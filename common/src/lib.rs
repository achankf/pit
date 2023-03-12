mod all_time_in_year;
mod all_time_until_now;
mod bool_from_str;
mod create_fetch_client;
mod days_prior_until_now;
mod deserialize_into_map;
mod excel_date_format;
mod excel_date_optional_time_format;
mod excel_datetime_format;
mod fetch_symbol_data;
mod trim_string;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub struct Exchange {
    pub short: &'static str,
    pub name: &'static str,
}

pub const EXCHANGES: &[Exchange] = &[Exchange {
    short: "TSX",
    name: "Toronto Stock Exchange",
}];

#[derive(Serialize, Deserialize)]
pub struct Quote {
    pub quote_type: String,
    pub long_name: String,
}

pub type SymbolList = BTreeMap<String, BTreeSet<String>>;
pub type SymbolMetaData = BTreeMap<String, BTreeMap<String, Quote>>;

pub trait Id {
    type IdType: Clone + Ord + std::fmt::Debug;
    fn id(&self) -> Self::IdType;
}

pub use all_time_in_year::all_time_in_year;
pub use all_time_until_now::all_time_until_now;
pub use bool_from_str::bool_from_str;
pub use days_prior_until_now::days_prior_until_end_of_today;
pub use deserialize_into_map::deserialize_into_map;
pub use excel_date_format::excel_date_format;
pub use excel_date_optional_time_format::excel_date_optional_time_format;
pub use excel_datetime_format::excel_datetime_format;
pub use fetch_symbol_data::fetch_symbol_data;
pub use trim_string::trim_string;
