use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

// UTILS
fn u32_from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    s.parse::<u32>().map_err(serde::de::Error::custom)
}

fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(serde::de::Error::custom("Invalid boolean value")),
    }
}

// BASE SCHEMAS ===============================================================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseOptions {
    pub limit: u16,
    pub page: u16,
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseMbidText {
    pub mbid: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseObject {
    pub mbid: String,
    pub url: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackImage {
    pub size: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Date {
    #[serde(deserialize_with = "u32_from_str")]
    pub uts: u32,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Streamable {
    pub fulltrack: String,
    #[serde(rename = "#text")]
    pub text: String,
}

// USER SCHEMAS ===============================================================
// Loved Track Schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LovedTrack {
    pub artist: BaseObject,
    pub date: Date,
    pub image: Vec<TrackImage>,
    pub streamable: Streamable,
}

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

// Recent Track Schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecentTrack {
    pub artist: BaseMbidText,
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    pub image: Vec<TrackImage>,
    pub album: BaseMbidText,
    #[serde(rename = "@attr")]
    pub attr: Option<HashMap<String, String>>,
    pub date: Option<Date>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
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
}

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
