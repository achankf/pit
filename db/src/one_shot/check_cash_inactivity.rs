use chrono::{DateTime, Local};

use crate::{SqlResult, Transaction};

pub struct InactivityCheckResult {
    pub first_name: String,
    pub last_name: String,
    pub inactive_fee_months: i64,
    pub latest_transaction: DateTime<Local>,
    pub account_name: String,
    pub should_make_a_move: bool,
}

impl Transaction<'_> {
    pub async fn check_cash_inactivity(&mut self) -> SqlResult<Vec<InactivityCheckResult>> {
        struct Inactivity {
            first_name: String,
            last_name: String,
            inactive_fee_months: i64,
            latest_transaction: DateTime<Local>,
            account_name: String,
        }

        let checks = sqlx::query_as!(
            Inactivity,
            r#"
WITH LatestTransaction AS (
    SELECT
        cash_account_holder_id,
        MAX(date) AS latest_transaction
    FROM
        FinancialEntry
        INNER JOIN CashAccountEntry USING (account_id)
        INNER JOIN CashAccountHolder USING (cash_account_holder_id)
        INNER JOIN CashAccountProduct USING (account_type_id)
        INNER JOIN AccountSubtype USING (account_subtype_id)
    WHERE
        inactive_fee_months < 120
    GROUP BY
        cash_account_holder_id
)
SELECT
    first_name,
    last_name,
    inactive_fee_months,
    latest_transaction AS "latest_transaction!:DateTime<Local>",
    account_name
FROM
    LatestTransaction
    INNER JOIN CashAccountHolder USING (cash_account_holder_id)
    INNER JOIN CashAccountProduct USING (account_type_id)
    INNER JOIN Person USING (person_id)
ORDER BY
    person_id,
    account_name;
"#
        )
        .fetch_all(&mut *self.0)
        .await?;

        let result = checks
            .into_iter()
            .map(|record| {
                //
                let last_activity_days = (Local::now() - record.latest_transaction).num_days();

                const AVG_NUM_DAYS_IN_MONTH: f64 = 30.437;
                let account_inactivity_fee_period =
                    (record.inactive_fee_months as f64 * AVG_NUM_DAYS_IN_MONTH) as i64;

                // make a transaction 2 weeks before you're charged with inactivity fees
                let should_make_a_move = account_inactivity_fee_period - last_activity_days < 10;

                InactivityCheckResult {
                    first_name: record.first_name,
                    last_name: record.last_name,
                    inactive_fee_months: record.inactive_fee_months,
                    latest_transaction: record.latest_transaction,
                    account_name: record.account_name,
                    should_make_a_move,
                }
            })
            .collect();

        Ok(result)
    }
}
