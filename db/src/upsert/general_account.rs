use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct GeneralAccount {
    pub general_account_key: String,
    pub general_account_name: String,
    pub general_account_type: String,
    pub stock_ticker: Option<String>,
    pub description: String,
    pub note: String,
}

impl Id for GeneralAccount {
    type IdType = String;

    fn id(&self) -> String {
        self.general_account_key.to_string()
    }
}

impl Query for GeneralAccount {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    GeneralAccount (
        general_account_key,
        general_account_name_id,
        general_account_type_id,
        description,
        note
    )
VALUES
    (
        ?,
        (
            SELECT
                general_account_name_id
            FROM
                GeneralAccountName
            WHERE
                general_account_name = ?
        ),
        (
            SELECT
                general_account_type_id
            FROM
                GeneralAccountType
            WHERE
                general_account_type = ?
        ),
        ?,
        ?
    ) ON CONFLICT(general_account_key) DO
UPDATE
SET
    general_account_name_id = excluded.general_account_name_id,
    general_account_type_id = excluded.general_account_type_id,
    general_account_type_id = excluded.general_account_type_id,
    description = excluded.description,
    note = excluded.note;
"#,
            self.general_account_key,
            self.general_account_name,
            self.general_account_type,
            self.description,
            self.note
        )
    }
}
