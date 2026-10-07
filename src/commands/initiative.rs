use std::collections::HashMap;
use std::ops::Index;
use env_logger::init;
use serenity::all::{Color, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponseMessage, Interaction, User, UserId};
use crate::commands::Commands;
use crate::helpers::errors::{ErrorContext};
use crate::helpers::interactions::{get_boolean_option, get_integer_option, get_string_option, get_subcommand_option, get_user_option};
use crate::InitiativeTracker;
use crate::mechanics::initiative_tracker::types::{Entity, EntityState, Initiative};

const BEFORE_ENTITY: char = '🔴';
const CURRENT_ENTITY: char = '🟢';
const AFTER_ENTITY: char = '🔵';

pub struct InitiativeCommand;

impl Commands for InitiativeCommand {
    const NAME: &'static str = "initiative";
    const DESCRIPTION: &'static str = "Initiative Tracker";

    fn builder() -> CreateCommand {
        CreateCommand::new(InitiativeCommand::NAME)
            .description(InitiativeCommand::DESCRIPTION)
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "clear",
                "Clears the current tracker",
            ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "get",
                "Gets the current tracker",
            )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::User,
                        "user",
                        "User's current tracker",
                    )
                ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Adds new entity to the tracker",
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "name",
                    "Name of the entity",
                ).required(true))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "hp",
                    "Hit points of the entity",
                ).required(true))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "initiative_value",
                    "Initiative value of the entity",
                ).required(true))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "state",
                    "State of the entity (e.g., alive, dead, unconscious)",
                )
                    .add_string_choice("Healthy", "healthy")
                    .add_string_choice("Injured", "injured")
                    .add_string_choice("Critical", "critical")
                    .add_string_choice("Dead", "dead")
                    .add_string_choice("Immune", "immune")
                    .add_string_choice("Stunned", "stunned")
                    .add_string_choice("Alerted", "alerted")
                    .add_string_choice("Escaped", "escaped")
                ))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "jump",
                "Jump to the next entity (smart jump)",
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Boolean,
                    "smart",
                    "Enable smart jump (auto-skip dead/unconscious entities. Default true)",
                )))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "damage",
                "Deals damage to an entity",
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "value",
                    "Amount of damage to deal",
                ).required(true))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "identifier",
                    "Name or ID of the entity",
                ).required(true)))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "heal",
                "Heals an entity",
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "value",
                    "Amount of health to restore",
                ).required(true))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "identifier",
                    "Name or ID of the entity",
                ).required(true))
                .add_sub_option(CreateCommandOption::new(
                   CommandOptionType::Boolean,
                   "overheal",
                   "If healing can go outside entity max HP (default false)"
                )))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "remove",
                "Removes an entity from the tracker",
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "identifier",
                    "Name or ID of the entity",
                ).required(true)))
            .add_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "effect",
                "Adds or removes an effect from an entity",
            )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "effect_name",
                    "Name of the effect",
                ).required(true))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "action",
                    "Action to perform (add or remove)",
                )
                    .add_string_choice("Add", "add")
                    .add_string_choice("Remove", "remove")
                    .required(true))
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    "identifier",
                    "Name or ID of the entity",
                ).required(true)))
    }

    async fn execute(ctx: &Context, interaction: Interaction) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();
        let (name, params) = get_subcommand_option(&command.data.options);

        let user = command.user;
        let mut data = ctx.data.write().await;
        let mut initiative_trackers = data.get_mut::<InitiativeTracker>().expect("Tracker not found");

        println!("{:?}", params);

        let embed = match name.as_str() {
            "add" => {
                let name = get_string_option(&params, "name").unwrap();
                let hp = get_integer_option(&params, "hp").unwrap();
                let initiative_value = get_integer_option(&params, "initiative_value").unwrap();
                let state = match get_string_option(&params, "state") {
                    Some(s) => EntityState::from_string(s),
                    None => None
                };

                add_execute(
                    user,
                    &mut initiative_trackers,
                    name, hp, initiative_value, state,
                )
            },
            "clear" => {
                clear_execute(user, &mut initiative_trackers)
            }
            "remove" => {
                let identifier = get_string_option(&params, "identifier").unwrap();
                remove_execute(user, &mut initiative_trackers, identifier)
            }
            "get" => {
                let user = {
                    get_user_option(&params, "user").unwrap_or_else(|| user.id).to_user(ctx).await.unwrap()
                };
                get_execute(user, &mut initiative_trackers)
            }
            "jump" => {
                let smart = get_boolean_option(&params, "smart").unwrap_or_else(|| true);
                jump_execute(
                    user,
                    initiative_trackers,
                    smart
                )
            }
            "damage" => {
                let identifier = get_string_option(&params, "identifier").unwrap();

                let value = get_integer_option(&params, "value").unwrap();
                damage_execute(user, initiative_trackers, identifier, value)
            }
            "heal" => {
                let identifier = get_string_option(&params, "identifier").unwrap();
                let value = get_integer_option(&params, "value").unwrap();
                let overheal = get_boolean_option(&params, "overheal").unwrap_or_else(|| false);

                heal_execute(user, initiative_trackers, identifier, value, overheal)
            }
            "effect" => {
                let identifier = get_string_option(&params, "identifier").unwrap();
                let effect_name = get_string_option(&params, "effect_name").unwrap();
                let action = get_string_option(&params, "action").unwrap();
                effect_execute(user, initiative_trackers, identifier, effect_name, action)
            }
            _ => unreachable!()
        }?;
        
        Ok(CreateInteractionResponseMessage::new().embed(embed))
    }
}

