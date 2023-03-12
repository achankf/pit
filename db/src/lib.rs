mod assertion;
mod config;
mod db;
mod insert;
mod one_shot;
mod transaction;
mod update_price;
mod upsert;
mod yahoo_symbol;

pub use assertion::*;
pub use insert::*;
pub use one_shot::*;
use sqlx::SqliteConnection;
pub use upsert::*;

pub type SqlResult<T> = sqlx::Result<T>;

pub struct Db(SqliteConnection);

pub struct Transaction<'c>(sqlx::Transaction<'c, sqlx::Sqlite>);

pub type SqlQuery<'q> = sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>;

pub trait Query {
    fn query(&self) -> SqlQuery;
}
