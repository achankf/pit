use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    path::PathBuf,
};

use common::{deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::de;
use sqlx::Sqlite;

use crate::{Query, SqlQuery, SqlResult, Transaction};

impl<'c> Transaction<'c> {
    pub async fn commit(self) -> SqlResult<()> {
        self.0.commit().await
    }

    pub async fn rollback(self) -> SqlResult<()> {
        self.0.rollback().await
    }

    pub async fn execute(
        &mut self,
        query: SqlQuery<'_>,
    ) -> SqlResult<<Sqlite as sqlx::Database>::QueryResult> {
        Ok(query.execute(&mut *self.0).await?)
    }

    pub async fn upsert_all<T>(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<BTreeMap<T::IdType, T>, Box<dyn std::error::Error>>
    where
        T: Id + de::DeserializeOwned + Query + std::fmt::Debug,
    {
        let parsed_records = deserialize_into_map::<T>(csv_path).await?;

        for (_, record) in &parsed_records {
            match self.execute(record.query()).await {
                Ok(result) => {
                    if result.rows_affected() != 1 {
                        println!(
                            "{}: no row was affected, record: {:?}",
                            "Warning".bold().yellow(),
                            record
                        );
                    }
                }
                Err(err) => {
                    return Err(format!("{}, record: {:?}", err.to_string(), record).into());
                }
            }
        }

        Ok(parsed_records)
    }

    pub async fn upsert_all_in_order<T>(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        T: Id + de::DeserializeOwned + Query + std::fmt::Debug,
    {
        let mut parsed_records = Vec::new();

        let file = File::open(csv_path)?;
        let mut set = BTreeSet::new();

        for result in csv::Reader::from_reader(file).deserialize::<T>() {
            let record = result?;
            let id = record.id();

            if !set.insert(id.clone()) {
                return Err(format!("duplicate id: {id:#?}").into());
            }

            parsed_records.push(record);
        }

        for record in &parsed_records {
            match self.execute(record.query()).await {
                Ok(result) => {
                    if result.rows_affected() != 1 {
                        println!(
                            "{}: no row was affected, record: {:?}",
                            "Warning".bold().yellow(),
                            record
                        );
                    }
                }
                Err(err) => {
                    return Err(format!("{}, record: {:?}", err.to_string(), record).into());
                }
            }
        }

        Ok(parsed_records)
    }
}

impl<'c> std::ops::Deref for Transaction<'c> {
    type Target = sqlx::Transaction<'c, sqlx::Sqlite>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'c> std::ops::DerefMut for Transaction<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
