use chrono::Local;
use db::{BalanceRecord, Transaction};
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table};

use crate::command::report::{
    stock_unit::get_stock_unit_str, BalanceRecordWithOwnerFormatted, NetBalanceFormatted,
};

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
            .with(Columns::single(1).modify().with(Alignment::right()))
            .to_string()
    );
    println!();

    fn make_table_str(vec: Vec<BalanceRecord>) -> String {
        Table::new(vec.into_iter().map(BalanceRecordWithOwnerFormatted::from))
            .with(Style::rounded())
            .with(Columns::single(2).modify().with(Alignment::right()))
            .to_string()
    }

    let balance_sheet_by_kind = vec![
        (
            "Cash and Cash Equivalent (Asset)",
            make_table_str(transaction.get_cash_balance(None).await?),
        ),
        (
            "Stock (Asset)",
            get_stock_unit_str(transaction.get_stock_unit(&Local::now()).await?),
        ),
        (
            "Equity",
            make_table_str(transaction.get_equity_balance(None).await?),
        ),
        (
            "Liabilities",
            make_table_str(transaction.get_liabilities_balance(None).await?),
        ),
    ];

    for (kind, table) in balance_sheet_by_kind {
        println!("{}", kind.bold());
        println!("{}", table);
        println!();
    }

    Ok(())
}
