use std::ops::Range;

use chrono::Local;
use chrono::{DateTime, Datelike, Days, TimeZone};

pub fn days_prior_until_end_of_today(days_prior: u64) -> Range<DateTime<Local>> {
    let now = Local::now();
    let today = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 23, 59, 59)
        .single()
        .expect("cannot get the start of today");

    let start_date = today
        .checked_sub_days(Days::new(days_prior))
        .expect("unable to subtract days");

    start_date..now
}
