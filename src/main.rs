extern crate core;

use std::collections::HashMap;
use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use serenity::all::{CommandId, CreateInteractionResponseMessage, CreateSelectMenuKind, CreateSelectMenuOption, EmojiId, Guild, Interaction, InteractionType, Message, MessageId, Reaction, ReactionType, UserId};
use serenity::model::{
    application::{Command},
    gateway::{Ready},
};
use serenity::{async_trait};
use serenity::builder::{CreateActionRow, CreateButton, CreateInteractionResponse, CreateMessage, CreateSelectMenu};
use serenity::prelude::*;
use tokio::task;
use crate::commands::{execute, get_builders};
// use crate::database::models::Model;
use crate::events::activity_handler::activity_handler;
use dotenv::dotenv;
use std::env;
use crate::database::ConnectionPool;
use crate::database::models::character::Character;
use crate::database::models::deck::Deck;
use crate::database::models::Model;
use crate::database::models::skill::Skill;
use crate::database::models::collection::Collection;
use crate::database::models::hand_card::HandCard;
use crate::events::component_handler::handle_component_interaction;
use crate::events::server_whitelist::is_server_whitelisted;
use crate::mechanics::initiative_tracker::types::Initiative;

pub mod commands;
pub mod helpers;
pub mod events;
pub mod types;
mod database;
pub mod mechanics;

#[async_trait]
impl EventHandler for Bot {
    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: Option<bool>) {
        if !is_server_whitelisted(&guild)  {
            guild.leave(&ctx).await;
        }
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        if new_message.author.bot {
            if rand::random_range(0..=100) == 67 {
                let reaction: ReactionType = ReactionType::Unicode("🫃".to_string());

                if let Err(why) = new_message.react(ctx, reaction).await {
                    eprintln!("Error sending reaction: {:?}", why);
                }
            }

            return;
        }

        let content = &new_message.content;

        if content.to_lowercase().contains("sigma") {
            let reaction: ReactionType = ReactionType::Custom {
                animated: false,
                id: EmojiId::new(1494342250648436937),
                name: Some("sigma_face".to_string()),
            };

            if let Err(why) = new_message.react(&ctx, reaction).await {
                eprintln!("Error while reacting message: {}", why);
            }

            return;
        }

        if content.to_lowercase() == "co" || content.to_lowercase() == "co?" {
            if let Err(why) = new_message.reply_mention(&ctx, "chujów Sto").await {
                eprintln!("Error while replying message: {}", why);
            }

            return;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is Ready!", ready.user.name);

        if true {
            ctx.http.get_global_commands().await.unwrap().iter().for_each(|command| println!("COMMAND: {} - {}", command.name, command.id));
        }

        let command_to_delete: Vec<u64> = vec![];

        for cmd in command_to_delete {
            ctx.http.delete_global_command(CommandId::new(cmd)).await.unwrap();
        }

        for builder in get_builders() {
            if let Err(why) = Command::create_global_command(&ctx.http, builder).await {
                println!("Error when creating command: {:?}", why);
            }
        }

        let arc_ctx = Arc::new(ctx);
        activity_handler(arc_ctx.clone());
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction.kind() {
            InteractionType::Command => {
                let command = interaction.clone().command().unwrap();

                // command.defer(&ctx).await.expect("Failed to defer");

                task::spawn(async move {
                    let response = execute(&command.data.name, &interaction, &ctx).await;

                    let response = response.unwrap_or_else(|e| {
                        CreateInteractionResponseMessage::new().embed(e.to_embed())
                    });

                    let response = CreateInteractionResponse::Message(response);
                    command.create_response(&ctx, response).await.expect("Failed to send message");
                });
            },
            InteractionType::Component => {
                let component = interaction.clone().message_component().unwrap();

                task::spawn(async move {
                    handle_component_interaction(&ctx, &interaction, &component).await
                });
            }
            _ => {}
        }
    }
}

struct Bot;

pub struct InitiativeTracker;
impl TypeMapKey for InitiativeTracker {
    type Value = Box<HashMap<UserId, Initiative>>;
}

pub struct CommandLimiterTracker;
impl TypeMapKey for CommandLimiterTracker {
    type Value = Box<HashMap<UserId, usize>>;
}

pub struct CasterTracker;
impl TypeMapKey for CasterTracker {
    type Value = Box<HashMap<MessageId, CasterTracker>>;
}

pub struct ConnectionPoolProvider;
impl TypeMapKey for ConnectionPoolProvider {
    type Value = Arc<ConnectionPool>;
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    dotenv().ok();
    let intents =
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::GUILDS |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let manager = SqliteConnectionManager::file(env::var("SQLITE_FILE")?.as_str());
    let pool = r2d2::Pool::builder().build(manager)?;

    let initiative_data = Box::new(HashMap::new());
    let command_limiter_tracker = Box::new(HashMap::new());

    {
        let pool_clone = pool.clone();
        task::spawn_blocking(async move || {
            Character::init(&pool_clone);
            Deck::init(&pool_clone);
            Skill::init(&pool_clone);
            Collection::init(&pool_clone);
            HandCard::init(&pool_clone);
        }).await?.await;
    }

    let bot_token = env::var("BOT_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(bot_token, intents)
        .event_handler(Bot)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<InitiativeTracker>(initiative_data);
        data.insert::<CommandLimiterTracker>(command_limiter_tracker);
        data.insert::<CasterTracker>(Box::new(HashMap::new()));
        data.insert::<ConnectionPoolProvider>(Arc::new(pool));
    }

    if let Err(why) = client.start().await {
        eprintln!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}