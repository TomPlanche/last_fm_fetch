use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::types::utils::{bool_from_str, u32_from_str};

use super::track_list::TrackList;

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

impl PartialEq for RecentTrack {
    fn eq(&self, other: &Self) -> bool {
        self.date.as_ref().map(|d| d.uts) == other.date.as_ref().map(|d| d.uts)
    }
}

impl Eq for RecentTrack {}

impl PartialOrd for RecentTrack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RecentTrack {
    fn cmp(&self, other: &Self) -> Ordering {
        // None (now playing) is treated as the most recent
        match (self.date.as_ref(), other.date.as_ref()) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.uts.cmp(&b.uts),
        }
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

impl PartialEq for RecentTrackExtended {
    fn eq(&self, other: &Self) -> bool {
        self.date.as_ref().map(|d| d.uts) == other.date.as_ref().map(|d| d.uts)
    }
}

impl Eq for RecentTrackExtended {}

impl PartialOrd for RecentTrackExtended {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RecentTrackExtended {
    fn cmp(&self, other: &Self) -> Ordering {
        // None (now playing) is treated as the most recent
        match (self.date.as_ref(), other.date.as_ref()) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.uts.cmp(&b.uts),
        }
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

impl PartialEq for LovedTrack {
    fn eq(&self, other: &Self) -> bool {
        self.date.uts == other.date.uts
    }
}

impl Eq for LovedTrack {}

impl PartialOrd for LovedTrack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LovedTrack {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.uts.cmp(&other.date.uts)
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

impl PartialEq for TopTrack {
    fn eq(&self, other: &Self) -> bool {
        self.playcount == other.playcount
    }
}

impl Eq for TopTrack {}

impl PartialOrd for TopTrack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopTrack {
    fn cmp(&self, other: &Self) -> Ordering {
        self.playcount.cmp(&other.playcount)
    }
}

// SCORED TRACK ===============================================================

/// A track aggregated from a [`RecentTrack`] list with a computed play count.
///
/// This is the output of [`TrackList<RecentTrack>::to_set`]. Tracks sharing
/// the same `(name, artist)` pair are merged; `play_count` reflects how many
/// times that track appears in the source list. Useful as a substitute for
/// the Top Tracks API when the built-in [`crate::types::Period`] options don't
/// cover the time range you need.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScoredTrack {
    /// Track name
    pub name: String,
    /// Artist name
    pub artist: String,
    /// Artist `MusicBrainz` ID (may be empty string)
    pub artist_mbid: String,
    /// Album name (may be empty string)
    pub album: String,
    /// `MusicBrainz` track ID (may be empty string)
    pub mbid: String,
    /// Last.fm URL for this track
    pub url: String,
    /// Track images in various sizes
    pub image: Vec<TrackImage>,
    /// Number of times this track appears in the source recent tracks list
    pub play_count: u32,
    /// 1-indexed rank ordered by `play_count` descending (1 = most played)
    pub rank: u32,
}

impl fmt::Display for ScoredTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} {} - {} ({} play{})",
            self.rank,
            self.name,
            self.artist,
            self.play_count,
            if self.play_count == 1 { "" } else { "s" }
        )
    }
}

impl PartialEq for ScoredTrack {
    fn eq(&self, other: &Self) -> bool {
        self.play_count == other.play_count
    }
}

impl Eq for ScoredTrack {}

impl PartialOrd for ScoredTrack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredTrack {
    fn cmp(&self, other: &Self) -> Ordering {
        self.play_count.cmp(&other.play_count)
    }
}

// SCORED ARTIST ==============================================================

/// An artist aggregated from a [`RecentTrack`] list with a computed play count.
///
/// This is the output of [`TrackList<RecentTrack>::top_artists`]. Every track
/// by the same artist is counted; `play_count` is the total number of
/// scrobbles by that artist in the source list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScoredArtist {
    /// Artist name
    pub name: String,
    /// `MusicBrainz` artist ID (may be empty string)
    pub mbid: String,
    /// Number of scrobbles by this artist in the source list
    pub play_count: u32,
    /// 1-indexed rank ordered by `play_count` descending (1 = most played)
    pub rank: u32,
}

impl fmt::Display for ScoredArtist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} {} ({} play{})",
            self.rank,
            self.name,
            self.play_count,
            if self.play_count == 1 { "" } else { "s" }
        )
    }
}

impl PartialEq for ScoredArtist {
    fn eq(&self, other: &Self) -> bool {
        self.play_count == other.play_count
    }
}

impl Eq for ScoredArtist {}

