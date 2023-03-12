use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct PrepaidAccount {
    #[serde(deserialize_with = "string_trim")]
    account_type: String,
}

impl Id for PrepaidAccount {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.account_type.clone()
    }
}

impl Query for PrepaidAccount {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO PrepaidAccount (account_type_id)
VALUES
    (
        (
            SELECT
                account_type_id
            FROM
                AccountType
            WHERE
                account_type = ?
        )
    )
"#,
            self.account_type
        )
    }
}
