use std::str::FromStr;
use chrono::Utc;
use rusqlite::fallible_iterator::FallibleIterator;
use serenity::all::{ChannelId, CommandOptionType, Context, CreateCommand, CreateEmbed, Interaction, UserId};
use serenity::all::CommandOptionType::{Channel, Integer, String as DString, SubCommand, SubCommandGroup, User};
use serenity::builder::{CreateCommandOption, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponseMessage};
use serenity::model::Color;
use uuid::Uuid;
use crate::commands::Commands;
use crate::ConnectionPoolProvider;
use crate::database::ConnectionPool;
use crate::database::models::character::Character;
use crate::database::models::{Model, ModelId, TokenType};
use crate::database::models::collection::Collection;
use crate::database::models::deck::Deck;
use crate::helpers::character_stats::get_character_health;
use crate::helpers::errors::{ErrorContext, ProgramError};
use crate::helpers::errors::ProgramError::ParseError;
use crate::helpers::interactions::{get_channel_option, get_integer_option, get_string_option, get_subcommand_option, get_user_option};
use crate::mechanics::spell_cards::{deparse_inner_key, parse_inner_key};
use crate::types::character::Stat;

pub struct CharacterCommand;

impl Commands for CharacterCommand {
    const NAME: &'static str = "character";
    const DESCRIPTION: &'static str = "Character Manager";

