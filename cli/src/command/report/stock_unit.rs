use chrono::{DateTime, Local};
use db::{SqlResult, StockUnit, Transaction};
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table, Tabled};

pub fn get_stock_unit_str(records: Vec<StockUnit>) -> String {
    #[derive(Tabled)]
    struct StockUnitFormatted {
        #[tabled(rename = "Holder")]
        pub name: String,
        #[tabled(rename = "Account Name")]
        pub account_name: String,
        #[tabled(rename = "Ticker")]
        pub ticker: String,
        #[tabled(rename = "Unit")]
        pub unit: String,
        #[tabled(rename = "Market Value")]
        pub market_value: String,
    }

    impl From<StockUnit> for StockUnitFormatted {
        fn from(value: StockUnit) -> Self {
            Self {
                name: value.name,
                account_name: value.account_name,
                ticker: value.ticker,
                unit: format!("{:.4}", value.total_unit),
                market_value: format!("{:.2}", value.market_value),
            }
        }
    }

    let formatted = records.into_iter().map(StockUnitFormatted::from);

    Table::new(formatted)
        .with(Style::rounded())
        .with(Columns::single(2).modify().with(Alignment::center()))
        .with(Columns::single(3).modify().with(Alignment::right()))
        .with(Columns::single(4).modify().with(Alignment::right()))
        .to_string()
}

pub async fn report_stock_unit(
    transaction: &mut Transaction<'_>,
    datetime: DateTime<Local>,
) -> SqlResult<()> {
    let records = transaction.get_stock_unit(&datetime).await?;

    println!();
    println!(
        r#"Stock ownership by {} (i.e. strictly "less than")"#,
        datetime.date_naive()
    );
    println!("{}", get_stock_unit_str(records));

    Ok(())
}
