use std::path::PathBuf;

use common::{deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::{SqlResult, Transaction};

#[derive(Debug, Deserialize)]
pub struct CashAccountProduct {
    #[serde(deserialize_with = "string_trim")]
    pub account_name: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
    #[serde(deserialize_with = "string_trim")]
    pub institution_name: String,
    #[serde(deserialize_with = "string_trim")]
    pub tax_shelter_type: String,
    #[serde(deserialize_with = "string_trim")]
    pub currency: String,
    pub min_balance_waiver: f64,
    #[serde(default = "i64max")]
    pub inactive_fee_months: i64,
}

pub fn i64max() -> i64 {
    // essentially no inactivity fee
    i64::MAX
}

impl Id for CashAccountProduct {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.account_type.clone()
    }
}

impl Transaction<'_> {
    pub async fn upsert_cash_account(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<CashAccountProduct>(csv_path).await?;

        for (_, record) in &parsed_records {
            self.upsert_account_type_helper(&record.account_type)
                .await?;

            self.upsert_cash_cash_account_helper(&record).await?;
        }

        Ok(())
    }

    async fn upsert_cash_cash_account_helper(
        &mut self,
        record: &CashAccountProduct,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("> inserting cash account");

        let result = sqlx::query!(
            r#"
INSERT INTO
    CashAccountProduct (
        account_type_id,
        institution_id,
        tax_shelter_type_id,
        currency_id,
        min_balance_waiver,
        inactive_fee_months,
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
                tax_shelter_type_id
            FROM
                TaxShelterType
            WHERE
                tax_shelter_type = ?
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
    tax_shelter_type_id = excluded.tax_shelter_type_id,
    currency_id = excluded.currency_id,
    min_balance_waiver = excluded.min_balance_waiver,
    inactive_fee_months = excluded.inactive_fee_months,
    account_name = excluded.account_name
WHERE
    institution_id <> excluded.institution_id
    OR tax_shelter_type_id <> excluded.tax_shelter_type_id
    OR currency_id <> excluded.currency_id
    OR min_balance_waiver <> excluded.min_balance_waiver
    OR inactive_fee_months <> excluded.inactive_fee_months
    OR account_name <> excluded.account_name
"#,
            record.account_type,
            record.institution_name,
            record.tax_shelter_type,
            record.currency,
            record.min_balance_waiver,
            record.inactive_fee_months,
            record.account_name
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
            Err(err) => {
                return Err(format!("{}, record: {:?}", err.to_string(), record).into());
            }
        }
    }

    pub async fn has_cash_account(&mut self, account_type: &str) -> SqlResult<bool> {
        let result = sqlx::query!(
            r#"
SELECT
    EXISTS (
        SELECT
            *
        FROM
            CashAccountProduct
            INNER JOIN AccountType USING (account_type_id)
        WHERE
            account_type = ?
    ) AS "is_account_exist!:bool";
"#,
            account_type
        )
        .fetch_one(&mut *self.0)
        .await?;

        Ok(result.is_account_exist)
    }
}
