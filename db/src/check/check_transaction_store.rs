use crate::{SqlResult, Transaction};

pub struct CheckTransactionStore {
    pub transaction_id: i64,
    pub description: String,
}

impl Transaction<'_> {
    pub async fn check_transaction_store(&mut self) -> SqlResult<Vec<CheckTransactionStore>> {
        let records = sqlx::query_as!(
            CheckTransactionStore,
            r#"
SELECT
    transaction_id,
    description
FROM
    CashbackTransaction
WHERE
    transaction_id NOT IN (
        SELECT
            transaction_id
        FROM
            TransactionStore
    );
"#
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(records)
    }
}
