use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct AccountSubtype {
    #[serde(deserialize_with = "string_trim")]
    pub account_kind: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_subtype: String,
}

impl Id for AccountSubtype {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.account_subtype.clone()
    }
}

impl Query for AccountSubtype {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    AccountSubtype (
        account_kind_id,
        account_subtype
    )
VALUES
    (
        (
            SELECT
                account_kind_id
            FROM
                AccountKind
            WHERE
                account_kind = ?
        ),
        ?
    ) ON CONFLICT(account_subtype) DO
UPDATE
SET
    account_kind_id = excluded.account_kind_id
WHERE
    excluded.account_kind_id <> account_kind_id;
"#,
            self.account_kind,
            self.account_subtype,
        )
    }
}
