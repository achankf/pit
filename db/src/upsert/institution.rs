use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Institution {
    #[serde(deserialize_with = "string_trim")]
    institution_name: String,
}

impl Id for Institution {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.institution_name.clone()
    }
}

impl Query for Institution {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO Institution (institution_name)
VALUES
    (?);
"#,
            self.institution_name
        )
    }
}
