use common::{excel_date_optional_time_format, Id};
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct FinancialEntry {
    pub transaction_id: i64,
    pub item_id: i64,
    #[serde(deserialize_with = "excel_date_optional_time_format")]
    pub date: i64,
    #[serde(deserialize_with = "string_trim")]
    pub account_key: String,
    pub unit: f64,
    pub debit: Option<f64>,
    pub credit: Option<f64>,
    #[serde(deserialize_with = "string_trim")]
    pub description: String,
}

impl Id for FinancialEntry {
    type IdType = (i64, i64);

    fn id(&self) -> (i64, i64) {
        (self.transaction_id, self.item_id)
    }
}

impl Query for FinancialEntry {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    FinancialEntry (
        transaction_id,
        item_id,
        date,
        account_id,
        unit,
        debit,
        credit,
        description
    )
VALUES
    (
        ?,
        ?,
        ?,
        (
            SELECT
                account_id
            FROM
                Account
            WHERE
                account_key = ?
        ),
        ?,
        ?,
        ?,
        ?
    ) ON CONFLICT (transaction_id, item_id) DO
UPDATE
SET
    date = excluded.date,
    account_id = excluded.account_id,
    unit = excluded.unit,
    debit = excluded.debit,
    credit = excluded.credit,
    description = excluded.description
WHERE
    date <> excluded.date
    OR account_id <> excluded.account_id
    OR unit <> excluded.unit
    OR debit <> excluded.debit
    OR credit <> excluded.credit
    OR description <> excluded.description;
"#,
            self.transaction_id,
            self.item_id,
            self.date,
            self.account_key,
            self.unit,
            self.debit,
            self.credit,
            self.description
        )
    }
}
