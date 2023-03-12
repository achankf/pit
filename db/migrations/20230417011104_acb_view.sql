CREATE VIEW AcbBaseData AS WITH BaseData AS (
    SELECT
        transaction_id,
        person_id,
        date,
        security_id,
        SUM(
            -- only the STOCK accounts can change number of units in a position
            CASE
                WHEN account_subtype = 'STOCK' THEN unit * CASE
                    WHEN debit IS NOT NULL THEN 1 -- buying
                    ELSE -1 -- selling
                END
                ELSE 0.0
            END
        ) AS unit,
        COALESCE(exchange_rate, 1.0) AS exchange_rate,
        SUM(
            CASE
                WHEN account_subtype = 'STOCK' THEN ROUND(
                    ROUND(
                        unit * (COALESCE(debit, 0.0) - COALESCE(credit, 0.0)),
                        2
                    ) * COALESCE(exchange_rate, 1.0),
                    2
                )
                ELSE 0.0
            END
        ) AS book_value,
        SUM(
            CASE
                WHEN account_subtype = 'DISTRIBUTION' THEN ROUND(
                    ROUND(unit * credit, 2) * COALESCE(exchange_rate, 1.0),
                    2
                )
                ELSE 0.0
            END
        ) AS distribution,
        SUM(
            CASE
                WHEN account_subtype = 'COMMISSION' THEN debit
                ELSE 0.0
            END
        ) AS commission,
        MIN(
            CASE
                -- distributions come before stock purchases
                WHEN account_subtype = 'DISTRIBUTION' THEN 1
                WHEN account_subtype = 'STOCK' THEN 2
                WHEN account_subtype = 'COMMISSION' THEN 3
            END
        ) AS sort_order
    FROM
        FinancialEntry
        INNER JOIN StockAccountEntry USING (account_id)
        INNER JOIN StockAccountHolder USING (stock_account_holder_id)
        INNER JOIN CashAccountProduct USING (account_type_id)
        INNER JOIN AccountSubtype USING (account_subtype_id)
        INNER JOIN AccountKind USING (account_kind_id)
        INNER JOIN TaxShelterType USING (tax_shelter_type_id)
        LEFT JOIN TransactionForex USING (transaction_id)
    WHERE
        account_subtype IN ('STOCK', 'DISTRIBUTION', 'COMMISSION')
        AND tax_shelter_type = 'NON-REGISTERED'
    GROUP BY
        transaction_id
)
SELECT
    transaction_id,
    person_id,
    date,
    security_id,
    exchange_rate,
    unit,
    book_value - commission AS book_value,
    distribution,
    sort_order
FROM
    BaseData;

CREATE VIEW AcbRunningShareTotal AS
SELECT
    *,
    ROUND(
        SUM(unit) OVER(
            PARTITION BY person_id,
            security_id
            ORDER BY
                date,
                sort_order
        ),
        4
    ) AS acc_units
FROM
    AcbBaseData;

CREATE VIEW AcbInjectPrevRunningTotal AS
SELECT
    *,
    LAG(acc_units, 1, 0) OVER (
        PARTITION BY person_id,
        security_id
        ORDER BY
            date,
            sort_order
    ) AS prev_acc_units
FROM
    AcbRunningShareTotal;

CREATE VIEW AcbPrecomputation AS
SELECT
    *,
    CASE
        WHEN unit > 0 THEN book_value
        ELSE 0.0
    END AS acb_increase,
    CASE
        WHEN unit < 0.0 THEN -- COALESCE for handling an edge case where I'm exiting a position, i.e. acb_decrease_factor = 0 meaning ACB becomes 0
        COALESCE((prev_acc_units + unit) / prev_acc_units, 0.0)
        ELSE 1.0
    END AS acb_decrease_factor,
    ROW_NUMBER() OVER(
        PARTITION BY person_id,
        security_id
        ORDER BY
            date,
            sort_order
    ) AS group_order
FROM
    AcbInjectPrevRunningTotal;

CREATE VIEW Acb AS WITH RECURSIVE RecurseAcb (
    sort_order,
    person_id,
    transaction_id,
    date,
    security_id,
    unit,
    book_value,
    acc_units,
    prev_acc_units,
    acb_increase,
    acb_decrease_factor,
    distribution,
    group_order,
    acb,
    capital_gl
) AS (
    SELECT
        sort_order,
        person_id,
        transaction_id,
        date,
        security_id,
        unit,
        book_value,
        acc_units,
        prev_acc_units,
        acb_increase,
        acb_decrease_factor,
        distribution,
        group_order,
        -- acb for the first buy is the book cost
        book_value,
        -- first entry should be a buy, so no capital gain/loss
        0.0
    FROM
        AcbPrecomputation
    WHERE
        group_order = 1
    UNION
    ALL
    SELECT
        a.sort_order,
        a.person_id,
        a.transaction_id,
        a.date,
        a.security_id,
        a.unit,
        a.book_value,
        a.acc_units,
        a.prev_acc_units,
        a.acb_increase,
        a.acb_decrease_factor,
        a.distribution,
        a.group_order,
        ROUND(
            (b.acb + a.acb_increase) * a.acb_decrease_factor,
            2
        ),
        ROUND(
            CASE
                WHEN a.unit < 0.0 THEN ABS(a.book_value) - (b.acb + a.acb_increase) * (1 - a.acb_decrease_factor)
                ELSE 0.0
            END,
            2
        )
    FROM
        AcbPrecomputation a
        INNER JOIN RecurseAcb b USING (person_id, security_id)
    WHERE
        a.group_order = b.group_order + 1
)
SELECT
    *
FROM
    RecurseAcb;

-- quick test
SELECT
    *
FROM
    Acb;