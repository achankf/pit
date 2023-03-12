use core::fmt;

struct TrimStringVisitor;

impl<'de> serde::de::Visitor<'de> for TrimStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a formatted date string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s.trim().to_string())
    }
}

pub fn trim_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    d.deserialize_str(TrimStringVisitor)
}
