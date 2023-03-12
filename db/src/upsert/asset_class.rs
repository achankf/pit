use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct AssetClass {
    pub person: String,
    pub parent: String,
    pub class: String,
    pub weight: f64,
}

impl Id for AssetClass {
    type IdType = (String, String);

    fn id(&self) -> (String, String) {
        (self.person.clone(), self.class.clone())
    }
}

impl Query for AssetClass {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    AssetClass (person_id, parent_id, class, weight)
VALUES
    (
        (
            SELECT
                person_id
            FROM
                Person
            WHERE
                short_name = ?
        ),
        (
            SELECT
                asset_class_id
            FROM
                AssetClass
            WHERE
                class = ?
                AND person_id = (
                    SELECT
                        person_id
                    FROM
                        Person
                    WHERE
                        short_name = ?
                )
        ),
        ?,
        ?
    ) ON CONFLICT(person_id, class) DO
UPDATE
SET
    weight = excluded.weight
"#,
            self.person,
            self.parent,
            self.person,
            self.class,
            self.weight,
        )
    }
}
