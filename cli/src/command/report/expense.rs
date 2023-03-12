use common::days_prior_until_end_of_today;
use db::{SqlResult, Transaction};
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table};

use crate::command::report::BalanceRecordFormatted;

pub async fn report_expense(transaction: &mut Transaction<'_>, days_prior: u64) -> SqlResult<()> {
    let spendings = transaction
        .get_expense_balance(Some(days_prior_until_end_of_today(days_prior)))
        .await?;

    let formatted = spendings.into_iter().map(BalanceRecordFormatted::from);

    println!();
    println!("Amount of money spend since {days_prior} days prior");
    println!(
        "{}",
        Table::new(formatted)
            .with(Style::rounded())
            .with(Columns::new(3..).modify().with(Alignment::right()))
            .to_string()
    );

    Ok(())
}
