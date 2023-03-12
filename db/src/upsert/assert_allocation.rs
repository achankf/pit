use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct AssetAllocation {
    #[serde(deserialize_with = "string_trim")]
    pub ticker: String,
    #[serde(deserialize_with = "string_trim")]
    pub asset_class_name: String,
    pub weight: f64,
}

impl Id for AssetAllocation {
    type IdType = (String, String);

    fn id(&self) -> Self::IdType {
        (self.ticker.clone(), self.asset_class_name.clone())
    }
}

impl Query for AssetAllocation {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    AssetAllocation (
        asset_class_name_id,
        security_id,
        weight
    )
VALUES
    (
        (
            SELECT
                asset_class_name_id
            FROM
                AssetClassName
            WHERE
                asset_class_name = ?
        ),
        (
            SELECT
                security_id
            FROM
                SECURITY
            WHERE
                ticker = ?
        ),
        ?
    ) ON CONFLICT(
        asset_class_name_id,
        security_id
    ) DO
UPDATE
SET
    weight = excluded.weight
WHERE
    weight <> excluded.weight
"#,
            self.asset_class_name,
            self.ticker,
            self.weight
        )
    }
}
