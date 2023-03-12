CREATE TABLE LastMarketPriceUpdate (date INTEGER NOT NULL) STRICT;

INSERT INTO
    LastMarketPriceUpdate (date)
VALUES
    (0);

CREATE TABLE Currency (
    currency_id INTEGER NOT NULL PRIMARY KEY,
    currency TEXT NOT NULL UNIQUE CHECK (currency = UPPER(currency)),
    currency_name TEXT NOT NULL UNIQUE,
    currency_symbol TEXT,
    -- This is used when doing aggregation which requirs market price calculation
    market_exchange_rate REAL NOT NULL DEFAULT 1.0 CHECK (market_exchange_rate >= 0),
    -- Only the "main" currency can be NULL
    yahoo_ticker TEXT UNIQUE
) STRICT;

CREATE TRIGGER Currency_trigger_insert_AtMostOneYahooTicker
AFTER
INSERT
    ON Currency BEGIN
SELECT
    CASE
        WHEN COUNT(*) <> 1 THEN RAISE(
            FAIL,
            'Currency table must have one NULL yahoo_ticker for the main currency'
        )
    END
FROM
    Currency
WHERE
    yahoo_ticker IS NULL;

END;

CREATE TRIGGER Currency_trigger_update_AtMostOneYahooTicker
AFTER
UPDATE
    ON Currency BEGIN
SELECT
    CASE
        WHEN COUNT(*) <> 1 THEN RAISE(
            FAIL,
            'Currency table must have one NULL yahoo_ticker for the main currency'
        )
    END
FROM
    Currency
WHERE
    yahoo_ticker IS NULL;

END;

CREATE TABLE Exchange (
    exchange_id INTEGER NOT NULL PRIMARY KEY,
    exchange_name TEXT UNIQUE NOT NULL CHECK (exchange_name = UPPER(exchange_name)),
    long_name TEXT UNIQUE NOT NULL UNIQUE
) STRICT;

CREATE TABLE SECURITY(
    security_id INTEGER NOT NULL PRIMARY KEY,
    exchange_id INTEGER NOT NULL REFERENCES Exchange(exchange_id),
    currency_id INTEGER NOT NULL REFERENCES Currency(currency_id),
    ticker TEXT NOT NULL UNIQUE,
    yahoo_ticker TEXT NOT NULL UNIQUE,
    long_name TEXT NOT NULL UNIQUE,
    price REAL NOT NULL DEFAULT 0.0 CHECK(price >= 0)
) STRICT;

CREATE INDEX Security_idx_Exchange ON SECURITY(exchange_id);

CREATE TABLE Person (
    person_id INTEGER NOT NULL PRIMARY KEY,
    short_name TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL
) STRICT;

CREATE INDEX Person_idx_Name ON Person(first_name, last_name);

CREATE INDEX Person_idx_LastName ON Person(last_name);

CREATE TABLE AssetClass(
    asset_class_id INTEGER NOT NULL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES Person(person_id),
    parent_id INTEGER REFERENCES AssetClass(asset_class_id),
    class TEXT NOT NULL,
    weight REAL NOT NULL CHECK(weight >= 0),
    UNIQUE(person_id, class)
) STRICT;

CREATE INDEX AssetClass_idx_Class ON AssetClass(class);

CREATE INDEX AssetClass_idx_ParentId ON AssetClass(parent_id);

CREATE TRIGGER AssetClass_trigger_insert_ExactlyOneParentPerPerson
AFTER
INSERT
    ON AssetClass BEGIN
SELECT
    CASE
        WHEN (
            SELECT
                EXISTS(
                    -- model exist for a person but doesn't have exactly one "root" that represents the portfolio
                    SELECT
                        COUNT(*)
                    FROM
                        AssetClass
                        INNER JOIN Person USING (person_id)
                    WHERE
                        parent_id IS NULL
                    GROUP BY
                        person_id
                    HAVING
                        COUNT(*) <> 1
                )
        ) THEN RAISE(
            FAIL,
            'Each person must have 1 root (i.e. parent is null) in AssetClass that represents their asset allocation model'
        )
    END;

END;

CREATE TABLE AssetAllocation (
    asset_allocation_id INTEGER NOT NULL PRIMARY KEY,
    asset_class_id INTEGER NOT NULL REFERENCES AssetClass(asset_class_id),
    security_id INTEGER NOT NULL REFERENCES SECURITY(security_id),
    weight REAL NOT NULL CHECK(weight >= 0),
    UNIQUE(asset_class_id, security_id)
) STRICT;

CREATE INDEX AssetAllocation_idx_Holding ON AssetAllocation(security_id);

