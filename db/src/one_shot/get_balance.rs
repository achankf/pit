use std::ops::Range;

use chrono::{DateTime, Local};
use common::all_time_until_now;
use serde::Deserialize;

use crate::{SqlResult, Transaction};

#[derive(Clone, Deserialize, Debug)]
pub struct BalanceRecord {
    pub first_name: String,
    pub last_name: String,
    pub general_account_type: String,
    pub description: String,
    pub currency: String,
    pub currency_symbol: String,
    pub balance: f64,
}

impl<'c> Transaction<'c> {
    async fn get_balance(
        &mut self,
        kind: &str,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        let range = range.unwrap_or_else(all_time_until_now);

        let start_ts = range.start.timestamp();
        let end_ts = range.end.timestamp();

        let rows = sqlx::query_as!(
            BalanceRecord,
            r#"
WITH BaseData AS (
    SELECT
        person_id,
        general_account_kind_id,
        general_account_type_id,
        general_account_id,
        currency_id,
        ROUND(
            SUM(
                ROUND(
                    unit * ROUND(
                        CASE
                            WHEN general_account_kind IN ('ASSET', 'EXPENSE') THEN COALESCE(debit, 0) - COALESCE(credit, 0)
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
        GeneralTransaction t
        INNER JOIN GeneralAccount USING (general_account_id)
        INNER JOIN GeneralAccountType USING (general_account_type_id)
        INNER JOIN GeneralAccountKind USING (general_account_kind_id)
    WHERE
        general_account_kind = ?
        AND t.date BETWEEN ?
        AND ?
    GROUP BY
        person_id,
        general_account_kind_id,
        general_account_type_id,
        general_account_id,
        currency_id
)
SELECT
    first_name,
    last_name,
    general_account_type,
    GeneralAccount.description,
    currency AS "currency!:String",
    currency_symbol AS "currency_symbol!:String",
    balance AS "balance!:f64"
FROM
    BaseData
    INNER JOIN GeneralAccount USING (general_account_id)
    INNER JOIN GeneralAccountType USING (general_account_type_id)
    INNER JOIN GeneralAccountKind USING (general_account_kind_id)
    INNER JOIN GeneralAccountName USING (general_account_name_id)
    INNER JOIN Person USING (person_id)
    INNER JOIN Currency USING (currency_id)
ORDER BY
    general_account_kind,
    general_account_type,
    first_name,
    last_name
"#,
            kind,
            start_ts,
            end_ts,
        )
        .fetch_all(&mut *self.0)
        .await?;

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

    pub async fn get_asset_balance(
        &mut self,
        range: Option<Range<DateTime<Local>>>,
    ) -> SqlResult<Vec<BalanceRecord>> {
        self.get_balance("ASSET", range).await
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
