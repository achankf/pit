use std::path::PathBuf;

use common::{deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Transaction;

#[derive(Debug, Deserialize)]
pub struct AccountType {
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
}

impl Id for AccountType {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.account_type.clone()
    }
}

impl Transaction<'_> {
    pub async fn upsert_account_type(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<AccountType>(csv_path).await?;

        for (_, record) in parsed_records {
            self.upsert_account_type_helper(&record.account_type)
                .await?;
        }

        Ok(())
    }

    pub async fn upsert_account_type_helper(
        &mut self,
        account_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("inserting account type {account_type}");

        let result = sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO AccountType (account_type)
VALUES
    (?)
"#,
            account_type,
        )
        .execute(&mut *self.0)
        .await;

        match result {
            Ok(result) => {
                if result.rows_affected() != 1 {
                    println!(
                        "{}: account type already existed",
                        "Warning".bold().yellow(),
                    );
                }
                Ok(())
            }
            Err(err) => Err(format!("{}, record: {}", err.to_string(), account_type).into()),
        }
    }
}
