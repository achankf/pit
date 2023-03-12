use std::path::PathBuf;

use chrono::NaiveDate;
use common::{deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Transaction;

#[derive(Debug, Deserialize)]
pub struct GicAccountHolder {
    #[serde(deserialize_with = "string_trim")]
    pub person_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
    pub issue_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub apr: f64,
}

impl Id for GicAccountHolder {
    type IdType = (String, String, NaiveDate, NaiveDate, String);

    fn id(&self) -> Self::IdType {
        (
            self.person_key.clone(),
            self.account_type.clone(),
            self.issue_date.clone(),
            self.maturity_date.clone(),
            self.apr.to_string(),
        )
    }
}

impl GicAccountHolder {
    pub fn get_account_key(&self) -> String {
        format!(
            "{}-{}-{}-{}-{:.2}%",
            self.person_key,
            self.account_type,
            self.issue_date,
            self.maturity_date,
            self.apr * 100.0
        )
    }
}

impl Transaction<'_> {
    pub async fn upsert_gic_account_holder(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<GicAccountHolder>(csv_path).await?;

        for (_, record) in &parsed_records {
            let account_key = record.get_account_key();

            println!("> inserting GIC account {}", account_key.yellow());
            self.upsert_gic_account_helper(&account_key, &record)
                .await?;

            println!("> inserting GIC account holder {}", account_key.yellow());
            let id = self.upsert_gic_account_holder_helper(record).await?;

            if let Some(gic_account_holder_id) = id {
                println!("> inserting GIC account mapping {}", account_key.yellow());
                self.upsert_gic_account_mapping_helper(gic_account_holder_id, &account_key)
                    .await?
            }
        }

        Ok(())
    }

    async fn upsert_gic_account_helper(
        &mut self,
        account_key: &str,
        record: &GicAccountHolder,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        let apr = format!("{:.2}%", record.apr * 100.0);

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
    account_name || ', APR=' || ? || ', ' || account_subtype
FROM
    WithAccountName
    CROSS JOIN GicAccountSubtype
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
            apr
        )
        .execute(&mut *self.0)
        .await;

        match result {
            Ok(result) => {
                // principal, interest, open balance
                const NUM_SUBTYPE_ACCOUNTS: u64 = 3;

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

    async fn upsert_gic_account_holder_helper(
        &mut self,
        record: &GicAccountHolder,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO GicAccountHolder (
        person_id,
        account_type_id,
        issue_date,
        maturity_date,
        apr
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
        ?,
        ?
    )
"#,
            record.person_key,
            record.account_type,
            record.issue_date,
            record.maturity_date,
            record.apr
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

    async fn upsert_gic_account_mapping_helper(
        &mut self,
        gic_account_holder_id: i64,
        account_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let search_term = format!("{account_key}%");

        sqlx::query!(
            r#"
INSERT INTO
    GicEntry (
        gic_account_holder_id,
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
    account_key LIKE ? ON CONFLICT(gic_account_holder_id, account_subtype_id) DO
UPDATE
SET
    account_id = excluded.account_id
WHERE
    account_id <> excluded.account_id
"#,
            gic_account_holder_id,
            search_term
        )
        .execute(&mut *self.0)
        .await?;

        Ok(())
    }
}
