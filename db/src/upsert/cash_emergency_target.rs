use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct CashEmergencyTarget {
    pub person: String,
    pub general_account_key: String,
    pub currency: String,
    pub emergency_target: f64,
}

impl Id for CashEmergencyTarget {
    type IdType = (String, String);

    fn id(&self) -> (String, String) {
        (self.person.clone(), self.general_account_key.clone())
    }
}

impl Query for CashEmergencyTarget {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    CashEmergencyTarget (
        person_id,
        general_account_id,
        currency_id,
        emergency_target
    )
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
                general_account_id
            FROM
                GeneralAccount
            WHERE
                general_account_key = ?
        ),
        (
            SELECT
                currency_id
            FROM
                Currency
            WHERE
                currency = ?
        ),
        ?
    ) ON CONFLICT(person_id, general_account_id, currency_id) DO
UPDATE
SET
    emergency_target = excluded.emergency_target
"#,
            self.person,
            self.general_account_key,
            self.currency,
            self.emergency_target
        )
    }
}
