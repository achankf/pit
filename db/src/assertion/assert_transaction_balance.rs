use crate::{SqlResult, Transaction};

pub struct AssertTransactionBalance {
    pub transaction_id: i64,
    pub debit: f64,
    pub credit: f64,
    pub balance: f64,
}

impl<'c> Transaction<'c> {
    pub async fn assert_transaction_balance(&mut self) -> SqlResult<Vec<AssertTransactionBalance>> {
        let results = sqlx::query_as!(
            AssertTransactionBalance,
            r#"
SELECT
    transaction_id,
    SUM(ROUND(unit * COALESCE(debit, 0), 2)) AS "debit!:f64",
    SUM(ROUND(unit * COALESCE(credit, 0), 2)) AS "credit!:f64",
    ROUND(SUM(ROUND((unit * COALESCE(debit, 0)), 2)) - SUM(ROUND((unit * COALESCE(credit, 0)), 2)), 2) AS "balance!:f64"
FROM
    GeneralTransaction
GROUP BY
    transaction_id
HAVING
    "balance!:f64" <> 0;
    "#
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(results)
    }
}
