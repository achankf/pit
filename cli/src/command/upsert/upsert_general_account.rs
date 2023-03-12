use std::path::PathBuf;

use db::{GeneralAccount, Transaction};

pub async fn upsert_general_account(
    transaction: &mut Transaction<'_>,
    csv_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let accounts = transaction.upsert_all::<GeneralAccount>(&csv_path).await?;

    for (_, record) in accounts {
        if let Some(ticker) = record.stock_ticker {
            match record.general_account_type.as_ref() {
                "COMMISSION" | "DIVIDEND" | "STOCK" | "ETF" => {
                    transaction
                        .upsert_stock_account(record.general_account_key, ticker)
                        .await?;
                }
                _ => {
                    continue; // continue is not needed, just to it clear
                }
            };
        }
    }

    Ok(())
}
