use common::{excel_date_optional_time_format, trim_string, Id};
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct GeneralTransaction {
    pub row_id: i64,
    pub person: String,
    #[serde(deserialize_with = "excel_date_optional_time_format")]
    pub date: i64,
    pub transaction_id: i64,
    pub general_account_key: String,
    pub currency: String,
    pub unit: f64,
    pub debit: Option<f64>,
    pub credit: Option<f64>,
    pub book_exchange_rate: f64,
    #[serde(deserialize_with = "trim_string")]
    pub description: String,
}

impl Id for GeneralTransaction {
    type IdType = i64;

    fn id(&self) -> i64 {
        self.row_id
    }
}

impl Query for GeneralTransaction {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO GeneralTransaction (
        row_id,
        person_id,
        date,
        transaction_id,
        general_account_id,
        currency_id,
        unit,
        debit,
        credit,
        book_exchange_rate,
        description
    )
VALUES
    (
        ?,
        (
            SELECT
                person_id
            FROM
                Person
            WHERE
                short_name = ?
        ),
        ?,
        ?,
        (
            SELECT
                general_account_id
            FROM
                GeneralAccount
            WHERE
                general_account_key = ?
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
        ?,
        ?,
        ?
    );
"#,
            self.row_id,
            self.person,
            self.date,
            self.transaction_id,
            self.general_account_key,
            self.currency,
            self.unit,
            self.debit,
            self.credit,
            self.book_exchange_rate,
            self.description
        )
    }
}
