use std::path::PathBuf;

use common::{deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Transaction;

#[derive(Deserialize, Debug)]
pub struct CreditCard {
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
    #[serde(deserialize_with = "string_trim")]
    pub institution_name: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_name: String,
    pub annual_fee: f64,
    pub credit_limit: Option<f64>,
    #[serde(deserialize_with = "string_trim")]
    pub currency: String,
}

impl Id for CreditCard {
    type IdType = String;

    fn id(&self) -> String {
        self.account_type.clone()
    }
}

impl Transaction<'_> {
    pub async fn upsert_credit_card(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<CreditCard>(csv_path).await?;

        for (_, record) in &parsed_records {
            self.upsert_account_type_helper(&record.account_type)
                .await?;

            self.upsert_credit_card_helper(record).await?;
        }

        Ok(())
    }

    async fn upsert_credit_card_helper(
        &mut self,
        record: &CreditCard,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        println!("inserting credit card {}", record.account_type);

        let result = sqlx::query!(
            r#"
INSERT INTO
    CreditCardProduct (
        account_type_id,
        institution_id,
        currency_id,
        annual_fee,
        credit_limit,
        account_name
    )
VALUES
    (
        (
            SELECT
                account_type_id
            FROM
                AccountType
            WHERE
                account_type = ?
        ),
        (
            SELECT
                institution_id
            FROM
                Institution
            WHERE
                institution_name = ?
        ),
        (
            SELECT
                currency_id
            FROM
                Currency
            WHERE
                currency = ?
        ),
        ?,
        ?,
        ?
    ) ON CONFLICT(account_type_id) DO
UPDATE
SET
    institution_id = excluded.institution_id,
    annual_fee = excluded.annual_fee,
    credit_limit = excluded.credit_limit
"#,
            record.account_type,
            record.institution_name,
            record.currency,
            record.annual_fee,
            record.credit_limit,
            record.account_name
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

                    Ok(None)
                } else {
                    Ok(Some(result.last_insert_rowid()))
                }
            }
            Err(err) => Err(format!("{}, record: {:?}", err.to_string(), record).into()),
        }
    }
}
