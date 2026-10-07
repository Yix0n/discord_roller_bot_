use crate::database::models::deck::{Deck};
use crate::database::models::{Model, ModelId, ToOptionString};
use crate::database::ConnectionPool;
use chrono::{DateTime, Utc};
use rusqlite::{params, Result as SqliteResult, Row};
use rusqlite::types::Value;
use rusqlite::types::Value::{Integer, Null, Text};
use serenity::futures::StreamExt;
use uuid::Uuid;
use crate::database::models::collection::Collection;
use crate::mechanics::spell_cards::parse_inner_key;

#[derive(Clone, Debug)]
pub struct Character {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub binded_deck: Option<ModelId<Deck>>,
    pub binded_channel: Option<isize>,
    pub binded_collection: Option<ModelId<Collection>>,
    pub owner_id: isize,
    pub detail_channel: isize,
    pub durability: i16,
    pub strength: i16,
    pub intelligence: i16,
    pub dexterity: i16,
    pub perception: i16,
    pub created_at: DateTime<Utc>,
}

impl Model for Character {
    const DATABASE_NAME: &'static str = "characters";

    fn init(connection: &ConnectionPool) {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table} (
                id TEXT PRIMARY KEY NOT NULL,
                key TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                binded_deck TEXT,
                binded_channel INTEGER,
                binded_collection TEXT,
                owner_id INTEGER NOT NULL,
                detail_channel INTEGER NOT NULL,
                durability INTEGER NOT NULL,
                strength INTEGER NOT NULL,
                intelligence INTEGER NOT NULL,
                dexterity INTEGER NOT NULL,
                perception INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
            "#,
            table = Self::DATABASE_NAME
        );

        let index_value: Vec<&[&str]> = vec![
            &["key"], &["binded_deck"], &["binded_channel"], &["binded_collection"], &["owner_id"],
            &["detail_channel"], &["key", "owner_id"], &["binded_channel", "owner_id"]
        ];

        let mut index_query = vec![sql];

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
            r#"
                SELECT id, key, name, binded_deck,
                binded_channel, binded_collection,
                owner_id, detail_channel,
                durability, strength, intelligence,
                dexterity, perception, created_at
                FROM {table}
                WHERE id = :id
            "#,
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
            let id: String = row.get(0)?;
            Uuid::parse_str(&id).unwrap()
        };
        let key: String = row.get(1)?;
        let name: String = row.get(2)?;
        let binded_deck: Option<ModelId<Deck>> = {
            let deck_id: Option<String> = row.get(3)?;
            match deck_id {
                Some(v) => Some(ModelId::new(Uuid::parse_str(&v).unwrap())),
                None => None,
            }
        };
        let binded_channel: Option<isize> = row.get(4)?;
        let binded_collection: Option<ModelId<Collection>> = {
            let collection_id: Option<String> = row.get(5)?;
            match collection_id {
                Some(v) => Some(ModelId::new(Uuid::parse_str(&v).unwrap())),
                None => None,
            }
        };
        let owner_id: isize = row.get(6)?;
        let detail_channel: isize = row.get(7)?;
        let durability: i16 = row.get(8)?;
        let strength: i16 = row.get(9)?;
        let intelligence: i16 = row.get(10)?;
        let dexterity: i16 = row.get(11)?;
        let perception: i16 = row.get(12)?;
        let created_at: DateTime<Utc> = {
            let dt: String = row.get(13)?;
            DateTime::from(DateTime::parse_from_rfc3339(&dt).unwrap())
        };

        Ok(Character {
            id,
            key,
            name,
            binded_deck,
            binded_channel,
            binded_collection,
            owner_id,
            detail_channel,
            durability,
            strength,
            intelligence,
            dexterity,
            perception,
            created_at,
        })
    }
}

