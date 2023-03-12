use std::path::PathBuf;

use db::{FinancialEntry, Transaction};

use crate::command::print_transaction_check::print_transaction_check;

pub async fn upsert_transaction(
    transaction: &mut Transaction<'_>,
    csv_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    transaction.upsert_all::<FinancialEntry>(csv_path).await?;

    print_transaction_check(transaction).await?;

    Ok(())
}