    fn builder() -> CreateCommand {
        CreateCommand::new(Self::NAME)
            .description(Self::DESCRIPTION)
            .add_option(CreateCommandOption::new(
                SubCommand,
                "create",
                "Creates new character"
            ).add_sub_option(
                CreateCommandOption::new(
                    DString,
                    "name",
                    "Name of character"
                )
                    .required(true)
            ).add_sub_option(
                CreateCommandOption::new(
                    DString,
                    "key",
                    "Key of character"
                )
                    .required(true)
            ).add_sub_option(
                CreateCommandOption::new(
                    Channel,
                    "sheet",
                    "Channel with character Sheet"
                )
                    .required(true)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "durability",
                    "Durability value"
                )
                    .required(true)
                    .min_int_value(0)
                    .max_int_value(20)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "strength",
                    "Strength value"
                )
                    .required(true)
                    .min_int_value(0)
                    .max_int_value(20)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "intelligence",
                    "Intelligence value"
                )
                    .required(true)
                    .min_int_value(0)
                    .max_int_value(20)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "dexterity",
                    "Dexterity value"
                )
                    .required(true)
                    .min_int_value(0)
                    .max_int_value(20)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "perception",
                    "Perception value"
                )
                    .required(true)
                    .min_int_value(0)
                    .max_int_value(20)
            )
            ).add_option(
            CreateCommandOption::new(
                SubCommandGroup,
                "bind",
                "Bind character to a deck or channel"
            ).add_sub_option(
                CreateCommandOption::new(
                    SubCommand,
                    "deck",
                    "Bind Deck to a character"
                ).add_sub_option(
                    CreateCommandOption::new(
                        DString,
                        "character_identifier",
                        "Key, Name or ID for character"
                    )
                        .required(true)
                ).add_sub_option(
                    CreateCommandOption::new(
                        DString,
                        "deck_identifier",
                        "Key, Name or ID for deck"
                    )
                        .required(true)
                )
            ).add_sub_option(
                CreateCommandOption::new(
                    SubCommand,
                    "channel",
                    "Bind Channel to a character"
                ).add_sub_option(
                    CreateCommandOption::new(
                        DString,
                        "character_identifier",
                        "Key, Name or ID for character"
                    )
                        .required(true)
                ).add_sub_option(
                    CreateCommandOption::new(
                        Channel,
                        "channel",
                        "Channel to bind (default this). To unbind give the same channel as parameter"
                    )
                )
            )
                .add_sub_option(
                    CreateCommandOption::new(
                        SubCommand,
                        "collection",
                        "Bind Collection to character"
                    )
                        .add_sub_option(
                            CreateCommandOption::new(
                                DString,
                                "collection_identifier",
                                "Key, Name or ID for collection"
                            ).required(true)
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                DString,
                                "character_identifier",
                                "Key, Name or ID for character. Defaults to channel binded"
                            )
                        )
                )
        ).add_option(
            CreateCommandOption::new(
                SubCommand,
                "list",
                "List all characters"
            ).add_sub_option(
                CreateCommandOption::new(
                    User,
                    "user",
                    "User to list"
                )
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "page",
                    "Page of list"
                )
                    .min_int_value(0)
                    .max_int_value(255)
            )
        ).add_option(
            CreateCommandOption::new(
                SubCommand,
                "sheet",
                "Get character details"
            ).add_sub_option(
                CreateCommandOption::new(
                    DString,
                    "character_identifier",
                    "Name, Key or ID of the character. Defaults to channel binded"
                )
            )
        ).add_option(
            CreateCommandOption::new(
                SubCommand,
                "update",
                "Update character with given ID or by sheet channel"
            ).add_sub_option(
                CreateCommandOption::new(
                    DString,
                    "character_identifier",
                    "Name, Key or ID of the character. Defaults to channel binded"
                )
            ).add_sub_option(
                CreateCommandOption::new(
                    DString,
                    "name",
                    "Name of character"
                )
            ).add_sub_option(
                CreateCommandOption::new(
                    Channel,
                    "sheet",
                    "Channel with character sheet"
                )
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "durability",
                    "Durability value"
                )
                    .min_int_value(0)
                    .max_int_value(20)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "strength",
                    "Strength value"
                )
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "intelligence",
                    "Intelligence value"
                )
                    .min_int_value(0)
                    .max_int_value(20)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "dexterity",
                    "Dexterity value"
                )
                    .min_int_value(0)
                    .max_int_value(20)
            ).add_sub_option(
                CreateCommandOption::new(
                    Integer,
                    "perception",
                    "Perception value"
                )
                    .min_int_value(0)
                    .max_int_value(20)
            )
        ).add_option(
            CreateCommandOption::new(
                SubCommand,
                "delete",
                "Deletes character"
            ).add_sub_option(
                CreateCommandOption::new(
                    DString,
                    "character_identifier",
                    "Name, Key or ID for character"
                ).required(true)
            )
        )
    }

    async fn execute(ctx: &Context, interaction: Interaction) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();
        let (group_name, group_params) = get_subcommand_option(&command.data.options);

        let connection_pool = ctx.data.read().await.get::<ConnectionPoolProvider>().cloned().unwrap().clone();

        let embed = match group_name.as_str() {
            "create" => {
                let name = get_string_option(&group_params, "name").unwrap();
                let key = get_string_option(&group_params, "key").unwrap();
                let sheet = get_channel_option(&group_params, "sheet").unwrap();
                let durability = get_integer_option(&group_params, "durability").unwrap();
                let strength = get_integer_option(&group_params, "strength").unwrap();
                let intelligence = get_integer_option(&group_params, "intelligence").unwrap();
                let dexterity = get_integer_option(&group_params, "dexterity").unwrap();
                let perception = get_integer_option(&group_params, "perception").unwrap();

                let user = command.user.id;

                execute_create(
                    &connection_pool,
                    user,
                    name,
                    key,
                    sheet,
                    durability,
                    strength,
                    intelligence,
                    dexterity,
                    perception,
                )
            },
            "bind" => {
                let (sub_name, sub_params) = get_subcommand_option(&group_params);

                match sub_name.as_str() {
                    "deck" => {
                        let character_identifier = get_string_option(&sub_params, "character_identifier").unwrap();
                        let deck_identifier = get_string_option(&sub_params, "deck_identifier").unwrap();

                        let user_id = command.user.id;

                        execute_bind_deck(
                            &connection_pool,
                            user_id,
                            character_identifier,
                            deck_identifier,
                        )
                    },
                    "channel" => {
                        let character_identifier = get_string_option(&sub_params, "character_identifier").unwrap();
                        let channel = get_channel_option(&sub_params, "channel").unwrap_or(command.channel_id);

                        let user_id = command.user.id;

                        execute_bind_channel(
                            &ctx,
                            &connection_pool,
                            user_id,
                            character_identifier,
                            channel,
                        ).await
                    },
                    "collection" => {
                        let character_identifier = get_string_option(&sub_params, "character_identifier");
                        let channel = command.channel_id;
                        let collection_identifier = get_string_option(&sub_params, "collection_identifier").unwrap();

                        let user_id = command.user.id;

                        execute_bind_collection(
                            &ctx,
                            &connection_pool,
                            user_id,
                            character_identifier,
                            channel,
                            collection_identifier,
                        ).await
                    }
                    _ => unreachable!()
                }
            },
            "list" => {
                let user = get_user_option(&group_params, "user").unwrap_or(command.user.id);
                let page = get_integer_option(&group_params, "page").unwrap_or(0);


                execute_list(
                    &connection_pool,
                    user,
                    page,
                )
            },
            "sheet" => {
                let character_identifier = get_string_option(&group_params, "character_identifier");

                let channel_id = command.channel_id;
                let user = command.user.id;

                execute_sheet(
                    &ctx,
                    &connection_pool,
                    user,
                    character_identifier,
                    channel_id,
                ).await
            },
            "update" => {
                let character_identifier = get_string_option(&group_params, "character_identifier");
                let name = get_string_option(&group_params, "name");
                let sheet_channel = get_channel_option(&group_params, "sheet");
                let durability = get_integer_option(&group_params, "durability");
                let strength = get_integer_option(&group_params, "strength");
                let intelligence = get_integer_option(&group_params, "intelligence");
                let dexterity = get_integer_option(&group_params, "dexterity");
                let perception = get_integer_option(&group_params, "perception");

                let user_id = command.user.id;

                execute_update(
                    &connection_pool,
                    user_id,
                    character_identifier,
                    command.channel_id,
                    name,
                    sheet_channel,
                    durability,
                    strength,
                    intelligence,
                    dexterity,
                    perception,
                )
            },
            "delete" => {
                let character_identifier = get_string_option(&group_params, "character_identifier").unwrap();
                let user = command.user.id;

                execute_delete(
                    &connection_pool,
                    user,
                    character_identifier,
                )
            },

            _ => unreachable!()
        }?;

        Ok(CreateInteractionResponseMessage::new().embed(embed))
    }
}

