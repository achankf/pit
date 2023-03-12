use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct AssetAllocation {
    pub ticker: String,
    pub class: String,
    pub weight: f64,
}

impl Id for AssetAllocation {
    type IdType = (String, String);

    fn id(&self) -> (String, String) {
        (self.ticker.clone(), self.class.clone())
    }
}

impl Query for AssetAllocation {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    AssetAllocation (
        asset_class_id,
        security_id,
        weight
    )
VALUES
    (
        (
            SELECT
                asset_class_id
            FROM
                AssetClass
            WHERE
                class = ?
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
        asset_class_id,
        security_id
    ) DO
UPDATE
SET
    weight = excluded.weight
WHERE
    weight <> excluded.weight
"#,
            self.class,
            self.ticker,
            self.weight
        )
    }
}
