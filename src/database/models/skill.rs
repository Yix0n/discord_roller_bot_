use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::ConnectionPool;
use crate::database::models::{Model, ModelId};
use rusqlite::{Result as SqliteResult, Row};
use crate::database::models::collection::Collection;
use crate::mechanics::spell_cards::parse_inner_key;

#[derive(Clone, Debug)]
pub struct Skill {
    pub id: Uuid,
    pub user_id: isize,
    pub cost: i32,
    pub skill_name: String,
    pub skill_key: String,
    pub collection: ModelId<Collection>,
    pub description: String,
    pub created_at: DateTime<Utc>
}

impl Model for Skill {
    const DATABASE_NAME: &'static str = "skills";

    fn init(connection: &ConnectionPool) {
        let sql = format!(
            r#"
                CREATE TABLE IF NOT EXISTS {table} (
                    id TEXT PRIMARY KEY,
                    user_id INTEGER NOT NULL,
                    cost INTEGER NOT NULL,
                    skill_name TEXT NOT NULL,
                    skill_key TEXT NOT NULL,
                    collection TEXT NOT NULL,
                    description TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
            "#,
            table = Self::DATABASE_NAME
        );

        let index_value: Vec<&[&str]> = vec![
            &["user_id"], &["skill_key"], &["collection"]
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
            r#"
                SELECT id, user_id, cost,
                skill_name, skill_key, collection,
                description, created_at
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
        let id = {
            let d: String = row.get(0)?;
            Uuid::parse_str(&d).unwrap()
        };
        let user_id: isize = row.get(1)?;
        let cost: i32 = row.get(2)?;
        let skill_name: String = row.get(3)?;
        let skill_key: String = row.get(4)?;
        let collection: ModelId<Collection> = {
            let d: String = row.get(5)?;
            let id = Uuid::parse_str(&d).unwrap();
            ModelId::new(id)
        };
        let description: String = row.get(6)?;
        let created_at: DateTime<Utc> = {
            let dt: String = row.get(7)?;
            DateTime::from(DateTime::parse_from_rfc3339(&dt).unwrap())
        };
        
        Ok(Self {
            id, user_id, cost, skill_name, skill_key, collection, description, created_at
        })
    }
}

impl Skill {
    pub fn create(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            INSERT INTO {table} (
                id, user_id, cost, skill_name,
                skill_key, collection,
                description, created_at
            )
            VALUES (
                :id, :user_id, :cost, :skill_name,
                :skill_key, :collection,
                :description, :created_at
            );
        "#,
        table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":id", &self.id.to_string()),
            (":user_id", &self.user_id.to_string()),
            (":cost", &self.cost.to_string()),
            (":skill_name", &self.skill_name),
            (":skill_key", &self.skill_key),
            (":collection", &self.collection.to_string()),
            (":description", &self.description),
            (":created_at", &self.created_at.to_rfc3339()),
        ])?;

        Ok(())
    }

    pub fn find(
        conn: &ConnectionPool,
        user_id: u64,
        value: String,
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();

        let key = parse_inner_key(value.clone(), user_id);

        let sql = format!(r#"
            SELECT id, user_id, cost, skill_name,
            skill_key, collection,
            description, created_at
            FROM {table}
            WHERE skill_key = :key
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
    
    pub fn get_by_key(
        conn: &ConnectionPool,
        key: &String,
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            SELECT id, user_id, cost, skill_name,
            skill_key, collection,
            description, created_at
            FROM {table}
            WHERE skill_key = :key
        "#,
        table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":key", &key.to_string()),
        ])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
    
    pub fn get_by_user_id_paginated (
        conn: &ConnectionPool,
        user_id: isize,
        page: isize,
    ) -> SqliteResult<Vec<Self>> {
        let conn = conn.get().unwrap();

        let sql = format!(r#"
            SELECT id, user_id, cost, skill_name,
            skill_key, collection,
            description, created_at
            FROM {table}
            WHERE user_id = :user_id
            ORDER BY created_at DESC
            LIMIT {page_size} OFFSET {page_offset};
        "#,
        table = Self::DATABASE_NAME,
        page_size = 10,
        page_offset = page * 10);

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":user_id", &user_id.to_string()),
        ])?;

        let mut skills = vec![];

        while let Some(row) = rows.next()? {
            skills.push(Self::from_row(&row)?);
        };

        Ok(skills)
    }

    pub fn get_by_collection(
        conn: &ConnectionPool,
        collection_id: Uuid
    ) -> SqliteResult<Vec<Self>> {
        let conn = conn.get().unwrap();
        
        let sql = format!(r#"
            SELECT id, user_id, cost, skill_name,
            skill_key, collection,
            description, created_at
            FROM {table}
            WHERE collection = :collection
        "#, table = Self::DATABASE_NAME);
        
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":collection", &collection_id.to_string()),
        ])?;
        
        let mut skills = vec![];
        while let Some(row) = rows.next()? {
            skills.push(Self::from_row(&row)?);
        };
        
        Ok(skills)
    }
    
    pub fn update(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();

        let sql = format!(r#"
            UPDATE {table}
            SET
            skill_name = :skill_name,
            skill_key = :skill_key,
            collection = :collection,
            description = :description,
            WHERE id = :id
        "#,
            table = Self::DATABASE_NAME
        );

        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":skill_name", &self.skill_name),
            (":skill_key", &self.skill_key),
            (":collection", &self.collection.to_string()),
            (":description", &self.description),
            (":id", &self.id.to_string()),
        ])?;

        Ok(())
    }
    
    pub fn remove(
        &self, 
        conn: &ConnectionPool
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();
        
        let sql = format!(r#"DELETE FROM {table} 
        WHERE id = :id"#,
        table = Self::DATABASE_NAME);
        
        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":id", &self.id.to_string()),
        ])?;
        
        Ok(())
    }
}