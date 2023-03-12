use std::path::PathBuf;

use common::{deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Transaction;

#[derive(Deserialize, Debug)]
pub struct StockAccount {
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
}

impl Id for StockAccount {
    type IdType = String;

    fn id(&self) -> String {
        self.account_type.clone()
    }
}

impl Transaction<'_> {
    pub async fn upsert_stock_account(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<StockAccount>(csv_path).await?;

        for (_, record) in &parsed_records {
            if !self.has_cash_account(&record.account_type).await? {
                return Err(format!(
                    "cannot insert {} as a stock account unless it is also a cash account",
                    record.account_type
                )
                .into());
            }

            self.upsert_stock_stock_account_helper(record).await?;
        }

        Ok(())
    }

    async fn upsert_stock_stock_account_helper(
        &mut self,
        record: &StockAccount,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO StockAccount (account_type_id)
VALUES
    (
        (
            SELECT
                account_type_id
            FROM
                AccountType
            WHERE
                account_type = ?
        )
    )
"#,
            record.account_type
        )
        .execute(&mut *self.0)
        .await;

        match result {
            Ok(result) => {
                if result.rows_affected() != 1 {
                    println!(
                        "{}: no row was affected, record: {:?}",
                        "Warning".bold().yellow(),
                        record
                    );
                }

                Ok(())
            }
            Err(err) => Err(format!("{}, record: {:?}", err.to_string(), record).into()),
        }
    }
}
