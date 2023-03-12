use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct CashbackCategory {
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
    #[serde(deserialize_with = "string_trim")]
    pub cashback_category_name: String,
    pub cashback_rate: f64,
}

impl Id for CashbackCategory {
    type IdType = (String, String);

    fn id(&self) -> Self::IdType {
        (
            self.account_type.clone(),
            self.cashback_category_name.clone(),
        )
    }
}

impl Query for CashbackCategory {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    CashbackCategory (
        account_type_id,
        cashback_category_name_id,
        cashback_rate
    )
VALUES
    (
        (
            SELECT
                account_type_id
            FROM
                AccountType
            WHERE
                account_type = ?
        ),
        (
            SELECT
                cashback_category_name_id
            FROM
                CashbackCategoryName
            WHERE
                cashback_category_name = ?
        ),
        ?
    ) ON CONFLICT(account_type_id, cashback_category_name_id) DO
UPDATE
SET
    cashback_rate = excluded.cashback_rate
WHERE
    cashback_rate <> excluded.cashback_rate
"#,
            self.account_type,
            self.cashback_category_name,
            self.cashback_rate
        )
    }
}
