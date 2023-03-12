use std::path::PathBuf;

use db::{AssertTransactionBalance, GeneralTransaction, Transaction};
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table, Tabled};

#[derive(Tabled)]
struct RecordFormatted {
    transaction_id: i64,
    debit: String,
    credit: String,
    balance: String,
}

impl From<AssertTransactionBalance> for RecordFormatted {
    fn from(value: AssertTransactionBalance) -> Self {
        Self {
            transaction_id: value.transaction_id,
            debit: format!("${:.2}", value.debit),
            credit: format!("${:.2}", value.credit),
            balance: format!("${:.2}", value.balance),
        }
    }
}

pub async fn upsert_transaction(
    transaction: &mut Transaction<'_>,
    csv_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    transaction
        .upsert_all::<GeneralTransaction>(csv_path)
        .await?;

    let results = transaction.assert_transaction_balance().await?;

    if !results.is_empty() {
        let formatted = results.into_iter().map(RecordFormatted::from);

        println!(
            "{}",
            format!("Transaction credit & debit have non-zero balance")
                .to_string()
                .red()
                .bold()
        );
        println!(
            "{}",
            Table::new(formatted)
                .with(Style::rounded())
                .with(Columns::new(2..).modify().with(Alignment::right()))
                .to_string()
        );

        return Err("Transaction credit & debit have non-zero balance".into());
    }

    transaction.assert_accounting_indentity().await?;

    Ok(())
}
