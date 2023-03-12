use crate::{SqlResult, Transaction};

impl<'c> Transaction<'c> {
    pub async fn upsert_stock_account(
        &mut self,
        general_account_key: String,
        ticker: String,
    ) -> SqlResult<()> {
        sqlx::query!(
            r#"
INSERT INTO
    StockAccount (general_account_id, security_id)
VALUES
    (
        (
            SELECT
                general_account_id
            FROM
                GeneralAccount
            WHERE
                general_account_key = ?
        ),
        (
            SELECT
                security_id
            FROM
                SECURITY
            WHERE
                ticker = ?
        )
    ) ON CONFLICT(general_account_id) DO
UPDATE
SET
    security_id = excluded.security_id
WHERE
    security_id <> excluded.security_id
"#,
            general_account_key,
            ticker
        )
        .execute(&mut *self.0)
        .await?;

        Ok(())
    }
}
