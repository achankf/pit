use std::ops::Range;

use chrono::{DateTime, Local};
use serde::Deserialize;
use sqlx::{sqlite::SqliteRow, Row};

use crate::{SqlResult, Transaction};

#[derive(Clone, Deserialize, Debug)]
pub struct Acb {
    pub first_name: String,
    pub last_name: String,
    pub ticker: String,
    pub acc_units: f64,
    pub avg_price: f64,
    pub acb: f64,
    pub total_distribution: f64,
    pub total_capital_gl: f64,
    pub person_id: i64,
}

impl From<SqliteRow> for Acb {
    fn from(row: SqliteRow) -> Self {
        Self {
            first_name: row.get(0),
            last_name: row.get(1),
            ticker: row.get(2),
            acc_units: row.get(3),
            avg_price: row.get(4),
            acb: row.get(5),
            total_distribution: row.get(6),
            total_capital_gl: row.get(7),
            person_id: row.get(8),
        }
    }
}

impl Transaction<'_> {
    pub async fn get_acb(&mut self, range: Range<DateTime<Local>>) -> SqlResult<Vec<Acb>> {
        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let rows = sqlx::query(
            r#"
WITH CapitalGl AS (
    SELECT
        person_id,
        security_id,
        SUM(distribution) AS total_distribution,
        SUM(capital_gl) AS total_capital_gl
    FROM
        Acb
    WHERE
        date BETWEEN ?
        AND ?
    GROUP BY
        person_id,
        security_id
)
SELECT
    first_name,
    last_name,
    ticker,
    acc_units,
    COALESCE(ROUND(acb / acc_units, 2), 0.0) AS avg_price,
    COALESCE(acb, 0.0),
    total_distribution,
    total_capital_gl,
    person_id
FROM
    Acb
    INNER JOIN SECURITY USING (security_id)
    LEFT JOIN CapitalGl USING (person_id, security_id)
    INNER JOIN (
        SELECT
            person_id,
            security_id,
            max(group_order) AS group_order
        FROM
            Acb
        WHERE
            date <= ?
        GROUP BY
            person_id,
            security_id
    ) USING (person_id, security_id, group_order)
    INNER JOIN Person USING (person_id)
ORDER BY
    person_id,
    ticker;
"#,
        )
        .bind(start_ts)
        .bind(end_ts)
        .bind(end_ts)
        .fetch_all(&mut *self.0)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
