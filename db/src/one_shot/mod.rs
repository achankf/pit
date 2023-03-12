mod check_cash_inactivity;
mod get_acb;
mod get_asset_last_update;
mod get_asset_rebalance;
mod get_balance;
mod get_credit_card_pad_injection;
mod get_current_credit_card_balance;
mod get_emergency_rebalance;
mod get_expense_by_category;
mod get_net_asset_balance;
mod get_net_revenue_balance;
mod get_next_transaction_id;
mod get_stock_transaction;
mod get_stock_unit;
mod get_transaction_by_account_key;
mod justify_amex;

pub use get_acb::Acb;
pub use get_asset_last_update::AccountLatestTransaction;
pub use get_asset_rebalance::AssetRebalance;
pub use get_balance::BalanceRecord;
pub use get_credit_card_pad_injection::CreditCardPadInjection;
pub use get_emergency_rebalance::EmergencyRebalance;
pub use get_stock_transaction::StockTransaction;
pub use get_stock_unit::StockUnit;
pub use get_transaction_by_account_key::TransactionByAccountKey;
pub use justify_amex::JustifyAmex;

#[derive(Clone, serde::Deserialize, Debug)]
pub struct NetBalanceRecord {
    pub first_name: String,
    pub last_name: String,
    pub currency: String,
    pub currency_symbol: String,
    pub balance: f64,
}
