use serde::{Deserialize, Serialize};

use crate::create_fetch_client::create_fetch_client;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub quote_type: String,
    pub long_name: String,
    pub symbol: String,
    pub regular_market_price: f64,
}

#[derive(Deserialize, Debug)]
struct QuoteResponse {
    result: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Response {
    quote_response: QuoteResponse,
}

pub async fn fetch_symbol_data(
    yahoo_tickers: &[String],
) -> Result<Vec<Quote>, Box<dyn std::error::Error>> {
    if yahoo_tickers.is_empty() {
        return Err("no symbol".into());
    }

    let symbols = yahoo_tickers.clone();
    let url = "https://query1.finance.yahoo.com/v7/finance/quote";
    let symbols_str = symbols.join(",");
    let params = [
        ("symbols", symbols_str.as_ref()),
        ("fields", "symbol,currency,longName,regularMarketPrice"),
        ("lang", "en-CA"),
        ("region", "CA"),
    ];

    let fetch = create_fetch_client();
    let request = fetch.get(url).query(&params).build()?;

    println!("getting quote information from {}", request.url());

    let response: Response = fetch.execute(request).await?.json().await?;

    Ok(response
        .quote_response
        .result
        .into_iter()
        .map(Quote::deserialize)
        .map(|quote| quote.expect("cannot parse Yahoo response for securities pricing data"))
        .collect())
}
