use std::ops::Range;

use chrono::{DateTime, Local};
use sqlx::Row;

use crate::{NetBalanceRecord, SqlResult, Transaction};

impl Transaction<'_> {
    pub async fn get_net_revenue_balance(
        &mut self,
        range: Range<DateTime<Local>>,
    ) -> SqlResult<Vec<NetBalanceRecord>> {
        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let rows = sqlx::query(
            r#"
SELECT
    first_name,
    last_name,
    currency AS "currency!:String",
    currency_symbol AS "currency_symbol!:String",
    ROUND(
        SUM(
            ROUND(
                unit * ROUND(
                    COALESCE(credit, 0) - COALESCE(debit, 0),
                    2
                ),
                2
            )
        ),
        2
    ) AS "balance!:f64"
FROM
    FinancialEntry
    INNER JOIN OwnedAccount USING (account_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    INNER JOIN AccountKind USING (account_kind_id)
    INNER JOIN Person USING (person_id)
    INNER JOIN Currency USING (currency_id)
WHERE
    account_kind IN ('REVENUE', 'EXPENSE')
    AND FinancialEntry.date BETWEEN ?
    AND ?
GROUP BY
    person_id,
    currency_id
ORDER BY
    person_id,
    currency_id
"#,
        )
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&mut *self.0)
        .await?
        .into_iter()
        .map(|record| NetBalanceRecord {
            first_name: record.get(0),
            last_name: record.get(1),
            currency: record.get(2),
            currency_symbol: record.get(3),
            balance: record.get(4),
        })
        .collect();

        Ok(rows)
    }
}
