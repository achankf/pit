use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct GeneralAccountName {
    pub general_account_name: String,
}

impl Id for GeneralAccountName {
    type IdType = String;

    fn id(&self) -> String {
        self.general_account_name.clone()
    }
}

impl Query for GeneralAccountName {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO GeneralAccountName (general_account_name)
VALUES
    (?);
"#,
            self.general_account_name,
        )
    }
}
