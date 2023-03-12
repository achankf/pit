use serde::Serialize;

use crate::{SqlResult, Transaction};

#[derive(Serialize, Debug)]
pub struct CreditCardPadInjection {
    pub name: String,
    pub account_name: String,
    pub min_injection: f64,
}

impl Transaction<'_> {
    pub async fn get_credit_card_pad_injection(
        &mut self,
    ) -> SqlResult<Vec<CreditCardPadInjection>> {
        let result = sqlx::query_as!(
            CreditCardPadInjection,
            r#"
SELECT
    name,
    account_name,
    min_injection AS "min_injection!:f64"
FROM
    RequiredPadInjection
"#
        )
        .fetch_all(&mut *self.0)
        .await?;

        Ok(result)
    }
}
