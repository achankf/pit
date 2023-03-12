mod print_justify_amex;
mod print_transaction_check;
mod rebalance;
mod report;
mod upsert;

use clap::Subcommand;
use db::Db;
use owo_colors::OwoColorize;
use rebalance::rebalance;

use self::{
    print_justify_amex::print_justify_amex, print_transaction_check::print_transaction_check,
    report::ReportCommand, upsert::UpsertCommand,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Verify the coherence of the database.
    Check,
    /// Calculates and evaluates whether using the Amex SimplyCash Preferred card is justified, based on the specified number of days.
    JustifyAmex {
        /// Specifies the number of days prior to the current date to be used for backtesting the Amex SimplyCash Preferred card justification.
        #[clap(default_value_t = 30)]
        num_days: u32,
    },
    /// Retrieves the next available transaction ID.
    Next,
    /// Displays the allocation required to rebalance the asset.
    Rebalance,
    /// Provides a report of important data from the database.
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Synchronizes the database with the provided CSV file.
    Upsert {
        #[command(subcommand)]
        command: UpsertCommand,
    },
}

impl Command {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::JustifyAmex { num_days } => {
                let mut db = Db::new().await?;
                let mut transaction = db.begin_wrapped_transaction().await?;

                print_justify_amex(&mut transaction, *num_days).await?;

                transaction.commit().await?;
                db.optimize().await?;

                Ok(())
            }
            Self::Check => {
                let mut db = Db::new().await?;
                let mut transaction = db.begin_wrapped_transaction().await?;

                print_transaction_check(&mut transaction).await?;

                transaction.commit().await?;
                db.optimize().await?;

                Ok(())
            }
            Self::Next => {
                let mut db = Db::new().await?;
                let mut transaction = db.begin_wrapped_transaction().await?;

                print_transaction_check(&mut transaction).await?;

                let next_transaction_id = transaction.get_next_transaction_id().await?;

                println!(
                    "The succeeding transaction ID is: {}",
                    next_transaction_id.to_string().bold().yellow()
                );

                transaction.commit().await?;
                db.optimize().await?;

                Ok(())
            }
            Self::Upsert { command } => command.run().await,
            Self::Rebalance => {
                rebalance().await?;
                Ok(())
            }
            Self::Report { command } => command.run().await,
        }
    }
}
