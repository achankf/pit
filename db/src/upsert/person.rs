use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Deserialize, Debug)]
pub struct Person {
    pub short_name: String,
    pub first_name: String,
    pub last_name: String,
}

impl Id for Person {
    type IdType = String;

    fn id(&self) -> String {
        self.short_name.clone()
    }
}

impl Query for Person {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    Person (short_name, first_name, last_name)
VALUES
    (?, ?, ?) ON CONFLICT(short_name) DO
UPDATE
SET
    first_name = excluded.first_name,
    last_name = excluded.last_name;
"#,
            self.short_name,
            self.first_name,
            self.last_name
        )
    }
}
