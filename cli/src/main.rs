mod command;

use clap::Parser;
use command::Command;

/// Add banking transaction records to the database.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Args::parse().command.run().await?;

    Ok(())
}
