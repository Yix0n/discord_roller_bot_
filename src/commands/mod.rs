use serenity::all::{CreateEmbed, CreateInteractionResponseMessage, CreateMessage, Interaction};
use serenity::builder::CreateCommand;
use serenity::client::Context;
use crate::commands::character::CharacterCommand;
use crate::commands::collection::CollectionCommand;
use crate::commands::deck::DeckCommand;
use crate::commands::npc_gen::NpcGenCommand;
use crate::commands::roll::RollCommand;
use crate::commands::skill::SkillCommand;
use crate::commands::weather::WeatherCommand;
use crate::helpers::errors::{ErrorContext, ProgramError};

pub mod roll;
pub mod weather;
pub mod npc_gen;
pub mod initiative;
pub mod skill;
pub mod collection;
pub mod character;
pub mod deck;

pub trait Commands {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;

    fn builder() -> CreateCommand;
    async fn execute(
        ctx: &Context,
        interaction: Interaction,
    ) -> Result<CreateInteractionResponseMessage, ErrorContext>;
}

pub fn get_builders() -> Vec<CreateCommand> {
    vec![
        RollCommand::builder(),
        WeatherCommand::builder(),
        NpcGenCommand::builder(),
        // InitiativeCommand::builder(),
        SkillCommand::builder(),
        CharacterCommand::builder(),
        DeckCommand::builder(),
        CollectionCommand::builder(),
    ]
}

pub async fn execute(command: &String, context: &Interaction, ctx: &Context) -> Result<CreateInteractionResponseMessage, ErrorContext> {
    let response = match command.to_lowercase().as_str() {
        "roll" => RollCommand::execute(ctx, context.clone()).await,
        "weather" => WeatherCommand::execute(ctx, context.clone()).await,
        "npc_generator" => NpcGenCommand::execute(ctx, context.clone()).await,
        // "initiative" => InitiativeCommand::execute(ctx, context.clone()).await,
        "skill" => SkillCommand::execute(ctx, context.clone()).await,
        "collections" => CollectionCommand::execute(ctx, context.clone()).await,
        "character" => CharacterCommand::execute(ctx, context.clone()).await,
        "deck" => DeckCommand::execute(ctx, context.clone()).await,

        _ => Err(ErrorContext {
            module: "Executioner".to_string(),
            error_type: ProgramError::CommandError,
            message: "Unknown Command".to_string(),
        })
    };

    response
}