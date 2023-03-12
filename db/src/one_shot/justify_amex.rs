use sqlx::Row;

use crate::Transaction;

pub struct JustifyAmex {
    pub year: i64,
    pub month: i64,
    pub balance: f64,
    pub extra_cashback_rate: f64,
    pub with_amex_cashback: f64,
    pub without_amex_cashback: f64,
    pub extra_cashback: f64,
    pub extra_cashback_after_fee: f64,
    pub missed_opportunities: f64,
}

impl Transaction<'_> {
    pub async fn justify_amex(
        &mut self,
        num_days: u32,
    ) -> Result<Vec<JustifyAmex>, Box<dyn std::error::Error>> {
        const AVG_DAYS_IN_MONTH: f64 = 30.437;
        let num_months = (num_days as f64 / AVG_DAYS_IN_MONTH).ceil();

        let records = sqlx::query(
            r#"
SELECT
    year,
    month,
    balance,
    extra_cashback_rate,
    with_amex_cashback,
    without_amex_cashback,
    extra_cashback,
    extra_cashback_after_fee,
    missed_opportunities
FROM
    JustifyAmex;
"#,
        )
        .bind(num_months)
        .bind(num_days)
        .fetch_all(&mut *self.0)
        .await?
        .into_iter()
        .map(|row| JustifyAmex {
            year: row.get(0),
            month: row.get(1),
            balance: row.get(2),
            extra_cashback_rate: row.get(3),
            with_amex_cashback: row.get(4),
            without_amex_cashback: row.get(5),
            extra_cashback: row.get(6),
            extra_cashback_after_fee: row.get(7),
            missed_opportunities: row.get(8),
        })
        .collect();

        Ok(records)
    }
}
