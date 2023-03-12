use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct Security {
    pub exchange_name: String,
    pub currency: String,
    pub ticker: String,
    pub yahoo_ticker: String,
    pub long_name: String,
}

impl Id for Security {
    type IdType = String;

    fn id(&self) -> String {
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
        yahoo_ticker,
        long_name
    )
VALUES
    (
        (
            SELECT
                exchange_id
            FROM
                Exchange
            WHERE
                exchange_name = ?
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
    long_name = excluded.long_name,
    yahoo_ticker = excluded.yahoo_ticker,
    currency_id = excluded.currency_id
"#,
            self.exchange_name,
            self.currency,
            self.ticker,
            self.yahoo_ticker,
            self.long_name,
        )
    }
}
