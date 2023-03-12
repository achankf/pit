use std::ops::Range;

use chrono::{DateTime, Local};
use common::all_time_until_now;
use serde::Deserialize;
use sqlx::Row;

use crate::{SqlResult, Transaction};

#[derive(Clone, Deserialize, Debug)]
pub struct BalanceRecord {
    pub name: String,
    pub account_type: String,
    pub account_name: String,
    pub balance: f64,
}

impl Transaction<'_> {
    async fn get_balance(
        &mut self,
        kind: &str,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        let range = range.unwrap_or_else(all_time_until_now);

        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let rows = sqlx::query(
            r#"
WITH BaseData AS (
    SELECT
        person_id,
        account_kind_id,
        account_type_id,
        account_id,
        ROUND(
            SUM(
                ROUND(
                    unit * ROUND(
                        CASE
                            WHEN account_kind IN ('ASSET', 'EXPENSE') THEN COALESCE(debit, 0) - COALESCE(credit, 0)
                            ELSE COALESCE(credit, 0) - COALESCE(debit, 0)
                        END,
                        2
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
        account_kind = ?
        AND date BETWEEN ?
        AND ?
    GROUP BY
        person_id,
        account_kind_id,
        account_type_id,
        account_id
)
SELECT
    first_name || ' ' || last_name as name,
    account_type,
    account_name,
    balance AS "balance!:f64"
FROM
    BaseData
    INNER JOIN Account USING (account_id)
    INNER JOIN AccountType USING (account_type_id)
    INNER JOIN AccountKind USING (account_kind_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    LEFT JOIN Person USING (person_id)
WHERE
    balance <> 0
ORDER BY
    name,
    account_kind,
    account_type
"#)
        .bind(kind)
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&mut *self.0)
        .await?
        .into_iter()
        .map(|row| BalanceRecord {
            name: row.get(0),
            account_type: row.get(1),
            account_name: row.get(2),
            balance: row.get(3),
        }).collect();

        Ok(rows)
    }

    pub async fn get_revenue_balance(
        &mut self,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        self.get_balance("REVENUE", range).await
    }

    pub async fn get_expense_balance(
        &mut self,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        self.get_balance("EXPENSE", range).await
    }

    pub async fn get_cash_balance(
        &mut self,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        let range = range.unwrap_or_else(all_time_until_now);

        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let rows = sqlx::query(
            r#"
WITH BaseData AS (
    SELECT
        person_id,
        account_kind_id,
        account_type_id,
        account_id,
        ROUND(
            SUM(
                ROUND(
                    unit * ROUND(
                        CASE
                            WHEN account_kind IN ('ASSET', 'EXPENSE') THEN COALESCE(debit, 0) - COALESCE(credit, 0)
                            ELSE COALESCE(credit, 0) - COALESCE(debit, 0)
                        END,
                        2
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
        account_kind = 'ASSET'
        AND account_subtype IN ('CASH', 'PRINCIPAL')
        AND date BETWEEN ?
        AND ?
    GROUP BY
        person_id,
        account_kind_id,
        account_type_id,
        account_id
    HAVING
        balance <> 0
)
SELECT
    first_name || ' ' || last_name AS name,
    account_type,
    account_name,
    balance AS "balance!:f64"
FROM
    BaseData
    INNER JOIN Account USING (account_id)
    INNER JOIN AccountType USING (account_type_id)
    INNER JOIN AccountKind USING (account_kind_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    LEFT JOIN Person USING (person_id)
ORDER BY
    name,
    account_kind,
    account_type;
"#)
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&mut *self.0)
        .await?
        .into_iter()
        .map(|row| BalanceRecord {
            name: row.get(0),
            account_type: row.get(1),
            account_name: row.get(2),
            balance: row.get(3),
        }).collect();

        Ok(rows)
    }

    pub async fn get_liabilities_balance(
        &mut self,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        self.get_balance("LIABILITIES", range).await
    }

    pub async fn get_equity_balance(
        &mut self,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        self.get_balance("EQUITY", range).await
    }
}
