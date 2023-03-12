use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct GeneralAccountType {
    pub general_account_kind: String,
    pub general_account_type: String,
}

impl Id for GeneralAccountType {
    type IdType = String;

    fn id(&self) -> String {
        self.general_account_type.clone()
    }
}

impl Query for GeneralAccountType {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    GeneralAccountType (
        general_account_kind_id,
        general_account_type
    )
VALUES
    (
        (
            SELECT
                general_account_kind_id
            FROM
                GeneralAccountKind
            WHERE
                general_account_kind = ?
        ),
        ?
    ) ON CONFLICT(general_account_type) DO
UPDATE
SET
    general_account_kind_id = excluded.general_account_kind_id
WHERE
    excluded.general_account_kind_id = general_account_kind_id;
"#,
            self.general_account_kind,
            self.general_account_type,
        )
    }
}
