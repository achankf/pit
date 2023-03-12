use std::ops::Range;

use chrono::{DateTime, Local};
use serde::Deserialize;
use sqlx::Row;

use crate::{SqlResult, Transaction};

#[derive(Deserialize, Debug)]
pub struct TransactionByAccountKey {
    pub transaction_id: i64,
    pub item_id: i64,
    pub date: DateTime<Local>,
    pub unit: f64,
    pub debit: Option<f64>,
    pub credit: Option<f64>,
    pub exchange_rate: Option<f64>,
    pub total_amount: f64,
    pub description: String,
}

impl Transaction<'_> {
    pub async fn get_transaction_by_account_key(
        &mut self,
        account_key: &str,
        range: Range<DateTime<Local>>,
    ) -> SqlResult<Vec<TransactionByAccountKey>> {
        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let result = sqlx::query(
            r#"
SELECT
    transaction_id,
    item_id,
    date AS "date!:DateTime<Local>",
    unit AS "unit!:f64",
    debit,
    credit,
    exchange_rate AS "exchange_rate:f64",
    ROUND(
        ROUND(
            (
                CASE
                    WHEN account_kind IN ('ASSET', 'EXPENSE') THEN COALESCE(debit, 0) - COALESCE(credit, 0)
                    ELSE COALESCE(credit, 0) - COALESCE(debit, 0)
                END
            ) * unit,
            2
        ) * COALESCE(exchange_rate, 1.0),
        2
    ) AS "total_amount!:f64",
    description
FROM
    FinancialEntry
    INNER JOIN Account USING (account_id)
    INNER JOIN AccountSubtype USING(account_subtype_id)
    INNER JOIN AccountKind USING(account_kind_id)
    LEFT JOIN TransactionForex USING (transaction_id)
WHERE
    account_key = ?
    AND date BETWEEN ?
    AND ?
"#)
        .bind(account_key)
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&mut *self.0)
        .await?.into_iter().map(|record| TransactionByAccountKey {
            transaction_id: record.get(0),
            item_id: record.get(1),
            date: record.get(2),
            unit: record.get(3),
            debit: record.get(4),
            credit: record.get(5),
            exchange_rate: record.get(6),
            total_amount: record.get(7),
            description: record.get(8),
        }).collect();

        Ok(result)
    }
}
