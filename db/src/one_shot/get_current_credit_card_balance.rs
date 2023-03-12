use serde::Serialize;
use sqlx::types::chrono::{DateTime, Local};

use crate::{SqlResult, Transaction};

#[derive(Serialize, Debug)]
pub struct CurrentCredit {
    pub account_name: String,
    pub last_payment_date: Option<DateTime<Local>>,
    pub currency: String,
    pub currency_symbol: String,
    pub balance: f64,
}

impl<'c> Transaction<'c> {
    pub async fn get_current_credit_card_balance(&mut self) -> SqlResult<Vec<CurrentCredit>> {
        let result = sqlx::query_as!(
            CurrentCredit,
            r#"
SELECT
    account_name AS "account_name",
    last_payment_date AS "last_payment_date:DateTime<Local>",
    currency AS "currency!:String",
    currency_symbol AS "currency_symbol!:String",
    balance AS "balance!:f64"
FROM
    CreditCardView;
"#
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(result)
    }
}
