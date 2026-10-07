use std::sync::Arc;
use chrono::Utc;
use serenity::all::{Color, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponseMessage, Interaction};
use serenity::all::CommandOptionType::SubCommand;
use serenity::builder::CreateEmbedFooter;
use uuid::Uuid;
use crate::commands::Commands;
use crate::ConnectionPoolProvider;
use crate::database::ConnectionPool;
use crate::database::models::skill::Skill;
use crate::database::models::collection::Collection;
use crate::database::models::{Model, ModelId, TokenType};
use crate::helpers::errors::{ErrorContext, ProgramError};
use crate::helpers::interactions::{get_integer_option, get_string_option, get_subcommand_option, get_user_option};
use crate::mechanics::spell_cards::{deparse_inner_key, parse_inner_key};

pub struct SkillCommand;

impl Commands for SkillCommand {
    const NAME: &'static str = "skill";
    const DESCRIPTION: &'static str = "Skill Manager";

    fn builder() -> CreateCommand {
        CreateCommand::new(Self::NAME)
            .description(Self::DESCRIPTION)
            .add_option(CreateCommandOption::new(SubCommand, "new", "Creates a new skill")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "name of the skill", )
                    .required(true))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "key", "Key for the skill", )
                    .required(true))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "description", "Description for the skill", )
                    .required(true))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "collection", "Collection key",
                ).required(true))
            )
            .add_option(CreateCommandOption::new(SubCommand, "remove", "Removes a skill")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "key", "Key for the skill", ))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "id", "id of the skill", )))
            .add_option(CreateCommandOption::new(SubCommand, "info", "Show information about skill")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "key", "Key for the skill", ))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "id", "id of the skill", ))
            )
            .add_option(CreateCommandOption::new(SubCommand, "list", "List all skills", )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "user", "Target user"))
                .add_sub_option(CreateCommandOption::new(CommandOptionType::Integer, "page", "Number of page"))
            )
    }

    async fn execute(ctx: &Context, interaction: Interaction) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();
        let (name, params) = get_subcommand_option(&command.data.options);

        let connection_pool = ctx.data.read().await.get::<ConnectionPoolProvider>().cloned().unwrap().clone();

        let embed = match name.as_str() {
            "new" => {
                let name = get_string_option(&params, "name").unwrap();
                let key = get_string_option(&params, "key").unwrap();
                let description = get_string_option(&params, "description").unwrap();
                let collection = get_string_option(&params, "collection").unwrap();

                execute_new(connection_pool, name, command.user.id, key, description, collection)
            },
            "remove" => {
                let collection_token = get_string_option(&params, "key")
                    .map(crate::commands::collection::TokenType::Key)
                    .or_else(|| {
                        get_string_option(&params, "id")
                            .and_then(|id| Uuid::try_parse(&id).ok())
                            .map(crate::commands::collection::TokenType::ID)
                    })
                    .ok_or_else(|| ErrorContext::new(
                        "Collection::executor",
                        ProgramError::ParseError,
                        "Missing key or id"
                    ))?;

                execute_remove(connection_pool, collection_token, command.user.id)
            },
            "info" => {
                let collection_token = get_string_option(&params, "key")
                    .map(crate::commands::collection::TokenType::Key)
                    .or_else(|| {
                        get_string_option(&params, "id")
                            .and_then(|id| Uuid::try_parse(&id).ok())
                            .map(crate::commands::collection::TokenType::ID)
                    })
                    .ok_or_else(|| ErrorContext::new(
                        "Collection::executor",
                        ProgramError::ParseError,
                        "Missing key or id"
                    ))?;

                execute_info(connection_pool, collection_token, command.user.id)
            },
            "list" => {
                let user = get_user_option(&params, "user").unwrap_or(command.user.id);
                let page = get_integer_option(&params, "page").unwrap_or(0) as u8;

                execute_list(connection_pool, user, page)
            },
            _ => unreachable!(),
        }?;
        
        Ok(CreateInteractionResponseMessage::new()
            .embed(embed))
    }
}

