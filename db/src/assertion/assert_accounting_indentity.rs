use crate::Transaction;

impl<'c> Transaction<'c> {
    pub async fn assert_accounting_indentity(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        struct CheckResult {
            asset_balance: f64,
            equity_liabilities_balance: f64,
            is_balance: bool,
        }

        let result = sqlx::query_as!(
            CheckResult,
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
        GeneralTransaction
        INNER JOIN GeneralAccount USING(general_account_id)
        INNER JOIN GeneralAccountType USING(general_account_type_id)
        INNER JOIN GeneralAccountKind USING(general_account_kind_id)
    WHERE
        general_account_kind = 'ASSET'
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
        GeneralTransaction
        INNER JOIN GeneralAccount USING(general_account_id)
        INNER JOIN GeneralAccountType USING(general_account_type_id)
        INNER JOIN GeneralAccountKind USING(general_account_kind_id)
    WHERE
        general_account_kind <> 'ASSET'
)
SELECT
    Lhs.balance AS "asset_balance!:f64",
    Rhs.balance AS "equity_liabilities_balance!:f64",
    Lhs.balance = Rhs.balance AS "is_balance:bool"
FROM
    Lhs
    CROSS JOIN Rhs;
"#
        )
        .fetch_one(&mut *self.0)
        .await?;

        if !result.is_balance {
            eprintln!(
                "Asset balance doesn't match its double-entry counterpart, {}|{}",
                result.asset_balance, result.equity_liabilities_balance
            );
            return Err("Asset balance doesn't match its double-entry counterpart".into());
        }

        Ok(())
    }
}
