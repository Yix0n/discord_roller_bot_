use crate::database::models::skill::Skill;
use crate::database::models::Model;
use crate::database::ConnectionPool;
use crate::mechanics::spell_cards::special_decks;
use rusqlite::Result as SqliteResult;
use uuid::{uuid, Uuid};

#[derive(Clone, Copy, Debug)]
pub enum CardType {
    Curse,
    Basic,
    CharacterSkill,
}

impl CardType {
    pub fn to_db_type(self) -> crate::database::models::hand_card::CardType {
        match self {
            special_decks::CardType::Curse => crate::database::models::hand_card::CardType::Other,
            special_decks::CardType::Basic => {
                crate::database::models::hand_card::CardType::Secondary
            }
            special_decks::CardType::CharacterSkill => {
                crate::database::models::hand_card::CardType::Primary
            }
        }
    }

    pub fn from_db_type(card_type: crate::database::models::hand_card::CardType) -> Self {
        match card_type {
            crate::database::models::hand_card::CardType::Secondary => CardType::Basic,
            crate::database::models::hand_card::CardType::Primary => CardType::CharacterSkill,
            crate::database::models::hand_card::CardType::Other => CardType::Curse,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InnerCards {
    pub name: String,
    pub key: String,
    pub description: String,
    pub r#type: CardType,
    pub id: Uuid,
    pub cost: i32,
}

impl From<Skill> for InnerCards {
    fn from(skill: Skill) -> Self {
        Self {
            name: skill.skill_name,
            key: skill.skill_key,
            description: skill.description,
            r#type: CardType::CharacterSkill,
            id: skill.id,
            cost: skill.cost,
        }
    }
}

impl From<&Skill> for InnerCards {
    fn from(skill: &Skill) -> Self {
        Self {
            name: skill.skill_name.clone(),
            key: skill.skill_key.clone(),
            description: skill.description.clone(),
            r#type: CardType::CharacterSkill,
            id: skill.id.clone(),
            cost: skill.cost.clone(),
        }
    }
}

/*
InnerCards {
            name: "xxx"
                .to_string(),
            key: "xxxx"
                .to_string(),
            description: "xxxx"
                .to_string(),
            r#type: CardType::Curse,
            id: uuid!(),
            cost: 0,
        }
*/

fn init_curses() -> Vec<InnerCards> {
    let builder = vec![
        InnerCards {
                name: "Gamble"
                    .to_string(),
                key: "0.gamblecurse"
                    .to_string(),
                description: "Throw d2. 1 halves the effect of the spell. 2 doubles it"
                    .to_string(),
                r#type: CardType::Curse,
            id: uuid!("01a08fa5-9f20-778e-be85-47efe7e0794b"),
            cost: 1,
            },
        InnerCards {
            name: "Overdrive"
                .to_string(),
            key: "0.overdrivecurse"
                .to_string(),
            description: "Deal 1dX damage to self, but next attack cannot be dodged. X is character's current HP. This can kill caster"
                .to_string(),
            r#type: CardType::Curse,
            id: uuid!("01a08fa8-31a3-7549-abcf-6ff52c12edb1"),
            cost: 1,
        },
        InnerCards {
                name: "Penitent"
                    .to_string(),
                key: "0.penitentcurse"
                    .to_string(),
                description: "Purges deck. For every card that got purged your next action has +2 on final value"
                    .to_string(),
                r#type: CardType::Curse,
            id: uuid!("01a08fae-1cd4-7619-88af-096e81c4e0fd"),
                cost: 1,
            },
        InnerCards {
            name: "Silence"
                .to_string(),
            key: "0.silencecurse"
                .to_string(),
            description: "Next spell doubles the throws (2d2 changes to 4d2 etc), but you cannot cast this spell again in this battle"
                .to_string(),
            r#type: CardType::Curse,
            id: uuid!("01a08fae-4f81-75fd-9919-a3eab54abc68"),
            cost: 1,
        },
        InnerCards {
            name: "Sacrifice"
                .to_string(),
            key: "0.sacraficecurse"
                .to_string(),
            description: "You give your HP to selected ally until he is fully healed or your HP falls to 0"
                .to_string(),
            r#type: CardType::Curse,
            id: uuid!("01a0a609-adb5-7302-8076-f80e1d9692fc"),
            cost: 1,
        },
        InnerCards {
            name: "Rage"
                .to_string(),
            key: "0.ragecurse"
                .to_string(),
            description: "You gain disadvantege to hit, but deal double the damage for the rest of the fight. If spell has guaranteed hit, then it's standard roll to hit"
                .to_string(),
            r#type: CardType::Curse,
            id: uuid!("01a0a612-0f9d-777a-804e-0859bacbdef4"),
            cost: 1,
        },
    ];

    builder
}

fn init_basics() -> Vec<InnerCards> {
    let builder = vec![
        InnerCards {
            name: "Basic Attack"
                .to_string(),
            key: "0.basicattackbasic"
                .to_string(),
            description: "Deals `1d3 + wpn` damage to one enemy. `wpn` is current weapon bonus"
                .to_string(),
            r#type: CardType::Basic,
            id: uuid!("01a08fae-66b5-72dd-8416-9577c1e1ae5e"),
            cost: 15,
        },
        InnerCards {
            name: "Follow-up Attack"
                .to_string(),
            key: "0.supriseattackbasic"
                .to_string(),
            description:
                "Can be casted anytime. Deals `1d3 + wpn` damage to random enemy target. `wpn` is current weapon bonus"
                    .to_string(),
            r#type: CardType::Basic,
            id: uuid!("01a08fae-9434-7213-8a50-e657b6f79586"),
            cost: 6,
        },
        InnerCards {
            name: "Emergency Healing"
                .to_string(),
            key: "0.emergencyhealingbasic"
                .to_string(),
            description: "Heals `3d3` HP for the caster"
                .to_string(),
            r#type: CardType::Basic,
            id: uuid!("01a08fae-b182-72d7-b506-542957d82d76"),
            cost: 4,
        },
        InnerCards {
            name: "Pipe Bomb"
                .to_string(),
            key: "0.pipebombbasic"
                .to_string(),
            description: "Throws a bomb, that upon hit deals `2d3`"
                .to_string(),
            r#type: CardType::Basic,
            id: uuid!("01a08fae-b8b5-76a8-b55d-1caa7cd53037"),
            cost: 4,
        },
        InnerCards {
            name: "Advantage"
                .to_string(),
            key: "0.advantagebasic"
                .to_string(),
            description: "For the next `d3` turns caster gains advantage on their rolls"
                .to_string(),
            r#type: CardType::Basic,
            id: uuid!("01a0a4d1-c887-72ca-a4a9-a823360f9a1b"),
            cost: 1,
        },
        InnerCards {
            name: "Revive"
                .to_string(),
            key: "0.saviourbasic"
                .to_string(),
            description: "Revive fallen ally. Revived ally is back with 1 HP and `3d3` Shield Points. This shield lasts 2 turns"
                .to_string(),
            r#type: CardType::Basic,
            id: uuid!("01a0a614-8ea1-75a8-b532-a32fca5834b4"),
            cost: 8,
        },
        InnerCards {
            name: "Guard Up"
                .to_string(),
            key: "0.guardupbasic"
                .to_string(),
            description: "Caster gain `2d5` Shield Point until his next action. If this shield breaks, then caster gain extra 1 card"
                .to_string(),
            r#type: CardType::Curse,
            id: uuid!("01a0a618-a806-7518-9e27-5dc91dab11a3"),
            cost: 0,
        },
    ];

    builder
}

pub fn inner_by_id(uuid: Uuid, pool: Option<&ConnectionPool>) -> SqliteResult<Option<InnerCards>> {
    let mut all_cards = get_curses();
    all_cards.append(&mut get_basics());

    if let Some(c) = all_cards.into_iter().find(|c| c.id == uuid) {
        Ok(Some(c))
    } else if let Some(pool) = pool {
        match Skill::get_by_id(&pool, uuid)? {
            Some(s) => Ok(Some(InnerCards::from(s))),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

pub fn get_curses() -> Vec<InnerCards> {
    init_curses()
}

pub fn get_basics() -> Vec<InnerCards> {
    init_basics()
}
