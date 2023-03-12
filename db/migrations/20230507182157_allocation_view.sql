-- returns stock balance by asset class, normalized to currency
CREATE VIEW NormalizedStockBalance AS WITH StockCount AS (
    SELECT
        person_id,
        security_id,
        ROUND(
            ROUND(
                ROUND(
                    SUM(
                        CASE
                            WHEN debit IS NOT NULL THEN unit
                            ELSE - unit
                        END
                    ),
                    4
                ) * price,
                2
            ) * market_exchange_rate,
            2
        ) AS balance
    FROM
        FinancialEntry
        INNER JOIN StockAccountEntry USING (account_id)
        INNER JOIN StockAccountHolder USING (stock_account_holder_id)
        INNER JOIN AccountSubtype USING (account_subtype_id)
        INNER JOIN AccountType USING (account_type_id)
        INNER JOIN SECURITY USING(security_id)
        INNER JOIN Currency USING (currency_id)
    WHERE
        account_subtype = 'STOCK'
    GROUP BY
        person_id,
        account_type_id,
        security_id
    HAVING
        balance <> 0
)
SELECT
    person_id,
    asset_class_id,
    SUM(ROUND(balance * rate, 2)) AS balance
FROM
    StockCount
    INNER JOIN PerClassAllocationRate USING (person_id, security_id)
GROUP BY
    person_id,
    asset_class_id;

-- quick test
SELECT
    *
FROM
    NormalizedStockBalance;

-- return fixed income balanced, normalized to main currency
CREATE VIEW NormalizedFixedIncomeBalance AS
SELECT
    person_id,
    ROUND(
        SUM(
            ROUND(
                ROUND(
                    unit * (COALESCE(debit, 0) - COALESCE(credit, 0)),
                    2
                ) * market_exchange_rate,
                2
            )
        ),
        2
    ) AS balance
FROM
    Account
    INNER JOIN GicEntry USING (account_id)
    INNER JOIN GicAccountHolder USING (gic_account_holder_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    INNER JOIN AccountKind USING (account_kind_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
    INNER JOIN FinancialEntry USING (account_id)
    INNER JOIN Currency USING (currency_id)
WHERE
    account_kind = 'ASSET'
GROUP BY
    person_id;

-- quick test
SELECT
    *
FROM
    NormalizedFixedIncomeBalance;

CREATE VIEW BalanceByAssetClass (
    person_id,
    asset_class_id,
    liquid_balance,
    reserve_balance,
    total_balance
) AS
SELECT
    person_id,
    asset_class_id,
    COALESCE(balance, 0),
    0.0,
    COALESCE(balance, 0)
FROM
    NormalizedStockBalance
UNION
ALL
SELECT
    person_id,
    asset_class_id,
    COALESCE(SUM(unallocated_fund), 0) * market_exchange_rate,
    COALESCE(SUM(emergency_fund), 0) * market_exchange_rate,
    COALESCE(SUM(unallocated_fund) + SUM(emergency_fund), 0) * market_exchange_rate
FROM
    CashView
    INNER JOIN AssetClass USING (person_id)
    INNER JOIN AssetClassName USING (asset_class_name_id)
    INNER JOIN Currency USING (currency_id)
WHERE
    asset_class_name = 'Cash'
    AND -- exclude prepaid accounts (i.e. Presto) from rebalancing
    cash_account_holder_id NOT IN (
        SELECT
            cash_account_holder_id
        FROM
            PrepaidAccount
            INNER JOIN CashAccountHolder USING (account_type_id)
    )
GROUP BY
    person_id,
    asset_class_id
UNION
ALL
SELECT
    person_id,
    asset_class_id,
    0.0,
    COALESCE(balance, 0.0),
    COALESCE(balance, 0.0)
FROM
    NormalizedFixedIncomeBalance
    INNER JOIN AssetClass USING (person_id)
    INNER JOIN AssetClassName USING (asset_class_name_id)
WHERE
    asset_class_name = 'Fixed Income';

-- quick test
SELECT
    *
FROM
    BalanceByAssetClass;

CREATE VIEW TotalAssetBalance AS
SELECT
    person_id,
    SUM(liquid_balance) AS liquid_sum,
    SUM(total_balance) AS total_sum
FROM
    BalanceByAssetClass
GROUP BY
    person_id;

-- quick test
SELECT
    *
FROM
    TotalAssetBalance;

CREATE VIEW AllocationView AS WITH RebalanceRate AS (
    SELECT
        COALESCE(liquid_balance, 0.0) / liquid_sum AS liquid_rate,
        COALESCE(total_balance, 0.0) / total_sum AS total_rate,
        asset_class_id,
        person_id,
        parent_id,
        -- class,
        class_rate,
        real_rate,
        COALESCE(liquid_balance, 0.0) AS liquid_balance,
        COALESCE(reserve_balance, 0.0) AS reserve_balance,
        COALESCE(total_balance, 0.0) AS total_balance,
        liquid_sum,
        total_sum
    FROM
        PortfolioAllocationRate
        LEFT JOIN BalanceByAssetClass b USING (person_id, asset_class_id)
        INNER JOIN TotalAssetBalance USING (person_id)
)
SELECT
    person_id,
    asset_class_id,
    ROUND((real_rate - liquid_rate) * liquid_sum, 2) AS current_rebalance_amount,
    ROUND((real_rate - total_rate) * total_sum, 2) AS potential_rebalance_amount
FROM
    RebalanceRate
WHERE
    current_rebalance_amount <> 0
    OR potential_rebalance_amount <> 0;

-- quick test
SELECT
    *
FROM
    AllocationView;