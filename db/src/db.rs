use std::{fs, path::PathBuf, str::FromStr};

use sqlx::{
    migrate::MigrateError,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    ConnectOptions, Connection, SqliteConnection,
};

use crate::{Db, SqlResult, Transaction};

impl Db {
    pub async fn new() -> SqlResult<Self> {
        let dir = home::home_dir()
            .expect("cannot find home directory")
            .join(".pit");
        fs::create_dir_all(dir.clone()).expect("cannot create directory for storing data");

        Self::from_path(dir.join("data.sqlite3")).await
    }

    pub async fn from_path(path: PathBuf) -> SqlResult<Self> {
        let path = path.to_str().unwrap();
        let conn = SqliteConnectOptions::from_str(path)?
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .connect()
            .await?;

        let mut db = Self(conn);
        db.migrate_to_latest().await?;

        Ok(db)
    }

    pub async fn migrate_to_latest(&mut self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations").run(&mut self.0).await
    }

    pub async fn begin_wrapped_transaction(&mut self) -> SqlResult<Transaction> {
        Ok(Transaction(self.begin().await?))
    }

    pub async fn optimize(&mut self) -> SqlResult<()> {
        sqlx::query("PRAGMA optimize;").execute(&mut self.0).await?;

        Ok(())
    }
}

impl std::ops::Deref for Db {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Db {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
