use std::sync::Arc;
use std::time::Duration;
use rand::Rng;
use serenity::all::ActivityData;
use serenity::client::Context;
use tokio::time::interval;

pub fn activity_handler(ctx: Arc<Context>) {

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_mins(5));

        let activities = vec![
            ActivityData::custom("Wale Konia (swojego)"),
            ActivityData::playing("Poker with other constellations"),
            ActivityData::listening("Brainrot"),
            ActivityData::watching("Skibidi Toilet"),
            ActivityData::competing("Against John Discord"),
            ActivityData::playing("on Ban's nerves"),
            ActivityData::playing("Sex dungeon prezydenta Nawrockiego")
        ];

        loop {
            interval.tick().await;

            let new_activity = activities[rand::rng().random_range(0..activities.len())].clone();

            ctx.set_activity(Some(new_activity));
        }
    });
}