impl Character {
    pub fn create(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            INSERT INTO {table} (
                id, key, name, binded_deck, binded_channel,
                binded_collection, owner_id, detail_channel,
                durability, strength, intelligence, dexterity,
                perception, created_at
            )
            VALUES (
                :id, :key, :name, :binded_deck, :binded_channel,
                :binded_collection, :owner_id, :detail_channel,
                :durability, :strength, :intelligence, :dexterity,
                :perception, :created_at
            );
        "#, table = Self::DATABASE_NAME);
        let mut stmt = pool.prepare(&sql)?;

        let id = self.id.to_string();
        let created_at = &self.created_at.to_rfc3339();

        let binded_deck = &self.binded_deck.to_option_string();
        let binded_collection = &self.binded_collection.to_option_string();

        let params: Vec<(&str, &dyn rusqlite::ToSql)> = vec![
            (":id", &id),
            (":key", &self.key),
            (":name", &self.name),
            (":binded_deck", binded_deck),
            (":owner_id", &self.owner_id),
            (":detail_channel", &self.detail_channel),
            (":binded_channel", &self.binded_channel),
            (":binded_collection", &binded_collection),
            (":durability", &self.durability),
            (":strength", &self.strength),
            (":intelligence", &self.intelligence),
            (":dexterity", &self.dexterity),
            (":perception", &self.perception),
            (":created_at", &created_at),
        ];


        stmt.execute(params.as_slice())?;

        Ok(())
    }

