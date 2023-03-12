mod get_acb;
mod get_asset_rebalance;
mod get_balance;
mod get_current_credit_card_balance;
mod get_emergency_rebalance;
mod get_net_asset_balance;
mod get_net_revenue_balance;
mod refresh_market_price;
mod check_cash_inactivity;

pub use get_acb::Acb;
pub use get_asset_rebalance::AssetRebalance;
pub use get_balance::BalanceRecord;
pub use get_emergency_rebalance::EmergencyRebalance;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct NetBalanceRecord {
    pub first_name: String,
    pub last_name: String,
    pub currency: String,
    pub currency_symbol: String,
    pub balance: f64,
}
