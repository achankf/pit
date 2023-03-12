CREATE VIEW CreditCardView (
    account_name,
    last_payment_date,
    currency,
    currency_symbol,
    balance
) AS WITH CreditCardRecords AS (
    SELECT
        general_account_id
    FROM
        GeneralAccount
        INNER JOIN GeneralAccountType USING (general_account_type_id)
    WHERE
        general_account_type = 'CREDIT-CARD'
)
SELECT
    GeneralAccount.description,
    (
        SELECT
            MAX(date)
        FROM
            CreditCardRecords
        WHERE
            general_account_id = t.general_account_id
            AND debit > 0
    ),
    currency,
    currency_symbol,
    ROUND(
        SUM(
            ROUND(
                unit * (COALESCE(credit, 0) - COALESCE(debit, 0)),
                2
            )
        ),
        2
    )
FROM
    CreditCardRecords t
    INNER JOIN GeneralTransaction USING (general_account_id)
    INNER JOIN GeneralAccount USING (general_account_id)
    INNER JOIN GeneralAccountType USING (general_account_type_id)
    INNER JOIN Currency USING (currency_id)
    INNER JOIN Person USING (person_id)
GROUP BY
    general_account_id;