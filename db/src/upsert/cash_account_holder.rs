use std::path::PathBuf;

use common::{bool_from_str, deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::string_trim;
use sqlx::Sqlite;

use crate::{SqlResult, Transaction};

#[derive(Debug, Deserialize)]
pub struct CashAccountHolder {
    #[serde(deserialize_with = "string_trim")]
    pub person_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
    pub emergency_target: f64,
    #[serde(deserialize_with = "bool_from_str")]
    pub is_closed: bool,
}

impl Id for CashAccountHolder {
    type IdType = (String, String);

    fn id(&self) -> Self::IdType {
        (self.person_key.clone(), self.account_type.clone())
    }
}

impl CashAccountHolder {
    pub fn to_account_key(&self) -> String {
        format!("{}-{}", self.person_key, self.account_type).to_uppercase()
    }
}

impl Transaction<'_> {
    pub async fn upsert_cash_account_holder(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<CashAccountHolder>(csv_path).await?;

        for (_, record) in &parsed_records {
            let account_key = record.to_account_key();

            println!(
                "> inserting supplementary accounts into Account {}",
                account_key
            );
            self.upsert_cash_account_helper(record, &account_key)
                .await?;

            println!("> inserting account holder");
            let cash_account_holder_id = self.upsert_cash_account_holder_helper(record).await?;

            if let Some(cash_account_holder_id) = cash_account_holder_id {
                println!("> upsert mapping");
                self.upsert_cash_account_mapping_helper(cash_account_holder_id, &account_key)
                    .await?;
            }
        }

        Ok(())
    }

    async fn upsert_cash_account_holder_helper(
        &mut self,
        record: &CashAccountHolder,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
INSERT INTO
    CashAccountHolder (
        person_id,
        account_type_id,
        emergency_target,
        is_closed
    )
VALUES
    (
        (
            SELECT
                person_id
            FROM
                Person
            WHERE
                person_key = ?
        ),
        (
            SELECT
                account_type_id
            FROM
                AccountType
            WHERE
                account_type = ?
        ),
        ?,
        ?
    ) ON CONFLICT(person_id, account_type_id) DO
UPDATE
SET
    emergency_target = excluded.emergency_target,
    is_closed = excluded.is_closed
WHERE
    emergency_target <> excluded.emergency_target
    OR is_closed <> excluded.is_closed
"#,
            record.person_key,
            record.account_type,
            record.emergency_target,
            record.is_closed
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

                    Ok(None)
                } else {
                    Ok(Some(result.last_insert_rowid()))
                }
            }
            Err(err) => {
                return Err(format!("{}, record: {:?}", err.to_string(), record).into());
            }
        }
    }

    async fn upsert_cash_account_mapping_helper(
        &mut self,

        cash_account_holder_id: i64,
        account_key: &str,
    ) -> SqlResult<<Sqlite as sqlx::Database>::QueryResult> {
        let search_term = format!("{account_key}%");

        sqlx::query!(
            r#"
INSERT INTO
    CashAccountEntry (
        cash_account_holder_id,
        account_subtype_id,
        account_id
    )
SELECT
    ?,
    account_subtype_id,
    account_id
FROM
    Account
WHERE
    account_key LIKE ? ON CONFLICT(cash_account_holder_id, account_subtype_id) DO
UPDATE
SET
    account_id = excluded.account_id
WHERE
    account_id <> excluded.account_id
"#,
            cash_account_holder_id,
            search_term
        )
        .execute(&mut *self.0)
        .await
    }

    async fn upsert_cash_account_helper(
        &mut self,
        record: &CashAccountHolder,
        account_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
INSERT INTO
    Account (
        account_key,
        account_type_id,
        account_subtype_id,
        account_name
    ) WITH BaseRecord (account_key, account_type_id) AS (
        SELECT
            ?,
            (
                SELECT
                    account_type_id
                FROM
                    AccountType
                WHERE
                    account_type = ?
            )
    ),
    WithAccountName AS (
        SELECT
            *,
            (
                SELECT
                    account_name
                FROM
                    BaseRecord
                    INNER JOIN CashAccountProduct USING(account_type_id)
            ) AS account_name
        FROM
            BaseRecord
    )
SELECT
    account_key || '-' || account_subtype,
    account_type_id,
    account_subtype_id,
    account_name || ', ' || account_subtype
FROM
    WithAccountName
    CROSS JOIN CashAccountEntryType
    INNER JOIN AccountSubtype USING (account_subtype_id) ON CONFLICT(account_key) DO
UPDATE
SET
    account_type_id = excluded.account_type_id,
    account_subtype_id = excluded.account_subtype_id,
    account_name = excluded.account_name
WHERE
    account_type_id <> excluded.account_type_id
    OR account_subtype_id <> excluded.account_subtype_id
    OR account_name <> excluded.account_name
"#,
            account_key,
            record.account_type,
        )
        .execute(&mut *self.0)
        .await;

        match result {
            Ok(result) => {
                // cash, fees, interest, open balance (digit & fiat), withholding tax, bonus, forex (revenue, expense)
                const NUM_SUBTYPE_ACCOUNTS: u64 = 9;

                if result.rows_affected() != NUM_SUBTYPE_ACCOUNTS {
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
