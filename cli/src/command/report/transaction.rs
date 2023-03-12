use chrono::{DateTime, Local};
use common::days_prior_until_end_of_today;
use db::{SqlResult, Transaction, TransactionByAccountKey};
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table, Tabled};

pub async fn report_transaction(
    transaction: &mut Transaction<'_>,
    account_key: &str,
    days_prior: u64,
) -> SqlResult<()> {
    #[derive(Tabled)]
    struct TransactionByAccountKeyFormatted {
        #[tabled(rename = "Transaction ID")]
        pub transaction_id: i64,
        #[tabled(rename = "Item ID")]
        pub item_id: i64,
        #[tabled(rename = "Transaction Date")]
        pub date: DateTime<Local>,
        #[tabled(rename = "Unit")]
        pub unit: String,
        #[tabled(rename = "Debit")]
        pub debit: String,
        #[tabled(rename = "Credit")]
        pub credit: String,
        #[tabled(rename = "Forex")]
        pub exchange_rate: String,
        #[tabled(rename = "Total $")]
        pub total_amount: String,
        #[tabled(rename = "Description")]
        pub description: String,
    }

    impl From<TransactionByAccountKey> for TransactionByAccountKeyFormatted {
        fn from(value: TransactionByAccountKey) -> Self {
            Self {
                transaction_id: value.transaction_id,
                item_id: value.item_id,
                date: value.date,
                unit: format!("{:.4}", value.unit),
                debit: if let Some(debit) = value.debit {
                    format!("${:.2}", debit)
                } else {
                    "".into()
                },
                credit: if let Some(credit) = value.credit {
                    format!("${:.2}", credit)
                } else {
                    "".into()
                },
                exchange_rate: if let Some(exchange_rate) = value.exchange_rate {
                    format!("{:.4}", exchange_rate)
                } else {
                    "N/A".into()
                },
                total_amount: format!("${:.2}", value.total_amount),
                description: value.description,
            }
        }
    }

    let records = transaction
        .get_transaction_by_account_key(account_key, days_prior_until_end_of_today(days_prior))
        .await?;

    let (total_debit, total_credit, total) =
        records
            .iter()
            .fold((0.0, 0.0, 0.0), |(debit, credit, _), record| {
                let unit = record.unit;
                let debit = debit + unit * record.debit.unwrap_or_default();
                let credit = credit + unit * record.credit.unwrap_or_default();
                let total = debit - credit;
                (debit, credit, total)
            });

    let formatted = records
        .into_iter()
        .map(TransactionByAccountKeyFormatted::from);

    println!();
    println!("Transaction for {account_key} since {days_prior} days prior; total debit=${total_debit:.2}, credit=${total_credit:.2}, balance=${total:.2}");
    println!(
        "{}",
        Table::new(formatted)
            .with(Style::rounded())
            .with(Columns::new(0..=1).modify().with(Alignment::right()))
            .with(Columns::new(3..=7).modify().with(Alignment::right()))
            .to_string()
    );

    Ok(())
}
