use std::path::PathBuf;

use crate::{AssetClass, Transaction};

impl<'c> Transaction<'c> {
    pub async fn upsert_asset_class(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.upsert_all_in_order::<AssetClass>(&csv_path.join("asset_class.csv"))
            .await?;

        struct Check {
            is_each_person_has_model: bool,
        }

        // make sure each person has a model
        let result = sqlx::query_as!(
            Check,
            r#"
SELECT
    count(*) > 0 as "is_each_person_has_model!:bool"
FROM
    Person
WHERE
    person_id NOT IN (
        SELECT
            person_id
        FROM
            AssetClass
    )
"#
        )
        .fetch_optional(&mut *self.0)
        .await?;

        if let Some(Check {
            is_each_person_has_model,
        }) = result
        {
            if is_each_person_has_model {
                return Ok(());
            }
        }

        Err("Someone does not have a model (AssetClass)".into())
    }
}
