use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Store {
    #[serde(deserialize_with = "string_trim")]
    pub store_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub store_name: String,
}

impl Id for Store {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.store_key.clone()
    }
}

impl Query for Store {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    Store (
        store_key,
        store_name
    )
VALUES
    (
        ?,
        ?
    ) ON CONFLICT(store_key) DO
UPDATE
SET
    store_name = excluded.store_name
WHERE
    store_name <> excluded.store_name
"#,
            self.store_key,
            self.store_name
        )
    }
}
