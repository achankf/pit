use serde::Serialize;
use sqlx::types::chrono::{DateTime, Local};

use crate::{SqlResult, Transaction};

#[derive(Serialize, Debug)]
pub struct CurrentCredit {
    pub first_name: String,
    pub last_name: String,
    pub account_name: String,
    pub last_payment_date: Option<DateTime<Local>>,
    pub balance: f64,
    pub has_pad: bool,
}

impl Transaction<'_> {
    pub async fn get_current_credit_card_balance(&mut self) -> SqlResult<Vec<CurrentCredit>> {
        let result = sqlx::query_as!(
            CurrentCredit,
            r#"
SELECT
    first_name,
    last_name,
    account_name AS "account_name",
    last_payment_date AS "last_payment_date:DateTime<Local>",
    balance AS "balance!:f64",
    has_pad AS "has_pad:bool"
FROM
    CreditCardView;
"#
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(result)
    }
}
