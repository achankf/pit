use std::path::PathBuf;

use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::{Query, Transaction};

#[derive(Deserialize, Debug)]
pub struct AssetClass {
    #[serde(deserialize_with = "string_trim")]
    pub person: String,
    #[serde(deserialize_with = "string_trim")]
    pub parent: String,
    #[serde(deserialize_with = "string_trim")]
    pub asset_class_name: String,
    pub weight: f64,
}

impl Id for AssetClass {
    type IdType = (String, String);

    fn id(&self) -> Self::IdType {
        (self.person.clone(), self.asset_class_name.clone())
    }
}

impl Query for AssetClass {
    fn query(&self) -> crate::SqlQuery {
        println!("{:#?}", self);
        sqlx::query!(
            r#"
INSERT INTO
    AssetClass (
        person_id,
        parent_id,
        asset_class_name_id,
        weight
    )
VALUES
    (
        (
            SELECT
                person_id
            FROM
                Person
            WHERE
                person_key = ?
        ),
        (
            SELECT
                asset_class_id
            FROM
                AssetClass
            WHERE
                asset_class_name_id = (
                    SELECT
                        asset_class_name_id
                    FROM
                        AssetClassName
                    WHERE
                        asset_class_name = ?
                )
                AND person_id = (
                    SELECT
                        person_id
                    FROM
                        Person
                    WHERE
                        person_key = ?
                )
        ),
        (
            SELECT
                asset_class_name_id
            FROM
                AssetClassName
            WHERE
                asset_class_name = ?
        ),
        ?
    ) ON CONFLICT(person_id, asset_class_name_id) DO
UPDATE
SET
    weight = excluded.weight,
    parent_id = excluded.parent_id
WHERE
    weight <> excluded.weight
    OR parent_id <> excluded.parent_id
"#,
            self.person,
            self.parent,
            self.person,
            self.asset_class_name,
            self.weight,
        )
    }
}

impl Transaction<'_> {
    pub async fn upsert_asset_class(
        &mut self,
        csv_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.upsert_all_in_order::<AssetClass>(&csv_path).await?;

        struct Check {
            is_each_person_has_model: bool,
        }

        // make sure each person has a model
        let result = sqlx::query_as!(
            Check,
            r#"
SELECT
    NOT EXISTS (
        SELECT
            person_id
        FROM
            Person
        WHERE
            person_id NOT IN (
                SELECT
                    person_id
                FROM
                    AssetClass
            )
    ) AS "is_each_person_has_model!:bool"
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
