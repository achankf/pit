use db::Transaction;
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table, Tabled};

pub async fn print_justify_amex(
    transaction: &mut Transaction<'_>,
    num_days: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let records = transaction.justify_amex(num_days).await?;

    #[derive(Tabled)]
    struct Justification {
        #[tabled(rename = "Year")]
        year: i64,
        #[tabled(rename = "Month")]
        month: i64,
        #[tabled(rename = "Spending Balance")]
        balance: String,
        #[tabled(rename = "Cashback ex. Amex")]
        without_amex_cashback: String,
        #[tabled(rename = "Cashback w/ Amex")]
        with_amex_cashback: String,
        #[tabled(rename = "Diff")]
        extra_cashback: String,
        #[tabled(rename = "Diff Rate")]
        extra_cashback_rate: String,
        #[tabled(rename = "Diff, net of fees")]
        extra_cashback_after_fee: String,
        #[tabled(rename = "Missed Opportunities")]
        missed_opportunities: String,
    }

    fn colourize(value: f64, formatted: String) -> String {
        if value >= 0.0 {
            formatted.green().to_string()
        } else {
            formatted.red().to_string()
        }
    }

    fn format_balance(value: f64) -> String {
        if value >= 0.0 {
            format!("${:.2}", value)
        } else {
            format!("(${:.2})", value.abs())
        }
    }

    fn format_coloured_balance(value: f64) -> String {
        colourize(value, format_balance(value))
    }

    let records = records.into_iter().map(|row| {
        let extra_cashback_rate = row.extra_cashback_rate * 100.0;

        Justification {
            year: row.year,
            month: row.month,
            balance: format_balance(row.balance),
            with_amex_cashback: format_coloured_balance(row.with_amex_cashback),
            without_amex_cashback: format_coloured_balance(row.without_amex_cashback),
            extra_cashback: format_coloured_balance(row.extra_cashback),
            extra_cashback_rate: colourize(
                extra_cashback_rate,
                format!("{:.2}%", extra_cashback_rate),
            ),
            extra_cashback_after_fee: format_coloured_balance(row.extra_cashback_after_fee),
            missed_opportunities: format_coloured_balance(row.missed_opportunities),
        }
    });

    println!();
    println!(
        "{}",
        Table::new(records)
            .with(Style::rounded())
            .with(Columns::new(1..).modify().with(Alignment::right()))
            .to_string()
    );

    Ok(())
}
