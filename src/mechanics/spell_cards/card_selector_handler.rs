use chrono::Utc;
use serenity::all::{ComponentInteractionDataKind, Context, CreateEmbed, CreateInteractionResponseMessage, CreateMessage, EditInteractionResponse, Interaction, MessageId, UserId};
use uuid::{Uuid};
use crate::ConnectionPoolProvider;
use crate::database::models::hand_card::{CardAction, HandCard};
use crate::database::models::{Model, ModelId};
use crate::events::component_handler::DeckComponentTypes;
use crate::helpers::errors::{ErrorContext};
use serenity::all::colours::roles::GOLD;
use serenity::builder::CreateInteractionResponse;
use crate::database::models::character::Character;
use crate::database::models::deck::Deck;
use crate::database::models::skill::Skill;
use crate::helpers::deck_interaction_creator::{create_deck_use_interaction, interaction_id_to_params};
use crate::mechanics::spell_cards::draw_alg::draw_cards;
use crate::mechanics::spell_cards::special_decks::{inner_by_id, InnerCards};

pub async fn handle_interaction (
    ctx: &Context,
    interaction: &Interaction,
    user_id: u64,
    action: DeckComponentTypes,
) -> Result<(), ErrorContext> {
    let connection_pool = ctx.data.read().await.get::<ConnectionPoolProvider>().cloned().unwrap().clone();
    let message_component = interaction.clone().message_component().unwrap();

    message_component.defer(&ctx);

    if message_component.user.id != user_id {
        let _ = message_component.create_response(
            &ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .embed(CreateEmbed::new()
                        .title("You can't interact with this object"))
            )
        ).await;

        return Ok(())
    }

    match action {
        DeckComponentTypes::CardSelector { values } => {
            let mut hand: Vec<HandCard> = vec![];

            for value in values {
                let id = Uuid::try_parse(value.as_str())?;
                let card = HandCard::get_by_id(&connection_pool, id)?;
                if let Some(card) = card {
                    hand.push(card);
                } else { continue; };
            }

            if hand.is_empty() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Hand is Empty")))
                ).await;
                return Ok(());
            }

            let deck = Deck::get_by_id(&connection_pool, hand[0].deck.id)?;

            if deck.is_none() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Deck is Empty")))
                ).await;
                return Ok(());
            }

            let deck = deck.unwrap();

            let character = Character::get_by_binded_deck(&connection_pool, deck.id)?;

            if character.is_none() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Character is Empty")))
                ).await;
                return Ok(());
            }
            let character = character.unwrap();

            for mut card in hand.clone() {
                match card.card_action {
                    CardAction::None => card.card_action = CardAction::Selected,
                    CardAction::Selected => card.card_action = CardAction::None,
                }

                card.update(&connection_pool)?
            }

            let full_hand = HandCard::get_cards_by_character_and_deck_id(
                &connection_pool,
                character.id.clone(),
                deck.id.clone()
            )?;

            let response = create_deck_use_interaction(
                &connection_pool, deck, character, full_hand
            )?;

            let v = CreateInteractionResponse::UpdateMessage( CreateInteractionResponseMessage::new().embed(response.1).components(response.0));

            match message_component.create_response(&ctx, v).await {
                Ok(a) => {},
                Err(e) => println!("{}", e),
            }
        }

        DeckComponentTypes::ConfirmButton { deck_id } => {
            let deck = Deck::get_by_id(&connection_pool, deck_id)?;

            if deck.is_none() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Deck is Empty")))
                ).await;
                return Ok(());
            }

            let deck = deck.unwrap();
            let character = Character::get_by_binded_deck(&connection_pool, deck.id)?;

            if character.is_none() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Character is Empty")))
                ).await;
                return Ok(());
            }
            let character = character.unwrap();

            let hand = HandCard::get_cards_by_character_and_deck_id(
                &connection_pool,
                character.id,
                deck.id,
            )?;

            if hand.is_empty() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Hand is Empty")))
                ).await;
                return Ok(());
            }

            let casted_cards: Vec<HandCard> = hand.into_iter().filter(|c| c.card_action == CardAction::Selected).collect();

            if casted_cards.is_empty() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("No card is selected")))
                ).await;
                return Ok(());
            }

            let mut casted_inner: Vec<InnerCards> = Vec::with_capacity(casted_cards.len());
            for c in casted_cards.clone() {
                if let Some(c) = inner_by_id(c.skill.id, Some(&connection_pool))? {
                    casted_inner.push(c);
                }
            }


            casted_cards.into_iter().for_each(|d| {
                d.remove(&connection_pool);
            });

            let embed = CreateEmbed::new()
                .title(format!("{} cast", character.name))
                .color(GOLD)
                .fields(casted_inner.iter().map(|c| {
                    (
                        format!("{}", c.name),
                        format!("{}", c.description),
                        true
                    )
                }))
                .timestamp(Utc::now());

            let _ = message_component.create_response(&ctx, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed)
            )).await;
        }

        DeckComponentTypes::DiscardButton { deck_id } => {
            let deck = Deck::get_by_id(&connection_pool, deck_id)?;

            if deck.is_none() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Deck is Empty")))
                ).await;
                return Ok(());
            }

            let deck = deck.unwrap();
            let character = Character::get_by_binded_deck(&connection_pool, deck.id)?;

            if character.is_none() {
                let _ = message_component.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new()
                            .title("Character is Empty")))
                ).await;
                return Ok(());
            }

            let character = character.unwrap();

            let full_hand = HandCard::get_cards_by_character_and_deck_id(
                &connection_pool,
                character.id,
                deck.id,
            )?;

            let affected_cards: Vec<&HandCard> = full_hand.iter().filter(|c|
            c.card_action == CardAction::Selected).collect();

            let affected = affected_cards.len();

            for cards in affected_cards {
                cards.remove(&connection_pool)?;
            }

            if affected / 2 != 0 {
                let after_cards: Vec<HandCard> = full_hand.into_iter().filter(|c|
                    c.card_action == CardAction::None).collect();

                let collection = {
                    let collection_id = character.binded_collection;

                    match collection_id {
                        Some(c) => c,
                        None => return Ok(()),
                    }
                }.get(&connection_pool)?;

                let collection_cards = if collection.is_none() {
                    vec![]
                } else {
                    match Skill::get_by_collection(&connection_pool, collection.clone().unwrap().id) {
                        Ok(e) => e,
                        Err(e) => { println!("{:?}", e); return Ok(()); },
                    }
                };

                let new_cards = draw_cards(
                    collection_cards,
                    after_cards,
                    affected as isize / 2
                );

                let _: Vec<HandCard> = new_cards.clone().iter().map(|c| {
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

                    hand.create(&connection_pool).unwrap();

                    hand
                }).collect();

                let user_id = UserId::new(character.owner_id as u64);

                let user = user_id.to_user(ctx).await.unwrap();
                let result = user.direct_message(&ctx, CreateMessage::new()
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

                match result {
                    Err(e) => {
                        println!("{:?}", e)
                    },
                    _ => {}
                }
            }


        },
    }

    Ok(())
}