impl PartialOrd for ScoredArtist {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredArtist {
    fn cmp(&self, other: &Self) -> Ordering {
        self.play_count.cmp(&other.play_count)
    }
}

// SCORED ALBUM ===============================================================

/// An album aggregated from a [`RecentTrack`] list with a computed play count.
///
/// This is the output of [`TrackList<RecentTrack>::top_albums`]. Tracks are
/// grouped by `(album name, artist)` pair; tracks with an empty album field
/// are excluded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScoredAlbum {
    /// Album name
    pub name: String,
    /// `MusicBrainz` album ID (may be empty string)
    pub mbid: String,
    /// Artist name
    pub artist: String,
    /// Number of scrobbles from this album in the source list
    pub play_count: u32,
    /// 1-indexed rank ordered by `play_count` descending (1 = most played)
    pub rank: u32,
}

impl fmt::Display for ScoredAlbum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} {} — {} ({} play{})",
            self.rank,
            self.name,
            self.artist,
            self.play_count,
            if self.play_count == 1 { "" } else { "s" }
        )
    }
}

impl PartialEq for ScoredAlbum {
    fn eq(&self, other: &Self) -> bool {
        self.play_count == other.play_count
    }
}

impl Eq for ScoredAlbum {}

impl PartialOrd for ScoredAlbum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredAlbum {
    fn cmp(&self, other: &Self) -> Ordering {
        self.play_count.cmp(&other.play_count)
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

// AGGREGATION ================================================================

impl TrackList<RecentTrack> {
    /// Aggregate recent tracks into unique entries with computed play counts.
    ///
    /// Groups tracks by `(name, artist)` pair and counts occurrences, producing
    /// a list equivalent to what the Top Tracks API returns — but computed
    /// locally from your recent listening history. This is particularly useful
    /// when none of the predefined [`crate::types::Period`] options cover the
    /// exact date range you care about.
    ///
    /// - Tracks are identified by an exact `(name, artist)` match (as returned
    ///   by Last.fm, which normalises both fields).
    /// - The representative metadata (image, URL, album, etc.) is taken from
    ///   the **most recently played** scrobble of that track.
    /// - Currently-playing tracks (no timestamp) are included in the count.
    /// - The returned list is sorted by `play_count` descending; `rank` is
    ///   1-indexed (rank 1 = most played).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use chrono::{Duration, Utc};
    ///
    /// let now = Utc::now();
    /// let one_week_ago = now - Duration::weeks(1);
    ///
    /// let recent = client
    ///     .recent_tracks("username")
    ///     .between(one_week_ago.timestamp(), now.timestamp())
    ///     .fetch()
    ///     .await?;
    ///
    /// let top = recent.to_set();
    /// println!("{top}"); // prints tracks sorted by play count
    /// ```
    #[must_use]
    pub fn to_set(&self) -> TrackList<ScoredTrack> {
        // Map (name, artist) → (representative RecentTrack, count).
        // The first occurrence is kept as the representative because Last.fm
        // returns tracks in reverse-chronological order, so index 0 is the
        // most recently played scrobble of that track.
        let mut groups: HashMap<(String, String), (RecentTrack, u32)> = HashMap::new();

        for track in self {
            let key = (track.name.clone(), track.artist.text.clone());
            let entry = groups.entry(key).or_insert_with(|| (track.clone(), 0));
            entry.1 += 1;
        }

        let mut scored: Vec<ScoredTrack> = groups
            .into_values()
            .map(|(rep, play_count)| ScoredTrack {
                name: rep.name,
                artist: rep.artist.text,
                artist_mbid: rep.artist.mbid,
                album: rep.album.text,
                mbid: rep.mbid,
                url: rep.url,
                image: rep.image,
                play_count,
                rank: 0,
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.play_count.cmp(&a.play_count));

        for (i, track) in scored.iter_mut().enumerate() {
            track.rank = u32::try_from(i).map_or(u32::MAX, |n| n.saturating_add(1));
        }

        TrackList::from(scored)
    }

    /// Aggregate recent tracks by artist, computing play counts.
    ///
    /// Groups all tracks by artist name and counts scrobbles per artist.
    /// Useful as a local substitute for the Top Artists API when the
    /// built-in [`crate::types::Period`] options don't suit your needs.
    ///
    /// The returned list is sorted by play count descending, with 1-indexed
    /// ranks (rank 1 = most played).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let top_artists = recent.top_artists();
    /// for artist in &top_artists {
    ///     println!("{artist}"); // "#1 Radiohead (42 plays)"
    /// }
    /// ```
    #[must_use]
    pub fn top_artists(&self) -> TrackList<ScoredArtist> {
        let mut groups: HashMap<(String, String), u32> = HashMap::new();

        for track in self {
            let key = (track.artist.text.clone(), track.artist.mbid.clone());
            *groups.entry(key).or_insert(0) += 1;
        }

        let mut scored: Vec<ScoredArtist> = groups
            .into_iter()
            .map(|((name, mbid), play_count)| ScoredArtist {
                name,
                mbid,
                play_count,
                rank: 0,
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.play_count.cmp(&a.play_count));

        for (i, artist) in scored.iter_mut().enumerate() {
            artist.rank = u32::try_from(i).map_or(u32::MAX, |n| n.saturating_add(1));
        }

        TrackList::from(scored)
    }

    /// Aggregate recent tracks by album, computing play counts.
    ///
    /// Groups all tracks by `(album name, artist)` pair and counts scrobbles.
    /// Tracks with an empty album field are excluded. Useful as a local
    /// substitute for the Top Albums API.
    ///
    /// The returned list is sorted by play count descending, with 1-indexed
    /// ranks (rank 1 = most played).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let top_albums = recent.top_albums();
    /// for album in &top_albums {
    ///     println!("{album}"); // "#1 OK Computer — Radiohead (12 plays)"
    /// }
    /// ```
    #[must_use]
    pub fn top_albums(&self) -> TrackList<ScoredAlbum> {
        let mut groups: HashMap<(String, String, String), u32> = HashMap::new();

        for track in self {
            if track.album.text.is_empty() {
                continue;
            }
            let key = (
                track.album.text.clone(),
                track.album.mbid.clone(),
                track.artist.text.clone(),
            );
            *groups.entry(key).or_insert(0) += 1;
        }

        let mut scored: Vec<ScoredAlbum> = groups
            .into_iter()
            .map(|((name, mbid, artist), play_count)| ScoredAlbum {
                name,
                mbid,
                artist,
                play_count,
                rank: 0,
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.play_count.cmp(&a.play_count));

        for (i, album) in scored.iter_mut().enumerate() {
            album.rank = u32::try_from(i).map_or(u32::MAX, |n| n.saturating_add(1));
        }

        TrackList::from(scored)
    }

    /// Count plays per hour of the day (UTC).
    ///
    /// Returns a fixed-size array of 24 counters indexed by UTC hour
    /// (index 0 = midnight, index 23 = 11 PM). Currently-playing tracks
    /// (no timestamp) are excluded.
    ///
    /// Useful for building a "listening clock" visualisation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let hours = recent.by_hour();
    /// let peak = hours.iter().enumerate().max_by_key(|(_, &c)| c);
    /// if let Some((hour, count)) = peak {
    ///     println!("Most active at {hour}:00 UTC ({count} plays)");
    /// }
    /// ```
    #[must_use]
    pub fn by_hour(&self) -> [u32; 24] {
        let mut counts = [0u32; 24];
        for track in self {
            if let Some(date) = &track.date {
                let hour = usize::try_from(date.uts % 86_400 / 3600).unwrap_or(0);
                counts[hour] = counts[hour].saturating_add(1);
            }
        }
        counts
    }

    /// Count plays per calendar date (UTC).
    ///
    /// Returns a [`BTreeMap`] from [`NaiveDate`] to play count, sorted
    /// chronologically. Currently-playing tracks (no timestamp) are excluded.
    ///
    /// Useful for heatmap data, streak analysis, and day-by-day history.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use chrono::NaiveDate;
    ///
    /// let by_date = recent.by_date();
    /// for (date, count) in &by_date {
    ///     println!("{date}: {count} plays");
    /// }
    /// ```
    #[must_use]
    pub fn by_date(&self) -> BTreeMap<NaiveDate, u32> {
        let mut counts: BTreeMap<NaiveDate, u32> = BTreeMap::new();
        for track in self {
            if let Some(date) = &track.date
                && let Some(dt) = DateTime::<Utc>::from_timestamp(i64::from(date.uts), 0)
            {
                *counts.entry(dt.date_naive()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Return the length of the longest consecutive listening-day streak.
    ///
    /// A streak is a run of calendar days (UTC) on which at least one track
    /// was scrobbled. Returns `0` if the list is empty or contains only
    /// currently-playing tracks (no timestamps).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let streak = recent.streak();
    /// println!("Longest streak: {streak} day(s)");
    /// ```
    #[must_use]
    pub fn streak(&self) -> u32 {
        let dates = self.by_date();
        if dates.is_empty() {
            return 0;
        }

        // BTreeMap keys are already sorted chronologically.
        let sorted: Vec<NaiveDate> = dates.into_keys().collect();
        let mut max_streak = 1u32;
        let mut current = 1u32;

        for window in sorted.windows(2) {
            if let [prev, next] = window {
                if next.signed_duration_since(*prev).num_days() == 1 {
                    current += 1;
                    if current > max_streak {
                        max_streak = current;
                    }
                } else {
                    current = 1;
                }
            }
        }

        max_streak
    }

    /// Return a new list with the currently-playing track removed.
    ///
    /// The currently-playing track is identified by `attr.nowplaying == "true"`
    /// and has no timestamp. All other tracks are preserved in their original
    /// order.
    #[must_use]
    pub fn without_now_playing(&self) -> Self {
        self.iter()
            .filter(|t| t.attr.as_ref().is_none_or(|a| a.nowplaying != "true"))
            .cloned()
            .collect()
    }

    /// Count the number of distinct artists in the list.
    ///
    /// Artists are matched by their exact name as returned by Last.fm.
    #[must_use]
    pub fn unique_artist_count(&self) -> usize {
        self.iter()
            .map(|t| &t.artist.text)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    /// Count the number of distinct `(track name, artist)` pairs in the list.
    #[must_use]
    pub fn unique_track_count(&self) -> usize {
        self.iter()
            .map(|t| (&t.name, &t.artist.text))
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

impl TrackList<RecentTrackExtended> {
    /// Aggregate extended recent tracks into unique entries with computed play counts.
    ///
    /// Identical semantics to [`TrackList<RecentTrack>::to_set`] but operates on
    /// extended track data. Groups by `(name, artist)` pair; the representative
    /// metadata is taken from the **most recently played** scrobble of each track.
    ///
    /// The returned list is sorted by `play_count` descending with 1-indexed ranks.
    #[must_use]
    pub fn to_set(&self) -> TrackList<ScoredTrack> {
        let mut groups: HashMap<(String, String), (RecentTrackExtended, u32)> = HashMap::new();

        for track in self {
            let key = (track.name.clone(), track.artist.name.clone());
            let entry = groups.entry(key).or_insert_with(|| (track.clone(), 0));
            entry.1 += 1;
        }

        let mut scored: Vec<ScoredTrack> = groups
            .into_values()
            .map(|(rep, play_count)| ScoredTrack {
                name: rep.name,
                artist: rep.artist.name,
                artist_mbid: rep.artist.mbid,
                album: rep.album.name,
                mbid: rep.mbid,
                url: rep.url,
                image: rep.image,
                play_count,
                rank: 0,
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.play_count.cmp(&a.play_count));

        for (i, track) in scored.iter_mut().enumerate() {
            track.rank = u32::try_from(i).map_or(u32::MAX, |n| n.saturating_add(1));
        }

        TrackList::from(scored)
    }

    /// Aggregate extended recent tracks by artist, computing play counts.
    ///
    /// Identical semantics to [`TrackList<RecentTrack>::top_artists`].
    /// The returned list is sorted by play count descending with 1-indexed ranks.
    #[must_use]
    pub fn top_artists(&self) -> TrackList<ScoredArtist> {
        let mut groups: HashMap<(String, String), u32> = HashMap::new();

        for track in self {
            let key = (track.artist.name.clone(), track.artist.mbid.clone());
            *groups.entry(key).or_insert(0) += 1;
        }

        let mut scored: Vec<ScoredArtist> = groups
            .into_iter()
            .map(|((name, mbid), play_count)| ScoredArtist {
                name,
                mbid,
                play_count,
                rank: 0,
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.play_count.cmp(&a.play_count));

        for (i, artist) in scored.iter_mut().enumerate() {
            artist.rank = u32::try_from(i).map_or(u32::MAX, |n| n.saturating_add(1));
        }

        TrackList::from(scored)
    }

    /// Aggregate extended recent tracks by album, computing play counts.
    ///
    /// Identical semantics to [`TrackList<RecentTrack>::top_albums`].
    /// Tracks with an empty album field are excluded. The returned list is
    /// sorted by play count descending with 1-indexed ranks.
    #[must_use]
    pub fn top_albums(&self) -> TrackList<ScoredAlbum> {
        let mut groups: HashMap<(String, String, String), u32> = HashMap::new();

        for track in self {
            if track.album.name.is_empty() {
                continue;
            }
            let key = (
                track.album.name.clone(),
                track.album.mbid.clone(),
                track.artist.name.clone(),
            );
            *groups.entry(key).or_insert(0) += 1;
        }

        let mut scored: Vec<ScoredAlbum> = groups
            .into_iter()
            .map(|((name, mbid, artist), play_count)| ScoredAlbum {
                name,
                mbid,
                artist,
                play_count,
                rank: 0,
            })
            .collect();

        scored.sort_unstable_by(|a, b| b.play_count.cmp(&a.play_count));

        for (i, album) in scored.iter_mut().enumerate() {
            album.rank = u32::try_from(i).map_or(u32::MAX, |n| n.saturating_add(1));
        }

        TrackList::from(scored)
    }

    /// Count plays per hour of the day (UTC).
    ///
    /// Identical semantics to [`TrackList<RecentTrack>::by_hour`].
    /// Returns a fixed-size array of 24 counters indexed by UTC hour (0–23).
    /// Currently-playing tracks (no timestamp) are excluded.
    #[must_use]
    pub fn by_hour(&self) -> [u32; 24] {
        let mut counts = [0u32; 24];
        for track in self {
            if let Some(date) = &track.date {
                let hour = usize::try_from(date.uts % 86_400 / 3600).unwrap_or(0);
                counts[hour] = counts[hour].saturating_add(1);
            }
        }
        counts
    }

    /// Count plays per calendar date (UTC).
    ///
    /// Identical semantics to [`TrackList<RecentTrack>::by_date`].
    /// Returns a [`BTreeMap`] from [`NaiveDate`] to play count, sorted chronologically.
    /// Currently-playing tracks (no timestamp) are excluded.
    #[must_use]
    pub fn by_date(&self) -> BTreeMap<NaiveDate, u32> {
        let mut counts: BTreeMap<NaiveDate, u32> = BTreeMap::new();
        for track in self {
            if let Some(date) = &track.date
                && let Some(dt) = DateTime::<Utc>::from_timestamp(i64::from(date.uts), 0)
            {
                *counts.entry(dt.date_naive()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Return the length of the longest consecutive listening-day streak.
    ///
    /// Identical semantics to [`TrackList<RecentTrack>::streak`].
    #[must_use]
    pub fn streak(&self) -> u32 {
        let dates = self.by_date();
        if dates.is_empty() {
            return 0;
        }

        let sorted: Vec<NaiveDate> = dates.into_keys().collect();
        let mut max_streak = 1u32;
        let mut current = 1u32;

        for window in sorted.windows(2) {
            if let [prev, next] = window {
                if next.signed_duration_since(*prev).num_days() == 1 {
                    current += 1;
                    if current > max_streak {
                        max_streak = current;
                    }
                } else {
                    current = 1;
                }
            }
        }

        max_streak
    }

    /// Return a new list with the currently-playing track removed.
    ///
    /// For extended tracks the currently-playing entry is identified by
    /// `attr["nowplaying"] == "true"`. All other tracks are preserved in
    /// their original order.
    #[must_use]
    pub fn without_now_playing(&self) -> Self {
        self.iter()
            .filter(|t| {
                t.attr
                    .as_ref()
                    .is_none_or(|a| a.get("nowplaying").is_none_or(|v| v != "true"))
            })
            .cloned()
            .collect()
    }

    /// Count the number of distinct artists in the list.
    ///
    /// Artists are matched by their exact name as returned by Last.fm.
    #[must_use]
    pub fn unique_artist_count(&self) -> usize {
        self.iter()
            .map(|t| &t.artist.name)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    /// Count the number of distinct `(track name, artist)` pairs in the list.
    #[must_use]
    pub fn unique_track_count(&self) -> usize {
        self.iter()
            .map(|t| (&t.name, &t.artist.name))
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

// SQLITE EXPORT ==============================================================

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteExportable for RecentTrack {
    fn table_name() -> &'static str {
        "recent_tracks"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS recent_tracks (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT    NOT NULL,
            url        TEXT    NOT NULL,
            artist     TEXT    NOT NULL,
            artist_mbid TEXT   NOT NULL,
            album      TEXT    NOT NULL,
            album_mbid TEXT    NOT NULL,
            date_uts   INTEGER,
            loved      INTEGER NOT NULL DEFAULT 0
        )"
    }

    fn insert_sql() -> &'static str {
        "INSERT INTO recent_tracks (name, url, artist, artist_mbid, album, album_mbid, date_uts, loved)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    }

    fn bind_and_execute(&self, stmt: &mut rusqlite::Statement<'_>) -> rusqlite::Result<usize> {
        stmt.execute(rusqlite::params![
            self.name,
            self.url,
            self.artist.text,
            self.artist.mbid,
            self.album.text,
            self.album.mbid,
            self.date.as_ref().map(|d| d.uts),
            0_i32,
        ])
    }
}

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteExportable for RecentTrackExtended {
    fn table_name() -> &'static str {
        "recent_tracks_extended"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS recent_tracks_extended (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT    NOT NULL,
            url         TEXT    NOT NULL,
            mbid        TEXT    NOT NULL,
            artist      TEXT    NOT NULL,
            artist_mbid TEXT    NOT NULL,
            artist_url  TEXT    NOT NULL,
            album       TEXT    NOT NULL,
            album_mbid  TEXT    NOT NULL,
            album_url   TEXT    NOT NULL,
            date_uts    INTEGER,
            loved       INTEGER NOT NULL DEFAULT 0
        )"
    }

    fn insert_sql() -> &'static str {
        "INSERT INTO recent_tracks_extended
             (name, url, mbid, artist, artist_mbid, artist_url, album, album_mbid, album_url, date_uts, loved)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    }

    fn bind_and_execute(&self, stmt: &mut rusqlite::Statement<'_>) -> rusqlite::Result<usize> {
        stmt.execute(rusqlite::params![
            self.name,
            self.url,
            self.mbid,
            self.artist.name,
            self.artist.mbid,
            self.artist.url,
            self.album.name,
            self.album.mbid,
            self.album.url,
            self.date.as_ref().map(|d| d.uts),
            0_i32,
        ])
    }
}

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteExportable for LovedTrack {
    fn table_name() -> &'static str {
        "loved_tracks"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS loved_tracks (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT    NOT NULL,
            url         TEXT    NOT NULL,
            artist      TEXT    NOT NULL,
            artist_mbid TEXT    NOT NULL,
            date_uts    INTEGER NOT NULL
        )"
    }

    fn insert_sql() -> &'static str {
        "INSERT INTO loved_tracks (name, url, artist, artist_mbid, date_uts)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    }

    fn bind_and_execute(&self, stmt: &mut rusqlite::Statement<'_>) -> rusqlite::Result<usize> {
        stmt.execute(rusqlite::params![
            self.name,
            self.url,
            self.artist.name,
            self.artist.mbid,
            self.date.uts,
        ])
    }
}

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteExportable for TopTrack {
    fn table_name() -> &'static str {
        "top_tracks"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS top_tracks (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            name      TEXT    NOT NULL,
            url       TEXT    NOT NULL,
            artist    TEXT    NOT NULL,
            mbid      TEXT    NOT NULL,
            playcount INTEGER NOT NULL,
            rank      INTEGER NOT NULL
        )"
    }

    fn insert_sql() -> &'static str {
        "INSERT INTO top_tracks (name, url, artist, mbid, playcount, rank)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    }

    fn bind_and_execute(&self, stmt: &mut rusqlite::Statement<'_>) -> rusqlite::Result<usize> {
        let rank: u32 = self.attr.rank.parse().unwrap_or_default();
        stmt.execute(rusqlite::params![
            self.name,
            self.url,
            self.artist.name,
            self.mbid,
            self.playcount,
            rank,
        ])
    }
}

// SQLITE LOAD ================================================================

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteLoadable for RecentTrack {
    fn select_sql() -> &'static str {
        // Columns: name(0) url(1) artist(2) artist_mbid(3) album(4) album_mbid(5) date_uts(6)
        // NULL date_uts → now-playing; ORDER puts those first (NULLS FIRST in DESC).
        "SELECT name, url, artist, artist_mbid, album, album_mbid, date_uts
         FROM recent_tracks
         ORDER BY CASE WHEN date_uts IS NULL THEN 0 ELSE 1 END, date_uts DESC"
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let date_uts: Option<u32> = row.get(6)?;
        Ok(Self {
            name: row.get(0)?,
            url: row.get(1)?,
            artist: BaseMbidText {
                text: row.get(2)?,
                mbid: row.get(3)?,
            },
            album: BaseMbidText {
                text: row.get(4)?,
                mbid: row.get(5)?,
            },
            date: date_uts.map(|uts| Date {
                uts,
                text: String::new(),
            }),
            // Fields not stored in the schema — reconstructed with defaults.
            mbid: String::new(),
            streamable: false,
            image: vec![],
            attr: None,
        })
    }
}

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteLoadable for RecentTrackExtended {
    fn select_sql() -> &'static str {
        // Columns: name(0) url(1) mbid(2) artist(3) artist_mbid(4) artist_url(5)
        //          album(6) album_mbid(7) album_url(8) date_uts(9)
        "SELECT name, url, mbid, artist, artist_mbid, artist_url,
                album, album_mbid, album_url, date_uts
         FROM recent_tracks_extended
         ORDER BY CASE WHEN date_uts IS NULL THEN 0 ELSE 1 END, date_uts DESC"
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let date_uts: Option<u32> = row.get(9)?;
        Ok(Self {
            name: row.get(0)?,
            url: row.get(1)?,
            mbid: row.get(2)?,
            artist: BaseObject {
                name: row.get(3)?,
                mbid: row.get(4)?,
                url: row.get(5)?,
            },
            album: BaseObject {
                name: row.get(6)?,
                mbid: row.get(7)?,
                url: row.get(8)?,
            },
            date: date_uts.map(|uts| Date {
                uts,
                text: String::new(),
            }),
            // Fields not stored in the schema — reconstructed with defaults.
            streamable: false,
            image: vec![],
            attr: None,
        })
    }
}

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteLoadable for LovedTrack {
    fn select_sql() -> &'static str {
        // Columns: name(0) url(1) artist(2) artist_mbid(3) date_uts(4)
        "SELECT name, url, artist, artist_mbid, date_uts
         FROM loved_tracks
         ORDER BY date_uts DESC"
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            name: row.get(0)?,
            url: row.get(1)?,
            artist: BaseObject {
                name: row.get(2)?,
                mbid: row.get(3)?,
                url: String::new(),
            },
            date: Date {
                uts: row.get(4)?,
                text: String::new(),
            },
            // Fields not stored in the schema — reconstructed with defaults.
            mbid: String::new(),
            image: vec![],
            streamable: Streamable {
                fulltrack: String::new(),
                text: String::new(),
            },
        })
    }
}

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteLoadable for TopTrack {
    fn select_sql() -> &'static str {
        // Columns: name(0) url(1) artist(2) mbid(3) playcount(4) rank(5)
        "SELECT name, url, artist, mbid, playcount, rank
         FROM top_tracks
         ORDER BY rank ASC"
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            name: row.get(0)?,
            url: row.get(1)?,
            artist: BaseObject {
                name: row.get(2)?,
                mbid: String::new(),
                url: String::new(),
            },
            mbid: row.get(3)?,
            playcount: row.get(4)?,
            attr: RankAttr {
                rank: row.get::<_, u32>(5)?.to_string(),
            },
            // Fields not stored in the schema — reconstructed with defaults.
            duration: 0,
            streamable: Streamable {
                fulltrack: String::new(),
                text: String::new(),
            },
            image: vec![],
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_track(name: &str, artist: &str, album: &str, uts: Option<u32>) -> RecentTrack {
        RecentTrack {
            artist: BaseMbidText {
                mbid: String::new(),
                text: artist.to_string(),
            },
            streamable: false,
            image: vec![],
            album: BaseMbidText {
                mbid: String::new(),
                text: album.to_string(),
            },
            attr: None,
            date: uts.map(|u| Date {
                uts: u,
                text: String::new(),
            }),
            name: name.to_string(),
            mbid: String::new(),
            url: String::new(),
        }
    }

    fn make_now_playing(name: &str, artist: &str) -> RecentTrack {
        RecentTrack {
            attr: Some(Attributes {
                nowplaying: "true".to_string(),
            }),
            date: None,
            ..make_track(name, artist, "", None)
        }
    }

    #[test]
    fn test_to_set_counts_and_ranks() {
        let list = TrackList::from(vec![
            make_track("Song A", "Artist 1", "Album", Some(300)),
            make_track("Song B", "Artist 1", "Album", Some(200)),
            make_track("Song A", "Artist 1", "Album", Some(100)),
        ]);
        let set = list.to_set();
        assert_eq!(set.len(), 2);
        // Song A has 2 plays → rank 1
        let top = set.iter().find(|t| t.name == "Song A").unwrap();
        assert_eq!(top.play_count, 2);
        assert_eq!(top.rank, 1);
    }

    #[test]
    fn test_top_artists() {
        let list = TrackList::from(vec![
            make_track("T1", "Radiohead", "OK Computer", Some(100)),
            make_track("T2", "Radiohead", "OK Computer", Some(200)),
            make_track("T3", "Portishead", "Dummy", Some(300)),
        ]);
        let artists = list.top_artists();
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].name, "Radiohead");
        assert_eq!(artists[0].play_count, 2);
        assert_eq!(artists[0].rank, 1);
        assert_eq!(artists[1].play_count, 1);
        assert_eq!(artists[1].rank, 2);
    }

    #[test]
    fn test_top_albums_excludes_empty_album() {
        let list = TrackList::from(vec![
            make_track("T1", "Artist", "Dummy", Some(100)),
            make_track("T2", "Artist", "Dummy", Some(200)),
            make_track("T3", "Artist", "", Some(300)), // no album → excluded
        ]);
        let albums = list.top_albums();
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].name, "Dummy");
        assert_eq!(albums[0].play_count, 2);
    }

    #[test]
    fn test_by_hour() {
        // 3600 = 01:00 UTC, 7200 = 02:00 UTC
        let list = TrackList::from(vec![
            make_track("T1", "A", "", Some(3_600)),
            make_track("T2", "A", "", Some(7_200)),
            make_track("T3", "A", "", Some(7_300)), // also 02:xx
        ]);
        let hours = list.by_hour();
        assert_eq!(hours[1], 1);
        assert_eq!(hours[2], 2);
        assert_eq!(hours[0], 0);
    }

    #[test]
    fn test_by_date_and_streak() {
        // Day 0 = 1970-01-01, day 1 = 1970-01-02, day 3 = 1970-01-04 (gap)
        let list = TrackList::from(vec![
            make_track("T1", "A", "", Some(0)),          // 1970-01-01
            make_track("T2", "A", "", Some(86_400)),     // 1970-01-02
            make_track("T3", "A", "", Some(86_400 * 3)), // 1970-01-04 (gap)
        ]);
        let by_date = list.by_date();
        assert_eq!(by_date.len(), 3);
        // Streak: days 01 + 02 = 2; then day 04 alone = 1
        assert_eq!(list.streak(), 2);
    }

    #[test]
    fn test_without_now_playing() {
        let list = TrackList::from(vec![
            make_track("T1", "A", "", Some(100)),
            make_now_playing("Live Track", "A"),
        ]);
        let filtered = list.without_now_playing();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "T1");
    }

    #[test]
    fn test_unique_counts() {
        let list = TrackList::from(vec![
            make_track("Song", "Artist 1", "", Some(100)),
            make_track("Song", "Artist 1", "", Some(200)), // duplicate
            make_track("Song", "Artist 2", "", Some(300)), // same name, different artist
        ]);
        assert_eq!(list.unique_artist_count(), 2);
        assert_eq!(list.unique_track_count(), 2);
    }

    // ── RecentTrackExtended helpers ──────────────────────────────────────────

    fn make_ext(name: &str, artist: &str, album: &str, uts: Option<u32>) -> RecentTrackExtended {
        RecentTrackExtended {
            artist: BaseObject {
                name: artist.to_string(),
                mbid: String::new(),
                url: String::new(),
            },
            streamable: false,
            image: vec![],
            album: BaseObject {
                name: album.to_string(),
                mbid: String::new(),
                url: String::new(),
            },
            attr: None,
            date: uts.map(|u| Date {
                uts: u,
                text: String::new(),
            }),
            name: name.to_string(),
            mbid: String::new(),
            url: String::new(),
        }
    }

    fn make_ext_now_playing(name: &str, artist: &str) -> RecentTrackExtended {
        use std::collections::HashMap;
        RecentTrackExtended {
            attr: Some(HashMap::from([(
                "nowplaying".to_string(),
                "true".to_string(),
            )])),
            date: None,
            ..make_ext(name, artist, "", None)
        }
    }

    #[test]
    fn test_ext_to_set() {
        let list = TrackList::from(vec![
            make_ext("Song A", "Artist 1", "Album", Some(300)),
            make_ext("Song B", "Artist 1", "Album", Some(200)),
            make_ext("Song A", "Artist 1", "Album", Some(100)),
        ]);
        let set = list.to_set();
        assert_eq!(set.len(), 2);
        let top = set.iter().find(|t| t.name == "Song A").unwrap();
        assert_eq!(top.play_count, 2);
        assert_eq!(top.rank, 1);
        assert_eq!(top.artist, "Artist 1");
    }

    #[test]
    fn test_ext_top_artists() {
        let list = TrackList::from(vec![
            make_ext("T1", "Radiohead", "OK Computer", Some(100)),
            make_ext("T2", "Radiohead", "OK Computer", Some(200)),
            make_ext("T3", "Portishead", "Dummy", Some(300)),
        ]);
        let artists = list.top_artists();
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].name, "Radiohead");
        assert_eq!(artists[0].play_count, 2);
        assert_eq!(artists[0].rank, 1);
    }

    #[test]
    fn test_ext_top_albums_excludes_empty() {
        let list = TrackList::from(vec![
            make_ext("T1", "Artist", "Dummy", Some(100)),
            make_ext("T2", "Artist", "Dummy", Some(200)),
            make_ext("T3", "Artist", "", Some(300)),
        ]);
        let albums = list.top_albums();
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].name, "Dummy");
        assert_eq!(albums[0].play_count, 2);
    }

    #[test]
    fn test_ext_by_hour_and_streak() {
        let list = TrackList::from(vec![
            make_ext("T1", "A", "", Some(3_600)),      // 01:00 UTC
            make_ext("T2", "A", "", Some(86_400)),     // 1970-01-02
            make_ext("T3", "A", "", Some(86_400 * 3)), // 1970-01-04 (gap)
        ]);
        let hours = list.by_hour();
        assert_eq!(hours[1], 1); // one play at 01:xx
        assert_eq!(list.streak(), 2); // days 01+02, then gap
    }

    #[test]
    fn test_ext_without_now_playing() {
        let list = TrackList::from(vec![
            make_ext("T1", "A", "", Some(100)),
            make_ext_now_playing("Live", "A"),
        ]);
        let filtered = list.without_now_playing();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "T1");
    }

    #[test]
    fn test_ext_unique_counts() {
        let list = TrackList::from(vec![
            make_ext("Song", "Artist 1", "", Some(100)),
            make_ext("Song", "Artist 1", "", Some(200)),
            make_ext("Song", "Artist 2", "", Some(300)),
        ]);
        assert_eq!(list.unique_artist_count(), 2);
        assert_eq!(list.unique_track_count(), 2);
    }

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