fn execute_new(
    pool: Arc<ConnectionPool>,
    name: String,
    user_id: impl Into<u64>,
    key: String,
    description: String,
    collection_key: String,
) -> Result<CreateEmbed, ErrorContext> {
    let user_id: u64 = user_id.into();

    let skill_key = &parse_inner_key(key.clone(), user_id.clone());
    let collection_key = &parse_inner_key(collection_key.clone(), user_id.clone());

    let collection = Collection::get_by_key(&pool, &collection_key)?;

    if collection.is_none() {
        return Err(ErrorContext::new(
            "Skill::new",
            ProgramError::NotFound,
            "Collection not found",
        ))
    }

    let skill = Skill::get_by_key(&pool, &key.clone())?;

    if skill.is_some() {
        return Err(ErrorContext::new(
            "Skill::new",
            ProgramError::ParseError,
            "Skill with the given key already exists",
        ))
    }

    let skill: Skill = Skill {
        id: Uuid::now_v7(),
        user_id: user_id as isize,
        cost: 10,
        skill_name: name,
        skill_key: skill_key.clone(),
        description,
        created_at: Utc::now(),
        collection: ModelId::new(collection.unwrap().id),
    };

    skill.create(&pool)?;

    Ok(CreateEmbed::new()
        .title("Skill created")
        .description(format!("Skill {} created", skill.skill_name))
        .color(Color::GOLD)
        .footer(CreateEmbedFooter::new(format!("ID: {}", skill.id)))
    )
}

fn execute_remove(
    pool: Arc<ConnectionPool>,
    token: TokenType,
    user_id: impl Into<u64>,
) -> Result<CreateEmbed, ErrorContext> {
    let user_id: u64 = user_id.into();

    let skill = match token {
        TokenType::ID(id) => {
            Skill::get_by_id(&pool, id)
        },
        TokenType::Key(key) => {
            let key = parse_inner_key(key, user_id);

            Skill::get_by_key(&pool, &key)
        }
    }?;

    if skill.is_none() {
        return Err(ErrorContext::new(
            "Skill::remove",
            ProgramError::NotFound,
            "Skill not found",
        ))
    }

    let skill = skill.unwrap();

    if skill.user_id != user_id as isize {
        return Err(ErrorContext::new(
            "Skill::remove",
            ProgramError::NotFound,
            "You don't have permission to execute this action"
        ))
    }
    
    let _ = skill.remove(&pool)?;

    Ok(CreateEmbed::new()
        .title("Skill removed")
        .description(format!("Skill {} removed", skill.skill_name))
        .color(Color::GOLD)
    )
}

fn execute_info(
    pool: Arc<ConnectionPool>,
    token: TokenType,
    user_id: impl Into<u64>,
) -> Result<CreateEmbed, ErrorContext> {
    let user_id: u64 = user_id.into();
    let skill = match token {
        TokenType::ID(id) => {
            Skill::get_by_id(&pool, id)
        },
        TokenType::Key(key) => {
            let key = parse_inner_key(key, user_id);
            Skill::get_by_key(&pool, &key)
        }
    }?;

    if skill.is_none() {
        return Err(ErrorContext::new(
            "Skill::info",
            ProgramError::NotFound,
            "Skill not found",
        ))
    }

    let skill = skill.unwrap();
    let collection = skill.collection.get(&pool)?.unwrap();

    let embed = CreateEmbed::new()
        .title("Skill info")
        .description(format!("**{}**\n\n{}", skill.skill_name, skill.description))
        .footer(CreateEmbedFooter::new(format!("ID: {} | Created at:", skill.id)))
        .field("Collection Name:", collection.name, true)
        .field("Skill Key:", deparse_inner_key(skill.skill_key).0, true)
        .timestamp(skill.created_at);

    Ok(embed)
}

fn execute_list(
    pool: Arc<ConnectionPool>,
    user_id: impl Into<u64>,
    page: u8,
) -> Result<CreateEmbed, ErrorContext> {
    let user_id: u64 = user_id.into();
    let skills_list = Some(Skill::get_by_user_id_paginated(&pool, user_id as isize, page as isize)?);
    
    let mut embed = CreateEmbed::new()
        .title("Skills list")
        .color(Color::DARK_GREEN);

    match skills_list {
        Some(skills) if skills.len() > 0 => {
            for skill in skills {
                embed = embed.field(
                    skill.skill_name,
                    format!("Skill ID: {}\n Skill Key: {}\n Collection ID: {}", skill.id, deparse_inner_key(skill.skill_key).0, &skill.collection.id.to_string()),
                    false
                )
            };
        },
        Some(_) => {
            embed = embed.description("No collections found");
        }
        None => {
            embed = embed.field(
                "No collections found",
                "This user does not have any collections",
                false
            )
        }
    }

    Ok(embed)
}