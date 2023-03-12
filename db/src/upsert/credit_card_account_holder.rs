use std::path::PathBuf;

use common::{bool_from_str, deserialize_into_map, Id};
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_trim::{option_string_trim, string_trim};

use crate::{SqlResult, Transaction};

#[derive(Debug, Deserialize)]
pub struct CreditCardHolder {
    #[serde(deserialize_with = "string_trim")]
    pub person_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
    #[serde(deserialize_with = "option_string_trim")]
    pub pad_source: Option<String>,
    #[serde(deserialize_with = "bool_from_str")]
    pub is_closed: bool,
}

impl Id for CreditCardHolder {
    type IdType = (String, String);

    fn id(&self) -> Self::IdType {
        (self.person_key.clone(), self.account_type.clone())
    }
}

impl CreditCardHolder {
    pub fn get_account_key(&self) -> String {
        format!("{}-{}", self.person_key, self.account_type)
    }
}

impl Transaction<'_> {
    pub async fn upsert_credit_card_holder(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("updating data with {}", csv_path.to_string_lossy());

        let parsed_records = deserialize_into_map::<CreditCardHolder>(csv_path).await?;

        for (_, record) in &parsed_records {
            let account_key = record.get_account_key();

            println!("> inserting credit card account {account_key}");
            self.upsert_credit_card_account_helper(&account_key, &record.account_type)
                .await?;

            println!("> inserting credit card account holder");
            let id = self
                .upsert_credit_card_account_holder_helper(record)
                .await?;

            if let Some(credit_card_holder_id) = id {
                println!("> inserting credit card account mapping");
                self.upsert_credit_card_account_mapping_helper(credit_card_holder_id, &account_key)
                    .await?;

                println!("> inserting credit card PAD source");
                self.upsert_credit_card_pad_source(credit_card_holder_id, &record.pad_source)
                    .await?;
            }
        }

        Ok(())
    }

    async fn upsert_credit_card_pad_source(
        &mut self,
        credit_card_holder_id: i64,
        pad_source: &Option<String>,
    ) -> SqlResult<()> {
        if let Some(pad_source) = pad_source {
            sqlx::query!(
                r#"
INSERT INTO
    CreditCardPadSource (credit_card_holder_id, cash_account_holder_id)
VALUES
    (
        ?,
        (
            SELECT
                cash_account_holder_id
            FROM
                Account
                INNER JOIN CashAccountEntry USING (account_id)
            WHERE
                account_key = ?
        )
    ) ON CONFLICT (credit_card_holder_id) DO
UPDATE
SET
    cash_account_holder_id = excluded.cash_account_holder_id
WHERE
    cash_account_holder_id <> excluded.cash_account_holder_id
"#,
                credit_card_holder_id,
                pad_source
            )
            .execute(&mut *self.0)
            .await?;
        } else {
            sqlx::query!(
                r#"
DELETE FROM
    CreditCardPadSource
WHERE
    credit_card_holder_id = ?
"#,
                credit_card_holder_id
            )
            .execute(&mut *self.0)
            .await?;
        }

        Ok(())
    }

    async fn upsert_credit_card_account_helper(
        &mut self,
        account_key: &str,
        account_type: &str,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO Account (
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
                    INNER JOIN CreditCardProduct USING(account_type_id)
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
    CROSS JOIN CreditCardEntryType
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
            account_type,
        )
        .execute(&mut *self.0)
        .await;

        match result {
            Ok(result) => {
                // debt, fees, interest (debt), cashback, open balance, bonus
                const NUM_SUBTYPE_ACCOUNTS: u64 = 6;

                if result.rows_affected() != NUM_SUBTYPE_ACCOUNTS {
                    println!(
                        "{}: no row was affected, record: {}, {}",
                        "Warning".bold().yellow(),
                        account_key,
                        account_type
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
                account_type
            )
            .into()),
        }
    }

    async fn upsert_credit_card_account_holder_helper(
        &mut self,
        record: &CreditCardHolder,
    ) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
INSERT INTO
    CreditCardHolder (
        person_id,
        account_type_id,
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
        ?
    ) ON CONFLICT(person_id, account_type_id) DO
UPDATE
SET
    is_closed = excluded.is_closed
WHERE
    is_closed <> excluded.is_closed
"#,
            record.person_key,
            record.account_type,
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

    async fn upsert_credit_card_account_mapping_helper(
        &mut self,
        credit_card_holder_id: i64,
        account_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let search_term = format!("{account_key}%");

        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO CreditCardEntry (
        credit_card_holder_id,
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
    account_key LIKE ? ON CONFLICT(credit_card_holder_id, account_subtype_id) DO
UPDATE
SET
    account_id = excluded.account_id
WHERE
    account_id <> excluded.account_id
"#,
            credit_card_holder_id,
            search_term
        )
        .execute(&mut *self.0)
        .await?;

        Ok(())
    }
}
