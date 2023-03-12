mod upsert_general_account;
mod upsert_transaction;

use std::path::PathBuf;

use clap::Subcommand;
use db::{
    AssetAllocation, AssetClass, CashAccount, CashEmergencyTarget, Currency, Db, Exchange,
    GeneralAccountName, GeneralAccountType, Institution, Person, Security, TaxShelterType,
};

use self::upsert_general_account::upsert_general_account;
use self::upsert_transaction::upsert_transaction;

#[derive(Debug, Subcommand)]
pub enum UpsertCommand {
    All {
        /// the folder where all csv files are located
        csv_folder: PathBuf,
    },
    AssetAllocation {
        csv_path: PathBuf,
    },
    AssetClass {
        csv_path: PathBuf,
    },
    CashAccount {
        csv_path: PathBuf,
    },
    CashEmergencyTarget {
        csv_path: PathBuf,
    },
    Currency {
        csv_path: PathBuf,
    },
    Exchange {
        csv_path: PathBuf,
    },
    GeneralAccount {
        csv_path: PathBuf,
    },
    GeneralAccountName {
        csv_path: PathBuf,
    },
    GeneralAccountType {
        csv_path: PathBuf,
    },
    GeneralTransaction {
        csv_path: PathBuf,
    },
    Institution {
        csv_path: PathBuf,
    },
    Person {
        csv_path: PathBuf,
    },
    Security {
        csv_path: PathBuf,
    },
    TaxShelterType {
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
                    .upsert_all_in_order::<AssetClass>(&csv_path.join("asset_class.csv"))
                    .await?;
                transaction
                    .upsert_all::<AssetAllocation>(&csv_path.join("asset_allocation.csv"))
                    .await?;
                transaction
                    .upsert_all::<TaxShelterType>(&csv_path.join("tax_shelter_type.csv"))
                    .await?;
                transaction
                    .upsert_all::<GeneralAccountName>(&csv_path.join("general_account_name.csv"))
                    .await?;
                transaction
                    .upsert_all::<GeneralAccountType>(&csv_path.join("general_account_type.csv"))
                    .await?;
                upsert_general_account(&mut transaction, &csv_path.join("general_account.csv"))
                    .await?;
                transaction
                    .upsert_all::<CashAccount>(&csv_path.join("cash_account.csv"))
                    .await?;
                transaction
                    .upsert_all::<CashEmergencyTarget>(&csv_path.join("cash_emergency_target.csv"))
                    .await?;
                upsert_transaction(&mut transaction, &csv_path.join("general_transaction.csv"))
                    .await?;
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
                transaction
                    .upsert_all_in_order::<AssetClass>(&csv_path)
                    .await?;
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

            Self::GeneralAccountName { csv_path } => {
                transaction
                    .upsert_all::<GeneralAccountName>(&csv_path)
                    .await?;
            }
            Self::GeneralAccountType { csv_path } => {
                transaction
                    .upsert_all::<GeneralAccountType>(&csv_path)
                    .await?;
            }
            Self::GeneralAccount { csv_path } => {
                upsert_general_account(&mut transaction, csv_path).await?;
            }
            Self::GeneralTransaction { csv_path } => {
                upsert_transaction(&mut transaction, csv_path).await?;
            }
            Self::CashAccount { csv_path } => {
                transaction.upsert_all::<CashAccount>(&csv_path).await?;
            }
            Self::TaxShelterType { csv_path } => {
                transaction.upsert_all::<TaxShelterType>(&csv_path).await?;
            }

            Self::CashEmergencyTarget { csv_path } => {
                transaction
                    .upsert_all::<CashEmergencyTarget>(&csv_path)
                    .await?;
            }
        };

        transaction.commit().await?;
        db.optimize().await?;

        Ok(())
    }
}
