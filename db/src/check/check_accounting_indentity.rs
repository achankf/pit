use crate::{SqlResult, Transaction};

pub struct AccountingIdentityResult {
    pub asset_balance: f64,
    pub equity_liabilities_balance: f64,
    pub is_balance: bool,
}

impl Transaction<'_> {
    pub async fn check_accounting_indentity(&mut self) -> SqlResult<AccountingIdentityResult> {
        let result = sqlx::query_as!(
            AccountingIdentityResult,
            r#"
WITH Lhs AS (
    SELECT
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
        INNER JOIN Account USING(account_id)
        INNER JOIN AccountSubtype USING(account_subtype_id)
        INNER JOIN AccountKind USING(account_kind_id)
    WHERE
        account_kind = 'ASSET'
),
Rhs AS (
    SELECT
        ROUND(
            SUM(
                ROUND(
                    unit * (
                        COALESCE(credit, 0) - COALESCE(debit, 0)
                    ),
                    2
                )
            ),
            2
        ) AS balance
    FROM
        FinancialEntry
        INNER JOIN Account USING(account_id)
        INNER JOIN AccountSubtype USING(account_subtype_id)
        INNER JOIN AccountKind USING(account_kind_id)
    WHERE
        account_kind <> 'ASSET'
)
SELECT
    (
        SELECT
            COALESCE(balance, 0.0)
        FROM
            Lhs
    ) AS "asset_balance!:f64",
    (
        SELECT
            COALESCE(balance, 0.0)
        FROM
            Rhs
    ) AS "equity_liabilities_balance!:f64",
    (
        SELECT
            COALESCE(balance, 0.0)
        FROM
            Lhs
    ) = (
        SELECT
            COALESCE(balance, 0.0)
        FROM
            Rhs
    ) AS "is_balance!:bool"
"#
        )
        .fetch_one(&mut *self.0)
        .await?;

        Ok(result)
    }
}
