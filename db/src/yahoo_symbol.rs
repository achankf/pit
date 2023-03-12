use std::collections::HashMap;

use crate::{SqlResult, Transaction};

pub enum TickerId {
    Security(i64),
    Currency(i64),
}

impl<'c> Transaction<'c> {
    pub async fn get_yahoo_symbols(&mut self) -> SqlResult<HashMap<String, TickerId>> {
        struct RawData {
            pub id: i64,
            pub yahoo_ticker: String,
        }

        let result = sqlx::query_as!(
            RawData,
            r#"
SELECT
    security_id AS id,
    yahoo_ticker
FROM
    SECURITY
"#
        )
        .fetch_all(&mut *self.0)
        .await?
        .into_iter()
        .map(|ticker| (ticker.yahoo_ticker, TickerId::Security(ticker.id)))
        .chain(
            sqlx::query_as!(
                RawData,
                r#"
SELECT
    currency_id AS id,
    yahoo_ticker AS "yahoo_ticker!:String"
FROM
    Currency
WHERE
    yahoo_ticker IS NOT NULL
"#
            )
            .fetch_all(&mut *self.0)
            .await?
            .into_iter()
            .map(|ticker| (ticker.yahoo_ticker, TickerId::Currency(ticker.id))),
        )
        .collect();

        Ok(result)
    }
}
