use std::ops::Range;

use chrono::{DateTime, Local};

use crate::{NetBalanceRecord, SqlResult, Transaction};

impl<'c> Transaction<'c> {
    pub async fn get_net_revenue_balance(
        &mut self,
        range: Range<DateTime<Local>>,
    ) -> SqlResult<Vec<NetBalanceRecord>> {
        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let rows = sqlx::query_as!(
            NetBalanceRecord,
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
    GeneralTransaction
    INNER JOIN Currency USING (currency_id)
    INNER JOIN GeneralAccount USING (general_account_id)
    INNER JOIN GeneralAccountType USING (general_account_type_id)
    INNER JOIN GeneralAccountKind USING (general_account_kind_id)
    INNER JOIN Person USING (person_id)
WHERE
    general_account_kind IN ('REVENUE', 'EXPENSE')
    AND GeneralTransaction.date BETWEEN ?
    AND ?
GROUP BY
    person_id,
    currency_id
ORDER BY
    person_id,
    currency_id;
"#,
            start_ts,
            end_ts
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(rows)
    }
}
