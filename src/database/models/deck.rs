use std::collections::HashMap;
use std::fmt::format;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::ConnectionPool;
use crate::database::models::Model;
use rusqlite::{Result as SqliteResult, Row};
use serenity::all::User;
use crate::database::models::character::Character;
use crate::database::models::hand_card::HandCard;
use crate::database::models::skill::Skill;
use crate::database::models::collection::Collection;
use crate::mechanics::spell_cards::parse_inner_key;

#[derive(Clone)]
#[derive(Debug)]
pub struct Deck {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub user_id: isize,
    pub created_at: DateTime<Utc>
}

impl Model for Deck {
    const DATABASE_NAME: &'static str = "decks";

    fn init(connection: &ConnectionPool) {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table} (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                key TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
            "#,
            table = Self::DATABASE_NAME
        );

        let index_value: Vec<&[&str]> = vec![
            &["user_id"], &["key"],
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
            r#"SELECT id, name, key, user_id, created_at
            FROM {table}
            WHERE id = :id;"#,
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
        let name: String = row.get(1)?;
        let key: String = row.get(2)?;
        let user_id: isize = row.get(3)?;
        let created_at: DateTime<Utc> = {
            let dt: String = row.get(4)?;
            DateTime::from(DateTime::parse_from_rfc3339(&dt).unwrap())
        };

        Ok(Self {
            id,
            name,
            key,
            user_id,
            created_at
        })
    }
}

impl Deck {
    pub fn create(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();
        
        let sql = format!(r#"
            INSERT INTO {table} (
                id, name, key,
                user_id,
                created_at
            ) VALUES (
                :id, :name, :key, :user_id, :created_at
            )
        "#, table = Self::DATABASE_NAME);
        
        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":id", &self.id.to_string()),
            (":name", &self.name),
            (":key", &self.key),
            (":user_id", &self.user_id.to_string()),
            (":created_at", &self.created_at.to_rfc3339()),
        ])?;
        
        Ok(())
    }

    pub fn get_by_key(
        conn: &ConnectionPool,
        key: &String,
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();
        let sql = format!(r#"
            SELECT id, name, key, user_id, created_at
            FROM {table}
            WHERE key = :key;
        "#, table = Self::DATABASE_NAME);
        
        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":key", &key),
        ])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_user_id_paginated (
        conn: &ConnectionPool,
        user_id: isize,
        page: isize,
    ) -> SqliteResult<Option<Vec<Self>>> {
        let pool = conn.get().unwrap();
        let sql = format!(r#"
            SELECT id, name, key, user_id, created_at
            FROM {table}
            WHERE user_id = :user_id
            ORDER BY created_at DESC
            LIMIT {page_size} OFFSET {page_offset};
        "#, table = Self::DATABASE_NAME,
        page_size = 10,
        page_offset = page * 10);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":user_id", &user_id.to_string()),
        ])?;
        
        let mut decks = vec![];
        
        while let Some(row) = rows.next()? {
            decks.push(Self::from_row(row)?);
        };
        
        Ok(Some(decks))
    }

    pub fn find(
        conn: &ConnectionPool,
        user_id: u64,
        value: String,
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();

        let key = parse_inner_key(value.clone(), user_id);

        let sql = format!(r#"
            SELECT id, name, key,
            user_id, created_at
            FROM {table}
            WHERE key = :key
            OR (name LIKE '%' || :name || '%' AND user_id = :user_id)
            OR id = :value
            LIMIT 1;
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

    pub fn get_characters(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<Vec<Character>> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            SELECT id, key, name, binded_deck,
            binded_channel, owner_id,
            detail_channel, durability,
            strength, intelligence, dexterity,
            perception, created_at
            FROM {character_table}
            WHERE binded_deck = :binded_deck
        "#, character_table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":binded_deck", &self.id.to_string()),
        ])?;

        let mut chars = vec![];
        while let Some(row) = rows.next()? {
            chars.push(Character::from_row(row)?);
        }
        Ok(chars)
    }

    pub fn update(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();
        
        let sql = format!(r#"
        UPDATE {table} SET
        (
            name = :name,
            key = :key,
        ) WHERE id = :id
        "#,
        table = Self::DATABASE_NAME);
        
        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":name", &self.name.to_string()),
            (":key", &self.key.to_string()),
            (":id", &self.id.to_string()),
        ])?;
        
        Ok(())
    }

    pub fn remove(
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
}