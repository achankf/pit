use db::Transaction;
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table};

use crate::command::report::{BalanceRecordFormatted, NetBalanceFormatted};

pub async fn report_balance(
    transaction: &mut Transaction<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let records = transaction.get_net_asset_balance().await?;

    let formatted = records.into_iter().map(NetBalanceFormatted::from);

    println!("{}", "Net Asset".bold());
    println!(
        "{}",
        Table::new(formatted)
            .with(Style::rounded())
            .with(Columns::new(1..).modify().with(Alignment::right()))
            .to_string()
    );
    println!();

    let balance_sheet_by_kind = vec![
        ("Asset", transaction.get_asset_balance(None).await?),
        ("Equity", transaction.get_equity_balance(None).await?),
        (
            "Liabilities",
            transaction.get_liabilities_balance(None).await?,
        ),
    ];

    for (kind, sheet) in balance_sheet_by_kind {
        let formatted = sheet.into_iter().map(BalanceRecordFormatted::from);

        println!("{}", kind.bold());
        println!(
            "{}",
            Table::new(formatted)
                .with(Style::rounded())
                .with(Columns::new(3..).modify().with(Alignment::right()))
                .to_string()
        );
        println!();
    }

    Ok(())
}
