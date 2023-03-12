use std::collections::HashMap;

use common::fetch_symbol_data;

use crate::{yahoo_symbol::TickerId, Transaction};

impl<'c> Transaction<'c> {
    pub async fn refresh_market_price(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let yahoo_symbols: HashMap<_, _> = self.get_yahoo_symbols().await?;

        let yahoo_tickers: Vec<_> = yahoo_symbols.keys().cloned().collect();

        println!("All tickers in database: {yahoo_tickers:?}");

        let quotes = fetch_symbol_data(yahoo_tickers.as_slice()).await?;

        for (quote, (expect, id)) in quotes.iter().zip(yahoo_symbols.iter()) {
            assert_eq!(expect.to_string(), quote.symbol, "sanity check");

            match id {
                TickerId::Currency(currency_id) => {
                    self.update_currency_exchange_rate(*currency_id, quote.regular_market_price)
                        .await?;
                }
                TickerId::Security(security_id) => {
                    self.update_security_price(*security_id, quote.regular_market_price)
                        .await?;
                }
            }
        }

        self.update_last_market_price_update().await?;

        Ok(())
    }
}
