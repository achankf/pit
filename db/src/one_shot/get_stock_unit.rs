use chrono::{DateTime, Local};

use crate::{SqlResult, Transaction};

pub struct StockUnit {
    pub name: String,
    pub account_name: String,
    pub ticker: String,
    pub total_unit: f64,
    pub market_value: f64,
}

impl Transaction<'_> {
    pub async fn get_stock_unit(&mut self, date: &DateTime<Local>) -> SqlResult<Vec<StockUnit>> {
        let timestamp = date.timestamp();

        let records = sqlx::query_as!(
            StockUnit,
            r#"
SELECT
    first_name || ' ' || last_name AS name,
    account_name,
    ticker,
    total_unit AS "total_unit!:f64",
    total_unit * price AS market_value
FROM
    (
        SELECT
            person_id,
            account_type_id,
            security_id,
            ROUND(
                SUM(
                    CASE
                        WHEN debit IS NOT NULL THEN unit
                        ELSE - unit
                    END
                ),
                4
            ) AS total_unit
        FROM
            FinancialEntry
            INNER JOIN StockAccountEntry USING (account_id)
            INNER JOIN StockAccountHolder USING (stock_account_holder_id)
            INNER JOIN AccountSubtype USING (account_subtype_id)
            INNER JOIN AccountType USING (account_type_id)
        WHERE
            date < ?
            AND account_subtype = 'STOCK'
        GROUP BY
            person_id,
            account_type_id,
            security_id
        HAVING
            total_unit <> 0
    )
    INNER JOIN Person USING (person_id)
    INNER JOIN SECURITY USING (security_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
"#,
            timestamp
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(records)
    }
}
