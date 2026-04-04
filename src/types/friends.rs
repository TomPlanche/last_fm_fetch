use serde::Deserialize;

use crate::types::TrackImage;
use crate::types::utils::{bool_from_str, u32_from_str};

// ── Internal deserialization helpers ─────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct FriendsResponse {
    friends: FriendsRaw,
}

#[derive(Deserialize, Debug)]
struct FriendsRaw {
    user: Vec<FriendRaw>,
    #[serde(rename = "@attr")]
    attr: FriendsAttr,
}

#[derive(Deserialize, Debug)]
struct FriendsAttr {
    #[serde(deserialize_with = "u32_from_str")]
    total: u32,
    #[serde(deserialize_with = "u32_from_str")]
    page: u32,
    #[serde(rename = "totalPages", deserialize_with = "u32_from_str")]
    total_pages: u32,
    #[serde(rename = "perPage", deserialize_with = "u32_from_str")]
    per_page: u32,
}

#[derive(Deserialize, Debug)]
struct RegisteredRaw {
    #[serde(rename = "unixtime", deserialize_with = "u32_from_str")]
    unixtime: u32,
}

#[derive(Deserialize, Debug)]
struct FriendRaw {
    name: String,
    realname: String,
    url: String,
    country: String,
    #[serde(deserialize_with = "bool_from_str")]
    subscriber: bool,
    image: Vec<TrackImage>,
    registered: RegisteredRaw,
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Profile information for a Last.fm friend.
///
/// Returned by [`user.getFriends`](https://www.last.fm/api/show/user.getFriends).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FriendProfile {
    /// Last.fm username
    pub name: String,
    /// Real name as set by the user
    pub real_name: String,
    /// Last.fm profile URL
    pub url: String,
    /// Country
    pub country: String,
    /// Whether the user has a Last.fm Pro subscription
    pub subscriber: bool,
    /// Profile images in various sizes
    pub images: Vec<TrackImage>,
    /// Account registration date as a Unix timestamp
    pub registered: u32,
}

impl From<FriendRaw> for FriendProfile {
    fn from(r: FriendRaw) -> Self {
        Self {
            name: r.name,
            real_name: r.realname,
            url: r.url,
            country: r.country,
            subscriber: r.subscriber,
            images: r.image,
            registered: r.registered.unixtime,
        }
    }
}

/// Paginated response from `user.getFriends`
#[derive(Debug)]
#[non_exhaustive]
pub struct FriendsPage {
    /// The friends on this page
    pub friends: Vec<FriendProfile>,
    /// Total number of friends
    pub total: u32,
    /// Current page number (1-indexed)
    pub page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Number of results per page
    pub per_page: u32,
}

impl From<FriendsResponse> for FriendsPage {
    fn from(r: FriendsResponse) -> Self {
        Self {
            total: r.friends.attr.total,
            page: r.friends.attr.page,
            total_pages: r.friends.attr.total_pages,
            per_page: r.friends.attr.per_page,
            friends: r
                .friends
                .user
                .into_iter()
                .map(FriendProfile::from)
                .collect(),
        }
    }
}
