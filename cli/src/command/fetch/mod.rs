mod refresh_market_price;

use clap::Subcommand;
use refresh_market_price::refresh_market_price;

#[derive(Clone, Debug, Subcommand)]
pub enum FetchCommand {
    /// refresh market price from Yahoo Finance
    MarketPrice,
}

impl FetchCommand {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::MarketPrice => refresh_market_price().await,
        }
    }
}
