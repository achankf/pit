// https://github.com/BurntSushi/rust-csv/issues/135#issuecomment-752783194

use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer,
};

pub fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.to_lowercase().as_str() {
        "t" | "true" | "1" | "on" | "y" | "yes" => Ok(true),
        "f" | "false" | "0" | "off" | "n" | "no" => Ok(false),
        other => Err(de::Error::invalid_value(
            Unexpected::Str(other),
            &"Must be truthy (t, true, 1, on, y, yes) or falsey (f, false, 0, off, n, no)",
        )),
    }
}
