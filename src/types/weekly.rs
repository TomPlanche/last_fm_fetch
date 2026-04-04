use serde::Deserialize;

use crate::types::utils::u32_from_str;

// ── WeeklyChartList ───────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct WeeklyChartListResponse {
    weeklychartlist: WeeklyChartListRaw,
}

#[derive(Deserialize, Debug)]
struct WeeklyChartListRaw {
    chart: Vec<WeeklyChartRangeRaw>,
}

#[derive(Deserialize, Debug)]
struct WeeklyChartRangeRaw {
    #[serde(deserialize_with = "u32_from_str")]
    from: u32,
    #[serde(deserialize_with = "u32_from_str")]
    to: u32,
}

/// A time range representing a weekly chart period.
///
/// Returned as part of [`user.getWeeklyChartList`](https://www.last.fm/api/show/user.getWeeklyChartList).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WeeklyChartRange {
    /// Start of the week as a Unix timestamp
    pub from: u32,
    /// End of the week as a Unix timestamp
    pub to: u32,
}

impl From<WeeklyChartRangeRaw> for WeeklyChartRange {
    fn from(r: WeeklyChartRangeRaw) -> Self {
        Self {
            from: r.from,
            to: r.to,
        }
    }
}

impl From<WeeklyChartListResponse> for Vec<WeeklyChartRange> {
    fn from(r: WeeklyChartListResponse) -> Self {
        r.weeklychartlist
            .chart
            .into_iter()
            .map(WeeklyChartRange::from)
            .collect()
    }
}

// ── WeeklyTrackChart ──────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct WeeklyTrackChartResponse {
    weeklytrackchart: WeeklyTrackChartRaw,
}

#[derive(Deserialize, Debug)]
struct WeeklyTrackChartRaw {
    track: Vec<WeeklyTrackRaw>,
}

#[derive(Deserialize, Debug)]
struct WeeklyTrackRaw {
    name: String,
    mbid: String,
    url: String,
    #[serde(deserialize_with = "u32_from_str")]
    playcount: u32,
    artist: WeeklyArtistRef,
    #[serde(rename = "@attr")]
    attr: WeeklyRankAttr,
}

#[derive(Deserialize, Debug)]
struct WeeklyArtistRef {
    #[serde(rename = "#text")]
    name: String,
    mbid: String,
}

#[derive(Deserialize, Debug)]
struct WeeklyRankAttr {
    #[serde(deserialize_with = "u32_from_str")]
    rank: u32,
}

/// A track entry in a weekly track chart.
///
/// Returned by [`user.getWeeklyTrackChart`](https://www.last.fm/api/show/user.getWeeklyTrackChart).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WeeklyTrack {
    /// Track title
    pub name: String,
    /// `MusicBrainz` track identifier (may be empty)
    pub mbid: String,
    /// Last.fm URL for this track
    pub url: String,
    /// Number of plays in this week
    pub playcount: u32,
    /// Artist name
    pub artist_name: String,
    /// `MusicBrainz` artist identifier (may be empty)
    pub artist_mbid: String,
    /// Position in the chart (1-indexed)
    pub rank: u32,
}

impl From<WeeklyTrackRaw> for WeeklyTrack {
    fn from(r: WeeklyTrackRaw) -> Self {
        Self {
            name: r.name,
            mbid: r.mbid,
            url: r.url,
            playcount: r.playcount,
            artist_name: r.artist.name,
            artist_mbid: r.artist.mbid,
            rank: r.attr.rank,
        }
    }
}

impl From<WeeklyTrackChartResponse> for Vec<WeeklyTrack> {
    fn from(r: WeeklyTrackChartResponse) -> Self {
        r.weeklytrackchart
            .track
            .into_iter()
            .map(WeeklyTrack::from)
            .collect()
    }
}

// ── WeeklyArtistChart ─────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct WeeklyArtistChartResponse {
    weeklyartistchart: WeeklyArtistChartRaw,
}

#[derive(Deserialize, Debug)]
struct WeeklyArtistChartRaw {
    artist: Vec<WeeklyArtistRaw>,
}

#[derive(Deserialize, Debug)]
struct WeeklyArtistRaw {
    name: String,
    mbid: String,
    url: String,
    #[serde(deserialize_with = "u32_from_str")]
    playcount: u32,
    #[serde(rename = "@attr")]
    attr: WeeklyRankAttr,
}

/// An artist entry in a weekly artist chart.
///
/// Returned by [`user.getWeeklyArtistChart`](https://www.last.fm/api/show/user.getWeeklyArtistChart).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WeeklyArtist {
    /// Artist name
    pub name: String,
    /// `MusicBrainz` artist identifier (may be empty)
    pub mbid: String,
    /// Last.fm URL for this artist
    pub url: String,
    /// Number of plays in this week
    pub playcount: u32,
    /// Position in the chart (1-indexed)
    pub rank: u32,
}

impl From<WeeklyArtistRaw> for WeeklyArtist {
    fn from(r: WeeklyArtistRaw) -> Self {
        Self {
            name: r.name,
            mbid: r.mbid,
            url: r.url,
            playcount: r.playcount,
            rank: r.attr.rank,
        }
    }
}

impl From<WeeklyArtistChartResponse> for Vec<WeeklyArtist> {
    fn from(r: WeeklyArtistChartResponse) -> Self {
        r.weeklyartistchart
            .artist
            .into_iter()
            .map(WeeklyArtist::from)
            .collect()
    }
}

// ── WeeklyAlbumChart ──────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct WeeklyAlbumChartResponse {
    weeklyalbumchart: WeeklyAlbumChartRaw,
}

#[derive(Deserialize, Debug)]
struct WeeklyAlbumChartRaw {
    album: Vec<WeeklyAlbumRaw>,
}

#[derive(Deserialize, Debug)]
struct WeeklyAlbumRaw {
    name: String,
    mbid: String,
    url: String,
    #[serde(deserialize_with = "u32_from_str")]
    playcount: u32,
    artist: WeeklyArtistRef,
    #[serde(rename = "@attr")]
    attr: WeeklyRankAttr,
}

/// An album entry in a weekly album chart.
///
/// Returned by [`user.getWeeklyAlbumChart`](https://www.last.fm/api/show/user.getWeeklyAlbumChart).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WeeklyAlbum {
    /// Album title
    pub name: String,
    /// `MusicBrainz` album identifier (may be empty)
    pub mbid: String,
    /// Last.fm URL for this album
    pub url: String,
    /// Number of plays in this week
    pub playcount: u32,
    /// Artist name
    pub artist_name: String,
    /// `MusicBrainz` artist identifier (may be empty)
    pub artist_mbid: String,
    /// Position in the chart (1-indexed)
    pub rank: u32,
}

impl From<WeeklyAlbumRaw> for WeeklyAlbum {
    fn from(r: WeeklyAlbumRaw) -> Self {
        Self {
            name: r.name,
            mbid: r.mbid,
            url: r.url,
            playcount: r.playcount,
            artist_name: r.artist.name,
            artist_mbid: r.artist.mbid,
            rank: r.attr.rank,
        }
    }
}

impl From<WeeklyAlbumChartResponse> for Vec<WeeklyAlbum> {
    fn from(r: WeeklyAlbumChartResponse) -> Self {
        r.weeklyalbumchart
            .album
            .into_iter()
            .map(WeeklyAlbum::from)
            .collect()
    }
}
