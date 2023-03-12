use chrono::{Datelike, Local};
use common::all_time_in_year;
use db::Transaction;
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table};

use crate::command::report::{BalanceRecordFormatted, NetBalanceFormatted};

pub async fn report_cashflow(
    transaction: &mut Transaction<'_>,
    year: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let this_year = Local::now().year();

    let year = year.unwrap_or(this_year);
    if year > this_year {
        return Err("No crytstal ball error: unable to show cashflow for the future".into());
    }

    let range = all_time_in_year(year);

    let records = transaction.get_net_revenue_balance(range.clone()).await?;

    if records.is_empty() {
        return Err(format!("No record for {year}").into());
    }

    let formatted = records.into_iter().map(NetBalanceFormatted::from);

    println!("{}", format!("Net Revenue of {year}").bold());
    println!(
        "{}",
        Table::new(formatted)
            .with(Style::rounded())
            .with(Columns::single(1).modify().with(Alignment::right()))
            .to_string()
    );
    println!();

    let balance_sheet_by_kind = vec![
        (
            "Revenue",
            transaction.get_revenue_balance(Some(range.clone())).await?,
        ),
        (
            "Expense",
            transaction.get_expense_balance(Some(range)).await?,
        ),
    ];

    for (kind, sheet) in balance_sheet_by_kind {
        let formatted = sheet.into_iter().map(BalanceRecordFormatted::from);

        println!("{}", kind.bold());
        println!(
            "{}",
            Table::new(formatted)
                .with(Style::rounded())
                .with(Columns::single(2).modify().with(Alignment::right()))
                .to_string()
        );
        println!();
    }

    Ok(())
}
