use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Account {
    pub account_key: String,
    pub account_subtype: String,
    pub account_type: String,
    pub stock_ticker: Option<String>,
    pub account_name: String,
}

impl Id for Account {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.account_key.to_string()
    }
}

impl Query for Account {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    Account (
        account_key,
        account_subtype_id,
        account_type_id,
        account_name
    )
VALUES
    (
        ?,
        (
            SELECT
                account_subtype_id
            FROM
                AccountSubtype
            WHERE
                account_subtype = ?
        ),
        (
            SELECT
                account_type_id
            FROM
                AccountType
            WHERE
                account_type = ?
        ),
        ?
    ) ON CONFLICT(account_key) DO
UPDATE
SET
    account_subtype_id = excluded.account_subtype_id,
    account_type_id = excluded.account_type_id,
    account_type_id = excluded.account_type_id,
    account_name = excluded.account_name
WHERE
    account_subtype_id <> excluded.account_subtype_id
    OR account_type_id <> excluded.account_type_id
    OR account_type_id <> excluded.account_type_id
    OR account_name <> excluded.account_name
"#,
            self.account_key,
            self.account_subtype,
            self.account_type,
            self.account_name,
        )
    }
}
