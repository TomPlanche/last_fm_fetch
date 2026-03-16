use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::types::utils::{bool_from_str, u32_from_str};

// BASE TYPES =================================================================

/// Basic type containing `MusicBrainz` ID and text content
///
/// Used for artist and album information in track responses
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BaseMbidText {
    /// `MusicBrainz` Identifier (may be empty string if not available)
    pub mbid: String,
    /// Text content (artist name, album name, etc.)
    #[serde(rename = "#text")]
    pub text: String,
}

/// Extended object type with `MusicBrainz` ID, URL, and name
///
/// Used for artist and album information in extended track responses
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BaseObject {
    /// `MusicBrainz` Identifier (may be empty string if not available)
    pub mbid: String,
    /// Last.fm URL for this object
    #[serde(default)]
    pub url: String,
    /// Name of the object (artist name, album name, etc.)
    #[serde(alias = "#text")]
    pub name: String,
}

/// Image information for tracks and albums
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TrackImage {
    /// Image size (e.g., "small", "medium", "large", "extralarge")
    pub size: String,
    /// URL to the image
    #[serde(rename = "#text")]
    pub text: String,
}

/// Streamability information for a track
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Streamable {
    /// Whether the full track is streamable ("0" or "1")
    pub fulltrack: String,
    /// Additional streamability information
    #[serde(rename = "#text")]
    pub text: String,
}

/// Detailed artist information
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Artist {
    /// Artist name
    pub name: String,
    /// `MusicBrainz` Identifier (may be empty string if not available)
    pub mbid: String,
    /// Last.fm URL for this artist
    #[serde(default)]
    pub url: String,
    /// Artist images in various sizes
    pub image: Vec<TrackImage>,
}

// DATE TYPE ==================================================================
// Unified - handles both API deserialization and storage

/// Date/timestamp information for tracks
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Date {
    /// Unix timestamp in seconds (not milliseconds) since January 1, 1970 UTC
    #[serde(deserialize_with = "u32_from_str")]
    pub uts: u32,
    /// Human-readable date string (e.g., "31 Jan 2024, 12:00")
    #[serde(rename = "#text")]
    pub text: String,
}

// ATTRIBUTES =================================================================

/// Attributes for recent tracks indicating current playback status
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Attributes {
    /// Whether this track is currently playing ("true" or "false")
    pub nowplaying: String,
}

/// Rank attributes for top tracks
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RankAttr {
    /// Numeric rank as a string (e.g., "1", "2", "3")
    pub rank: String,
}

// RECENT TRACK ===============================================================
// Unified - no more ApiRecentTrack vs RecentTrack split!

/// A track from a user's recent listening history
///
/// Retrieved from the `user.getrecenttracks` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RecentTrack {
    /// Artist information
    pub artist: BaseMbidText,
    /// Whether the track is streamable on Last.fm
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    /// Track/album images in various sizes
    pub image: Vec<TrackImage>,
    /// Album information
    pub album: BaseMbidText,
    /// Attributes (present if track is currently playing)
    #[serde(rename = "@attr")]
    pub attr: Option<Attributes>,
    /// When the track was played (None if currently playing)
    pub date: Option<Date>,
    /// Track name
    pub name: String,
    /// `MusicBrainz` track identifier (may be empty string)
    pub mbid: String,
    /// Last.fm URL for this track
    pub url: String,
}

impl fmt::Display for RecentTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.attr.is_some() {
            " [NOW PLAYING]"
        } else {
            ""
        };
        let date_str = self
            .date
            .as_ref()
            .map_or(String::new(), |d| format!(" ({})", d.text));

        write!(
            f,
            "{} - {} [{}]{date_str}{status}",
            self.name, self.artist.text, self.album.text
        )
    }
}

/// A track from recent listening history with extended artist/album information
///
/// Retrieved when using the `extended=1` parameter with `user.getrecenttracks`
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RecentTrackExtended {
    /// Extended artist information (includes URL)
    pub artist: BaseObject,
    /// Whether the track is streamable on Last.fm
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    /// Track/album images in various sizes
    pub image: Vec<TrackImage>,
    /// Extended album information (includes URL)
    pub album: BaseObject,
    /// Additional attributes (format varies, use `HashMap`)
    #[serde(rename = "@attr")]
    pub attr: Option<HashMap<String, String>>,
    /// When the track was played (None if currently playing)
    pub date: Option<Date>,
    /// Track name
    pub name: String,
    /// `MusicBrainz` track identifier (may be empty string)
    pub mbid: String,
    /// Last.fm URL for this track
    #[serde(default)]
    pub url: String,
}

