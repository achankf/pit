use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct Person {
    #[serde(deserialize_with = "string_trim")]
    pub person_key: String,
    #[serde(deserialize_with = "string_trim")]
    pub first_name: String,
    #[serde(deserialize_with = "string_trim")]
    pub last_name: String,
}

impl Id for Person {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.person_key.clone()
    }
}

impl Query for Person {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    Person (person_key, first_name, last_name)
VALUES
    (?, ?, ?) ON CONFLICT(person_key) DO
UPDATE
SET
    first_name = excluded.first_name,
    last_name = excluded.last_name
WHERE
    first_name <> excluded.first_name
    OR last_name <> excluded.last_name
"#,
            self.person_key,
            self.first_name,
            self.last_name
        )
    }
}
