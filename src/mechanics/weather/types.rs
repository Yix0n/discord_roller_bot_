
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct Weather {
    pub emote: char,
    pub name: String,
    pub temperature: f32,
    pub feels_like: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub cloud_coverage: f32,
    pub wind_speed: f32,
    pub wind_direction: Option<f32>,
    pub radio_disruption: f32,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct WeatherBuilder {
    pub name: String,
    pub emote: char,
    // min-max
    pub temperature: Range<f32>,
    // min-max
    pub feels_like_difference: Range<f32>,
    // Season, Chance (higher value = more common)
    pub seasons: Vec<(Season, f32)>,
    // wilgotność
    pub humidity: Range<f32>,
    // ciśnienie
    pub pressure: Range<f32>,
    // pokrycie chmur
    pub cloud_coverage: Range<f32>,
    // min-max
    pub wind_speed: Range<f32>,
    // R scale. 0-5
    pub radio_disruption: Range<f32>,
}

impl WeatherBuilder {
    pub fn n(emote: char, name: impl Into<String>) -> Self {
        Self {
            emote,
            name: name.into(),
            temperature: Range::default(),
            feels_like_difference: Range::default(),
            seasons: Vec::new(),
            humidity: Default::default(),
            pressure: Default::default(),
            cloud_coverage: Default::default(),
            wind_speed: Default::default(),
            radio_disruption: Default::default(),
        }
    }

    pub fn new(
        name: impl Into<String>,
        emote: char,
        temperature: Range<f32>,
        feels_like_difference: Range<f32>,
        seasons: Vec<(Season, f32)>,
        humidity: Range<f32>,
        pressure: Range<f32>,
        cloud_coverage: Range<f32>,
        wind_speed: Range<f32>,
        radio_disruption: Range<f32>,
    ) -> Self {
        WeatherBuilder {
            name: name.into(),
            emote, temperature, feels_like_difference,
            seasons, humidity, pressure, cloud_coverage,
            wind_speed, radio_disruption, 
        }
    }

    pub fn emote(mut self, emote: char) -> Self {
        self.emote = emote;
        self
    }

    pub fn temperature(&mut self, value: Range<f32>) -> &mut Self {
        self.temperature = value;
        self
    }

    pub fn temperature_feels_difference(&mut self, value: Range<f32>) -> &mut Self {
        self.feels_like_difference = value;
        self
    }

    pub fn add_season(&mut self, season: Season, chance: f32) -> &mut Self {
        self.seasons.push((season, chance));
        self
    }

    pub fn humidity(&mut self, value: Range<f32>) -> &mut Self {
        self.humidity = value;
        self
    }

    pub fn pressure(&mut self, value: Range<f32>) -> &mut Self {
        self.pressure = value;
        self
    }

    pub fn cloud_coverage(&mut self, value: Range<f32>) -> &mut Self {
        self.cloud_coverage = value;
        self
    }

    pub fn wind_speed(&mut self, value: Range<f32>) -> &mut Self {
        self.wind_speed = value;
        self
    }

    pub fn radio_disruption(&mut self, value: Range<f32>) -> &mut Self {
        self.radio_disruption = value;
        self
    }

    pub fn build(self) -> Weather {
        let temperature: f32 = rand::random_range(self.temperature.min..self.temperature.max);
        let feels_like_difference: f32 = rand::random_range(self.feels_like_difference.min..self.feels_like_difference.max);
        let humidity: f32 = rand::random_range(self.humidity.min..self.humidity.max);
        let pressure: f32 = rand::random_range(self.pressure.min..self.pressure.max);
        let cloud_coverage: f32 = rand::random_range(self.cloud_coverage.min..self.cloud_coverage.max);
        let wind_speed: f32 = rand::random_range(self.wind_speed.min..self.wind_speed.max);
        let wind_direction: Option<f32> = {
          if wind_speed == 0.0 {
              None
          } else {
              Some(rand::random_range(0.0..=360.0))
          }
        };
        let radio_disruption: f32 = rand::random_range(self.radio_disruption.min..self.radio_disruption.max);


        Weather {
            name: self.name,
            emote: self.emote,
            temperature,
            feels_like: temperature + feels_like_difference,
            humidity,
            pressure,
            cloud_coverage,
            wind_speed,
            wind_direction,
            radio_disruption,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct SimpleHour {
    pub hours: u8,
    pub minutes: u8,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

impl<T: Into<f64>> Range<T> {
    pub fn n(min: T, max: T) -> Self {
        Self { min, max }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Clone, PartialEq, Eq)]
pub enum Season {
    Winter,
    Spring,
    Summer,
    Autumn,
}

impl Season {
    pub fn from_month(month: u8) -> Self {
        match month {
            12 | 1 | 2 => Season::Winter,
            3 | 4 | 5 => Season::Spring,
            6 | 7 | 8 => Season::Summer,
            9 | 10 | 11 => Season::Autumn,
            _ => Season::Winter
        }
    }
}