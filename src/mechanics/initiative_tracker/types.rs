use std::cmp::PartialEq;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Initiative {
    pub current_entity: usize,
    pub round: u32,
    pub entities: Vec<Entity>,
}

impl PartialEq<usize> for &Entity {
    fn eq(&self, other: &usize) -> bool {
        self == other
    }
}

impl Initiative {
    pub fn smart_next(&mut self) {
        let starting_index = self.current_entity;
        let total_entities = self.entities.len();

        if total_entities == 0 {
            return;
        }

        loop {
            self.current_entity = (self.current_entity + 1) % total_entities;

            if self.current_entity == starting_index {
                self.round += 1;
                break;
            }

            let entity = match self.entities.get(self.current_entity) {
                Some(e) => e,
                None => continue,
            };

            let skip = match entity.state {
                EntityState::Stunned | EntityState::Dead => true,
                _ => false,
            };

            if !skip {
                break;
            }
        }
    }

    pub fn next(&mut self) {
        self.current_entity += 1;
        if self.current_entity >= self.entities.len() {
            self.current_entity = 0;
            self.round += 1;
        }
    }
    
    pub fn sort(&mut self) {
        let mut entities = self.entities.clone();
        entities.sort_by(|a, b| {
            b.cmp(a)
        })
    }

    pub fn add_entity(&mut self, entity: Entity) -> usize {
        let id = self.entities.len() + 1;
        let mut entity = entity.clone();
        entity.id = id;

        if self.entities.iter().find(|e| e.name == entity.name).is_some() {
            entity.name = format!("[{}] {}", entity.id, entity.name);
        }

        self.entities.push(entity);

        id
    }

    pub fn find_entity(&self, identifier: String) -> Option<&Entity> {
        if let Ok(id) = identifier.parse::<usize>() {
            self.entities.iter().find(|e| e.id == id)
        } else {
            self.entities.iter().find(|e| e.name == identifier)
        }
    }
    
    pub fn find_entity_index(&self, identifier: String) -> Option<usize> {
        if let Ok(id) = identifier.parse::<usize>() {
            self.entities.iter().position(|e| e.id == id)
        } else {
            self.entities.iter().position(|e| e.name == identifier)
        }
    }

    pub fn find_entity_mut(&mut self, identifier: String) -> Option<&mut Entity> {
        if let Ok(id) = identifier.parse::<usize>() {
            self.entities.iter_mut().find(|e| e.id == id)
        } else {
            self.entities.iter_mut().find(|e| e.name == identifier)
        }
    }

    pub fn get_entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn get_current_entity(&self) -> &Entity {
        &self.entities[self.current_entity]
    }

    pub fn get_current_entity_mut(&mut self) -> &mut Entity {
        &mut self.entities[self.current_entity]
    }

    pub fn get_entity_action_bar(&self) -> (Vec<&Entity>, &Entity, Vec<&Entity>) {
        let mut before_entities = Vec::new();
        let mut after_entities = Vec::new();

        let current_entity = self.get_current_entity();

        for (index, entity) in self.entities.iter().enumerate() {
            if index < self.current_entity {
                before_entities.push(entity);
            } else if index > self.current_entity {
                after_entities.push(entity);
            }
        }

        (before_entities, current_entity, after_entities)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[derive(Clone)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    pub id: usize,
    pub name: String,
    pub health_points: i128,
    pub current_health_points: i128,
    pub effects: Vec<String>,
    pub initiative_value: i64,
    pub state: EntityState
}

impl Entity {
    pub fn new(name: String, health_points: i128, initiative_value: i64, state: EntityState) -> Entity {
        Entity {
            id: 0,
            name,
            health_points,
            current_health_points: health_points,
            effects: vec![],
            initiative_value,
            state
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[derive(Clone)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum EntityState {
    #[default]
    Healthy,
    Injured,
    Critical,
    Dead,
    Immune,
    Stunned,
    Alerted,
    Escaped,
}

impl EntityState {
    pub fn from_string(string: String) -> Option<EntityState> {
        match string.to_lowercase().as_str() {
            "healthy" => Some(EntityState::Healthy),
            "injured" => Some(EntityState::Injured),
            "critical" => Some(EntityState::Critical),
            "dead" => Some(EntityState::Dead),
            "immune" => Some(EntityState::Immune),
            "stunned" => Some(EntityState::Stunned),
            "alerted" => Some(EntityState::Alerted),
            "escaped" => Some(EntityState::Escaped),
            _ => None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            EntityState::Healthy => String::from("healthy"),
            EntityState::Injured => String::from("injured"),
            EntityState::Critical => String::from("critical"),
            EntityState::Dead => String::from("dead"),
            EntityState::Immune => String::from("immune"),
            EntityState::Stunned => String::from("stunned"),
            EntityState::Alerted => String::from("alerted"),
            EntityState::Escaped => String::from("escaped"),
        }
    }
}