impl fmt::Display for RecentTrackExtended {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_now_playing = self
            .attr
            .as_ref()
            .and_then(|a| a.get("nowplaying"))
            .is_some_and(|v| v == "true");
        let status = if is_now_playing { " [NOW PLAYING]" } else { "" };
        let date_str = self
            .date
            .as_ref()
            .map_or(String::new(), |d| format!(" ({})", d.text));

        write!(
            f,
            "{} - {} [{}]{date_str}{status}",
            self.name, self.artist.name, self.album.name
        )
    }
}

// LOVED TRACK ================================================================

/// A track that a user has marked as "loved" on Last.fm
///
/// Retrieved from the `user.getlovedtracks` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct LovedTrack {
    /// Artist information with URL
    pub artist: BaseObject,
    /// When the track was loved
    pub date: Date,
    /// Track/album images in various sizes
    pub image: Vec<TrackImage>,
    /// Streamability information
    pub streamable: Streamable,
    /// Track name
    pub name: String,
    /// `MusicBrainz` track identifier (may be empty string)
    pub mbid: String,
    /// Last.fm URL for this track
    pub url: String,
}

impl fmt::Display for LovedTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} (loved {})",
            self.name, self.artist.name, self.date.text
        )
    }
}

// TOP TRACK ==================================================================

/// A track from a user's top tracks, ranked by play count
///
/// Retrieved from the `user.gettoptracks` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopTrack {
    /// Streamability information
    pub streamable: Streamable,
    /// `MusicBrainz` track identifier (may be empty string)
    pub mbid: String,
    /// Track name
    pub name: String,
    /// Track/album images in various sizes
    pub image: Vec<TrackImage>,
    /// Artist information with URL
    pub artist: BaseObject,
    /// Last.fm URL for this track
    pub url: String,
    /// Track duration in seconds
    #[serde(deserialize_with = "u32_from_str")]
    pub duration: u32,
    /// Rank attributes (position in top tracks)
    #[serde(rename = "@attr")]
    pub attr: RankAttr,
    /// Total number of times this track has been played
    #[serde(deserialize_with = "u32_from_str")]
    pub playcount: u32,
}

impl fmt::Display for TopTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} - {} by {} ({} plays)",
            self.attr.rank, self.name, self.artist.name, self.playcount
        )
    }
}

// RESPONSE WRAPPERS ==========================================================

/// Base response metadata included in all paginated API responses
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BaseResponse {
    /// Username the request was made for
    pub user: String,
    /// Total number of pages available
    #[serde(deserialize_with = "u32_from_str", rename = "totalPages")]
    pub total_pages: u32,
    /// Current page number (1-indexed)
    #[serde(deserialize_with = "u32_from_str")]
    pub page: u32,
    /// Number of items per page
    #[serde(deserialize_with = "u32_from_str", rename = "perPage")]
    pub per_page: u32,
    /// Total number of items available across all pages
    #[serde(deserialize_with = "u32_from_str")]
    pub total: u32,
}

/// Recent tracks response wrapper
#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct RecentTracks {
    /// List of recent tracks
    pub track: Vec<RecentTrack>,
    /// Response metadata
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

/// Top-level recent tracks API response
#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct UserRecentTracks {
    /// Recent tracks data
    pub recenttracks: RecentTracks,
}

/// Recent tracks extended response wrapper
#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct RecentTracksExtended {
    /// List of extended recent tracks
    pub track: Vec<RecentTrackExtended>,
    /// Response metadata
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

/// Top-level extended recent tracks API response
#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct UserRecentTracksExtended {
    /// Extended recent tracks data
    pub recenttracks: RecentTracksExtended,
}

/// Loved tracks response wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct LovedTracks {
    /// List of loved tracks
    pub track: Vec<LovedTrack>,
    /// Response metadata
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

/// Top-level loved tracks API response
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserLovedTracks {
    /// Loved tracks data
    pub lovedtracks: LovedTracks,
}

/// Top tracks response wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopTracks {
    /// List of top tracks
    pub track: Vec<TopTrack>,
    /// Response metadata
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

/// Top-level top tracks API response
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserTopTracks {
    /// Top tracks data
    pub toptracks: TopTracks,
}

// ANALYTICS ==================================================================

/// Represents a track's play count information
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct TrackPlayInfo {
    /// Track name
    pub name: String,
    /// Number of times played
    pub play_count: u32,
    /// Artist name
    pub artist: String,
    /// Album name (if available)
    pub album: Option<String>,
    /// Image URL (if available)
    pub image_url: Option<String>,
    /// Whether the track is currently playing
    pub currently_playing: bool,
    /// Unix timestamp of when played
    pub date: Option<u32>,
    /// Last.fm URL
    pub url: String,
}

// TRAITS =====================================================================

/// Trait for types that have a timestamp
pub trait Timestamped {
    /// Get the timestamp as a Unix epoch in seconds
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
#[allow(clippy::unwrap_used)]
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
