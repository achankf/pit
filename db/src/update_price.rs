use crate::{SqlResult, Transaction};

impl Transaction<'_> {
    pub async fn update_security_price(&mut self, security_id: i64, price: f64) -> SqlResult<()> {
        let result = sqlx::query!(
            "UPDATE Security SET price = ? WHERE security_id = ?",
            price,
            security_id
        )
        .execute(&mut *self.0)
        .await?;

        assert_eq!(
            result.rows_affected(),
            1,
            "sanity check: was not able to update price"
        );

        Ok(())
    }

    pub async fn update_currency_exchange_rate(
        &mut self,
        currency_id: i64,
        exchange_rate: f64,
    ) -> SqlResult<()> {
        let result = sqlx::query!(
            "UPDATE Currency SET market_exchange_rate = ? WHERE currency_id = ?",
            exchange_rate,
            currency_id
        )
        .execute(&mut *self.0)
        .await?;

        assert_eq!(
            result.rows_affected(),
            1,
            "sanity check: was not able to update exchange rate"
        );

        Ok(())
    }
}
