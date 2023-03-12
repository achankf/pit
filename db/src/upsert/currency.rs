use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Currency {
    pub currency: String,
    pub currency_name: String,
    pub currency_symbol: Option<String>,
    pub yahoo_ticker: Option<String>,
}

impl Id for Currency {
    type IdType = String;

    fn id(&self) -> String {
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
        yahoo_ticker
    )
VALUES
    (?, ?, ?, ?) ON CONFLICT(currency) DO
UPDATE
SET
    currency_name = excluded.currency_name,
    currency_symbol = excluded.currency_symbol,
    yahoo_ticker = excluded.yahoo_ticker
"#,
            self.currency,
            self.currency_name,
            self.currency_symbol,
            self.yahoo_ticker
        )
    }
}
