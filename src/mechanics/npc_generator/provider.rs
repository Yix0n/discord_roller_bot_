use std::collections::HashMap;
use serde::Deserialize;
use crate::mechanics::npc_generator::types::{Gender, NameSet, Race};

#[derive(Default, Deserialize)]
struct FirstNames {
    male: Vec<String>,
    female: Vec<String>,
    neutral: Vec<String>,
}

#[derive(Default, Deserialize)]
struct RaceData {
    first: FirstNames,
    last: Vec<String>
}
pub trait NameProvider: Send + Sync {
    fn get_names(&self, race: Race, gender: Gender) -> Option<&NameSet>;
    fn get_all_names(&self) -> &HashMap<(Race,Gender), NameSet>;
}

pub struct JsonNameProvider {
    names: HashMap<(Race, Gender), NameSet>,
}

impl JsonNameProvider {
    pub fn new() -> Self {
        let mut names = HashMap::new();

        let json_files = [
            (Race::Human, include_str!("data/json/human.json")),
            (Race::Elf, include_str!("data/json/elf.json")),
            (Race::Dwarf, include_str!("data/json/dwarf.json")),
            (Race::HalfAnimal, include_str!("data/json/half_animal.json")),
            (Race::Demon, include_str!("data/json/demon.json")),
            (Race::Angel, include_str!("data/json/angel.json")),
            (Race::Voidborn, include_str!("data/json/voidborn.json")),
        ];

        for (race, json_str) in json_files {
            if let Ok(race_data) = serde_json::from_str::<RaceData>(json_str) {
                let last_names = race_data.last;
                names.insert(
                    (race.clone(), Gender::Male),
                    NameSet::new(race_data.first.male.clone(), last_names.clone()),
                );

                names.insert(
                    (race.clone(), Gender::Female),
                    NameSet::new(race_data.first.female.clone(), last_names.clone()),
                );

                names.insert(
                    (race, Gender::Neutral),
                    NameSet::new(race_data.first.neutral.clone(), last_names.clone()),
                );
            }
        }
        
        Self { names }
    }
}

impl NameProvider for JsonNameProvider {
    fn get_names(&self, race: Race, gender: Gender) -> Option<&NameSet> {
        self.names.get(&(race, gender))
    }
    
    fn get_all_names(&self) -> &HashMap<(Race, Gender), NameSet> {
        &self.names
    }
}