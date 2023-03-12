use std::collections::BTreeMap;

use chrono::{Datelike, Local};
use common::all_time_in_year;
use db::{Acb, Transaction};
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table, Tabled};

pub async fn report_acb(
    transaction: &mut Transaction<'_>,
    year: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Tabled)]
    struct AcbFormatted {
        #[tabled(rename = "Holder Name")]
        pub name: String,
        #[tabled(rename = "Ticker")]
        pub ticker: String,
        #[tabled(rename = "Units")]
        pub units: String,
        #[tabled(rename = "Distribution")]
        pub total_distribution: String,
        #[tabled(rename = "Gain/Loss")]
        pub total_capital_gl: String,
        #[tabled(rename = "ACB")]
        pub acb: String,
    }

    impl From<Acb> for AcbFormatted {
        fn from(value: Acb) -> Self {
            let format_colored_cad = |v: f64| {
                if v == 0.0 {
                    "".into()
                } else {
                    let ret = format!("${:.2}", v);
                    if v > 0.0 {
                        ret.green().to_string()
                    } else {
                        ret.red().to_string()
                    }
                }
            };

            let format_cad = |v: f64| {
                if v == 0.0 {
                    "".into()
                } else {
                    format!("${:.2}", v)
                }
            };

            Self {
                name: format!("{} {}", value.first_name, value.last_name),
                ticker: value.ticker,
                units: format!("{:.4}", value.acc_units),
                acb: format_cad(value.acb),
                total_distribution: format_colored_cad(value.total_distribution),
                total_capital_gl: format_colored_cad(value.total_capital_gl),
            }
        }
    }

    let this_year = Local::now().year();

    let year = year.unwrap_or(this_year);
    if year > this_year {
        return Err("No crytstal ball error: unable to show acb for the future".into());
    }

    let range = all_time_in_year(year);

    let records = transaction.get_acb(range.clone()).await?;

    if records.is_empty() {
        return Err(format!("No record for {year}").into());
    }

    let everyone_capital_gl = records.iter().fold(
        BTreeMap::<i64, (String, f64, f64)>::new(),
        |mut acc, record| {
            let entry = acc.entry(record.person_id);
            let value = entry.or_default();

            if value.0.is_empty() {
                value.0 = format!("{} {}", record.first_name, record.last_name);
            }
            value.1 += record.total_distribution;
            value.2 += record.total_capital_gl;

            acc
        },
    );

    println!(
        "{}",
        format!("Adjusted Cost Base (ACB) with Capital Gain/Loss for {year}").bold()
    );

    for (_, (name, total_distribution, total_capital_gl)) in everyone_capital_gl {
        fn format_money(val: f64) -> String {
            if val > 0.0 {
                format!("${:.2}", val).green().to_string()
            } else if val < 0.0 {
                format!("${:.2}", val).red().to_string()
            } else {
                "$0.00".yellow().to_string()
            }
        }

        println!(
            "{}: distribution={}, GL={}",
            name,
            format_money(total_distribution),
            format_money(total_capital_gl)
        );
    }
    println!();

    let formatted = records.iter().cloned().map(AcbFormatted::from);

    println!("{}", format!("GL Breakdown By Security").bold());
    println!(
        "{}",
        Table::new(formatted)
            .with(Style::rounded())
            .with(Columns::single(2).modify().with(Alignment::center()))
            .with(Columns::single(5).modify().with(Alignment::center()))
            .with(Columns::new(2..).modify().with(Alignment::right()))
            .with(Columns::single(6).modify().with(Alignment::right()))
            .to_string()
    );
    println!();

    Ok(())
}