CREATE TABLE Institution (
    institution_id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE EventHost (
    host_id INTEGER NOT NULL PRIMARY KEY,
    host_name TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE Event(
    event_id INTEGER NOT NULL PRIMARY KEY,
    host_id INTEGER NOT NULL REFERENCES EventHost(host_id),
    date INTEGER NOT NULL CHECK(date > 0),
    event_name TEXT NOT NULL,
    UNIQUE (host_id, date)
) STRICT;

CREATE TABLE GeneralAccountKind (
    general_account_kind_id INTEGER NOT NULL PRIMARY KEY,
    general_account_kind TEXT NOT NULL UNIQUE CHECK(
        general_account_kind = UPPER (general_account_kind)
    )
) STRICT;

INSERT INTO
    GeneralAccountKind(general_account_kind)
VALUES
    ('ASSET'),
    ('EQUITY'),
    ('EXPENSE'),
    ('LIABILITIES'),
    ('REVENUE');

CREATE TABLE GeneralAccountType(
    general_account_type_id INTEGER NOT NULL PRIMARY KEY,
    general_account_kind_id INTEGER NOT NULL REFERENCES GeneralAccountKind(general_account_kind_id),
    general_account_type TEXT NOT NULL UNIQUE CHECK(
        general_account_type = UPPER (general_account_type)
    )
) STRICT;

CREATE INDEX GeneralAccountType_idx_GeneralAccountKind ON GeneralAccountType (general_account_kind_id);

CREATE TABLE GeneralAccountName (
    general_account_name_id INTEGER NOT NULL PRIMARY KEY,
    general_account_name TEXT NOT NULL UNIQUE CHECK(
        general_account_name = UPPER (general_account_name)
    )
) STRICT;

CREATE TABLE GeneralAccount (
    general_account_id INTEGER NOT NULL PRIMARY KEY,
    general_account_key TEXT NOT NULL UNIQUE,
    general_account_name_id INTEGER NOT NULL REFERENCES GeneralAccountName(general_account_name_id),
    general_account_type_id INTEGER NOT NULL REFERENCES GeneralAccountType(general_account_type_id),
    description TEXT NOT NULL,
    note TEXT
) STRICT;

CREATE INDEX GeneralAccount_idx_GeneralAccountName ON GeneralAccount(general_account_name_id);

CREATE INDEX GeneralAccount_idx_GeneralAccountType ON GeneralAccount(general_account_type_id);

CREATE INDEX GeneralAccount_idx_Type_Name ON GeneralAccount(general_account_type_id, general_account_name_id);

CREATE TABLE StockAccount (
    general_account_id INTEGER NOT NULL PRIMARY KEY REFERENCES GeneralAccount(general_account_id),
    security_id INTEGER NOT NULL REFERENCES SECURITY(security_id)
) STRICT;

CREATE TABLE GeneralTransaction (
    -- purpose for row_id is to upsert entries (allow for correction)
    row_id INTEGER NOT NULL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES Person(person_id),
    date INTEGER NOT NULL CHECK (date >= 0),
    -- this is not a primary key. Each transaction can have multiple journal entries,
    -- so there can be multiple rows having the same transaction id. 
    transaction_id INTEGER NOT NULL,
    general_account_id INTEGER NOT NULL REFERENCES GeneralAccount(general_account_id),
    currency_id INTEGER NOT NULL REFERENCES Currency(currency_id),
    -- can contain fractional shares
    unit REAL NOT NULL CHECK(unit > 0),
    debit REAL CHECK(
        debit IS NULL
        OR debit > 0
    ),
    credit REAL CHECK(
        credit IS NULL
        OR credit > 0
    ),
    description TEXT NOT NULL,
    book_exchange_rate REAL NOT NULL,
    -- one of debit or credit must be null
    CHECK(
        (
            debit IS NOT NULL
            OR credit IS NOT NULL
        )
        AND NOT (
            debit IS NULL
            AND credit IS NULL
        )
    )
) STRICT;

CREATE INDEX GeneralTransaction_idx_TransactionId ON GeneralTransaction(transaction_id);

CREATE INDEX GeneralTransaction_idx_Date ON GeneralTransaction(date);

CREATE INDEX GeneralTransaction_idx_GeneralAccountId ON GeneralTransaction(general_account_id, date);

CREATE INDEX GeneralTransaction_idx_PersonAccount ON GeneralTransaction(person_id, general_account_id);

CREATE TABLE TaxShelterType (
    tax_shelter_type_id INTEGER NOT NULL PRIMARY KEY,
    tax_shelter_type TEXT NOT NULL UNIQUE CHECK(tax_shelter_type = UPPER(tax_shelter_type)),
    tax_shelter_name TEXT NOT NULL
) STRICT;

CREATE TABLE CashAccount (
    general_account_id INTEGER NOT NULL PRIMARY KEY REFERENCES GeneralAccount(general_account_id),
    general_account_name_id INTEGER NOT NULL REFERENCES GeneralAccountName(general_account_name_id),
    institution_id INTEGER NOT NULL REFERENCES Institution(institution_id),
    tax_shelter_type_id INTEGER NOT NULL REFERENCES TaxShelterType(tax_shelter_type_id),
    -- how much is need for the bank to waive its monthly fees
    min_balance_waiver REAL NOT NULL CHECK(min_balance_waiver >= 0),
    -- how many months before banks charges you for inactivity
    inactive_fee_months INTEGER NOT NULL CHECK (inactive_fee_months >= 0)
) STRICT;

CREATE INDEX CashAccount_idx_GeneralAccountNameId ON CashAccount(general_account_name_id);

CREATE TABLE CashEmergencyTarget (
    person_id INTEGER NOT NULL REFERENCES Person(person_id),
    general_account_id INTEGER NOT NULL REFERENCES GeneralAccount(general_account_id),
    currency_id INTEGER NOT NULL REFERENCES Currency(currency_id),
    emergency_target REAL NOT NULL CHECK(emergency_target >= 0),
    PRIMARY KEY (person_id, general_account_id, currency_id)
) STRICT;