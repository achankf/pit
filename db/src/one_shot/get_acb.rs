use std::ops::Range;

use chrono::{DateTime, Local};
use serde::Deserialize;
use sqlx::{sqlite::SqliteRow, Row};

use crate::{SqlResult, Transaction};

#[derive(Clone, Deserialize, Debug)]
pub struct Acb {
    pub first_name: String,
    pub last_name: String,
    pub account_name: String,
    pub ticker: String,
    pub acc_units: f64,
    pub avg_price: f64,
    pub acb: f64,
    pub capital_gl: f64,
    pub person_id: i64,
    pub is_taxable: bool,
}

impl From<SqliteRow> for Acb {
    fn from(row: SqliteRow) -> Self {
        Self {
            first_name: row.get(0),
            last_name: row.get(1),
            account_name: row.get(2),
            ticker: row.get(3),
            acc_units: row.get(4),
            avg_price: row.get(5),
            acb: row.get(6),
            capital_gl: row.get(7),
            person_id: row.get(8),
            is_taxable: row.get(9),
        }
    }
}

impl<'c> Transaction<'c> {
    pub async fn get_acb(&mut self, range: Range<DateTime<Local>>) -> SqlResult<Vec<Acb>> {
        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let rows = sqlx::query(
            r#"
WITH LatestRow AS (
    SELECT
        person_id,
        general_account_name_id,
        security_id,
        SUM(capital_gl) AS capital_gl_sum,
        MAX(chronological_order) AS chronological_order
    FROM
        Acb
         WHERE
         date BETWEEN ?
         AND ?
    GROUP BY
        person_id,
        general_account_name_id,
        security_id
)
SELECT
    first_name,
    last_name,
    general_account_name AS account_name,
    ticker,
    acc_units,
    ROUND(acb / acc_units, 2) AS avg_price,
    acb,
    capital_gl_sum AS capital_gl,
    person_id,
    CASE
        WHEN tax_shelter_type_id = (
            SELECT
                tax_shelter_type_id
            FROM
                TaxShelterType
            WHERE
                tax_shelter_type = 'NON-REGISTERED'
        ) THEN TRUE
        ELSE FALSE
    END AS is_taxable
FROM
    Acb
    INNER JOIN LatestRow USING (
        person_id,
        general_account_name_id,
        security_id,
        chronological_order
    )
    INNER JOIN SECURITY USING (security_id)
    INNER JOIN Person USING (person_id)
    INNER JOIN CashAccount USING (general_account_name_id)
    INNER JOIN  GeneralAccountName USING (general_account_name_id)
ORDER BY
    person_id,
    ticker,
    group_order;
"#,
        )
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&mut *self.0)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
