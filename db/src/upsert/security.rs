use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct Security {
    #[serde(deserialize_with = "string_trim")]
    pub exchange_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub currency: String,
    #[serde(deserialize_with = "string_trim")]
    pub ticker: String,
    #[serde(deserialize_with = "string_trim")]
    pub security_name: String,
    pub price: f64,
}

impl Id for Security {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.ticker.to_string()
    }
}

impl Query for Security {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    SECURITY (
        exchange_id,
        currency_id,
        ticker,
        security_name,
        price
    )
VALUES
    (
        (
            SELECT
                exchange_id
            FROM
                Exchange
            WHERE
                exchange_key = ?
        ),
        (
            SELECT
                currency_id
            FROM
                Currency
            WHERE
                currency = ?
        ),
        ?,
        ?,
        ?
    ) ON CONFLICT(ticker) DO
UPDATE
SET
    exchange_id = excluded.exchange_id,
    security_name = excluded.security_name,
    currency_id = excluded.currency_id,
    price = excluded.price
WHERE
    exchange_id <> excluded.exchange_id
    OR security_name <> excluded.security_name
    OR currency_id <> excluded.currency_id
    OR price <> excluded.price
"#,
            self.exchange_key,
            self.currency,
            self.ticker,
            self.security_name,
            self.price
        )
    }
}
