use db::{AssertTransactionBalance, CheckTransactionStore, SqlResult, Transaction};
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table, Tabled};

pub async fn print_transaction_check(transaction: &mut Transaction<'_>) -> SqlResult<()> {
    {
        #[derive(Tabled)]
        struct RecordFormatted {
            transaction_id: i64,
            description: String,
        }

        impl From<CheckTransactionStore> for RecordFormatted {
            fn from(value: CheckTransactionStore) -> Self {
                Self {
                    transaction_id: value.transaction_id,
                    description: value.description,
                }
            }
        }

        let transaction_store = transaction.check_transaction_store().await?;

        if !transaction_store.is_empty() {
            let formatted = transaction_store.into_iter().map(RecordFormatted::from);

            println!("The following purchases have incomplete merchant information");
            println!(
                "{}",
                Table::new(formatted).with(Style::rounded()).to_string()
            );
            println!();
        }
    }

    let results = transaction.check_transaction_balance().await?;

    if !results.is_empty() {
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

        let formatted = results.into_iter().map(RecordFormatted::from);

        println!(
            "{}",
            format!("Credit & debit aren't balanced!")
                .to_string()
                .yellow()
                .bold()
        );
        println!(
            "{}",
            Table::new(formatted)
                .with(Style::rounded())
                .with(Columns::new(2..).modify().with(Alignment::right()))
                .to_string()
        );
    } else {
        let result = transaction.check_accounting_indentity().await?;

        if !result.is_balance {
            eprintln!(
                "{}",
                format!(
                    "Asset balance doesn't match its double-entry counterpart, {}|{}",
                    result.asset_balance, result.equity_liabilities_balance
                )
                .bold()
            );
        }
    }

    Ok(())
}
