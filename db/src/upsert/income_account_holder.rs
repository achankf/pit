use std::path::PathBuf;

use common::{deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Transaction;

#[derive(Debug, Deserialize)]
pub struct IncomeAccountHolder {
    #[serde(deserialize_with = "string_trim")]
    pub person_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
}

impl Id for IncomeAccountHolder {
    type IdType = (String, String);

    fn id(&self) -> Self::IdType {
        (self.person_key.clone(), self.account_type.clone())
    }
}

impl IncomeAccountHolder {
    pub fn get_account_key(&self) -> String {
        format!("{}-{}", self.person_key, self.account_type,)
    }
}

impl Transaction<'_> {
    pub async fn upsert_income_account_holder(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<IncomeAccountHolder>(csv_path).await?;

        for (_, record) in &parsed_records {
            let account_key = record.get_account_key();

            println!("> inserting income account {}", account_key.yellow());
            self.upsert_income_account_helper(&account_key, &record)
                .await?;

            println!("> inserting income account holder {}", account_key.yellow());
            let id = self.upsert_income_account_holder_helper(record).await?;

            if let Some(income_account_holder_id) = id {
                println!(
                    "> inserting income account mapping {}",
                    account_key.yellow()
                );
                self.upsert_income_account_mapping_helper(income_account_holder_id, &account_key)
                    .await?
            }
        }

        Ok(())
    }

    async fn upsert_income_account_helper(
        &mut self,
        account_key: &str,
        record: &IncomeAccountHolder,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
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
                    INNER JOIN IncomeAccount USING(account_type_id)
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
    CROSS JOIN IncomeAccountSubtype
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
                // CPP, EI, group benefits, salary, income tax
                const NUM_SUBTYPE_ACCOUNTS: u64 = 5;

                if result.rows_affected() != NUM_SUBTYPE_ACCOUNTS {
                    println!(
                        "{}: no row was affected, record: {}, {}",
                        "Warning".bold().yellow(),
                        account_key,
                        record.account_type
                    );

                    Ok(None)
                } else {
                    Ok(Some(result.last_insert_rowid()))
                }
            }
            Err(err) => Err(format!(
                "{}, record: {}, {}",
                err.to_string(),
                account_key,
                record.account_type
            )
            .into()),
        }
    }

    async fn upsert_income_account_holder_helper(
        &mut self,
        record: &IncomeAccountHolder,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO IncomeAccountHolder (
        person_id,
        account_type_id
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
        )
    )
"#,
            record.person_key,
            record.account_type,
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

    async fn upsert_income_account_mapping_helper(
        &mut self,
        income_account_holder_id: i64,
        account_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let search_term = format!("{account_key}%");

        sqlx::query!(
            r#"
INSERT INTO
    IncomeAccountMapping (
        income_account_holder_id,
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
    account_key LIKE ? ON CONFLICT(income_account_holder_id, account_subtype_id) DO
UPDATE
SET
    account_id = excluded.account_id
WHERE
    account_id <> excluded.account_id
"#,
            income_account_holder_id,
            search_term
        )
        .execute(&mut *self.0)
        .await?;

        Ok(())
    }
}
