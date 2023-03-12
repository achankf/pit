use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct StoreCashbackMapping {
    #[serde(deserialize_with = "string_trim")]
    pub store_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub account_type: String,
    #[serde(deserialize_with = "string_trim")]
    pub cashback_category_name: String,
}

impl Id for StoreCashbackMapping {
    type IdType = (String, String);

    fn id(&self) -> Self::IdType {
        (self.store_key.clone(), self.account_type.clone())
    }
}

impl Query for StoreCashbackMapping {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    StoreCashbackMapping (store_id, account_type_id, cashback_category_name_id)
VALUES
    (
        (
            SELECT
                store_id
            FROM
                Store
            WHERE
                store_key = ?
        ),
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
        )
    ) ON CONFLICT(store_id, account_type_id) DO
UPDATE
SET
    cashback_category_name_id = excluded.cashback_category_name_id
WHERE
    cashback_category_name_id <> excluded.cashback_category_name_id
"#,
            self.store_key,
            self.account_type,
            self.cashback_category_name
        )
    }
}
