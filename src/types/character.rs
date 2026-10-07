use serde::{Deserialize, Serialize};
use crate::helpers::errors::{ErrorContext, ProgramError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Character {
    pub key: String,
    pub name: String,
    pub strength: Stat,
    pub dexterity: Stat,
    pub intelligence: Stat,
    pub durability: Stat,
    pub charisma: Stat,
    pub perception: Stat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stat {
    pub points: i32,
    pub talents: i32
}

impl Stat {
    pub(crate) fn from_string(s: String) -> Result<Stat, ErrorContext> {
        let parts: Vec<&str> = s.split(":").collect();
        if parts.len() != 2 {
            return Err(ErrorContext::new(
                "Parser > Stat Parser",
                ProgramError::ParseError,
                "Invalid format for stat. Expected `points:talents`"
            ));
        }

        let points: i32 = parts[0].parse().map_err(|_|
            ErrorContext::new(
                "Parser > Stat Parser",
                ProgramError::ParseError,
                "Invalid format for stat. Expected `points` to be number"
            ))?;
        let talents: i32 = parts[1].parse().map_err(|_|
            ErrorContext::new(
                "Parser > Stat Parser",
                ProgramError::ParseError,
                "Invalid format for stat. Expected `talents` to be number"
            ))?;

        Ok(Stat {points, talents})
    }
}

fn parse_key(key: &str) -> String {
    format!("{}.skill_interpreter", key)
}