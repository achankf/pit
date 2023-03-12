use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct CashbackCategoryName {
    #[serde(deserialize_with = "string_trim")]
    pub cashback_category_name: String,
}

impl Id for CashbackCategoryName {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.cashback_category_name.clone()
    }
}

impl Query for CashbackCategoryName {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO CashbackCategoryName (cashback_category_name)
VALUES
    (?)
"#,
            self.cashback_category_name,
        )
    }
}
