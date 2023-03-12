mod acb;
mod balance;
mod cashflow;
mod expense;

use balance::report_balance;
use cashflow::report_cashflow;
use clap::Subcommand;
use db::{BalanceRecord, Db, NetBalanceRecord};
use expense::report_expense;
use tabled::Tabled;

use self::acb::report_acb;

#[derive(Tabled)]
struct BalanceRecordFormatted {
    #[tabled(rename = "Account Type")]
    pub account_type: String,
    #[tabled(rename = "Holder Name")]
    pub name: String,
    #[tabled(rename = "Account Name")]
    pub description: String,
    #[tabled(rename = "Balance")]
    pub balance: String,
}

impl From<BalanceRecord> for BalanceRecordFormatted {
    fn from(value: BalanceRecord) -> Self {
        Self {
            name: format!("{} {}", value.first_name, value.last_name),
            account_type: value.general_account_type,
            description: value.description,
            balance: format!(
                "{:.2}{} {}",
                value.balance, value.currency_symbol, value.currency
            ),
        }
    }
}

#[derive(Tabled)]
pub struct NetBalanceFormatted {
    #[tabled(rename = "Holder Name")]
    pub name: String,
    #[tabled(rename = "Balance")]
    pub balance: String,
}

impl From<NetBalanceRecord> for NetBalanceFormatted {
    fn from(value: NetBalanceRecord) -> Self {
        Self {
            name: format!("{} {}", value.first_name, value.last_name),
            balance: format!(
                "{:.2}{} {}",
                value.balance, value.currency_symbol, value.currency
            ),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ReportCommand {
    /// report your adjusted cost base for a particular year
    Acb {
        /// the year for the ACB, default to the current year
        year: Option<i32>,
    },
    /// Show your overall asset = equity - liabilities balances
    Balance,
    /// report your cashflow (revenue & expense) for a particular year
    Cashflow {
        /// show the cash flow for a specific year, default to the current year
        year: Option<i32>,
    },
    /// Show expenses in the past X days.
    Expense {
        /// how many days of history of expenses you'd like to see
        #[clap(default_value_t = 30)]
        days_prior: u64,
    },
}

impl ReportCommand {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut db = Db::new().await?;
        let mut transaction = db.begin_wrapped_transaction().await?;

        match self {
            Self::Acb { year } => {
                report_acb(&mut transaction, year.clone()).await?;
            }
            Self::Balance => {
                report_balance(&mut transaction).await?;
            }
            Self::Cashflow { year } => {
                report_cashflow(&mut transaction, year.clone()).await?;
            }
            Self::Expense { days_prior } => {
                report_expense(&mut transaction, *days_prior).await?;
            }
        };

        transaction.commit().await?;
        db.optimize().await?;

        Ok(())
    }
}
