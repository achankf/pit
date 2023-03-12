use serde::Deserialize;
use sqlx::{sqlite::SqliteRow, Row};

use crate::{SqlResult, Transaction};

#[derive(Clone, Deserialize, Debug)]
pub struct AssetRebalance {
    pub first_name: String,
    pub last_name: String,
    pub asset_class_name: String,
    pub current_rebalance_amount: f64,
    pub potential_rebalance_amount: f64,
}

impl From<SqliteRow> for AssetRebalance {
    fn from(row: SqliteRow) -> Self {
        Self {
            first_name: row.get(0),
            last_name: row.get(1),
            asset_class_name: row.get(2),
            current_rebalance_amount: row.get(3),
            potential_rebalance_amount: row.get(4),
        }
    }
}

impl Transaction<'_> {
    pub async fn get_asset_rebalance(&mut self) -> SqlResult<Vec<AssetRebalance>> {
        let rows = sqlx::query(
            r#"
SELECT
    first_name,
    last_name,
    asset_class_name,
    current_rebalance_amount,
    potential_rebalance_amount
FROM
    AllocationView
    INNER JOIN Person USING (person_id)
    INNER JOIN AssetClass USING (asset_class_id)
    INNER JOIN AssetClassName USING (asset_class_name_id)
ORDER BY
    Person.person_id, current_rebalance_amount DESC
"#,
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
