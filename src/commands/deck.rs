use std::cmp::min;
use std::io::ErrorKind;
use chrono::Utc;
use serenity::all::{ChannelId, Color, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateEmbed, Interaction, UserId};
use serenity::all::colours::roles::{BLUE, RED};
use serenity::builder::{CreateEmbedFooter, CreateInteractionResponseMessage, CreateMessage};
use uuid::Uuid;
use crate::commands::Commands;
use crate::ConnectionPoolProvider;
use crate::database::ConnectionPool;
use crate::database::models::character::Character;
use crate::database::models::deck::Deck;
use crate::database::models::{Model, ModelId};
use crate::database::models::collection::Collection;
use crate::database::models::hand_card::{CardAction, HandCard};
use crate::database::models::skill::Skill;
use crate::helpers::deck_interaction_creator::create_deck_use_interaction;
use crate::helpers::errors::{ErrorContext, ProgramError};
use crate::helpers::interactions::{get_channel_option, get_integer_option, get_string_option, get_subcommand_option};
use crate::mechanics::spell_cards::draw_alg::draw_cards;
use crate::mechanics::spell_cards::parse_inner_key;
use crate::mechanics::spell_cards::special_decks::CardType;

pub struct DeckCommand;

impl DeckCommand {

}

impl Commands for DeckCommand {
    const NAME: &'static str = "deck";
    const DESCRIPTION: &'static str = "Deck Manager";

    fn builder() -> CreateCommand {
        CreateCommand::new(Self::NAME)
            .description(Self::DESCRIPTION)
            .add_option(
                CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "new",
                "Creates new deck")
                    .add_sub_option(CreateCommandOption::new(
                        CommandOptionType::String,
                        "deck_key",
                        "Key to access deck"
                    )
                        .required(true)
                    )
                    .add_sub_option(CreateCommandOption::new(
                        CommandOptionType::String,
                        "name",
                        "Name of deck"
                    ).required(true))
                    .add_sub_option(CreateCommandOption::new(
                        CommandOptionType::String,
                        "character_key",
                        "Character key to bind to deck"
                    ))
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "use",
                "Generates deck with playable cards"
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "deck_identifier",
                    "Name, Key or ID of deck."
                ))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "character_identifier",
                    "Name, Key or ID of character."
                ))
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "purge",
                "Removes all cards from the deck"
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "target_channel",
                    "Purges all cards from all deck that are binded to this channel"
                ))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "deck_identifier",
                    "Name, Key or ID of the Deck to purge"
                ))
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "delete",
                    "Removes deck"
                )
                    .add_sub_option(CreateCommandOption::new(
                        CommandOptionType::String,
                        "deck_identifier",
                        "Name, Key or ID of the deck to delete"
                    ).required(true))
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "draw",
                "Draw a card(s)"
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "amount",
                    "Number of cards to draw. Defaults to 2"
                ))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "character_id",
                    "ID of the character to draw a card"
                ))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "target_channel",
                    "All characters binded to this channel draws a card"
                ))
            )
    }

    async fn execute(ctx: &Context, interaction: Interaction) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();
        let (name, params) = get_subcommand_option(&command.data.options);

        let connection_pool = ctx.data.read().await.get::<ConnectionPoolProvider>().cloned().unwrap().clone();

        match name.as_str() {
            "new" => {
                let deck_key = get_string_option(&params, "deck_key").unwrap();
                let name = get_string_option(&params, "name").unwrap();
                let character_key = get_string_option(&params, "character_key");
                let user_id = command.user.id;

                execute_new(&connection_pool, deck_key, name, character_key, user_id)
            },
            "use" => {
                let deck_identifier = get_string_option(&params, "deck_identifier");
                let character_identifier = get_string_option(&params, "character_identifier");
                let channel_id = command.channel_id;
                let user_id = command.user.id;

                // try to get by binded channel if both are none
                execute_use(&connection_pool, deck_identifier, character_identifier, user_id, channel_id)
            },
            "purge" => {
                let target_channel = get_channel_option(&params, "target_channel");
                let deck_identifier = get_string_option(&params, "deck_identifier");

                execute_purge(&connection_pool, target_channel, deck_identifier)
            },
            "delete" => {
                let deck_identifier = get_string_option(&params, "deck_identifier").unwrap();
                let user_id = command.user.id;

                execute_delete(&connection_pool, user_id, deck_identifier)
            },
            "draw" => {
                let amount = get_integer_option(&params, "amount").unwrap_or(2);
                let character_id = get_string_option(&params, "character_id");
                let target_channel = get_channel_option(&params, "target_channel").unwrap_or(command.channel_id);

                execute_draw(&connection_pool, ctx, amount, character_id, target_channel).await
            },
            _ => unimplemented!()
        }
    }
}

