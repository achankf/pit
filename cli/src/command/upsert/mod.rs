mod upsert_transaction;

use std::path::PathBuf;

use clap::Subcommand;
use db::{
    Account, AccountSubtype, AssetAllocation, AssetClassName, CashbackCategory,
    CashbackCategoryName, Currency, Db, Exchange, Institution, Person, PrepaidAccount, Security,
    Store, StoreCashbackMapping, TaxShelterType, TransactionForex, TransactionStore,
};

use self::upsert_transaction::upsert_transaction;

#[derive(Debug, Subcommand)]
pub enum UpsertCommand {
    All {
        /// The directory containing all the CSV files.
        csv_folder: PathBuf,
    },
    Account {
        csv_path: PathBuf,
    },
    AccountSubtype {
        csv_path: PathBuf,
    },
    AccountType {
        csv_path: PathBuf,
    },
    AssetAllocation {
        csv_path: PathBuf,
    },
    AssetClass {
        csv_path: PathBuf,
    },
    AssetClassName {
        csv_path: PathBuf,
    },
    CashbackCategory {
        csv_path: PathBuf,
    },
    CashAccountProduct {
        csv_path: PathBuf,
    },
    CashAccountHolder {
        csv_path: PathBuf,
    },
    CreditCardProduct {
        csv_path: PathBuf,
    },
    CreditCardHolder {
        csv_path: PathBuf,
    },
    Currency {
        csv_path: PathBuf,
    },
    Exchange {
        csv_path: PathBuf,
    },
    GicAccount {
        csv_path: PathBuf,
    },
    GicAccountHolder {
        csv_path: PathBuf,
    },
    IncomeAccount {
        csv_path: PathBuf,
    },
    IncomeAccountHolder {
        csv_path: PathBuf,
    },
    Institution {
        csv_path: PathBuf,
    },
    Person {
        csv_path: PathBuf,
    },
    PrepaidAccount {
        csv_path: PathBuf,
    },
    Security {
        csv_path: PathBuf,
    },
    StockAccount {
        csv_path: PathBuf,
    },
    StockAccountHolder {
        csv_path: PathBuf,
    },
    Store {
        csv_path: PathBuf,
    },
    StoreCashbackMapping {
        csv_path: PathBuf,
    },
    TaxShelterType {
        csv_path: PathBuf,
    },
    Transaction {
        csv_path: PathBuf,
    },
    TransactionForex {
        csv_path: PathBuf,
    },
    TransactionStore {
        csv_path: PathBuf,
    },
}

impl UpsertCommand {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut db = Db::new().await?;
        let mut transaction = db.begin_wrapped_transaction().await?;

