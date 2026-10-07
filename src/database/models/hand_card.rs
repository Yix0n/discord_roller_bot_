use chrono::{DateTime, Utc};
use rusqlite::{Result as SqliteResult, Row};
use uuid::Uuid;
use crate::database::ConnectionPool;
use crate::database::models::deck::{Deck};
use crate::database::models::{Model, ModelId};
use crate::database::models::skill::{Skill};
use crate::database::models::character::{Character};

#[derive(Clone, Debug)]
pub enum CardType {
    Primary,
    Secondary,
    Other
}

impl CardType {
    pub fn to_string(&self) -> String {
        match self {
            CardType::Primary => String::from("primary"),
            CardType::Secondary => String::from("secondary"),
            CardType::Other => String::from("other")
        }
    }
    pub fn from_string(o: impl Into<String>) -> SqliteResult<CardType> {
        let o: String = o.into();

        match o.as_ref() {
            "primary" => Ok(CardType::Primary),
            "secondary" => Ok(CardType::Secondary),
            "other" => Ok(CardType::Other),
            &_ => unreachable!(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum CardAction {
    None,
    Selected
}

impl CardAction {
    pub fn to_string(&self) -> String {
        match self {
            CardAction::Selected => String::from("selected"),
            CardAction::None => String::from("none"),
        }
    }

    pub fn from_string(o: impl Into<String>) -> SqliteResult<CardAction> {
        let o: String = o.into();
        match o.as_ref() {
            "selected" => Ok(CardAction::Selected),
            "none" => Ok(CardAction::None),
            _ => Err(rusqlite::Error::InvalidQuery)
        }
    }
}

#[derive(Clone, Debug)]
pub struct HandCard {
    pub id: Uuid,
    pub user_id: isize,
    pub card_type: CardType,
    pub character: ModelId<Character>,
    pub deck: ModelId<Deck>,
    pub skill: ModelId<Skill>,
    pub card_action: CardAction,
    pub created_at: DateTime<Utc>,
}

impl Model for HandCard {
    const DATABASE_NAME: &'static str = "hand_cards";

    fn init(connection: &ConnectionPool) {
        let sql = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {table} (
                    id TEXT PRIMARY KEY,
                    user_id INTEGER NOT NULL,
                    card_type TEXT NOT NULL,
                    character TEXT NOT NULL,
                    deck TEXT NOT NULL,
                    skill TEXT NOT NULL,
                    card_action TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
            "#,
            table = Self::DATABASE_NAME
        );

        let index_value: Vec<&[&str]> = vec![
            &["user_id"], &["card_type"],
            &["character"], &["deck"], &["skill"],
            &["card_action"]
        ];

        let mut index_query = vec![];

        index_query.push(sql);

        for v in index_value {
            index_query.push(format!(
                "CREATE INDEX IF NOT EXISTS idx_{table}_{param_v} ON {table} ({param_k});",
                table = Self::DATABASE_NAME,
                param_v = v.join("_"),
                param_k = v.join(", ")
            ))
        }

        let pool = connection.get().unwrap();

        pool.execute_batch(
            format!(
                r#"BEGIN;
                {v}
                COMMIT;"#,
                v = index_query.join("\n")
            ).as_str()
        ).unwrap();
    }

    fn get_by_id(conn: &ConnectionPool, id: Uuid) -> SqliteResult<Option<Self>>
    where
        Self: Sized
    {
        let pool = conn.get().unwrap();

        let sql = format!(
            r#"SELECT id, user_id, card_type,
            character, deck, skill, card_action,
            created_at
            FROM {table}
            WHERE id = :id"#,
            table = Self::DATABASE_NAME
        );

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":id", &id.to_string()),
        ])?;

        if let Some(row) = rows.next()? {
            Some(Self::from_row(row)).transpose()
        } else {
            Ok(None)
        }
    }

    fn from_row(row: &Row) -> SqliteResult<Self>
    where
        Self: Sized
    {
        let id: Uuid = {
            let d: String = row.get(0)?;
            Uuid::parse_str(&d.as_str()).unwrap()
        };
        let user_id: isize = row.get(1)?;
        let card_type: CardType = {
            let c: String = row.get(2)?;
            CardType::from_string(c.as_str()).unwrap()
        };
        let character: ModelId<Character> = {
            let d: String = row.get(3)?;
            ModelId::new(Uuid::parse_str(&d.as_str()).unwrap())
        };
        let deck: ModelId<Deck> = {
            let d: String = row.get(4)?;
            ModelId::new(Uuid::parse_str(&d.as_str()).unwrap())
        };
        let skill: ModelId<Skill> = {
            let d: String = row.get(5)?;
            ModelId::new(Uuid::parse_str(&d.as_str()).unwrap())
        };
        let card_action: CardAction = {
            let c: String = row.get(6)?;
            CardAction::from_string(c.as_str())?
        };
        let created_at: DateTime<Utc> = {
            let dt: String = row.get(7)?;
            DateTime::from(DateTime::parse_from_rfc3339(&dt).unwrap())
        };

        Ok(Self {
            id,
            user_id,
            card_type,
            character,
            deck,
            skill,
            card_action,
            created_at
        })
    }
}

impl HandCard {
    pub fn create(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            INSERT INTO {table} (
                id,
                user_id,
                card_type,
                character,
                deck,
                skill,
                card_action,
                created_at
            )
            VALUES (
                :id,
                :user_id,
                :card_type,
                :character,
                :deck,
                :skill,
                :card_action,
                :created_at
            );
        "#,
        table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":id", &self.id.to_string()),
            (":user_id", &self.user_id.to_string()),
            (":card_type", &self.card_type.to_string()),
            (":character", &self.character.to_string()),
            (":deck", &self.deck.to_string()),
            (":skill", &self.skill.to_string()),
            (":card_action", &self.card_action.to_string()),
            (":created_at", &self.created_at.to_rfc3339()),
        ])?;

        Ok(())
    }

    pub fn get_by_user_id_paginated (
        conn: &ConnectionPool,
        user_id: isize,
        page: isize,
    ) -> SqliteResult<Vec<Self>> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            SELECT id, user_id, card_type,
            character, deck, skill, card_action,
            created_at
            FROM {table}
            WHERE user_id = :user_id
            ORDER BY created_at DESC
            LIMIT {page_size} OFFSET {page_offset};
        "#,
        table = Self::DATABASE_NAME,
        page_size = 10,
        page_offset = page * 10);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":user_id", &user_id.to_string()),
        ])?;

        let mut cards = vec![];
        while let Some(row) =rows.next()? {
            cards.push(HandCard::from_row(row)?)
        };

        Ok(cards)
    }

    pub fn get_cards_by_character_and_deck_id(
        conn: &ConnectionPool,
        character_id: Uuid,
        deck_id: Uuid
    ) -> SqliteResult<Vec<Self>> {
        let pool = conn.get().unwrap();
        let sql = format!(r#"
        SELECT id, user_id, card_type,
        character, deck, skill, card_action,
        created_at
        FROM {table}
        WHERE
        character = :character_id
        AND deck = :deck_id"#,
        table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        stmt.raw_bind_parameter(c":character_id", &character_id.to_string())?;
        stmt.raw_bind_parameter(c":deck_id", &deck_id.to_string())?;

        let mut rows = stmt.raw_query();

        let mut cards = vec![];
        while let Some(row) =rows.next()? {
            cards.push(Self::from_row(row)?)
        };

        Ok(cards)
    }

    pub fn update(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            UPDATE {table} SET

            card_type = :card_type,
            character = :character,
            deck = :deck,
            skill = :skill,
            card_action = :card_action

            WHERE id = :id;
        "#,
        table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":card_type", &self.card_type.to_string()),
            (":character", &self.character.to_string()),
            (":deck", &self.deck.to_string()),
            (":skill", &self.skill.to_string()),
            (":card_action", &self.card_action.to_string()),
            (":id", &self.id.to_string()),
        ])?;

        Ok(())
    }

    pub fn remove(
        &self,
        conn: &ConnectionPool
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            DELETE FROM {table} WHERE id = :id;
        "#,
        table = Self::DATABASE_NAME);
        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":id", &self.id.to_string()),
        ])?;

        Ok(())
    }

    pub fn purge(
        conn: &ConnectionPool,
        deck_id: Uuid,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
        DELETE FROM {table} WHERE deck = :deck_id"#, table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":deck_id", &deck_id.to_string()),
        ])?;

        Ok(())
    }
}