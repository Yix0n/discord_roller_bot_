use std::cmp::PartialEq;
use serenity::all::{ButtonStyle, Color, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponseMessage, CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, Embed};
use serenity::builder::CreateEmbedFooter;
use uuid::Uuid;
use crate::database::ConnectionPool;
use crate::database::models::character::Character;
use crate::database::models::deck::Deck;
use crate::database::models::hand_card;
use crate::database::models::hand_card::{CardAction, HandCard};
use crate::helpers::errors::ErrorContext;
use crate::mechanics::spell_cards::deparse_inner_key;
use crate::mechanics::spell_cards::special_decks::{inner_by_id, InnerCards};

const SELECTED_EMOJI: char = '\u{2705}'; // ✅
const PRIMARY_EMOJI: char = '\u{1f4a5}'; // 💥
const SECONDARY_EMOJI: char = '\u{1f5e1}'; // 🗡
const OTHER_EMOJI: char = '\u{2623}'; // ☣

pub fn create_deck_use_interaction(
    pool: &ConnectionPool,
    deck: Deck,
    character: Character,
    cards: Vec<HandCard>,
) -> Result<(Vec<CreateActionRow>, CreateEmbed), ErrorContext> {
    let character_name = character.name.clone();
    let is_any_selected = cards.iter().any(|c| c.card_action == CardAction::Selected);
    
    let mut embed = {
        let e = CreateEmbed::new()
            .title(format!("{}'s Deck", character_name))
            .footer(CreateEmbedFooter::new(format!("ID: {}", deck.id)))
            .color(Color::DARK_GREEN)
            ;

            e
    };

    let select_menu_options = {
        let mut v: Vec<(String, String)> = vec![];

        for card in cards {
            let skill: InnerCards = if let Some(c) = inner_by_id(card.skill.id, Some(pool))? {
                c
            } else {
                continue;
            };

            let name = skill.name;
            let description = skill.description;
            let id = card.id.to_string();

            let should_select = match card.card_action {
                CardAction::None => false,
                CardAction::Selected => true
            };

            embed = embed.field(
                format!("{}{} | {}", {
                    if should_select {
                        SELECTED_EMOJI
                    } else {
                        '\0'
                    }
                }, {
                    match card.card_type {
                        hand_card::CardType::Primary => PRIMARY_EMOJI,
                        hand_card::CardType::Secondary => SECONDARY_EMOJI,
                        hand_card::CardType::Other => OTHER_EMOJI,
                    }
                }, name),
                description,
                true
            );

            let name = format!("{}{}",
                               match card.card_type {
                                   hand_card::CardType::Primary => PRIMARY_EMOJI,
                                   hand_card::CardType::Secondary => SECONDARY_EMOJI,
                                   hand_card::CardType::Other => OTHER_EMOJI,
                               },
                name
            );

            v.push((id, name));
        }

        v
    };

    let deck_id = deck.id.to_string();

    let discard_button = CreateButton::new(to_interaction_id("discard", &deck_id, character.owner_id as u64))
        .label("Discard")
        .style(ButtonStyle::Danger)
        .disabled(!is_any_selected);
    let confirm_button = CreateButton::new(to_interaction_id("confirm", &deck_id, character.owner_id as u64))
        .label("Confirm")
        .style(ButtonStyle::Primary)
        .disabled(!is_any_selected);
    let select_menu = CreateSelectMenu::new(to_interaction_id("select_cards", &deck_id, character.owner_id as u64), CreateSelectMenuKind::String {
        options: select_menu_options.iter().map(|(id, name)| {
            CreateSelectMenuOption::new(
                name, id
            )
        }).collect(),
    }).placeholder("Cards..");

    let select_row = CreateActionRow::SelectMenu(select_menu);
    let button_row = CreateActionRow::Buttons(vec![confirm_button, discard_button]);

    let components = if select_menu_options.is_empty() {
        vec![button_row]
    } else {
        vec![button_row, select_row]
    };

    Ok((components, embed))
}

pub fn interaction_id_to_params(
    value: String,
) -> (String, Uuid, u64) {

    let components = value.split("::").collect::<Vec<&str>>();
    let component_id = components[0];
    let inner_id = Uuid::parse_str(components[1]).unwrap();
    let user_id: u64 = components[2].parse().unwrap();

    (component_id.to_string(), inner_id, user_id)
}

pub fn to_interaction_id(
    name: impl Into<String>,
    id: impl Into<String>,
    user_id: u64
) -> String {
    let name = name.into();
    let id = id.into();
    let user_id = user_id.to_string();

    format!("{}::{}::{}", name, id, user_id)
}