fn execute_new(
    pool: &ConnectionPool,
    deck_key: String,
    name: String,
    character_key: Option<String>,
    user_id: UserId
) -> Result<CreateInteractionResponseMessage, ErrorContext> {
    let deck_key = parse_inner_key(deck_key.clone(), user_id);
    let deck_existing = Deck::get_by_key(&pool, &deck_key)?;

    if deck_existing.is_some() {
        return Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
            .title("Cannot create deck")
            .description("Deck with that key already exists.")
            .color(Color::RED)
        ))
    }

    drop(deck_existing);

    let deck = Deck {
        id: Uuid::now_v7(),
        name,
        key: deck_key,
        user_id: user_id.get() as isize,
        created_at: Utc::now(),
    };

    deck.create(&pool)?;

    if let Some(character_key) = character_key {
        let mut character = Character::get_by_key(&pool, &parse_inner_key(character_key, user_id))?;

        if let Some(mut character) = character {
            character.binded_deck = Some(ModelId::new(deck.id));

            character.update(&pool)?;
        }
    }

    Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
        .title("Deck created successfully.")
        .description("Deck created successfully.")
        .footer(CreateEmbedFooter::new(format!("ID: {}", deck.id)))
    ))
}

fn execute_use(
    pool: &ConnectionPool,
    deck_identifier: Option<String>,
    character_identifier: Option<String>,
    user_id: UserId,
    channel_id: ChannelId,
) -> Result<CreateInteractionResponseMessage, ErrorContext> {
    let character = {
        if let Some(character_id) = character_identifier {
            Character::find(&pool, user_id.get(), character_id)?
        } else {
            Character::get_by_bind_channel_and_user_id(&pool, channel_id.into(), user_id.into())?
        }
    };

    if character.is_none() {
        return Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
            .title("No character found")
            .description("No character found that has valid deck options")))
    }

    let character = character.unwrap();

    let deck = if let Some(deck) = &character.binded_deck {
        let deck = deck.get(&pool)?;
        deck
    } else {
        if let Some(deck) = deck_identifier {
            let deck = Deck::find(&pool, user_id.get(), deck)?;
            deck
        } else {
            return Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
                .title("Cannot find deck")
                .description("No deck found that has valid deck options")))
        }
    };

    let deck = if let Some(deck) = deck {
        deck
    } else {
        return Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
            .title("No deck found")
            .description("No deck found that has valid deck options")))

    };

    let hand_card = {
        let deck_id = deck.id;
        let character_id = character.id;

        let cards = HandCard::get_cards_by_character_and_deck_id(
            &pool,
            character_id,
            deck_id,
        )?;

        cards
    };

    let interaction = create_deck_use_interaction(
        &pool,
        deck, character,
        hand_card
    )?;

    Ok(CreateInteractionResponseMessage::new().embed(interaction.1).components(interaction.0))
}

fn execute_purge(
    pool: &ConnectionPool,
    target_channel: Option<ChannelId>,
    deck_identifier: Option<String>,
) -> Result<CreateInteractionResponseMessage, ErrorContext> {
    let characters = {
        if let Some(character_id) = deck_identifier {
            if let Some(character) = Character::get_by_id(&pool, Uuid::parse_str(character_id.as_str())?)? {
                vec![character]
            } else {
                return Err(ErrorContext::new(
                    "deck::purge",
                    ProgramError::NotFound,
                    "Character not found",
                ))
            }

        } else if let Some(channel) = target_channel {
            let channel_id = channel.get();

            Character::get_all_by_bind_channel(&pool, channel_id as isize)?
        } else {
            return Err(ErrorContext::new(
                "deck::purge",
                ProgramError::NotFound,
                "No character found",
            ))
        }
    };

    for character in &characters {
        let deck = match character.binded_deck.clone() {
            Some(deck) => deck.get(&pool)?,
            None => continue,
        };

        if deck.is_none() {
            continue;
        }

        let deck = deck.unwrap();

        HandCard::purge(&pool, deck.id)?
    }

    Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
        .title("Deck(s) purged successfully.")
        .description(format!("Purged {} decks", characters.len()))))
}

