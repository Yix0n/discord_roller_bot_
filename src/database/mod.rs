use r2d2_sqlite::SqliteConnectionManager;

pub mod models;

pub type ConnectionPool = r2d2::Pool<SqliteConnectionManager>;