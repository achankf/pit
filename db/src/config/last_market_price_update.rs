use chrono::{DateTime, Local};

use crate::{SqlResult, Transaction};

impl<'a> Transaction<'a> {
    pub async fn get_last_market_price_update(&mut self) -> SqlResult<DateTime<Local>> {
        struct LastUpdate {
            date: DateTime<Local>,
        }

        let result = sqlx::query_as!(
            LastUpdate,
            r#"
SELECT
    date AS "date!:DateTime<Local>"
FROM
    LastMarketPriceUpdate
"#
        )
        .fetch_one(&mut *self.0)
        .await?;

        Ok(result.date)
    }

    pub async fn update_last_market_price_update(&mut self) -> SqlResult<()> {
        sqlx::query!(
            r#"
UPDATE
    LastMarketPriceUpdate
SET
    date = UNIXEPOCH()
"#
        )
        .execute(&mut *self.0)
        .await?;

        Ok(())
    }
}
