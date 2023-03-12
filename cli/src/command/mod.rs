mod fetch;
mod rebalance;
mod report;
mod upsert;

use clap::Subcommand;
use rebalance::rebalance;

use self::{fetch::FetchCommand, report::ReportCommand, upsert::UpsertCommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Fetch and update database with 3rd-party sources
    Fetch {
        #[command(subcommand)]
        command: FetchCommand,
    },
    /// Show allocation to rebalance your asset.
    Rebalance,
    /// Report key data from the database
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Synchronize database with given CSV file. Basic a drop table and insert.
    Upsert {
        #[command(subcommand)]
        command: UpsertCommand,
    },
}

impl Command {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Fetch { command } => command.run().await,
            Self::Upsert { command } => command.run().await,
            Self::Rebalance => rebalance().await,
            Self::Report { command } => command.run().await,
        }
    }
}
