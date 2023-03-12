use chrono::{Days, Local};
use sqlx::Row;

use crate::{BalanceRecord, SqlResult, Transaction};

impl Transaction<'_> {
    pub async fn get_expense_balance_by_category(
        &mut self,
        days_prior: u64,
    ) -> SqlResult<Vec<BalanceRecord>> {
        let start_date = Local::now()
            .checked_sub_days(Days::new(days_prior))
            .expect("unable to subtract days");

        let records = sqlx::query(
            r#"
WITH BaseData AS (
    SELECT
        account_kind_id,
        account_type_id,
        ROUND(
            SUM(
                ROUND(
                    unit * (
                        COALESCE(debit, 0) - COALESCE(credit, 0)
                    ),
                    2
                )
            ),
            2
        ) AS balance
    FROM
        FinancialEntry
        INNER JOIN Account USING (account_id)
        INNER JOIN AccountSubtype USING (account_subtype_id)
        INNER JOIN AccountType USING (account_type_id)
        INNER JOIN AccountKind USING (account_kind_id)
        LEFT JOIN OwnedAccount USING (account_id, account_type_id, account_subtype_id)
    WHERE
        person_id IS NULL
        AND date > ?
        AND account_kind = 'EXPENSE'
    GROUP BY
        account_kind_id,
        account_type_id
)
SELECT
    account_kind,
    account_type,
    balance
FROM
    BaseData
    INNER JOIN AccountType USING (account_type_id)
    INNER JOIN AccountKind USING (account_kind_id)
ORDER BY
    balance DESC,
    account_type
"#,
        )
        .bind(start_date.timestamp())
        .fetch_all(&mut *self.0)
        .await?
        .into_iter()
        .map(|row| BalanceRecord {
            account_type: row.get(0),
            account_name: row.get(1),
            balance: row.get(2),
            name: "".to_string(),
        })
        .collect();

        Ok(records)
    }
}
