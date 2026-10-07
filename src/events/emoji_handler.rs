use rusqlite::fallible_iterator::FallibleIterator;
use serenity::all::{Context, Reaction};
use crate::CasterTracker;

pub async fn emoji_handler(ctx: Context, reaction: Reaction) {
    if reaction.user(&ctx).await.unwrap().bot {
        return;
    }

    let message = reaction.message(&ctx).await.unwrap();

    if reaction.user_id.unwrap() != message.author.id {
        return;
    }

    let data = ctx.data.write().await;
    let tracker_map = data.get::<CasterTracker>().unwrap();

    if let Some(tracker) = tracker_map.get(&message.id) {

    }
}