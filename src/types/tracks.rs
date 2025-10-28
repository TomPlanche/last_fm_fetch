use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

// UTILS - Custom deserializers
fn u32_from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
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

fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

// BASE TYPES =================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseMbidText {
    pub mbid: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseObject {
    pub mbid: String,
    #[serde(default)]
    pub url: String,
    #[serde(alias = "#text")]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackImage {
    pub size: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Streamable {
    pub fulltrack: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artist {
    pub name: String,
    pub mbid: String,
    #[serde(default)]
    pub url: String,
    pub image: Vec<TrackImage>,
}

// DATE TYPE ==================================================================
// Unified - handles both API deserialization and storage

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Date {
    #[serde(deserialize_with = "u32_from_str")]
    pub uts: u32,
    #[serde(rename = "#text")]
    pub text: String,
}

// ATTRIBUTES =================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attributes {
    pub nowplaying: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RankAttr {
    pub rank: String,
}

// RECENT TRACK ===============================================================
// Unified - no more ApiRecentTrack vs RecentTrack split!

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecentTrack {
    pub artist: BaseMbidText,
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    pub image: Vec<TrackImage>,
    pub album: BaseMbidText,
    #[serde(rename = "@attr")]
    pub attr: Option<Attributes>,
    pub date: Option<Date>,
    pub name: String,
    pub mbid: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecentTrackExtended {
    pub artist: BaseObject,
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    pub image: Vec<TrackImage>,
    pub album: BaseObject,
    #[serde(rename = "@attr")]
    pub attr: Option<HashMap<String, String>>,
    pub date: Option<Date>,
    pub name: String,
    pub mbid: String,
    #[serde(default)]
    pub url: String,
}

// LOVED TRACK ================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LovedTrack {
    pub artist: BaseObject,
    pub date: Date,
    pub image: Vec<TrackImage>,
    pub streamable: Streamable,
    pub name: String,
    pub mbid: String,
    pub url: String,
}

// TOP TRACK ==================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopTrack {
    pub streamable: Streamable,
    pub mbid: String,
    pub name: String,
    pub image: Vec<TrackImage>,
    pub artist: BaseObject,
    pub url: String,
    #[serde(deserialize_with = "u32_from_str")]
    pub duration: u32,
    #[serde(rename = "@attr")]
    pub attr: RankAttr,
    #[serde(deserialize_with = "u32_from_str")]
    pub playcount: u32,
}

// RESPONSE WRAPPERS ==========================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseResponse {
    pub user: String,
    #[serde(deserialize_with = "u32_from_str", rename = "totalPages")]
    pub total_pages: u32,
    #[serde(deserialize_with = "u32_from_str")]
    pub page: u32,
    #[serde(deserialize_with = "u32_from_str", rename = "perPage")]
    pub per_page: u32,
    #[serde(deserialize_with = "u32_from_str")]
    pub total: u32,
}

// Recent tracks response
#[derive(Serialize, Deserialize, Debug)]
pub struct RecentTracks {
    pub track: Vec<RecentTrack>,
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRecentTracks {
    pub recenttracks: RecentTracks,
}

// Recent tracks extended response
#[derive(Serialize, Deserialize, Debug)]
pub struct RecentTracksExtended {
    pub track: Vec<RecentTrackExtended>,
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRecentTracksExtended {
    pub recenttracks: RecentTracksExtended,
}

// Loved tracks response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LovedTracks {
    pub track: Vec<LovedTrack>,
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLovedTracks {
    pub lovedtracks: LovedTracks,
}

// Top tracks response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopTracks {
    pub track: Vec<TopTrack>,
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTopTracks {
    pub toptracks: TopTracks,
}

// TRAITS =====================================================================

pub trait Timestamped {
    fn get_timestamp(&self) -> Option<u32>;
}

impl Timestamped for RecentTrack {
    fn get_timestamp(&self) -> Option<u32> {
        self.date.as_ref().map(|d| d.uts)
    }
}

impl Timestamped for LovedTrack {
    fn get_timestamp(&self) -> Option<u32> {
        Some(self.date.uts)
    }
}

impl Timestamped for RecentTrackExtended {
    fn get_timestamp(&self) -> Option<u32> {
        self.date.as_ref().map(|d| d.uts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_deserialization() {
        use serde_json::json;
        let json_value = json!({
            "uts": "1_234_567_890",
            "#text": "2009-02-13 23:31:30"
        });
        let date: Date = serde_json::from_value(json_value).unwrap();
        assert_eq!(date.uts, 1_234_567_890);
        assert_eq!(date.text, "2009-02-13 23:31:30");
    }

    #[test]
    fn test_bool_from_str() {
        use serde_json::json;
        // Test that "1" deserializes to true
        let json_value = json!({
            "artist": {"mbid": "", "#text": "Test"},
            "streamable": "1",
            "image": [],
            "album": {"mbid": "", "#text": ""},
            "name": "Test",
            "mbid": "",
            "url": ""
        });
        let track: RecentTrack = serde_json::from_value(json_value).unwrap();
        assert!(track.streamable);
    }

    #[test]
    fn test_timestamped_trait() {
        let track = RecentTrack {
            artist: BaseMbidText {
                mbid: String::new(),
                text: "Artist".to_string(),
            },
            streamable: false,
            image: vec![],
            album: BaseMbidText {
                mbid: String::new(),
                text: String::new(),
            },
            attr: None,
            date: Some(Date {
                uts: 1_234_567_890,
                text: "test".to_string(),
            }),
            name: "Track".to_string(),
            mbid: String::new(),
            url: String::new(),
        };

        assert_eq!(track.get_timestamp(), Some(1_234_567_890));
    }
}
