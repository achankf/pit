use common::Id;
use serde::Deserialize;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct CashAccount {
    pub general_account_key: String,
    pub general_account_name: String,
    pub institution: String,
    pub tax_shelter_type: String,
    pub min_balance_waiver: f64,
    #[serde(default = "i64max")]
    pub inactive_fee_months: i64,
}

pub fn i64max() -> i64 {
    // essentially no inactivity fee
    i64::MAX
}

impl Id for CashAccount {
    type IdType = String;

    fn id(&self) -> String {
        self.general_account_key.clone()
    }
}

impl Query for CashAccount {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT INTO
    CashAccount (
        general_account_id,
        general_account_name_id,
        institution_id,
        tax_shelter_type_id,
        min_balance_waiver,
        inactive_fee_months
    )
VALUES
    (
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
                general_account_name_id
            FROM
                GeneralAccountName
            WHERE
                general_account_name = ?
        ),
        (
            SELECT
                institution_id
            FROM
                Institution
            WHERE
                name = ?
        ),
        (
            SELECT
                tax_shelter_type_id
            FROM
                TaxShelterType
            WHERE
                tax_shelter_type = ?
        ),
        ?,
        ?
    ) ON CONFLICT(general_account_id) DO
UPDATE
SET
    general_account_name_id = excluded.general_account_name_id,
    institution_id = excluded.institution_id,
    tax_shelter_type_id = excluded.tax_shelter_type_id,
    min_balance_waiver = excluded.min_balance_waiver,
    inactive_fee_months = excluded.inactive_fee_months
"#,
            self.general_account_key,
            self.general_account_name,
            self.institution,
            self.tax_shelter_type,
            self.min_balance_waiver,
            self.inactive_fee_months
        )
    }
}
