use chrono::{DateTime, NaiveTime, TimeZone};
use chrono::{Local, NaiveDate};

pub fn start_of_day(naive_date: &NaiveDate) -> DateTime<Local> {
    let naive_time =
        NaiveTime::from_hms_opt(0, 0, 0).expect("to create constant naive time of 12AM");
    let naive_date_time = naive_date.and_time(naive_time);

    Local::from_local_datetime(&Local, &naive_date_time)
        .single()
        .expect("to create local date time")
}
