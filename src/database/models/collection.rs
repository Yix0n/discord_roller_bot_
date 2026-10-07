use chrono::{DateTime, Utc};
use rusqlite::{Result as SqliteResult, Row};
use uuid::Uuid;
use crate::database::ConnectionPool;
use crate::database::models::Model;
use crate::mechanics::spell_cards::parse_inner_key;

#[derive(Clone)]
#[derive(Debug)]
pub struct Collection {
    pub id: Uuid,
    pub user_id: isize,
    pub name: String,
    pub key: String,
    pub created_at: DateTime<Utc>,
}

impl Model for Collection {
    const DATABASE_NAME: &'static str = "collections";

    fn init(connection: &ConnectionPool) {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table} (
                id TEXT PRIMARY KEY NOT NULL,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                key TEXT NOT NULL,
                created_at TIME NOT NULL
            );
            "#,
            table = Self::DATABASE_NAME
        );

        let index_value: Vec<&[&str]> = vec![
            &["key"], &["user_id"]
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
                SELECT id, user_id, name, key, created_at
                FROM {table}
                WHERE id = :id;
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
        let user_id: isize = row.get(1)?;
        let name: String = row.get(2)?;
        let key: String = row.get(3)?;
        let created_at:DateTime<Utc> = {
            let dt: String = row.get(4)?;
            DateTime::from(DateTime::parse_from_rfc3339(&dt).unwrap())
        } ;

        Ok(Self {
            id,
            user_id,
            name,
            key,
            created_at,
        })
    }
}

impl Collection {
    pub fn create(
        &self,
        conn: &ConnectionPool,
    ) -> SqliteResult<()> {
        let pool = conn.get().unwrap();
        let sql = format!(r#"
            INSERT INTO {table} (
                id, user_id, name,
                key, created_at
            )
            VALUES (
                :id, :user_id, :name, :key, :created_at
            );
        "#, table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":id", &self.id.to_string()),
            (":user_id", &self.user_id.to_string()),
            (":name", &self.name),
            (":key", &self.key),
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
            SElECT id, user_id, name, key, created_at
            FROM {table}
            WHERE key = :key
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

    pub fn find(
        conn: &ConnectionPool,
        user_id: u64,
        value: String,
    ) -> SqliteResult<Option<Self>> {
        let pool = conn.get().unwrap();

        let key = parse_inner_key(value.clone(), user_id);

        let sql = format!(r#"
            SELECT id, user_id, name, key, created_at
            FROM {table}
            WHERE key = :key
            OR (name LIKE '%' || :name || '%' AND user_id = :user_id)
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

    pub fn get_by_user_id_paginated (
        conn: &ConnectionPool,
        user_id: isize,
        page: isize,
    ) -> SqliteResult<Vec<Self>> {
        let pool = conn.get().unwrap();
        let sql = format!(r#"
        SELECT id, user_id, name, key, created_at
        FROM {table}
        WHERE user_id = :user_id
        ORDER BY created_at DESC
        LIMIT {page_size} OFFSET {offset}"#,
        table = Self::DATABASE_NAME,
        page_size = 10,
        offset = page * 10);

        let mut stmt = pool.prepare(&sql)?;
        let mut rows = stmt.query(&[
            (":user_id", &user_id.to_string()),
        ])?;

        let mut collections = vec![];

        while let Some(row) = rows.next()? {
            collections.push(Self::from_row(row)?);
        };

        Ok(collections)
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
            )
            WHERE id = :id;
        "#,
        table = Self::DATABASE_NAME);

        let mut stmt = pool.prepare(&sql)?;
        stmt.execute(&[
            (":name", &self.name),
            (":key", &self.key),
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