fn execute_create(
    pool: &ConnectionPool,
    user_id: UserId,
    name: String,
    key: String,
    sheet_channel: impl Into<u64>,
    durability: i64,
    strength: i64,
    intelligence: i64,
    dexterity: i64,
    perception: i64,
) -> Result<CreateEmbed, ErrorContext> {
    let user_id: u64 = user_id.into();
    let key = parse_inner_key(key, user_id);

    let character = Character::get_by_key(&pool, &key)?;

    if character.is_some() {
        return Err(ErrorContext::new(
            "Character::create",
            ProgramError::DatabaseError,
            "You already have character with this key"
        ))
    };

    let character = Character {
        id: Uuid::now_v7(),
        key,
        name,
        binded_deck: None,
        binded_channel: None,
        binded_collection: None,
        owner_id: user_id as isize,
        detail_channel: sheet_channel.into() as isize,
        durability: durability as i16,
        strength: strength as i16,
        intelligence: intelligence as i16,
        dexterity: dexterity as i16,
        perception: perception as i16,
        created_at: Utc::now(),
    };

    character.create(pool)?;

    Ok(CreateEmbed::new()
        .title("Created character")
        .description(format!("{} was created successfully", character.name))
        .footer(CreateEmbedFooter::new(format!("ID: {}", character.id)))
    )
}

fn execute_bind_deck(
    pool: &ConnectionPool,
    user_id: UserId,
    character_key: String,
    deck_key: String,
) -> Result<CreateEmbed, ErrorContext> {
    let character_key = parse_inner_key(character_key, user_id);
    let deck_key = parse_inner_key(deck_key, user_id);

    let mut character = Character::get_by_key(&pool, &character_key)?;

    if character.is_none() {
        return Err(ErrorContext::new(
            "Character::executor",
            ProgramError::NotFound,
            "No character found"
        ))
    }

    let mut character = character.unwrap();

    let deck = Deck::get_by_key(&pool, &deck_key)?;

    if deck.is_none() {
        return Err(ErrorContext::new(
            "Deck::executor",
            ProgramError::NotFound,
            "No deck found"
        ))
    }

    let deck = deck.unwrap();

    character.binded_deck = Some(ModelId::new(deck.id));

    character.update(pool)?;

    Ok(CreateEmbed::new().title("Deck binded")
        .description(format!("{} is now binded to {}", deck.name, character.name)))
}

