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

    match s.to_lowercase().as_str() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
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

// #[derive(Serialize, Debug, Deserialize, Clone)]
// pub struct Date {
//     #[serde(deserialize_with = "u32_from_str")]
//     pub uts: u32,
//     #[serde(rename = "#text")]
//     pub text: String,
// }

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
    pub url: String,
    image: Vec<TrackImage>,
}

// USER SCHEMAS ===============================================================
// Loved Track Schema
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiRecentTrackExtended {
    pub artist: BaseObject,
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    pub image: Vec<TrackImage>,
    pub album: BaseObject,
    #[serde(rename = "@attr")]
    pub attr: Option<HashMap<String, String>>,
    pub date: Option<ApiDate>,
    pub name: String,
    pub mbid: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecentTrackExtended {
    pub artist: BaseObject,
    pub streamable: bool,
    pub image: Vec<TrackImage>,
    pub album: BaseObject,
    #[serde(rename = "@attr")]
    pub attr: Option<HashMap<String, String>>,
    pub date: Option<Date>,
    pub name: String,
    pub mbid: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecentTracks {
    pub track: Vec<ApiRecentTrack>,
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRecentTracks {
    pub recenttracks: RecentTracks,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attributes {
    pub nowplaying: String,
}

// API response structs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiRecentTrack {
    pub artist: BaseMbidText,
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    pub image: Vec<TrackImage>,
    pub album: BaseMbidText,
    #[serde(rename = "@attr")]
    pub attr: Option<Attributes>,
    pub date: Option<ApiDate>,
    pub name: String,
    pub mbid: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiDate {
    #[serde(deserialize_with = "u32_from_str")]
    pub uts: u32,
    #[serde(rename = "#text")]
    pub text: String,
}

// File storage structs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecentTrack {
    pub artist: BaseMbidText,
    pub streamable: bool,
    pub image: Vec<TrackImage>,
    pub album: BaseMbidText,
    pub attr: Option<Attributes>,
    pub date: Option<Date>,
    pub name: String,
    pub mbid: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Date {
    pub uts: u32,
    #[serde(rename = "#text")]
    pub text: String,
}

// Conversion implementations
impl From<ApiRecentTrack> for RecentTrack {
    fn from(api_track: ApiRecentTrack) -> Self {
        RecentTrack {
            artist: api_track.artist,
            streamable: api_track.streamable,
            image: api_track.image,
            album: api_track.album,
            attr: api_track.attr,
            date: api_track.date.map(std::convert::Into::into),
            name: api_track.name,
            mbid: api_track.mbid,
            url: api_track.url,
        }
    }
}

impl From<ApiDate> for Date {
    fn from(api_date: ApiDate) -> Self {
        Date {
            uts: api_date.uts,
            text: api_date.text,
        }
    }
}

pub trait Timestamped {
    #[allow(dead_code)]
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

// TOP TRACKS SCHEMAS =========================================================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RankAttr {
    pub rank: String,
}

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
