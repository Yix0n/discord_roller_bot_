pub mod skill;
pub mod collection;
pub mod deck;
pub mod character;
pub mod hand_card;

use rusqlite::{Row, ToSql};
use rusqlite::Result as SqliteResult;
use rusqlite::types::{ToSqlOutput, Value};
use uuid::Uuid;
use crate::database::ConnectionPool;

pub trait Model {
    const DATABASE_NAME: &'static str;
    fn init(connection: &ConnectionPool);

    fn get_by_id(conn: &ConnectionPool, id: Uuid) -> SqliteResult<Option<Self>> where Self: Sized;
    
    fn from_row(row: &Row) -> SqliteResult<Self> where Self: Sized;
}

#[derive(Clone, Debug)]
pub struct ModelId<T: Model> {
    pub(crate) id: Uuid,
    #[allow(dead_code)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Model> ModelId<T> {
    pub fn new(id: Uuid) -> Self {
        ModelId { id, _phantom: std::marker::PhantomData }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn get(&self, pool: &ConnectionPool) -> SqliteResult<Option<T>> {
        T::get_by_id(pool, self.id)
    }

    pub fn to_string(&self) -> String {
        self.id.to_string()
    }
}

impl<T: Model> From<Uuid> for ModelId<T> {
    fn from(id: Uuid) -> Self {
        Self::new(id)
    }
}

impl<T: Model> std::fmt::Display for ModelId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<T: Model> ToOptionString for Option<ModelId<T>> {
    fn to_option_string(&self) -> Option<String> {
        self.as_ref().map(|m| m.id.to_string())
    }
}

#[derive(Debug)]
pub enum TokenType {
    ID(Uuid),
    Key(String),
}

pub trait ToOptionString {
    fn to_option_string(&self) -> Option<String>;
}