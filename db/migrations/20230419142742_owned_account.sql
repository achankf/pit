CREATE VIEW OwnedAccount AS
SELECT
    account_key,
    person_id,
    account_id,
    account_type_id,
    account_subtype_id,
    cash_account_holder_id AS holder_id,
    currency_id,
    Account.account_name
FROM
    CashAccountHolder
    INNER JOIN CashAccountEntry USING (cash_account_holder_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
    INNER JOIN Account USING (account_id, account_type_id, account_subtype_id)
UNION
ALL
SELECT
    account_key,
    person_id,
    account_id,
    account_type_id,
    account_subtype_id,
    credit_card_holder_id AS holder_id,
    currency_id,
    Account.account_name
FROM
    CreditCardHolder
    INNER JOIN CreditCardEntry USING (credit_card_holder_id)
    INNER JOIN CreditCardProduct USING (account_type_id)
    INNER JOIN Account USING (account_id, account_type_id, account_subtype_id)
UNION
ALL
SELECT
    account_key,
    person_id,
    account_id,
    account_type_id,
    account_subtype_id,
    stock_account_holder_id AS holder_id,
    currency_id,
    Account.account_name
FROM
    StockAccountHolder
    INNER JOIN StockAccountEntry USING (stock_account_holder_id)
    INNER JOIN Account USING (account_id, account_type_id, account_subtype_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
UNION
ALL
SELECT
    account_key,
    person_id,
    account_id,
    account_type_id,
    account_subtype_id,
    gic_account_holder_id AS holder_id,
    currency_id,
    Account.account_name
FROM
    GicAccountHolder
    INNER JOIN GicEntry USING (gic_account_holder_id)
    INNER JOIN Account USING (account_id, account_type_id, account_subtype_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
UNION
ALL
SELECT
    account_key,
    person_id,
    account_id,
    account_type_id,
    account_subtype_id,
    income_account_holder_id AS holder_id,
    currency_id,
    Account.account_name
FROM
    IncomeAccountHolder
    INNER JOIN IncomeAccountMapping USING (income_account_holder_id)
    INNER JOIN Account USING (account_id, account_type_id, account_subtype_id)
    INNER JOIN IncomeAccount USING (account_type_id);