async fn execute_bind_channel(
    ctx: &Context,
    pool: &ConnectionPool,
    user_id: UserId,
    character_key: String,
    channel: ChannelId,
) -> Result<CreateEmbed, ErrorContext> {
    let key = parse_inner_key(character_key, user_id);

    let mut character = Character::get_by_key(&pool, &key)?;

    if character.is_none() {
        return Err(ErrorContext::new(
            "Character::bind::channel",
            ProgramError::DatabaseError,
            "Character not found"
        ))
    };

    let mut character = character.unwrap();
    let channel_name= channel.name(&ctx).await.unwrap();

    match character.binded_channel {
        Some(chnl) if channel.get() as isize == chnl => {
            character.binded_channel = None;
        },
        _ => {
            character.binded_channel = Some(channel.get() as isize);
        }
    }

    character.update(&pool)?;

    Ok(CreateEmbed::new()
        .title("Binded character")
        .description(format!("{} was binded to {}", character.name, channel_name)))
}

async fn execute_bind_collection(
    ctx: &Context,
    pool: &ConnectionPool,
    user_id: UserId,
    character_identifier: Option<String>,
    channel: ChannelId,
    collection_identifier: String,
) -> Result<CreateEmbed, ErrorContext> {
    let character = {
        if let Some(character) = character_identifier {
            Character::find(
                &pool,
                user_id.into(),
                character,
            )?
        } else {
            Character::get_by_bind_channel_and_user_id(
                &pool,
                channel.get(),
                user_id.get()
            )?
        }
    };

    if character.is_none() {
        return Err(ErrorContext::new(
            "Character::bind::collection",
            ProgramError::DatabaseError,
            "Character not found"
        ))
    }

    let collection = Collection::find(&pool, user_id.get(), collection_identifier)?;

    if collection.is_none() {
        return Err(ErrorContext::new(
            "Collection::find",
            ProgramError::DatabaseError,
            "Collection not found"
        ))
    }

    let mut character = character.unwrap();
    let collection = collection.unwrap();

    character.binded_collection = Some(ModelId::new(collection.id));

    character.update(pool)?;

    Ok(CreateEmbed::new()
        .title("Binded collection")
        .description(format!("Bind between {} and {} created", collection.name, character.name))
    )
}

fn execute_list(
    pool: &ConnectionPool,
    user_id: UserId,
    page: i64,
) -> Result<CreateEmbed, ErrorContext> {
    let user_id = user_id.get() as isize;
    let character_list = Character::get_by_user_id_paginated(&pool, user_id, page as isize)?;

    println!("{:?}", user_id);

    let mut embed = CreateEmbed::new()
        .title("Characters list")
        .color(Color::DARK_GREEN);

    match character_list {
        characters if characters.len() > 0 => {
            for character in characters {
                embed = embed.field(
                    character.name,
                    format!(
                        "Character ID: {}\nCharacter Key: {}",
                        character.id, deparse_inner_key(character.key).0
                    ),
                    false
                )
            };
        },
        _ => {
            embed = embed.description("No characters found");
        },
    }

    Ok(embed)
}

