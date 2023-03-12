CREATE VIEW CashTotal AS
SELECT
    person_id,
    general_account_id,
    currency_id,
    ROUND(
        SUM(
            ROUND(
                unit * (COALESCE(debit, 0) - COALESCE(credit, 0)),
                2
            )
        ),
        2
    ) AS balance
FROM
    GeneralTransaction
    INNER JOIN GeneralAccount USING (general_account_id)
    INNER JOIN GeneralAccountType USING (general_account_type_id)
WHERE
    general_account_type = 'CASH'
GROUP BY
    person_id,
    general_account_id,
    currency_id;

CREATE VIEW WithEmergencyTarget AS
SELECT
    person_id,
    institution_id,
    general_account_id,
    currency_id,
    balance,
    tax_shelter_type_id,
    inactive_fee_months,
    COALESCE(emergency_target, 0) AS emergency_target,
    min_balance_waiver
FROM
    CashTotal
    INNER JOIN CashAccount USING (general_account_id)
    LEFT JOIN CashEmergencyTarget USING (person_id, general_account_id, currency_id);

CREATE VIEW CashView (
    person_id,
    institution_id,
    general_account_id,
    currency_id,
    balance,
    tax_shelter_type_id,
    inactive_fee_months,
    emergency_target,
    min_balance_waiver,
    emergency_fund,
    injection_needed,
    unallocated_fund
) AS
SELECT
    person_id,
    institution_id,
    general_account_id,
    currency_id,
    balance,
    tax_shelter_type_id,
    inactive_fee_months,
    emergency_target,
    min_balance_waiver,
    MAX(
        0,
        MIN(
            emergency_target,
            ROUND(balance - min_balance_waiver, 2)
        )
    ),
    MAX(
        0,
        ROUND(
            (
                emergency_target + min_balance_waiver
            ) - balance,
            2
        )
    ),
    MAX(
        0,
        ROUND(
            balance - (
                emergency_target + min_balance_waiver
            ),
            2
        )
    )
FROM
    WithEmergencyTarget