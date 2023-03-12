use crate::{SqlResult, Transaction};

impl Transaction<'_> {
    pub async fn get_next_transaction_id(&mut self) -> SqlResult<i64> {
        struct Record {
            transaction_id: i64,
        }

        let record = sqlx::query_as!(
            Record,
            r#"
SELECT
    COALESCE(MAX(transaction_id), 0) + 1 AS transaction_id
FROM
    FinancialEntry;
"#
        )
        .fetch_one(&mut *self.0)
        .await?;

        Ok(record.transaction_id)
    }
}
