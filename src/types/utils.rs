use serde::{Deserialize, Deserializer};

/// Custom deserializer that accepts either a sequence of items or a single item.
///
/// The Last.fm API returns list fields (e.g. `track`, `artist`, `album`) as a JSON array
/// in most cases, but collapses them to a single object when the result contains exactly
/// one item. This most notably happens on the `limit=1` probe request used to discover the
/// total count, when the user is not currently playing anything. This deserializer accepts
/// both shapes and always yields a `Vec`.
pub(crate) fn vec_or_single<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany<T> {
        Many(Vec<T>),
        One(T),
    }

    Ok(match OneOrMany::<T>::deserialize(deserializer)? {
        OneOrMany::Many(items) => items,
        OneOrMany::One(item) => vec![item],
    })
}

/// Custom deserializer that accepts both string and numeric u32 values
///
/// The Last.fm API sometimes returns numeric values as strings (e.g., "12345" instead of 12345).
/// This deserializer handles both formats and also removes underscores from string representations.
pub(crate) fn u32_from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
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
pub(crate) fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::vec_or_single;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Item {
        name: String,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Wrapper {
        #[serde(deserialize_with = "vec_or_single")]
        items: Vec<Item>,
    }

    #[test]
    fn vec_or_single_parses_array() {
        let json = r#"{ "items": [{ "name": "a" }, { "name": "b" }] }"#;
        let parsed: Wrapper = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.items.len(), 2);
    }

    #[test]
    fn vec_or_single_parses_single_object() {
        // Last.fm collapses single-element lists to a bare object (the limit=1 probe quirk).
        let json = r#"{ "items": { "name": "a" } }"#;
        let parsed: Wrapper = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.items, vec![Item { name: "a".to_string() }]);
    }

    #[test]
    fn vec_or_single_parses_empty_array() {
        let json = r#"{ "items": [] }"#;
        let parsed: Wrapper = serde_json::from_str(json).unwrap();
        assert!(parsed.items.is_empty());
    }
}
