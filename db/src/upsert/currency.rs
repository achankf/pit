use common::Id;
use serde::Deserialize;
use serde_trim::option_string_trim;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Currency {
    #[serde(deserialize_with = "string_trim")]
    pub currency: String,
    #[serde(deserialize_with = "string_trim")]
    pub currency_name: String,
    #[serde(deserialize_with = "option_string_trim")]
    pub currency_symbol: Option<String>,
    #[serde(deserialize_with = "option_string_trim")]
    pub market_exchange_rate: Option<String>,
}

impl Id for Currency {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.currency.clone()
    }
}

impl Query for Currency {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    Currency (
        currency,
        currency_name,
        currency_symbol,
        market_exchange_rate
    )
VALUES
    (?, ?, ?, ?) ON CONFLICT(currency) DO
UPDATE
SET
    currency_name = excluded.currency_name,
    currency_symbol = excluded.currency_symbol,
    market_exchange_rate = excluded.market_exchange_rate
WHERE
    currency_name <> excluded.currency_name
    OR currency_symbol <> excluded.currency_symbol
    OR market_exchange_rate <> excluded.market_exchange_rate
"#,
            self.currency,
            self.currency_name,
            self.currency_symbol,
            self.market_exchange_rate
        )
    }
}
