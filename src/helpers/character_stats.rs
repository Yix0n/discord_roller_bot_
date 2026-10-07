
// Strength (ilość unikalnych slotów? - 1 punkt daje 1 efekt który można dodać do umiejętności, 1 talent daje dodatkowy slot do umiejętności)
// - Dexterity (Agility)
// - Intelligence (Mana/Turn)
// - Durability (HP)
// - Charisma (Max Mana)
// Perception (Punkty wiedzy) - im więcej punktów tym więcej informacji zdobywają o przeciwniku

use crate::types::character::Stat;

pub fn get_character_health(durability: &Stat) -> i32 {
    (durability.points * 2) + (durability.talents * 5) + 3
}

pub fn get_character_agility(dexterity: &Stat) -> i32 {
    (dexterity.points + 1) * 2 + (dexterity.talents + 1) * 5 + 5
}

pub fn get_max_mana(charisma: &Stat) -> i32 {
    80 + (charisma.points * 8) + (charisma.talents * 20)
}

pub fn get_mana_regeneration(intelligence: &Stat) -> i32 {
    10 + (intelligence.points * 2) + (intelligence.talents * 8)
}

pub fn get_discovery_points(perception: &Stat) -> i32 {
    let total_points = perception.talents * 2 + perception.points;

    total_points
}

pub fn get_character_modifier(stat: &Stat) -> &str {
    match stat.points {
        0 => "-5",
        1..=2 => "-4",
        3..=5 => "-3",
        6..=7 => "-2",
        8..=9 => "-1",
        10 => "0",
        11..=13 => "+1",
        14..=15 => "+2",
        16..=18 => "+3",
        19 => "+4",
        20 => "+5",
        _ => "-0"
    }
}

pub fn get_modifier(stat: i8) -> String {
    match stat {
        0..=1 => "-2",
        2..=4 => "-1",
        5..=10 => "0",
        11..=14 => "+1",
        15..=18 => "+2",
        19..=20 => "+3",
        _ => "-0"
    }.to_string()
}