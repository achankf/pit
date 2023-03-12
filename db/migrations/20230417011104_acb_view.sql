CREATE VIEW Acb AS WITH RECURSIVE BaseData AS (
    SELECT
        person_id,
        date,
        general_account_id,
        general_account_name_id,
        security_id,
        general_account_type_id,
        unit,
        debit,
        credit,
        book_exchange_rate,
        CASE
            WHEN general_account_type_id = (
                SELECT
                    general_account_type_id
                FROM
                    GeneralAccountType
                WHERE
                    general_account_type = 'STOCK'
            ) THEN TRUE
            ELSE FALSE
        END AS is_stock,
        CASE
            WHEN general_account_type_id = (
                SELECT
                    general_account_type_id
                FROM
                    GeneralAccountType
                WHERE
                    general_account_type = 'DIVIDEND'
            ) THEN TRUE
            ELSE FALSE
        END AS is_dividend,
        CASE
            WHEN general_account_type_id = (
                SELECT
                    general_account_type_id
                FROM
                    GeneralAccountType
                WHERE
                    general_account_type = 'COMMISSION'
            ) THEN TRUE
            ELSE FALSE
        END AS is_commission
    FROM
        GeneralTransaction t
        INNER JOIN StockAccount USING (general_account_id)
        INNER JOIN GeneralAccount USING (general_account_id)
),
AugmentedData AS (
    SELECT
        *,
        ROW_NUMBER() OVER (
            ORDER BY
                date,
                is_dividend DESC,
                is_stock DESC,
                is_commission DESC
        ) AS chronological_order,
        CASE
            WHEN is_dividend
            OR is_commission THEN 0.0
            ELSE unit
        END AS unit_change,
        CASE
            WHEN is_dividend THEN credit -- dividend accounts (revenue) have credit balance
            WHEN is_commission THEN - debit
            ELSE debit -- stock accounts (asset) and commission accounts (expense) have debit balance
        END AS price
    FROM
        BaseData
),
RunningShareTotal AS (
    SELECT
        *,
        ROUND(ROUND(unit * price, 2) * book_exchange_rate, 2) AS book_value,
        SUM(unit_change) OVER(
            PARTITION BY person_id,
            general_account_id,
            security_id
            ORDER BY
                chronological_order
        ) AS acc_units
    FROM
        AugmentedData
),
InjectPrevRunningTotal AS (
    SELECT
        *,
        LAG(acc_units, 1, 0) OVER (
            PARTITION BY person_id,
            general_account_id,
            security_id
            ORDER BY
                chronological_order
        ) AS prev_acc_units
    FROM
        RunningShareTotal
),
Precomputation AS (
    SELECT
        *,
        COALESCE(
            CASE
                WHEN NOT is_dividend
                AND debit IS NOT NULL THEN book_value
            END,
            0.0
        ) AS acb_increase,
        COALESCE(
            CASE
                WHEN NOT is_dividend
                AND credit IS NOT NULL THEN (prev_acc_units + unit) / prev_acc_units
            END,
            1.0
        ) AS acb_decrease_factor,
        CASE
            WHEN is_dividend THEN book_value
            ELSE 0.0
        END AS dividend_capital_gain,
        DENSE_RANK() OVER(
            PARTITION BY person_id,
            general_account_id,
            security_id
            ORDER BY
                chronological_order
        ) AS group_order
    FROM
        InjectPrevRunningTotal
),
RecurseAcb (
    chronological_order,
    person_id,
    date,
    general_account_id,
    general_account_name_id,
    security_id,
    general_account_type_id,
    unit,
    debit,
    credit,
    is_dividend,
    is_commission,
    unit_change,
    book_value,
    acc_units,
    prev_acc_units,
    acb_increase,
    acb_decrease_factor,
    dividend_capital_gain,
    group_order,
    acb,
    capital_gl
) AS (
    SELECT
        chronological_order,
        person_id,
        date,
        general_account_id,
        general_account_name_id,
        security_id,
        general_account_type_id,
        unit,
        debit,
        credit,
        is_dividend,
        is_commission,
        unit_change,
        book_value,
        acc_units,
        prev_acc_units,
        acb_increase,
        acb_decrease_factor,
        dividend_capital_gain,
        group_order,
        -- acb for the first buy is the book cost
        book_value,
        ROUND(
            CASE
                WHEN is_dividend THEN dividend_capital_gain
            END,
            2
        )
    FROM
        Precomputation
    WHERE
        group_order = 1
    UNION
    ALL
    SELECT
        a.chronological_order,
        a.person_id,
        a.date,
        a.general_account_id,
        a.general_account_name_id,
        a.security_id,
        a.general_account_type_id,
        a.unit,
        a.debit,
        a.credit,
        a.is_dividend,
        a.is_commission,
        a.unit_change,
        a.book_value,
        a.acc_units,
        a.prev_acc_units,
        a.acb_increase,
        a.acb_decrease_factor,
        a.dividend_capital_gain,
        a.group_order,
        ROUND(
            (b.acb + a.acb_increase) * a.acb_decrease_factor,
            2
        ),
        ROUND(
            CASE
                WHEN a.is_stock
                AND a.credit IS NOT NULL THEN a.book_value - (b.acb + a.acb_increase) * (1 - a.acb_decrease_factor)
                ELSE a.dividend_capital_gain
            END,
            2
        )
    FROM
        Precomputation a
        INNER JOIN RecurseAcb b ON a.person_id = b.person_id
        AND a.general_account_id = b.general_account_id
        AND a.security_id = b.security_id
    WHERE
        a.group_order = b.group_order + 1
)
SELECT
    *
FROM
    RecurseAcb
ORDER BY
    person_id,
    general_account_id,
    security_id,
    group_order;