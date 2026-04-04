use serde::Deserialize;

use crate::types::utils::{bool_from_str, u32_from_str};

// ── Internal deserialization helpers ─────────────────────────────────────────

/// The `registered` field in a user.getInfo response
#[derive(Deserialize, Debug)]
struct RegisteredRaw {
    #[serde(rename = "unixtime", deserialize_with = "u32_from_str")]
    unixtime: u32,
}

/// Top-level envelope for user.getInfo
#[derive(Deserialize, Debug)]
pub(crate) struct UserInfoResponse {
    user: UserInfoRaw,
}

#[derive(Deserialize, Debug)]
struct UserInfoRaw {
    name: String,
    realname: String,
    url: String,
    country: String,
    #[serde(deserialize_with = "u32_from_str")]
    age: u32,
    gender: String,
    #[serde(deserialize_with = "bool_from_str")]
    subscriber: bool,
    #[serde(deserialize_with = "u32_from_str")]
    playcount: u32,
    #[serde(deserialize_with = "u32_from_str")]
    playlists: u32,
    registered: RegisteredRaw,
}

// ── Public type ───────────────────────────────────────────────────────────────

/// Profile information for a Last.fm user.
///
/// Returned by [`user.getInfo`](https://www.last.fm/api/show/user.getInfo).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct UserInfo {
    /// Last.fm username
    pub name: String,
    /// Real name as set by the user
    pub real_name: String,
    /// URL to the user's Last.fm profile page
    pub url: String,
    /// Country code (e.g. `"France"`)
    pub country: String,
    /// Age
    pub age: u32,
    /// Gender (`"m"`, `"f"`, or `""`)
    pub gender: String,
    /// Whether the user has a Last.fm Pro subscription
    pub subscriber: bool,
    /// Total number of scrobbles
    pub play_count: u32,
    /// Number of playlists
    pub playlist_count: u32,
    /// Account registration date as a Unix timestamp
    pub registered: u32,
}

impl From<UserInfoRaw> for UserInfo {
    fn from(r: UserInfoRaw) -> Self {
        Self {
            name: r.name,
            real_name: r.realname,
            url: r.url,
            country: r.country,
            age: r.age,
            gender: r.gender,
            subscriber: r.subscriber,
            play_count: r.playcount,
            playlist_count: r.playlists,
            registered: r.registered.unixtime,
        }
    }
}

impl From<UserInfoResponse> for UserInfo {
    fn from(r: UserInfoResponse) -> Self {
        Self::from(r.user)
    }
}