fn execute_delete(
    pool: &ConnectionPool,
    user_id: UserId,
    deck_identifier: String,
) -> Result<CreateInteractionResponseMessage, ErrorContext> {
    let deck = Deck::find(&pool, user_id.get(), deck_identifier)?;
    
    if deck.is_none() {
        return Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
        .title("No deck found")
        .description("No deck found that has valid deck options")
            .color(RED)));
    }
    
    let deck = deck.unwrap();
    
    deck.remove(&pool)?;
    
    Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
        .title("Deck has been deleted")
        .color(BLUE)))
}

async fn execute_draw(
    pool: &ConnectionPool,
    ctx: &Context,
    amount: i64,
    character_id: Option<String>,
    target_channel: ChannelId,
) -> Result<CreateInteractionResponseMessage, ErrorContext> {
    let characters = {
        if let Some(character_identifier) = character_id {
            vec![Character::get_by_id(
                &pool,
                Uuid::parse_str(&*character_identifier)?,
            )?].iter().filter(|c| c.is_some()).map(|c| c.clone().unwrap()).collect()
        } else {
            Character::get_all_by_bind_channel(
                &pool,
                target_channel.get() as isize
            )?
        }
    };

    if characters.is_empty() {
        return Err(ErrorContext::new(
            "No character found",
            ProgramError::NotFound,
            "No character found matching the criteria"
        ))
    }

    for character in characters.clone() {
        let deck = if let Some(deck_id) = character.binded_deck {
            let deck = deck_id.get(&pool)?;

            if deck.is_none() {
                continue;
            }

            deck.unwrap()
        } else {
            continue;
        };

        let collection = match character.binded_collection {
            Some(collection) => collection.get(&pool)?,
            None => None
        };

        let collection = if collection.is_none() {
            Collection {
                id: Default::default(),
                user_id: 0,
                name: "nil".to_string(),
                key: "nil".to_string(),
                created_at: Default::default(),
            }
        } else {
            collection.unwrap()
        };

        let hand = HandCard::get_cards_by_character_and_deck_id(
            &pool,
            character.id,
            deck.id
        )?;


        let collection_cards = if collection.id.to_string() == Uuid::nil().to_string() {
            vec![]
        } else {
            Skill::get_by_collection(&pool, collection.id)?
        };


        let new_cards = draw_cards(
            collection_cards,
            hand,
            min(10, amount as isize)
        );

        let card_initiator: Vec<HandCard> = new_cards.clone().iter().map(|c| {
            let hand = HandCard {
                id: Uuid::now_v7(),
                user_id: character.owner_id,
                card_type: c.r#type.to_db_type(),
                character: ModelId::new(character.id),
                deck: ModelId::new(deck.id),
                skill: ModelId::new(c.id),
                card_action: CardAction::None,
                created_at: Utc::now(),
            };

            hand.create(&pool).unwrap();

            hand


        }).collect();

        let user_id = UserId::new(character.owner_id as u64);

        let user = user_id.to_user(ctx).await.unwrap();
        let _ = user.direct_message(&ctx, CreateMessage::new()
            .embed(CreateEmbed::new()
                .title(format!("New cards drawn for {}", character.name))
                .fields(
                    new_cards.iter().map(|card| (
                        format!("{}", card.name),
                        format!("{}", card.description),
                        true
                        ))
                )
            )).await;
    }

    Ok(CreateInteractionResponseMessage::new().embed(CreateEmbed::new()
        .title("Draw successful.")
        .description(format!("Drawn cards to {} characters", characters.len()))
    ))
}