        match self {
            Self::All {
                csv_folder: csv_path,
            } => {
                transaction
                    .upsert_all::<Person>(&csv_path.join("person.csv"))
                    .await?;
                transaction
                    .upsert_all::<Institution>(&csv_path.join("institution.csv"))
                    .await?;
                transaction
                    .upsert_all::<Currency>(&csv_path.join("currency.csv"))
                    .await?;
                transaction
                    .upsert_all::<Exchange>(&csv_path.join("exchange.csv"))
                    .await?;
                transaction
                    .upsert_all::<Security>(&csv_path.join("security.csv"))
                    .await?;
                transaction
                    .upsert_all::<AssetClassName>(&csv_path.join("asset_class_name.csv"))
                    .await?;
                transaction
                    .upsert_asset_class(&csv_path.join("asset_class.csv"))
                    .await?;
                transaction
                    .upsert_all::<AssetAllocation>(&csv_path.join("asset_allocation.csv"))
                    .await?;
                transaction
                    .upsert_all::<TaxShelterType>(&csv_path.join("tax_shelter_type.csv"))
                    .await?;
                transaction
                    .upsert_all::<AccountSubtype>(&csv_path.join("account_subtype.csv"))
                    .await?;
                transaction
                    .upsert_account_type(&csv_path.join("account_type.csv"))
                    .await?;
                transaction
                    .upsert_all::<Account>(&csv_path.join("account.csv"))
                    .await?;
                transaction
                    .upsert_cash_account(&csv_path.join("cash_account.csv"))
                    .await?;
                transaction
                    .upsert_cash_account_holder(&csv_path.join("cash_account_holder.csv"))
                    .await?;
                transaction
                    .upsert_all::<CashbackCategoryName>(
                        &csv_path.join("cashback_category_name.csv"),
                    )
                    .await?;
                transaction
                    .upsert_credit_card(&csv_path.join("credit_card_account.csv"))
                    .await?;
                transaction
                    .upsert_credit_card_holder(&csv_path.join("credit_card_account_holder.csv"))
                    .await?;
                transaction
                    .upsert_all::<CashbackCategory>(&csv_path.join("cashback_category.csv"))
                    .await?;
                transaction
                    .upsert_all::<Store>(&csv_path.join("store.csv"))
                    .await?;
                transaction
                    .upsert_all::<StoreCashbackMapping>(
                        &csv_path.join("store_cashback_mapping.csv"),
                    )
                    .await?;
                transaction
                    .upsert_stock_account(&csv_path.join("stock_account.csv"))
                    .await?;
                transaction
                    .upsert_stock_account_holder(&csv_path.join("stock_account_holder.csv"))
                    .await?;
                transaction
                    .upsert_gic_account(&csv_path.join("gic_account.csv"))
                    .await?;
                transaction
                    .upsert_gic_account_holder(&csv_path.join("gic_account_holder.csv"))
                    .await?;
                transaction
                    .upsert_income_account(&csv_path.join("income_account.csv"))
                    .await?;
                transaction
                    .upsert_income_account_holder(&csv_path.join("income_account_holder.csv"))
                    .await?;
                transaction
                    .upsert_all::<PrepaidAccount>(&csv_path.join("prepaid_account.csv"))
                    .await?;
                transaction
                    .upsert_all::<TransactionStore>(&csv_path.join("transaction_store.csv"))
                    .await?;
                transaction
                    .upsert_all::<TransactionForex>(&csv_path.join("transaction_forex.csv"))
                    .await?;

                upsert_transaction(&mut transaction, &csv_path.join("transaction.csv")).await?;
            }
            Self::GicAccount { csv_path } => {
                transaction.upsert_gic_account(&csv_path).await?;
            }
            Self::GicAccountHolder { csv_path } => {
                transaction.upsert_gic_account_holder(&csv_path).await?;
            }
            Self::Exchange { csv_path } => {
                transaction.upsert_all::<Exchange>(&csv_path).await?;
            }
            Self::Currency { csv_path } => {
                transaction.upsert_all::<Currency>(&csv_path).await?;
            }
            Self::Security { csv_path } => {
                transaction.upsert_all::<Security>(&csv_path).await?;
            }
            Self::AssetClass { csv_path } => {
                transaction.upsert_asset_class(&csv_path).await?;
            }
            Self::AssetAllocation { csv_path } => {
                transaction.upsert_all::<AssetAllocation>(&csv_path).await?;
            }
            Self::Institution { csv_path } => {
                transaction.upsert_all::<Institution>(&csv_path).await?;
            }
            Self::Person { csv_path } => {
                transaction.upsert_all::<Person>(&csv_path).await?;
            }
            Self::AccountSubtype { csv_path } => {
                transaction.upsert_all::<AccountSubtype>(&csv_path).await?;
            }
            Self::AccountType { csv_path } => {
                transaction.upsert_account_type(&csv_path).await?;
            }
            Self::Account { csv_path } => {
                transaction.upsert_all::<Account>(&csv_path).await?;
            }
            Self::Transaction { csv_path } => {
                upsert_transaction(&mut transaction, csv_path).await?;
            }
            Self::CashAccountProduct { csv_path } => {
                transaction.upsert_cash_account(&csv_path).await?;
            }
            Self::TaxShelterType { csv_path } => {
                transaction.upsert_all::<TaxShelterType>(&csv_path).await?;
            }
            Self::CashAccountHolder { csv_path } => {
                transaction.upsert_cash_account_holder(csv_path).await?;
            }
            Self::Store { csv_path } => {
                transaction.upsert_all::<Store>(&csv_path).await?;
            }
            Self::TransactionStore { csv_path } => {
                transaction
                    .upsert_all::<TransactionStore>(&csv_path)
                    .await?;
            }
            Self::TransactionForex { csv_path } => {
                transaction
                    .upsert_all::<TransactionForex>(&csv_path)
                    .await?;
            }
            Self::StoreCashbackMapping { csv_path } => {
                transaction
                    .upsert_all::<StoreCashbackMapping>(&csv_path)
                    .await?;
            }
            Self::CreditCardProduct { csv_path } => {
                transaction.upsert_credit_card(&csv_path).await?;
            }
            Self::CreditCardHolder { csv_path } => {
                transaction.upsert_credit_card_holder(&csv_path).await?;
            }
            Self::StockAccount { csv_path } => {
                transaction.upsert_stock_account(&csv_path).await?;
            }
            Self::StockAccountHolder { csv_path } => {
                transaction.upsert_stock_account_holder(&csv_path).await?;
            }
            Self::IncomeAccount { csv_path } => {
                transaction.upsert_income_account(&csv_path).await?;
            }
            Self::IncomeAccountHolder { csv_path } => {
                transaction.upsert_income_account_holder(&csv_path).await?;
            }
            Self::CashbackCategory { csv_path } => {
                transaction
                    .upsert_all::<CashbackCategory>(&csv_path)
                    .await?;
            }
            Self::AssetClassName { csv_path } => {
                transaction.upsert_all::<AssetClassName>(&csv_path).await?;
            }
            Self::PrepaidAccount { csv_path } => {
                transaction.upsert_all::<PrepaidAccount>(&csv_path).await?;
            }
        };

        transaction.commit().await?;
        db.optimize().await?;

        Ok(())
    }
}
