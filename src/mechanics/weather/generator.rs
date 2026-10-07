use std::ops::Deref;
use crate::mechanics::weather::types::{Range, Season, Weather, WeatherBuilder};
use std::sync::OnceLock;
use weighted_rand::builder::{NewBuilder, WalkerTableBuilder};

static BUILDERS: OnceLock<Vec<Box<WeatherBuilder>>> = OnceLock::new();

fn init() -> Vec<Box<WeatherBuilder>>  {
    let builders = vec![
        // Heat wave - fala upałów
        Box::new(WeatherBuilder::new(
            "Heat wave",
            '🔥',
            Range::n(38.0, 45.0),
            Range::n(5.0, 15.0),
            vec![(Season::Summer, 0.5)],
            Range::n(0.0, 3.0),
            Range::n(1005.0, 1200.0),
            Range::n(0.0, 5.0),
            Range::n(0.0, 3.0),
            Range::n(0.0, 0.5),
        )),
        // Very Hot Sunny - ekstremalnie gorący słoneczny dzień
        Box::new(WeatherBuilder::new(
            "Very hot sunny",
            '☀',
            Range::n(24.0, 39.9),
            Range::n(2.0, 10.0),
            vec![(Season::Summer, 2.0)],
            Range::n(0.0, 5.0),
            Range::n(1000.0, 1200.0),
            Range::n(0.0, 5.0),
            Range::n(0.0, 5.0),
            Range::n(0.0, 1.0),
        )),
        // Mild Sunny - przyjemny słoneczny dzień
        Box::new(WeatherBuilder::new(
            "Mild Sunny",
            '☀',
            Range::n(18.0, 24.0),
            Range::n(-2.0, 5.0),
            vec![(Season::Spring, 12.0), (Season::Summer, 14.0), (Season::Autumn, 8.0)],
            Range::n(0.0, 15.0),
            Range::n(990.0, 1200.0),
            Range::n(0.0, 15.0),
            Range::n(0.0, 10.0),
            Range::n(0.0, 1.0),
        )),
        // Heavy Rain - ulewny deszcz
        Box::new(WeatherBuilder::new(
            "Heavy Rain",
            '🌧',
            Range::n(12.0, 18.0),
            Range::n(-2.0, 4.0),
            vec![(Season::Spring, 4.0), (Season::Autumn, 5.0)],
            Range::n(80.0, 100.0),
            Range::n(980.0, 1150.0),
            Range::n(90.0, 100.0),
            Range::n(20.0, 60.0),
            Range::n(0.5, 3.0),
        )),
        // Drizzle - mżawka
        Box::new(WeatherBuilder::new(
            "Drizzle",
            '🌧',
            Range::n(5.0, 12.0),
            Range::n(-1.0, 2.0),
            vec![(Season::Spring, 8.0), (Season::Autumn, 9.0), (Season::Winter, 4.0)],
            Range::n(40.0, 70.0),
            Range::n(990.0, 1200.0),
            Range::n(50.0, 85.0),
            Range::n(1.0, 20.0),
            Range::n(0.1, 1.5),
        )),
        // Heat Lightning - burza bez deszczu (suche burze)
        Box::new(WeatherBuilder::new(
            "Heat Lightning",
            '⚡',
            Range::n(28.0, 38.0),
            Range::n(4.0, 10.0),
            vec![(Season::Summer, 0.8)],
            Range::n(10.0, 30.0),
            Range::n(990.0, 1150.0),
            Range::n(30.0, 60.0),
            Range::n(5.0, 20.0),
            Range::n(3.0, 5.0),
        )),
        // Breezy - przyjemny wietrzyk
        Box::new(WeatherBuilder::new(
            "Breezy",
            '🍃',
            Range::n(18.0, 26.0),
            Range::n(-1.0, 5.0),
            vec![(Season::Summer, 10.0), (Season::Spring, 9.0)],
            Range::n(10.0, 30.0),
            Range::n(995.0, 1200.0),
            Range::n(15.0, 45.0),
            Range::n(15.0, 35.0),
            Range::n(0.0, 0.8),
        )),
        // Downpour - ulewa
        Box::new(WeatherBuilder::new(
            "Downpour",
            '🌧',
            Range::n(14.0, 22.0),
            Range::n(0.0, 4.0),
            vec![(Season::Summer, 2.5), (Season::Spring, 2.0)],
            Range::n(90.0, 100.0),
            Range::n(985.0, 1150.0),
            Range::n(95.0, 100.0),
            Range::n(25.0, 70.0),
            Range::n(0.5, 3.0),
        )),
        // Blizzard - zamieć śnieżna
        Box::new(WeatherBuilder::new(
            "Blizzard",
            '❄',
            Range::n(-25.0, -10.0),
            Range::n(-10.0, -3.0),
            vec![(Season::Winter, 1.5)],
            Range::n(90.0, 100.0),
            Range::n(970.0, 1100.0),
            Range::n(90.0, 100.0),
            Range::n(30.0, 80.0),
            Range::n(1.0, 4.0),
        )),
        // Hail - grad
        Box::new(WeatherBuilder::new(
            "Hail",
            '🌨',
            Range::n(2.0, 10.0),
            Range::n(-3.0, 2.0),
            vec![(Season::Spring, 1.5), (Season::Summer, 1.0)],
            Range::n(60.0, 90.0),
            Range::n(985.0, 1120.0),
            Range::n(70.0, 100.0),
            Range::n(5.0, 45.0),
            Range::n(1.0, 4.0),
        )),
        // Light Snow - lekki śnieg
        Box::new(WeatherBuilder::new(
            "Light Snow",
            '❄',
            Range::n(-8.0, 0.0),
            Range::n(-5.0, 1.0),
            vec![(Season::Winter, 10.0), (Season::Spring, 1.5)],
            Range::n(60.0, 90.0),
            Range::n(990.0, 1200.0),
            Range::n(50.0, 85.0),
            Range::n(1.0, 10.0),
            Range::n(0.0, 2.0),
        )),
        // Fog - mgła
        Box::new(WeatherBuilder::new(
            "Fog",
            '🌫',
            Range::n(0.0, 8.0),
            Range::n(-3.0, 2.0),
            vec![(Season::Autumn, 7.0), (Season::Spring, 5.0), (Season::Winter, 6.0)],
            Range::n(85.0, 100.0),
            Range::n(1000.0, 1200.0),
            Range::n(80.0, 100.0),
            Range::n(0.0, 5.0),
            Range::n(0.1, 1.0),
        )),
        // Thunderstorm - burza
        Box::new(WeatherBuilder::new(
            "Thunderstorm",
            '⛈',
            Range::n(15.0, 28.0),
            Range::n(-1.0, 5.0),
            vec![(Season::Summer, 5.0), (Season::Spring, 3.5)],
            Range::n(70.0, 100.0),
            Range::n(985.0, 1150.0),
            Range::n(80.0, 100.0),
            Range::n(10.0, 70.0),
            Range::n(2.0, 5.0), // wysokie zakłócenia radiowe
        )),
        // Windy - wietrznie
        Box::new(WeatherBuilder::new(
            "Windy",
            '💨',
            Range::n(8.0, 16.0),
            Range::n(-4.0, 3.0),
            vec![(Season::Autumn, 8.0), (Season::Spring, 7.0)],
            Range::n(20.0, 60.0),
            Range::n(990.0, 1200.0),
            Range::n(30.0, 70.0),
            Range::n(40.0, 90.0),
            Range::n(0.2, 2.0),
        )),
        // Partly Cloudy - częściowe zachmurzenie
        Box::new(WeatherBuilder::new(
            "Partly Cloudy",
            '⛅',
            Range::n(15.0, 25.0),
            Range::n(-1.0, 6.0),
            vec![(Season::Spring, 13.0), (Season::Summer, 12.0), (Season::Autumn, 10.0)],
            Range::n(0.0, 20.0),
            Range::n(995.0, 1200.0),
            Range::n(25.0, 65.0),
            Range::n(0.0, 12.0),
            Range::n(0.0, 1.0),
        )),
        // Cloudy - pochmurno
        Box::new(WeatherBuilder::new(
            "Cloudy",
            '☁',
            Range::n(10.0, 18.0),
            Range::n(-2.0, 4.0),
            vec![(Season::Autumn, 11.0), (Season::Spring, 9.0), (Season::Winter, 8.0)],
            Range::n(30.0, 70.0),
            Range::n(995.0, 1200.0),
            Range::n(70.0, 100.0),
            Range::n(0.0, 15.0),
            Range::n(0.0, 1.5),
        )),
        // Aurora - zorza polarna (bez opadów, bardzo specyficzna)
        Box::new(WeatherBuilder::new(
            "Aurora",
            '🌌',
            Range::n(-35.0, -15.0),
            Range::n(-10.0, -3.0),
            vec![(Season::Winter, 0.1)],
            Range::n(0.0, 10.0),
            Range::n(1000.0, 1200.0),
            Range::n(0.0, 10.0),
            Range::n(0.0, 5.0),
            Range::n(0.0, 0.5),
        )),
        // Polar Night - noc polarna (bardzo ciemno, zimno)
        Box::new(WeatherBuilder::new(
            "Polar Night",
            '🌑',
            Range::n(-40.0, -25.0),
            Range::n(-12.0, -5.0),
            vec![(Season::Winter, 0.05)],
            Range::n(0.0, 5.0),
            Range::n(1000.0, 1200.0),
            Range::n(0.0, 10.0),
            Range::n(0.0, 3.0),
            Range::n(0.0, 0.5),
        )),
    ];

    builders
}


pub fn generate_weather(
    season: Option<Season>,
) -> Weather {
    let builder = select_weather(season);

    builder.build()
}

fn select_weather(season: Option<Season>) -> WeatherBuilder {
    let builders = BUILDERS.get_or_init(init);
    let mut eligible: Vec<&WeatherBuilder> = vec![];
    let mut weights: Vec<f32> = vec![];

    if let Some(season) = season {
        for builder in builders {
            let weather = builder.deref();
            for (s, weight) in &weather.seasons {
                if *s == season {
                    eligible.push(builder);
                    weights.push(*weight);
                    break;
                }
            }
        }
    } else {
        for builder in builders {
            let weather = builder.deref();
            let total_weight: f32 = weather.seasons.iter().map(|(_, w)| w).sum();
            if total_weight > 0.0 {
                eligible.push(builder);
                weights.push(total_weight);
            }
        }
    }

    let rand_builder = WalkerTableBuilder::new(&weights);
    let wa_table = rand_builder.build();
    let index = wa_table.next();

    (*eligible[index]).clone()
}