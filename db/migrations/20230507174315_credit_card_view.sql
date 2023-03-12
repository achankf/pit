CREATE VIEW CreditCardView AS WITH LastPayment AS (
    SELECT
        credit_card_holder_id,
        MAX(date) AS last_payment_date
    FROM
        FinancialEntry
        INNER JOIN Account USING (account_id)
        INNER JOIN CreditCardEntry USING (account_id)
        INNER JOIN AccountSubtype USING (account_subtype_id)
    WHERE
        transaction_id IN (
            SELECT
                transaction_id
            FROM
                FinancialEntry
                INNER JOIN CashAccountEntry USING (account_id)
        )
        AND account_subtype = 'DEBT'
    GROUP BY
        credit_card_holder_id
)
SELECT
    first_name,
    last_name,
    -- i.e. the generic name of the account
    CreditCardProduct.account_name,
    last_payment_date,
    ROUND(
        SUM(
            ROUND(
                unit * (COALESCE(credit, 0) - COALESCE(debit, 0)),
                2
            )
        ),
        2
    ) AS balance,
    CreditCardPadSource.cash_account_holder_id IS NOT NULL AS has_pad
FROM
    FinancialEntry
    INNER JOIN Account USING (account_id)
    INNER JOIN CreditCardEntry USING (account_id)
    INNER JOIN CreditCardHolder USING (credit_card_holder_id)
    INNER JOIN CreditCardProduct USING (account_type_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    INNER JOIN Person USING (person_id)
    LEFT JOIN LastPayment USING (credit_card_holder_id)
    LEFT JOIN CreditCardPadSource USING (credit_card_holder_id)
WHERE
    account_subtype = 'DEBT'
GROUP BY
    account_id
HAVING
    balance <> 0;

CREATE VIEW DebtByPadSource AS
SELECT
    cash_account_holder_id,
    ROUND(
        SUM(
            ROUND(
                unit * (COALESCE(credit, 0.0) - COALESCE(debit, 0.0)),
                2
            )
        ),
        2
    ) AS total_debt
FROM
    CreditCardPadSource
    INNER JOIN CreditCardHolder USING (credit_card_holder_id)
    INNER JOIN CreditCardEntry USING (credit_card_holder_id)
    INNER JOIN FinancialEntry USING (account_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
WHERE
    account_subtype = 'DEBT'
GROUP BY
    cash_account_holder_id;

CREATE VIEW PadSourceBalance AS
SELECT
    cash_account_holder_id,
    SUM(
        unit * (COALESCE(debit, 0) - COALESCE(credit, 0))
    ) AS source_balance
FROM
    (
        SELECT
            DISTINCT cash_account_holder_id
        FROM
            CreditCardPadSource
    )
    INNER JOIN CashAccountEntry USING(cash_account_holder_id)
    INNER JOIN FinancialEntry USING (account_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
WHERE
    account_subtype = 'CASH'
GROUP BY
    cash_account_holder_id;

CREATE VIEW RequiredPadInjection AS
SELECT
    first_name || ' ' || last_name AS name,
    account_name,
    COALESCE(total_debt, 0.0) - source_balance AS min_injection
FROM
    PadSourceBalance
    LEFT JOIN DebtByPadSource USING (cash_account_holder_id)
    INNER JOIN CashAccountHolder USING (cash_account_holder_id)
    INNER JOIN Person USING (person_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
WHERE
    min_injection > 0;

-- quick test
SELECT
    *
FROM
    CreditCardView;

CREATE VIEW ExpenseTransaction AS
SELECT
    FinancialEntry.*
FROM
    FinancialEntry
    INNER JOIN Account USING (account_id)
    INNER JOIN AccountType USING (account_type_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    INNER JOIN AccountKind USING (account_kind_id)
WHERE
    account_kind IN ('ASSET', 'LIABILITIES')
    AND (
        transaction_id IN (
            SELECT
                transaction_id
            FROM
                FinancialEntry
                INNER JOIN Account USING (account_id)
                INNER JOIN AccountType USING (account_type_id)
                INNER JOIN AccountSubtype USING (account_subtype_id)
                INNER JOIN AccountKind USING (account_kind_id)
            WHERE
                account_kind = 'EXPENSE'
                AND account_type_id NOT IN (
                    -- this ensures we're not counting account fees
                    SELECT
                        account_type_id
                    FROM
                        CashAccountProduct
                    UNION
                    ALL -- this ensures we're not counting employment income
                    SELECT
                        account_type_id
                    FROM
                        IncomeAccount
                )
            EXCEPT
            SELECT
                -- exclude transactions related to prepaid credits (there's a separate query for PrepaidAccount)
                transaction_id
            FROM
                FinancialEntry
                INNER JOIN Account USING (account_id)
                INNER JOIN PrepaidAccount USING (account_type_id)
        )
    );

-- quick test
SELECT
    *
FROM
    ExpenseTransaction;

CREATE VIEW PrepaidTransaction AS
SELECT
    FinancialEntry.*
FROM
    FinancialEntry
    INNER JOIN Account USING (account_id) -- INNER JOIN PrepaidAccount USING (account_type_id)
    INNER JOIN AccountSubtype USING (account_subtype_id)
    INNER JOIN AccountKind USING (account_kind_id)
WHERE
    -- we're interested in cashback for *expense* (i.e. crediting asset accounts)
    credit IS NOT NULL
    AND -- is a prepaid account
    transaction_id IN (
        SELECT
            transaction_id
        FROM
            FinancialEntry
            INNER JOIN Account USING (account_id)
            INNER JOIN PrepaidAccount USING (account_type_id)
        EXCEPT
        SELECT
            -- include transactions that related to prepaid credit spendings
            transaction_id
        FROM
            FinancialEntry
            INNER JOIN Account USING (account_id)
            INNER JOIN AccountSubtype USING (account_subtype_id)
            INNER JOIN AccountKind USING (account_kind_id)
        WHERE
            account_kind <> 'ASSET'
    );

CREATE VIEW CashbackTransaction AS
SELECT
    *
FROM
    ExpenseTransaction
UNION
ALL
SELECT
    *
FROM
    PrepaidTransaction;

CREATE VIEW CashbackTransactionBalance AS
SELECT
    transaction_id,
    date,
    ROUND(
        unit * (COALESCE(credit, 0.0) - COALESCE(debit, 0.0)),
        2
    ) AS balance,
    COALESCE(cashback_rate, 0.0) AS cashback_rate
FROM
    CashbackTransaction
    INNER JOIN TransactionStore USING(transaction_id)
    INNER JOIN OwnedAccount USING (account_id)
    LEFT JOIN StoreCashbackMapping USING (store_id, account_type_id)
    LEFT JOIN CashbackCategory USING (account_type_id, cashback_category_name_id);

-- quick test
SELECT
    *
FROM
    CashbackTransactionBalance;

CREATE VIEW CashbackWithAmex AS
SELECT
    transaction_id,
    MAX(cashback_rate) AS cashback_rate
FROM
    TransactionStore
    INNER JOIN StoreCashbackMapping USING (store_id)
    INNER JOIN Store USING (store_id)
    INNER JOIN CashbackCategory USING(account_type_id, cashback_category_name_id)
GROUP BY
    transaction_id,
    store_id;

-- quick test
SELECT
    *
FROM
    CashbackWithAmex;

CREATE VIEW CashbackNoAmex AS
SELECT
    transaction_id,
    MAX(cashback_rate) AS cashback_rate
FROM
    TransactionStore
    INNER JOIN StoreCashbackMapping USING (store_id)
    INNER JOIN Store USING (store_id)
    INNER JOIN CashbackCategory USING (account_type_id, cashback_category_name_id)
    INNER JOIN CreditCardProduct USING (account_type_id)
    INNER JOIN Institution USING (institution_id)
WHERE
    institution_name <> 'American Express'
GROUP BY
    transaction_id,
    store_id;

-- quick test
SELECT
    *
FROM
    CashbackNoAmex;

CREATE VIEW CashbackEstimate AS
SELECT
    transaction_id,
    date,
    balance,
    Actual.cashback_rate AS actual_cashback_rate,
    ROUND (Actual.balance * Actual.cashback_rate, 2) AS actual_cashback,
    CashbackWithAmex.cashback_rate AS with_amex_cashback_rate,
    ROUND(balance * CashbackWithAmex.cashback_rate, 2) AS with_amex_cashback,
    CashbackNoAmex.cashback_rate AS without_amex_cashback_rate,
    ROUND(balance * CashbackNoAmex.cashback_rate, 2) AS without_amex_cashback
FROM
    CashbackTransactionBalance Actual
    INNER JOIN CashbackWithAmex USING (transaction_id)
    INNER JOIN CashbackNoAmex USING (transaction_id);

-- quick test
SELECT
    *
FROM
    CashbackEstimate;

-- Conduct a backtest of purchases made in the last 30 days to evaluate whether using the Amex SimplyCash Preferred Card is worthwhile.
CREATE VIEW JustifyAmex AS
SELECT
    CAST(
        strftime(
            '%Y',
            DATE(date, 'unixepoch')
        ) AS INTEGER
    ) AS year,
    CAST(
        strftime(
            '%m',
            DATE(date, 'unixepoch')
        ) AS INTEGER
    ) AS MONTH,
    -- Represents the total spending balance from 30 days prior to the current date.
    SUM(balance) AS balance,
    -- Calculate the average extra cashback rate for optimally using Amex.
    SUM(
        (
            with_amex_cashback_rate - without_amex_cashback_rate
        ) * balance
    ) / SUM (balance) AS extra_cashback_rate,
    -- Compute the theoretical cashback if my credit cards were used with Amex.
    SUM(with_amex_cashback) AS with_amex_cashback,
    -- Compute the theoretical cashback if my credit cards were used without Amex.
    SUM(without_amex_cashback) AS without_amex_cashback,
    -- Compute the theoretical extra cashback earned by using Amex compared to not using it.
    SUM(with_amex_cashback - without_amex_cashback) AS extra_cashback,
    -- Compute the theoretical extra cashback earned by using Amex compared to not using it, net of fees.
    SUM(with_amex_cashback - without_amex_cashback) - (
        SELECT
            annual_fee
        FROM
            CreditCardProduct
            INNER JOIN Institution USING (institution_id)
        WHERE
            institution_name = 'American Express'
    ) / 12 AS extra_cashback_after_fee,
    -- Calculate the missed cashback opportunities where Amex was available, but not used.
    SUM(actual_cashback - with_amex_cashback) AS missed_opportunities
FROM
    CashbackEstimate
GROUP BY
    strftime(
        '%Y-%m',
        DATE(date, 'unixepoch', 'localtime')
    )
ORDER BY
    year DESC,
    MONTH DESC
LIMIT
    12;

-- quick test
SELECT
    *
FROM
    JustifyAmex;