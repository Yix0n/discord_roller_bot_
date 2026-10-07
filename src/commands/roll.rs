
use serenity::builder::CreateCommand;
use serenity::all::{Color, CommandInteraction, CommandOptionType, CreateCommandOption, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponseMessage, Interaction};
use serenity::client::Context;
use crate::commands::Commands;
use crate::helpers::errors::{ErrorContext, ProgramError};
use crate::helpers::expressioner::{ExprType, Expression};
use crate::helpers::interactions::get_string_option;

pub struct RollCommand;

impl Commands for RollCommand {
    const NAME: &'static str = "roll";
    const DESCRIPTION: &'static str = "Roll a dice";

    fn builder() -> CreateCommand {
        CreateCommand::new(RollCommand::NAME)
            .description(RollCommand::DESCRIPTION)
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "notation",
                    "dice notation")
                    .required(true)
            )
    }


    async fn execute(_: &Context, interaction: Interaction) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();
        let notation = get_string_option(&command.data.options, "notation").unwrap();

        let embed = roll(notation, command)?;
        
        Ok(CreateInteractionResponseMessage::new()
            .embed(embed))
    }
}

fn roll(notation: String, command: CommandInteraction) -> Result<CreateEmbed, ErrorContext> {
    let expr = match Expression::new(notation) {
        Ok(expr) => expr,
        Err(e) => return Err(ErrorContext::new(
            "Roll > 2.0",
            ProgramError::ParseError,
            e.as_str()
        ))
    };

    let interaction_author = command.user;
    let mut embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(
            interaction_author.name.clone(),
        ).icon_url(interaction_author.avatar_url().unwrap_or_default()))
        .color(Color::DARK_GREEN);

    match expr.compute() {
        Ok(result) => {
            if let Some(vars) = result.variable_results {
                embed = embed.description(format!("{} = {}", result.original_notation, result.total));
                for (var_name, var_result) in vars {
                    embed = embed.field(
                        format!("{} = {}", var_name, var_result.total.to_string()),
                        format!("{}", var_result.expression),
                        true
                    )
                }
            } else {
                embed = embed.description(format!("`{}` = **{}**", result.tokenized_result, result.total))
                    .field("​", result.original_notation, true);
            }
        },
        Err(e) => {
            return Err(ErrorContext::new(
                "Roll > 2.0",
                ProgramError::ParseError,
                e.as_str()
            ))
        }
    }

    if !expr.tokens.iter().any(|a| match a {
        ExprType::ExplodingDice(_,_,_) |
        ExprType::HighestDice(_,_,_) |
        ExprType::LowestDice(_,_,_) |
        ExprType::StandardDice(_,_)
        => true,
        _ => false,
    }) {
        embed = embed.footer(CreateEmbedFooter::new("No roll detected"));
    }

    Ok(embed)
}