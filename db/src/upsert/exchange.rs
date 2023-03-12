use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Exchange {
    #[serde(deserialize_with = "string_trim")]
    pub exchange_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub exchange_name: String,
}

impl Id for Exchange {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.exchange_key.clone()
    }
}

impl Query for Exchange {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    Exchange (
        exchange_key,
        exchange_name
    )
VALUES
    (
        ?,
        ?
    ) ON CONFLICT(exchange_key) DO
UPDATE
SET
    exchange_name = excluded.exchange_name
WHERE
    exchange_name <> excluded.exchange_name
"#,
            self.exchange_key,
            self.exchange_name
        )
    }
}
