use serenity::all::{CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponseMessage, Interaction};
use serenity::builder::CreateEmbedAuthor;
use crate::commands::Commands;
use crate::helpers::errors::ErrorContext;
use crate::helpers::interactions::{get_integer_option, get_string_option};
use crate::mechanics::npc_generator::data::generator::NameGenerator;
use crate::mechanics::npc_generator::provider::JsonNameProvider;
use crate::mechanics::npc_generator::types::{Gender, GenerateNpcParams, Race};

pub struct NpcGenCommand;

impl Commands for NpcGenCommand {
    const NAME: &'static str = "npc_generator";
    const DESCRIPTION: &'static str = "generate new NPC details";

    fn builder() -> CreateCommand {
        CreateCommand::new(NpcGenCommand::NAME)
            .description(NpcGenCommand::DESCRIPTION)
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "gender",
                    "Gender of npc (random by default)")
                    .add_string_choice("Male", "male")
                    .add_string_choice("Female", "female")
                    .add_string_choice("Neutral", "neutral")
                    .required(false)
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "race",
                    "Race of npc (random by default)")
                    .add_string_choice("Human", "human")
                    .add_string_choice("Elf", "elf")
                    .add_string_choice("Half Animal", "half_animal")
                    .add_string_choice("Dwarf", "dwarf")
                    .add_string_choice("Demon", "demon")
                    .add_string_choice("Angel", "angel")
                    .add_string_choice("Voidborn", "voidborn")
                    .required(false)
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Number,
                    "trait_amount",
                "Number of traits")
                    .min_int_value(1)
                    .max_int_value(15)
            )
    }

    async fn execute(_: &Context, interaction: Interaction) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();

        let race = match get_string_option(&command.data.options, "race") {
            Some(race) => Race::from_string(&*race),
            None => None
        };
        let gender = match get_string_option(&command.data.options, "gender") {
            Some(gender) => Gender::from_string(&*gender),
            None => None,
        };
        let trait_amount = match get_integer_option(&command.data.options, "trait_amount") {
            Some(trait_amount) => Some(trait_amount as usize),
            None => None
        };


        let provider = JsonNameProvider::new();
        let mut generator = NameGenerator::new(provider);
        let npc = generator.generate_npc(GenerateNpcParams{ race, gender, trait_amount });


        let mut traits = vec![];
        for trt in &npc.traits {
            traits.push((trt.name(), trt.description(), true));
        }

        let embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::new(format!("{} {}", npc.name.first, npc.name.last)))
            .fields(traits)
            .title(format!("{} {}", npc.name.gender.as_str(), npc.name.race.as_str()));

        Ok(CreateInteractionResponseMessage::new().embed(embed))
    }
}