pub fn add_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
    name: String,
    hp: i64,
    initiative_value: i64,
    state: Option<EntityState>,
) -> Result<CreateEmbed, ErrorContext> {
    let tracker = tracker.entry(user.id).or_insert_with(Initiative::default);

    let state = state.unwrap_or_else(|| EntityState::Healthy);

    if tracker.entities.len() > 20 {
        return Ok(CreateEmbed::new()
            .title("Tracker is full")
            .description("Maximum length of single tracker is 20")
            .color(Color::RED))
    }

    let entity = Entity::new(
        name, hp as i128, initiative_value, state
    );

    let given_id = tracker.add_entity(entity.clone());

    Ok(CreateEmbed::new()
        .title("Created Entity")
        .description(format!("Entity `{}` added", entity.name))
        .field(format!("ID: {}", given_id), "", false))
}

pub fn clear_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
) -> Result<CreateEmbed, ErrorContext> {
    tracker.remove(&user.id);

    Ok(CreateEmbed::new()
    .title("Cleared Tracker")
        .color(Color::RED))
}

pub fn remove_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
    identifier: String,
) -> Result<CreateEmbed, ErrorContext> {
    let tracker_opt = tracker.get_mut(&user.id);
    let tracker = {
        if let Some(tracker) = tracker_opt {
            tracker
        } else {
            return Ok(CreateEmbed::new()
                .title("No tracker found")
                .description("You have no tracker"))
        }
    };

    let entity = tracker.find_entity_index(identifier.clone());

    if let Some(index) = entity {
        let entity = tracker.entities.get(index).unwrap().clone();
        tracker.entities.remove(index);

        Ok(CreateEmbed::new()
            .title("Entity Removed")
            .description(format!("Entity {} removed", entity.name)))

    } else {
        Ok(CreateEmbed::new()
            .title("No entity found")
            .description(format!("No entity found with identifier {}", identifier)))
    }
}

pub fn get_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
) -> Result<CreateEmbed, ErrorContext> {
    let tracker = tracker.entry(user.id).or_insert_with(Initiative::default);

    let mut embed = CreateEmbed::new()
        .title(format!("{}'s Tracker", user.name));

    let mut fields: Vec<(String, String, bool)> = Vec::with_capacity(20);
    let mut gds = tracker.get_entity_action_bar();

    // before
    for entity in gds.0 {
        fields.push((
            format!("[{}] {}", entity.id, entity.name),
            format!("{}\n Init: {}\n\
            HP: [{}/{}] ({})\n\
            {}",
                BEFORE_ENTITY,
                    entity.initiative_value, entity.current_health_points,
                    entity.health_points, entity.state.to_string(),
                    {
                        if entity.effects.is_empty() {
                            "No effects".to_string()
                        } else {
                            format!("**Effects:** {}", entity.effects.join(", "))
                        }
                    }

            ),
            true))
    }

    {
        let entity = gds.1;

        fields.push((
            format!("[{}] {}", entity.id, entity.name),
            format!("{}\nInit: {}\n\
            HP: [{}/{}] ({})\n\
            {}",
                    CURRENT_ENTITY,
                    entity.initiative_value, entity.current_health_points,
                    entity.health_points, entity.state.to_string(),
                    {
                        if entity.effects.is_empty() {
                            "No effects".to_string()
                        } else {
                            format!("**Effects:** {}", entity.effects.join(", "))
                        }
                    }

            ),
            true))
    }

    for entity in gds.2 {
        fields.push((
            format!("[{}] {}", entity.id, entity.name),
            format!("{}\nInit: {}\n\
            HP: [{}/{}] ({})\n\
            {}",
                AFTER_ENTITY,
                    entity.initiative_value, entity.current_health_points,
                    entity.health_points, entity.state.to_string(),
                    {
                        if entity.effects.is_empty() {
                            "No effects".to_string()
                        } else {
                            format!("**Effects:** {}", entity.effects.join(", "))
                        }
                    }

            ),
            true))
    }

    embed = embed.fields(fields);

    Ok(embed)
}

