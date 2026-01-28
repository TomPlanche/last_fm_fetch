use serde::{Deserialize, Deserializer};

/// Custom deserializer that accepts both string and numeric u32 values
///
/// The Last.fm API sometimes returns numeric values as strings (e.g., "12345" instead of 12345).
/// This deserializer handles both formats and also removes underscores from string representations.
pub fn u32_from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        String(String),
        Number(u32),
    }

    match StringOrNum::deserialize(deserializer)? {
        StringOrNum::String(s) => s
            .replace('_', "")
            .parse::<u32>()
            .map_err(serde::de::Error::custom),
        StringOrNum::Number(n) => Ok(n),
    }
}

/// Custom deserializer that accepts both string and boolean values
///
/// The Last.fm API returns boolean values as strings ("0"/"1" or "true"/"false").
/// This deserializer converts them to proper Rust boolean values.
pub fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        String(String),
        Bool(bool),
    }

    match StringOrBool::deserialize(deserializer)? {
        StringOrBool::String(s) => match s.to_lowercase().as_str() {
            "1" | "true" => Ok(true),
            "0" | "false" => Ok(false),
            _ => Err(serde::de::Error::custom("Invalid boolean value")),
        },
        StringOrBool::Bool(b) => Ok(b),
    }
}
