use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct Institution {
    name: String,
}

impl Id for Institution {
    type IdType = String;

    fn id(&self) -> String {
        self.name.clone()
    }
}

impl Query for Institution {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO Institution (name)
VALUES
    (?);
"#,
            self.name
        )
    }
}
