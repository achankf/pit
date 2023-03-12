-- returns stock balance by asset class, normalized to currency
CREATE VIEW NormalizedStockBalanceByAssetClass AS
SELECT
    person_id,
    asset_class_id,
    ROUND(
        SUM(
            ROUND(
                ROUND(ROUND(unit * price, 2) * rate, 2) * market_exchange_rate,
                2
            )
        ),
        2
    ) AS balance
FROM
    StockAccount
    INNER JOIN GeneralTransaction USING (general_account_id)
    INNER JOIN GeneralAccount USING (general_account_id)
    INNER JOIN GeneralAccountType USING (general_account_type_id)
    INNER JOIN PerClassAllocationRate USING (security_id)
    INNER JOIN SECURITY USING (security_id)
    INNER JOIN Currency USING (currency_id)
WHERE
    general_account_type = 'STOCK'
GROUP BY
    person_id,
    asset_class_id;

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
    GeneralTransaction
    INNER JOIN GeneralAccount USING (general_account_id)
    INNER JOIN GeneralAccountType USING (general_account_type_id)
    INNER JOIN Currency USING (currency_id)
WHERE
    general_account_type = 'GIC'
GROUP BY
    person_id;

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
    balance,
    0,
    balance
FROM
    NormalizedStockBalanceByAssetClass
UNION
ALL
SELECT
    person_id,
    asset_class_id,
    SUM(unallocated_fund),
    SUM(emergency_fund),
    SUM(unallocated_fund) + SUM(emergency_fund)
FROM
    CashView
    INNER JOIN AssetClass USING (person_id)
WHERE
    class = 'Cash'
GROUP BY
    person_id,
    asset_class_id
UNION
ALL
SELECT
    person_id,
    asset_class_id,
    0,
    balance,
    balance
FROM
    NormalizedFixedIncomeBalance
    INNER JOIN AssetClass USING (person_id)
WHERE
    class = 'Fixed Income';

CREATE VIEW TotalAssetBalance AS
SELECT
    person_id,
    SUM(liquid_balance) AS liquid_sum,
    SUM(total_balance) AS total_sum
FROM
    BalanceByAssetClass
GROUP BY
    person_id;

CREATE VIEW AllocationView (
    person_id,
    asset_class_id,
    current_rebalance_amount,
    potential_rebalance_amount
) AS WITH RebalanceRate AS (
    SELECT
        liquid_balance / liquid_sum AS liquid_rate,
        total_balance / total_sum AS total_rate,
        *
    FROM
        PortfolioAllocationRate
        INNER JOIN BalanceByAssetClass b USING (person_id, asset_class_id)
        INNER JOIN TotalAssetBalance USING (person_id)
)
SELECT
    person_id,
    asset_class_id,
    ROUND((real_rate - liquid_rate) * liquid_sum, 2) AS rebalance_diff,
    ROUND((real_rate - total_rate) * total_sum, 2) AS total_diff
FROM
    RebalanceRate;