use std::sync::Arc;
use chrono::{DateTime, Utc};
use serenity::all::{Color, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateInteractionResponseMessage, Interaction, UserId};
use uuid::Uuid;
use crate::commands::Commands;
use crate::ConnectionPoolProvider;
use crate::database::ConnectionPool;
use crate::database::models::collection::Collection;
use crate::database::models::Model;
pub(crate) use crate::database::models::TokenType;
use crate::helpers::errors::{ErrorContext, ProgramError};
use crate::helpers::interactions::{get_integer_option, get_string_option, get_subcommand_option, get_user_option};
use crate::mechanics::spell_cards::{deparse_inner_key, parse_inner_key};

pub struct CollectionCommand;

impl Commands for CollectionCommand {
    const NAME: &'static str = "collections";
    const DESCRIPTION: &'static str = "Skill Collection Manager";

    fn builder() -> CreateCommand {
        CreateCommand::new(Self::NAME)
            .description(Self::DESCRIPTION)
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "new", "Creates new collection")
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "name", "Name of the collection").required(true))
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "key", "key of the collection").required(true))
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "info", "Shows info about the collection. Requires Key or ID")
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "key", "key of the collection"))
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "id", "id of the collection"))
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "list", "List all available collections")
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "user", "Target user"))
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::Integer, "page", "Number of pages (skips 10 entries)"))
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::SubCommand, "remove", "Remove an existing collection. Requires key or ID")
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "key", "key of the collection"))
                    .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "id", "id of the collection"))
            )
    }

    async fn execute(
        ctx: &Context,
        interaction: Interaction
    ) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();
        let (name, params) = get_subcommand_option(&command.data.options);

        let connection_pool = ctx.data.read().await.get::<ConnectionPoolProvider>().cloned().unwrap().clone();

        let embed = match name.as_str() {
            "new" => {
                // name, key

                let collection_name = get_string_option(&params, "name").unwrap();
                let collection_key = get_string_option(&params, "key").unwrap();
                let user_id = command.user.id;

                execute_new(
                    connection_pool,
                    user_id,
                    collection_name,
                    collection_key,
                ).await
            },
            "info" => {
                // *key, *id

                let collection_token = get_string_option(&params, "key")
                    .map(TokenType::Key)
                    .or_else(|| {
                        get_string_option(&params, "id")
                            .and_then(|id| Uuid::try_parse(&id).ok())
                            .map(TokenType::ID)
                    })
                    .ok_or_else(|| ErrorContext::new(
                        "Collection::executor",
                        ProgramError::ParseError,
                        "Missing key or id"
                    ))?;
                let user_id = command.user.id;

                execute_into(connection_pool, ctx, user_id, collection_token).await
            },
            "list" => {
                // *user, *page

                let user: UserId = get_user_option(&params, "user").unwrap_or(command.user.id);
                let page = get_integer_option(&params, "page").unwrap_or(0) as u8;

                execute_list(connection_pool, user, page).await
            },
            "remove" => {
                // *key, *id

                let collection_token = get_string_option(&params, "key")
                    .map(TokenType::Key)
                    .or_else(|| {
                        get_string_option(&params, "id")
                            .and_then(|id| Uuid::try_parse(&id).ok())
                            .map(TokenType::ID)
                    })
                    .ok_or_else(|| ErrorContext::new(
                        "Collection::executor",
                        ProgramError::ParseError,
                        "Missing key or id"
                    ))?;
                let user_id = command.user.id;

                execute_remove(connection_pool, user_id, collection_token).await
            },
            _ => unreachable!()
        }?;
        
        Ok(CreateInteractionResponseMessage::new().embed(embed))
    }
}

async fn execute_new(
    pool: Arc<ConnectionPool>,
    user_id: UserId,
    name: String,
    key: String,
) -> Result<CreateEmbed, ErrorContext> {
    let user_id = user_id.get();

    let inner_key = parse_inner_key(key, user_id);

    let has_collection = Collection::get_by_key(&pool, &inner_key)?;

    match has_collection {
        Some(_) => return Ok(CreateEmbed::new().color(Color::RED).description("You already have collection with that key")),
        _ => {}
    }

    let collection = Collection {
        id: Uuid::now_v7(),
        user_id: user_id as isize,
        name,
        key: inner_key,
        created_at: Utc::now(),
    };

    collection.create(&pool)?;

    Ok(CreateEmbed::new()
        .color(Color::DARK_GREEN)
        .description(format!("Collection {} created!", collection.name))
        .title("New Collection")
        .footer(
            CreateEmbedFooter::new(format!("ID: {}", collection.id))
        )
    )
}

async fn execute_into(
    pool: Arc<ConnectionPool>,
    ctx: &Context,
    user_id: UserId,
    token: TokenType,
) -> Result<CreateEmbed, ErrorContext> {
    let collection = match token {
        TokenType::ID(id) => {
            Collection::get_by_id(&pool, id)?
        },
        TokenType::Key(key) => {
            Collection::get_by_key(&pool, &parse_inner_key(key, user_id))?
        }
    };

    match collection {
        Some(collection) => {
            let creator = UserId::new(collection.user_id as u64).to_user(ctx).await.unwrap();

            let embed = CreateEmbed::new()
                .title(format!("Collection {}", collection.name))
                .color(Color::BLUE)
                .field("Created by", creator.name, true)
                .timestamp(collection.created_at)
                .footer(CreateEmbedFooter::new(format!("ID: {}; Created: ", collection.id)));

            Ok(embed)
        }

        None => Err(ErrorContext::new(
            "Collection::executor",
            ProgramError::NotFound,
            "Collection not found",
        ))
    }
}

async fn execute_list(
    pool: Arc<ConnectionPool>,
    user_id: UserId,
    page: u8
) -> Result<CreateEmbed, ErrorContext> {
    let collection_list = Collection::get_by_user_id_paginated(&pool, user_id.get() as isize, page as isize)?;

    let mut embed = CreateEmbed::new()
    .title("Collections list")
        .color(Color::DARK_GREEN);

    match collection_list {
        collections if collections.len() > 0 => {
            for collection in collections {
                embed = embed.field(
                    collection.name,
                    format!("Collection ID: {}\n Collection Key: {}", collection.id, deparse_inner_key(collection.key).1),
                    true
                )
            };
        },
        _ => {
            embed = embed.description("No collections found");
        }
    }

    Ok(embed)
}

async fn execute_remove(
    pool: Arc<ConnectionPool>,
    user_id: UserId,
    token: TokenType
) -> Result<CreateEmbed, ErrorContext> {
    let collection = match token {
        TokenType::ID(id) => {
            Collection::get_by_id(&pool, id)?
        },
        TokenType::Key(key) => {
            Collection::get_by_key(&pool, &parse_inner_key(key, user_id))?
        }
    };

    let collection = match collection {
        Some(collection) => collection,
        None => {
            return Err(ErrorContext::new(
                "Collection::remove",
                ProgramError::NotFound,
                "Collection not found",
            ))
        }
    };

    collection.remove(&pool)?;

    Ok(CreateEmbed::new()
        .color(Color::FOOYOO)
        .title("Collections removed")
        .description(format!("Collection {} removed!", collection.name)))
}