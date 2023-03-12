use serde::Deserialize;

use crate::{SqlResult, Transaction};

#[derive(Deserialize, Debug)]
pub struct EmergencyRebalance {
    pub first_name: String,
    pub last_name: String,
    pub account_name: String,
    pub currency: String,
    pub currency_symbol: String,
    pub unallocated_fund: f64,
    pub injection_needed: f64,
}

impl<'c> Transaction<'c> {
    pub async fn get_emergency_rebalance(&mut self) -> SqlResult<Vec<EmergencyRebalance>> {
        let result = sqlx::query_as!(
            EmergencyRebalance,
            r#"
SELECT
    first_name,
    last_name,
    GeneralAccount.description AS account_name,
    currency AS "currency!:String",
    currency_symbol AS "currency_symbol!:String",
    unallocated_fund AS "unallocated_fund!:f64",
    injection_needed AS "injection_needed!:f64"
FROM
    CashView
    INNER JOIN GeneralAccount USING (general_account_id)
    INNER JOIN Person USING (person_id)
    INNER JOIN Currency USING (currency_id)
WHERE
    unallocated_fund <> 0 OR injection_needed <> 0
ORDER BY
    first_name,
    last_name,
    account_name;
"#
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(result)
    }
}