pub fn jump_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
    smart: bool
) -> Result<CreateEmbed, ErrorContext> {
    let mut tracker = tracker.entry(user.id).or_insert_with(Initiative::default);


    if tracker.entities.len() < 2 {
        return Ok(CreateEmbed::new()
            .title("Cannot jump")
            .description("You don't have enough entities to preform a jump"))
    }


    if smart {
        tracker.smart_next();
    } else {
        tracker.next();
    }

    let entity = tracker.get_current_entity();

    Ok(CreateEmbed::new()
        .title(format!("Jump to {}", entity.name))
        .description(format!("Round: {}", tracker.round)))
}

pub fn damage_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
    identifier: String,
    value: i64,
) -> Result<CreateEmbed, ErrorContext> {
    let tracker = tracker.entry(user.id).or_insert_with(Initiative::default);

    let entity = tracker.find_entity_mut(identifier.clone());

    if entity.is_none() {
        return Ok(CreateEmbed::new()
            .title("Cannot find entity")
            .description(format!("Entity {} not found", identifier)))
    }

    let entity = entity.unwrap();

    entity.current_health_points = entity.current_health_points - value as i128;

    Ok(CreateEmbed::new()
        .title(format!("{} damaged", entity.name))
        .field(
            format!("ID: {}", entity.id),
            format!("State: {}", entity.state.to_string()),
            true)
        .field(
            format!("Damage Taken: {}", value),
            format!("HP: {}/{}", entity.current_health_points, entity.health_points),
            false)
        .color(Color::RED))
}

pub fn heal_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
    identifier: String,
    value: i64,
    overflow: bool,
) -> Result<CreateEmbed, ErrorContext> {
    let tracker = tracker.entry(user.id).or_insert_with(Initiative::default);

    let entity = tracker.find_entity_mut(identifier.clone());

    if entity.is_none() {
        return Ok(CreateEmbed::new()
            .title("Cannot find entity")
            .description(format!("Entity {} not found", identifier)))
    }

    let entity = entity.unwrap();

    entity.current_health_points = {
        if overflow {
            entity.current_health_points + value as i128
        } else {
          std::cmp::min(entity.current_health_points + value as i128, entity.health_points)
        }
    };

    Ok(CreateEmbed::new()
        .title(format!("{} healed", entity.name))
        .field(
            format!("ID: {}", entity.id),
            format!("State: {}", entity.state.to_string()),
            true)
        .field(
            format!("Damage Taken: {}", value),
            format!("HP: {}/{}", entity.current_health_points, entity.health_points),
            false)
        .color(Color::RED))
}

pub fn effect_execute(
    user: User,
    tracker: &mut Box<HashMap<UserId, Initiative>>,
    identifier: String,
    effect_name: String,
    action: String,
) -> Result<CreateEmbed, ErrorContext> {
    let tracker = tracker.entry(user.id).or_insert_with(Initiative::default);

    let entity = tracker.find_entity_mut(identifier.clone());

    if entity.is_none() {
        return Ok(CreateEmbed::new()
            .title("Cannot find entity")
            .description(format!("Entity {} not found", identifier)))
    }

    let entity = entity.unwrap();

    let mut embed = CreateEmbed::new();

    if action == "add" {
        embed = embed.title("Effect added")
            .field("Effect name", effect_name.clone(), true);


        entity.effects.push(effect_name);
    } else {


        if let Some(index) = entity.effects.iter().position(|value| *value == effect_name) {
            entity.effects.swap_remove(index);

            return Ok(embed)
        }
    }

    Ok(embed)
}