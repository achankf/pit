use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct TransactionStore {
    transaction_id: i64,
    #[serde(deserialize_with = "string_trim")]
    store_key: String,
}

impl Id for TransactionStore {
    type IdType = i64;

    fn id(&self) -> Self::IdType {
        self.transaction_id
    }
}

impl Query for TransactionStore {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    TransactionStore (store_id, transaction_id)
VALUES
    (
        (
            SELECT
                store_id
            FROM
                Store
            WHERE
                store_key = ?
        ),
        ?
    ) ON CONFLICT(transaction_id) DO
UPDATE
SET
    store_id = excluded.store_id
WHERE
    store_id <> excluded.store_id
"#,
            self.store_key,
            self.transaction_id
        )
    }
}
