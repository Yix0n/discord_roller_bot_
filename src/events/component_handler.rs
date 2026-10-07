
use serenity::all::{ComponentInteraction, ComponentInteractionDataKind, Context, Interaction};
use uuid::Uuid;
use crate::helpers::deck_interaction_creator::interaction_id_to_params;
use crate::helpers::errors::ErrorContext;
use crate::mechanics::spell_cards::card_selector_handler::handle_interaction;

pub async fn handle_component_interaction(
    ctx: &Context,
    interaction: &Interaction,
    component_interaction: &ComponentInteraction
) -> Result<(), ErrorContext>{
    let component_id = component_interaction.data.custom_id.clone();

    let (interaction_id, action_id, user_id) = interaction_id_to_params(component_id);

    let component = match &component_interaction.data.kind {
        ComponentInteractionDataKind::Button if interaction_id == "confirm" => {
            DeckComponentTypes::ConfirmButton { deck_id: action_id }
        }
        ComponentInteractionDataKind::Button if interaction_id == "discard" => {
            DeckComponentTypes::DiscardButton { deck_id: action_id }
        }

        ComponentInteractionDataKind::StringSelect{ values } if interaction_id == "select_cards" => {
            let v = values.clone();
            DeckComponentTypes::CardSelector { values: v }
        }

        _ => unreachable!(),
    };

    let result = match &component {
        DeckComponentTypes::CardSelector { values: _ } |
        DeckComponentTypes::ConfirmButton { deck_id: _ } |
        DeckComponentTypes::DiscardButton { deck_id: _ } => {
            handle_interaction(
                ctx,
                interaction,
                user_id,
                component,
            ).await
        }
    };

    match result {
        Err(e) => {
            println!("{:?}", e)
        }
        Ok(_) => {}
    }

    Ok(())
}
 pub enum DeckComponentTypes {
     CardSelector { values: Vec<String> },
     ConfirmButton { deck_id: Uuid },
     DiscardButton{ deck_id: Uuid }
 }