async fn execute_sheet(
    ctx: &Context,
    pool: &ConnectionPool,
    user_id: UserId,
    character_identifier: Option<String>,
    channel: ChannelId,
) -> Result<CreateEmbed, ErrorContext> {
    let character = match character_identifier {
        Some(identifier) => Character::find(
            &pool,
            user_id.get(),
            identifier
        )?,
        None => Character::get_by_bind_channel_and_user_id(
            &pool, channel.get(), user_id.get()
        )?,
    };

    let character = match character {
        None => {
            return Err(ErrorContext::new(
                "Character::sheet",
                ProgramError::DatabaseError,
                "No character found"
            ))
        },
        Some(c) => c
    };
    let author = user_id.to_user(&ctx).await.unwrap();

    let embed = CreateEmbed::new()
        .title(character.name)
        .author(CreateEmbedAuthor::new(
            author.name.to_string())
            .icon_url(author.face()))
        .color(Color::DARK_GREEN)
        .footer(CreateEmbedFooter::new(
            format!("ID: {} | Created at:", character.id)))
        .timestamp(character.created_at)
        .field(
            "Sheet",
            format!("<#{}>", character.detail_channel),
            false,
        )
        .field(
            format!("Strength: {}", character.strength),
            "​",
            true
        )
        .field(
            format!("Durability: {}", character.durability),
            format!("Max HP: {}", get_character_health(&Stat { points: character.durability as i32, talents: 1 })),
            true
        )
        .field(
            format!("Intelligence: {}", character.intelligence),
            "​",
            true
        )
        .field(
            format!("Dexterity: {}", character.dexterity),
            "​",
            true
        )
        .field(
            format!("Perception: {}", character.perception),
            "​",
            true
        );

    Ok(embed)
}

fn execute_update(
    pool: &ConnectionPool,
    user_id: UserId,
    character_id: Option<String>,
    channel_id: ChannelId,
    name: Option<String>,
    sheet_channel: Option<ChannelId>,
    durability: Option<i64>,
    strength: Option<i64>,
    intelligence: Option<i64>,
    dexterity: Option<i64>,
    perception: Option<i64>,
) -> Result<CreateEmbed, ErrorContext> {
    let mut character: Option<Character> = {
        match character_id {
            Some(id) => {
                let id = match Uuid::parse_str(&id) {
                    Ok(id) => Ok(id),
                    Err(e) => Err(ErrorContext::new(
                        "Character::Update",
                        ParseError,
                        e.to_string().as_str()
                    ))
                }?;

                Character::get_by_id(&pool, id)
            },
            None => {
                let channel_id = channel_id.get();

                rusqlite::Result::Ok(
                    Character::get_by_details_channel_and_user_id(
                        &pool,
                        channel_id,
                        user_id.get()
                    )?
                )
            }
        }
    }?;

    let mut character = match character {
        Some(character) => character,
        None => Err(ErrorContext::new("Character::update", ParseError, "No character found"))?,
    };

    if character.owner_id as u64 != user_id.get() {
        return Err(ErrorContext::new(
            "Character::update",
            ProgramError::DatabaseError,
            "You don't have permission to execute this action"
        ))
    }

    if let Some(name) = name {
        character.name = name;
    }

    if let Some(sheet_channel) = sheet_channel {
        character.detail_channel = sheet_channel.get() as isize;
    }

    if let Some(durability) = durability {
        character.durability = durability as i16
    }

    if let Some(strength) = strength {
        character.strength = strength as i16
    }

    if let Some(intelligence) = intelligence {
        character.intelligence = intelligence as i16
    }

    if let Some(dexterity) = dexterity {
        character.dexterity = dexterity as i16
    }

    if let Some(perception) = perception {
        character.perception = perception as i16
    }

    character.update(&pool);

    Ok(CreateEmbed::new()
        .title("Character Updated")
        .description(format!("Updated character: {}", character.name))
        .color(Color::GOLD)
    )
}

fn execute_delete(
    pool: &ConnectionPool,
    user_id: UserId,
    token: String,
) -> Result<CreateEmbed, ErrorContext> {
    let character = Character::find(
        &pool,
        user_id.get(),
        token
    )?;

    let character = if let Some(character) = character {
        character
    } else {
        return Err(ErrorContext::new("Character::delete", ParseError, "No character found"));
    };

    if character.owner_id as u64 != user_id.get() {
        return Err(ErrorContext::new(
            "Character::delete",
            ProgramError::DatabaseError,
            "You don't have permission to execute this action"
        ))
    }

    let affected = character.remove(&pool, user_id.get() as isize)?;
    
    Ok(CreateEmbed::new()
        .title("Deleted character")
        .description(format!("Deleted character: {}", character.name))
        .color(Color::RED)
    )
}