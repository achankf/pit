use core::fmt;

use chrono::{Local, NaiveDateTime};

struct ExcelDateTimeVisitor;

impl<'de> serde::de::Visitor<'de> for ExcelDateTimeVisitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a formatted date string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S") {
            Ok(datetime) => {
                let datetime = datetime.and_local_timezone(Local).unwrap();
                Ok(datetime.timestamp())
            }
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s),
                &self,
            )),
        }
    }
}

pub fn excel_datetime_format<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    d.deserialize_str(ExcelDateTimeVisitor)
}
