CREATE VIEW CashView AS
SELECT
    cash_account_holder_id,
    person_id,
    emergency_target,
    MAX(
        0,
        MIN(
            emergency_target,
            ROUND(balance - min_balance_waiver, 2)
        )
    ) AS emergency_fund,
    MAX(
        0,
        ROUND(
            (
                emergency_target + min_balance_waiver
            ) - balance,
            2
        )
    ) AS injection_needed,
    MAX(
        0,
        ROUND(
            balance - (
                emergency_target + min_balance_waiver
            ),
            2
        )
    ) AS unallocated_fund,
    currency_id
FROM
    CashAccountEntry
    INNER JOIN CashAccountHolder USING (cash_account_holder_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    INNER JOIN (
        SELECT
            account_id,
            ROUND(
                SUM (
                    ROUND(
                        unit * (COALESCE(debit, 0.0) - COALESCE(credit, 0.0)),
                        2
                    )
                ),
                2
            ) AS balance
        FROM
            FinancialEntry
            INNER JOIN CashAccountEntry USING (account_id)
        GROUP BY
            account_id
    ) USING (account_id)
WHERE
    account_subtype = 'CASH';

-- quick test
SELECT
    *
FROM
    CashView;

SELECT
    CashVIew.*,
    account_type
FROM
    CashView
    INNER JOIN CashAccountHolder USING (cash_account_holder_id)
    INNER JOIN AccountType USING (account_type_id);