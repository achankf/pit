use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct AssetClassName {
    #[serde(deserialize_with = "string_trim")]
    pub asset_class_name: String,
}

impl Id for AssetClassName {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.asset_class_name.clone()
    }
}

impl Query for AssetClassName {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO AssetClassName (asset_class_name)
VALUES
    (?)
"#,
            self.asset_class_name
        )
    }
}
