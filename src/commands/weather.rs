use serenity::all::{Color, CommandOptionType, Context, CreateCommand, CreateEmbed, CreateInteractionResponseMessage, Interaction};
use serenity::builder::{CreateCommandOption, CreateEmbedAuthor};
use crate::commands::Commands;
use crate::helpers::errors::ErrorContext;
use crate::helpers::interactions::get_string_option;
use crate::mechanics::weather::generator::generate_weather;
use crate::mechanics::weather::types::Season;

pub struct WeatherCommand;

impl Commands for WeatherCommand {
    const NAME: &'static str = "weather";
    const DESCRIPTION: &'static str = "Generate Weather";

    fn builder() -> CreateCommand {
        CreateCommand::new(WeatherCommand::NAME)
            .description(WeatherCommand::DESCRIPTION)
            .add_option(CreateCommandOption::new(
                CommandOptionType::String,
                "season",
                "Season.. what else to say?"
            )
                .add_string_choice("Summer", "summer")
                .add_string_choice("Sping", "spring")
                .add_string_choice("Winter", "winter")
                .add_string_choice("Autumn", "autumn")
            )
    }

    async fn execute(_ctx: &Context, interaction: Interaction) -> Result<CreateInteractionResponseMessage, ErrorContext> {
        let command = interaction.command().unwrap();
        let season_str = get_string_option(&command.data.options, "season").unwrap_or("any".to_string());

        let season = match season_str.as_str() {
            "summer" => Some(Season::Summer),
            "spring" => Some(Season::Spring),
            "winter" => Some(Season::Winter),
            "autumn" => Some(Season::Autumn),
            _ => None
        };

        let weather = generate_weather(season);

        let embed = 
            CreateEmbed::new()
                .title(format!("{} {}", weather.emote, weather.name))
                .color(Color::BLURPLE)
                .author(CreateEmbedAuthor::new(
                    format!("Radio Disruption: {} R", weather.radio_disruption)
                ))
                .field(
                    format!("Temperature: {:.1} °C", weather.temperature),
                    format!("Feels Like: {:.1} °C", weather.feels_like),
                    true
                )
                .field(
                    format!("Pressure: {:.2} hPa", weather.pressure),
                    "​",
                    true
                )
                .field(
                    format!("Wind Speed: {:.2} km/h", weather.wind_speed),
                    {
                        if weather.wind_direction.is_none() {
                            "None".to_string()
                        } else {
                            let result = get_direction(weather.wind_direction.unwrap());
                            format!("{} ({})", result.0, result.1)
                        }
                    },
                    true
                )
                .field(
                    format!("Humidity: {:.1}%", weather.humidity),
                    format!("Cloud Coverage: {:.1}%", weather.cloud_coverage),
                    true
                );

        Ok(CreateInteractionResponseMessage::new().embed(embed))
    }
}

pub fn get_direction(direction: f32) -> (&'static str, &'static str) {
    let directions = [
        (348.75, "N", "North"),
        (11.25, "NNE", "North-Northeast"),
        (33.75, "NE", "Northeast"),
        (56.25, "ENE", "East-Northeast"),
        (78.75, "E", "East"),
        (101.25, "ESE", "East-Southeast"),
        (123.75, "SE", "Southeast"),
        (146.25, "SSE", "South-Southeast"),
        (168.75, "S", "South"),
        (191.25, "SSW", "South-Southwest"),
        (213.75, "SW", "Southwest"),
        (236.25, "WSW", "West-Southwest"),
        (258.75, "W", "West"),
        (281.25, "WNW", "West-Northwest"),
        (303.75, "NW", "Northwest"),
        (326.25, "NNW", "North-Northwest"),
    ];

    let mut result = ( "N", "North" );

    for i in 0..directions.len() {
        let (range, short, name) = directions[i];

        if i == 0 {
            if direction >= range || direction < directions[1].0 {
                result = (short, name);
                break;
            }
        } else if i == directions.len() - 1 {
            if direction >= range {
                result = (short, name);
                break;
            }
        } else {
            if direction >= range && direction < directions[i + 1].0 {
                result = (short, name);
                break;
            }
        }
    }

    result
}