use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Exchange {
    pub exchange_name: String,
    pub long_name: String,
}

impl Id for Exchange {
    type IdType = String;

    fn id(&self) -> String {
        self.exchange_name.clone()
    }
}

impl Query for Exchange {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    Exchange (
        exchange_name,
        long_name
    )
VALUES
    (
        ?,
        ?
    ) ON CONFLICT(exchange_name) DO
UPDATE
SET
    exchange_name = excluded.exchange_name,
    long_name = excluded.long_name
"#,
            self.exchange_name,
            self.long_name
        )
    }
}