    pub fn get_by_key(
        conn: &ConnectionPool,
        key: &String,
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();

        let sql = format!(
            r#"
                SELECT id, key, name, binded_deck,
                binded_channel, binded_collection,
                owner_id, detail_channel, durability,
                strength, intelligence, dexterity,
                perception, created_at
                FROM {table}
                WHERE key = :key
            "#,
            table = Self::DATABASE_NAME
        );

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":key", &key.to_string()),
        ])?;

        if let Some(row) = rows.next()? {
            Some(Self::from_row(row)).transpose()
        } else {
            Ok(None)
        }
    }

    pub fn get_by_binded_deck(
        conn: &ConnectionPool,
        binded_deck_id: impl Into<String>
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();
        let sql = format!(r#"
            SELECT id, key, name, binded_deck, binded_channel,
            binded_collection, owner_id, detail_channel, durability,
            strength, intelligence, dexterity, perception, created_at
            FROM {table}
            WHERE binded_deck = :binded_deck_id
        "#, table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":binded_deck_id", &binded_deck_id.into()),
        ])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find(
        conn: &ConnectionPool,
        user_id: u64,
        value: String,
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();

        let key = parse_inner_key(value.clone(), user_id);

        let sql = format!(r#"
            SELECT id, key, name, binded_deck,
            binded_channel, binded_collection,
            owner_id, detail_channel, durability,
            strength, intelligence, dexterity,
            perception, created_at
            FROM {table}
            WHERE key = :key
            OR (name LIKE '%' || :name || '%'
            AND owner_id = :user_id)
            OR id = :value;
        "#, table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":key", &key.to_string()),
            (":name", &value.to_string()),
            (":value", &value.to_string()),
            (":user_id", &user_id.to_string()),
        ])?;

        if let Some(row) = rows.next()? {
            Some(Self::from_row(row)).transpose()
        } else {
            Ok(None)
        }
    }

    pub fn get_by_user_id_paginated(
        conn: &ConnectionPool,
        user_id: isize,
        page: isize,
    ) -> SqliteResult<Vec<Self>> {
        let pool = conn.get().unwrap();

        let sql = format!(
            r#"
                SELECT id, key, name, binded_deck,
                binded_channel, binded_collection,
                owner_id, detail_channel, durability,
                strength, intelligence, dexterity,
                perception, created_at
                FROM {table}
                WHERE owner_id = :owner_id
                LIMIT {page_size} OFFSET {page_offset}
            "#,
            table = Self::DATABASE_NAME,
            page_size = 10,
            page_offset = page * 10,
        );

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":owner_id", &user_id.to_string()),
        ])?;

        let mut chars = vec![];

        while let Some(row) = rows.next()? {
            chars.push(Self::from_row(row)?);
        };

        Ok(chars)
    }

    pub fn get_by_details_channel_and_user_id(
        pool: &ConnectionPool,
        channel_id: u64,
        user_id: u64
    ) -> SqliteResult<Option<Self>> {
        let pool = pool.get().unwrap();

        let sql = format!(r#"
            SELECT id, key, name, binded_deck,
            binded_channel, binded_collection,
            owner_id, detail_channel, durability,
            strength, intelligence, dexterity,
            perception, created_at
            FROM {table}
            WHERE detail_channel = :detail_channel
            AND owner_id = :owner_id
        "#,
                          table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":detail_channel", &channel_id.to_string()),
            (":owner_id", &user_id.to_string()),
        ])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_bind_channel_and_user_id(
        pool: &ConnectionPool,
        channel_id: u64,
        user_id: u64
    ) -> SqliteResult<Option<Self>> {
        let pool = pool.get().unwrap();
            // TODO kilka postaci może być związanych z jednym kanałem i użytkownikiem
        let sql = format!(r#"
            SELECT id, key, name, binded_deck,
            binded_channel, binded_collection,
            owner_id, detail_channel, durability,
            strength, intelligence, dexterity,
            perception, created_at
            FROM {table}
            WHERE binded_channel = :binded_channel
            AND owner_id = :owner_id
        "#,
                          table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":binded_channel", &channel_id.to_string()),
            (":owner_id", &user_id.to_string()),
        ])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_by_bind_channel(
        pool: &ConnectionPool,
        channel_id: impl Into<isize>,
    ) -> SqliteResult<Vec<Self>> {
        let pool = pool.get().unwrap();
        let sql = format!(r#"
        SELECT id, key, name, binded_deck,
            binded_channel, binded_collection,
            owner_id, detail_channel, durability,
            strength, intelligence, dexterity,
            perception, created_at
            FROM {table}
            WHERE binded_channel = :bind_channel"#,
                          table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":bind_channel", &channel_id.into()),
        ])?;

        let mut chars = vec![];
        while let Some(char) = rows.next()? {
            chars.push(Self::from_row(char)?);
        };

        Ok(chars)
    }

    pub fn update(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            UPDATE {table} SET
                name = :name,
                binded_deck = :binded_deck,
                binded_channel = :binded_channel,
                binded_collection = :binded_collection,
                detail_channel = :detail_channel,
                durability = :durability,
                strength = :strength,
                intelligence = :intelligence,
                dexterity = :dexterity,
                perception = :perception
            WHERE id = :id
        "#, table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;

        let deck_param = {
            if self.binded_deck.is_some() {
                Text(self.binded_deck.as_ref().unwrap().to_string())
            } else {
                Null
            }
        };

        let channel_param = {
            if self.binded_channel.is_some() {
                Integer(*self.binded_channel.as_ref().unwrap() as i64)
            } else {
                Null
            }
        };

        let collection_param = {
            if self.binded_collection.is_some() {
                Text(self.binded_collection.as_ref().unwrap().to_string())
            } else {
                Null
            }
        };

        stmt.execute(&[
            (":id", &Text(self.id.to_string())),
            (":name", &Text(self.name.to_string())),
            (":binded_deck", &deck_param),
            (":detail_channel", &Integer(self.detail_channel as i64)),
            (":binded_channel", &channel_param),
            (":binded_collection", &collection_param),
            (":durability", &Integer(self.durability as i64)),
            (":strength", &Integer(self.strength as i64)),
            (":intelligence", &Integer(self.intelligence as i64)),
            (":dexterity", &Integer(self.dexterity as i64)),
            (":perception", &Integer(self.perception as i64)),
        ])?;

        Ok(())
    }

    pub fn force_remove(
        &self,
        conn: &ConnectionPool
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"DELETE FROM {table} WHERE id = :id"#, table = Self::DATABASE_NAME);
        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":id", &self.id.to_string()),
        ])?;

        Ok(())
    }

    pub fn remove(
        &self,
        conn: &ConnectionPool,
        user_id: isize,
    ) -> SqliteResult<usize> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"DELETE FROM {table} WHERE id = :id AND owner_id = :user_id"#, table = Self::DATABASE_NAME);
        let mut stmt = pool.prepare(&sql)?;
        let affected = stmt.execute(&[
            (":id", &self.id.to_string()),
            (":user_id", &user_id.to_string()),
        ])?;

        Ok(affected)
    }
}