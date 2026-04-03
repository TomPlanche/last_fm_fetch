use std::cmp::Ordering;
use std::fmt;

use crate::types::{BaseResponse, RankAttr, TrackImage};
use serde::{Deserialize, Serialize};

use crate::types::utils::{bool_from_str, u32_from_str};

/// An artist from a user's top artists, ranked by play count
///
/// Retrieved from the `user.gettopartists` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopArtist {
    /// Artist name
    pub name: String,
    /// `MusicBrainz` artist identifier (may be empty string)
    pub mbid: String,
    /// Last.fm URL for this artist
    pub url: String,
    /// Total number of times this artist has been played
    #[serde(deserialize_with = "u32_from_str")]
    pub playcount: u32,
    /// Whether the artist is streamable on Last.fm
    #[serde(deserialize_with = "bool_from_str")]
    pub streamable: bool,
    /// Artist images in various sizes
    pub image: Vec<TrackImage>,
    /// Rank attributes (position in top artists)
    #[serde(rename = "@attr")]
    pub attr: RankAttr,
}

impl fmt::Display for TopArtist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} - {} ({} plays)",
            self.attr.rank, self.name, self.playcount
        )
    }
}

impl PartialEq for TopArtist {
    fn eq(&self, other: &Self) -> bool {
        self.playcount == other.playcount
    }
}

impl Eq for TopArtist {}

impl PartialOrd for TopArtist {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopArtist {
    fn cmp(&self, other: &Self) -> Ordering {
        self.playcount.cmp(&other.playcount)
    }
}

// SQLITE EXPORT ==============================================================

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteExportable for TopArtist {
    fn table_name() -> &'static str {
        "top_artists"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS top_artists (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            name      TEXT    NOT NULL,
            mbid      TEXT    NOT NULL,
            url       TEXT    NOT NULL,
            playcount INTEGER NOT NULL,
            rank      INTEGER NOT NULL
        )"
    }

    fn insert_sql() -> &'static str {
        "INSERT INTO top_artists (name, mbid, url, playcount, rank)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    }

    fn bind_and_execute(&self, stmt: &mut rusqlite::Statement<'_>) -> rusqlite::Result<usize> {
        let rank: u32 = self.attr.rank.parse().unwrap_or_default();
        stmt.execute(rusqlite::params![
            self.name,
            self.mbid,
            self.url,
            self.playcount,
            rank,
        ])
    }
}

// SQLITE LOAD ================================================================

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteLoadable for TopArtist {
    fn select_sql() -> &'static str {
        // Columns: name(0) mbid(1) url(2) playcount(3) rank(4)
        "SELECT name, mbid, url, playcount, rank
         FROM top_artists
         ORDER BY rank ASC"
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            name: row.get(0)?,
            mbid: row.get(1)?,
            url: row.get(2)?,
            playcount: row.get(3)?,
            attr: crate::types::RankAttr {
                rank: row.get::<_, u32>(4)?.to_string(),
            },
            // Fields not stored in the schema — reconstructed with defaults.
            streamable: false,
            image: vec![],
        })
    }
}

/// Top artists response wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopArtists {
    /// List of top artists
    pub artist: Vec<TopArtist>,
    /// Response metadata
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

/// Top-level top artists API response
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserTopArtists {
    /// Top artists data
    pub topartists: TopArtists,
}
