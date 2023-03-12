use chrono::Local;
use db::{AssetRebalance, Db, EmergencyRebalance, SqlResult, Transaction};
use owo_colors::OwoColorize;
use tabled::{object::Columns, Alignment, ModifyObject, Style, Table, Tabled};

async fn print_emergency_injecion(transaction: &mut Transaction<'_>) -> SqlResult<()> {
    #[derive(Tabled)]
    pub struct EmergencyRebalanceFormatted {
        #[tabled(rename = "Name")]
        pub name: String,
        #[tabled(rename = "Account Name")]
        pub account_name: String,
        #[tabled(rename = "Unallocated Fund")]
        pub unallocated_fund: String,
        #[tabled(rename = "Injection Needed")]
        pub injection_needed: String,
    }

    impl From<EmergencyRebalance> for EmergencyRebalanceFormatted {
        fn from(value: EmergencyRebalance) -> Self {
            Self {
                name: format!("{} {}", value.first_name, value.last_name),
                account_name: value.account_name,
                unallocated_fund: format!(
                    "{:.2}{} {}",
                    value.unallocated_fund, value.currency_symbol, value.currency
                ),
                injection_needed: format!(
                    "{:.2}{} {}",
                    value.injection_needed, value.currency_symbol, value.currency
                ),
            }
        }
    }

    let values = transaction.get_emergency_rebalance().await?;

    if !values.is_empty() {
        let results = values.into_iter().map(EmergencyRebalanceFormatted::from);

        println!("• Fill up emergency funds:");
        println!(
            "{}",
            Table::new(results)
                .with(Style::rounded())
                .with(Columns::new(2..).modify().with(Alignment::right()))
                .to_string()
        );
        println!();
    }

    Ok(())
}

pub async fn print_current_credit(transaction: &mut Transaction<'_>) -> SqlResult<()> {
    let current_credit = transaction.get_current_credit_card_balance().await?;

    if !current_credit.is_empty() {
        println!(
            "{}",
            "(╬ಠ益ಠ) You have unpaid credits, repay them ASAP!"
                .red()
                .bold(),
        );
        println!();

        for record in current_credit {
            let last_payment_date = if let Some(last_payment_date) = record.last_payment_date {
                let duration = (Local::now() - last_payment_date).num_days();
                format!(
                    "(last payment: {}, {} days ago)",
                    last_payment_date.format("%d/%m/%Y").to_string(),
                    if duration > 20 {
                        duration.to_string().yellow().bold().to_string()
                    } else {
                        duration.to_string().bold().to_string()
                    }
                )
            } else {
                "".into()
            };

            println!(
                "\t• {}: {} {}",
                record.account_name,
                format!(
                    "{:.2}{} {}",
                    record.balance, record.currency_symbol, record.currency
                )
                .red(),
                last_payment_date
            );
        }
        println!();
    }

    Ok(())
}

async fn print_rebalance(transaction: &mut Transaction<'_>) -> SqlResult<()> {
    #[derive(Tabled)]
    pub struct RebalanceFormatted {
        #[tabled(rename = "Name")]
        pub name: String,
        #[tabled(rename = "Asset Class")]
        pub class: String,
        #[tabled(rename = "Full Rebalance")]
        pub current_rebalance_amount: String,
        #[tabled(rename = "Potential Rebalance")]
        pub potential_rebalance_amount: String,
    }

    impl From<AssetRebalance> for RebalanceFormatted {
        fn from(value: AssetRebalance) -> Self {
            Self {
                name: format!("{} {}", value.first_name, value.last_name),
                class: value.class,
                current_rebalance_amount: format!("{:.2}$ CAD", value.current_rebalance_amount),
                potential_rebalance_amount: format!("{:.2}$ CAD", value.potential_rebalance_amount),
            }
        }
    }

    let results = transaction.get_asset_rebalance().await?;
    let results = results.iter().cloned().map(RebalanceFormatted::from);

    println!("• Full rebalance");
    println!(
        "{}",
        Table::new(results)
            .with(Style::rounded())
            .with(Columns::new(2..).modify().with(Alignment::right()))
            .to_string()
    );
    println!();

    Ok(())
}

pub async fn rebalance() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Db::new().await?;
    let mut transaction = db.begin_wrapped_transaction().await?;

    let last_update = transaction.get_last_market_price_update().await?;
    println!("{last_update}");

    if (Local::now() - last_update).num_hours() > 4 {
        // automatically update after at least 4 hours since last update
        transaction.refresh_market_price().await?;
    }

    print_current_credit(&mut transaction).await?;

    print_emergency_injecion(&mut transaction).await?;

    print_rebalance(&mut transaction).await?;

    transaction.commit().await?;
    db.optimize().await?;

    Ok(())
}
