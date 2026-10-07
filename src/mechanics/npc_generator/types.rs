use rand::random_range;
use crate::mechanics::npc_generator::data::traits::Traits;

pub struct GeneratedName {
    pub first: String,
    pub last: String,
    pub race: Race,
    pub gender: Gender,
}

pub struct NpcGenerator {
    pub name: GeneratedName,
    pub traits: Vec<Traits>,
}

#[derive(Eq, PartialEq)]
#[derive(Hash, Clone)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    HalfAnimal,
    Demon,
    Angel,
    Voidborn,
}

impl Race {
    pub fn as_str(&self) -> &'static str {
        match self {
            Race::Human => "human",
            Race::Elf => "elf",
            Race::Dwarf => "dwarf",
            Race::HalfAnimal => "half_animal",
            Race::Demon => "demon",
            Race::Angel => "angel",
            Race::Voidborn => "voidborn",
        }
    }

    pub fn from_string(string: &str) -> Option<Race> {
        match string {
            "human" => Some(Race::Human),
            "elf" => Some(Race::Elf),
            "dwarf" => Some(Race::Dwarf),
            "half_animal" => Some(Race::HalfAnimal),
            "demon" => Some(Race::Demon),
            "angel" => Some(Race::Angel),
            "voidborn" => Some(Race::Voidborn),
            _ => None,
        }
    }

    pub fn random() -> Self {
        let range = random_range(1..=7);

        match range {
            1 => Race::Human,
            2 => Race::Elf,
            3 => Race::Dwarf,
            4 => Race::HalfAnimal,
            5 => Race::Demon,
            6 => Race::Angel,
            7 => Race::Voidborn,
            _ => unreachable!(),
        }
    }
}

#[derive(Eq, PartialEq)]
#[derive(Hash, Clone)]
pub enum Gender {
    Male,
    Female,
    Neutral,
}
impl Gender {
    pub fn as_str(&self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Neutral => "neutral",
        }
    }

    pub fn from_string(string: &str) -> Option<Gender> {
        match string {
            "male" => Some(Gender::Male),
            "female" => Some(Gender::Female),
            "neutral" => Some(Gender::Neutral),
            _ => unreachable!(),
        }
    }

    pub fn random() -> Self {

        let range = random_range(1..=3);
        match range {
            1 => Gender::Male,
            2 => Gender::Female,
            3 => Gender::Neutral,
            _ => Gender::Male,
        }
    }
}

#[derive(Default)]
pub struct GenerateNpcParams {
    pub race: Option<Race>,
    pub gender: Option<Gender>,
    pub trait_amount: Option<usize>,
}

#[derive(Clone)]
pub struct NameSet {
    pub first_names: Vec<String>,
    pub last_names: Vec<String>,
}

impl NameSet {
    pub fn new(first_names: Vec<String>, last_names: Vec<String>) -> Self {
        Self {
            first_names,
            last_names,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.first_names.is_empty() && self.last_names.is_empty()
    }
}