use std::ops::Range;

use chrono::Local;
use chrono::{DateTime, TimeZone};

pub fn all_time_in_year(year: i32) -> Range<DateTime<Local>> {
    let start_of_year = Local::with_ymd_and_hms(&Local, year, 1, 1, 0, 0, 0)
        .single()
        .expect("unable to create timestamp for start of year");
    let end_of_year = Local::with_ymd_and_hms(&Local, year, 12, 31, 23, 59, 59)
        .single()
        .expect("unable to create timestamp for end of year");

    start_of_year..end_of_year
}
