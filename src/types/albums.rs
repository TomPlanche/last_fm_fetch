use std::cmp::Ordering;
use std::fmt;

use crate::types::{BaseObject, BaseResponse, RankAttr, TrackImage};
use serde::{Deserialize, Serialize};

use crate::types::utils::{empty_string_as_none, u32_from_str, vec_or_single};

/// An album from a user's top albums, ranked by play count
///
/// Retrieved from the `user.gettopalbums` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopAlbum {
    /// Album name (`None` when blank)
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub name: Option<String>,
    /// Artist information
    pub artist: BaseObject,
    /// `MusicBrainz` album identifier (`None` when the API returns no identifier)
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub mbid: Option<String>,
    /// Last.fm URL for this album (`None` when blank)
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub url: Option<String>,
    /// Total number of times this album has been played
    #[serde(deserialize_with = "u32_from_str")]
    pub playcount: u32,
    /// Album images in various sizes
    pub image: Vec<TrackImage>,
    /// Rank attributes (position in top albums)
    #[serde(rename = "@attr")]
    pub attr: RankAttr,
}

impl fmt::Display for TopAlbum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} - {} by {} ({} plays)",
            self.attr.rank,
            self.name.as_deref().unwrap_or(""),
            self.artist.name.as_deref().unwrap_or(""),
            self.playcount
        )
    }
}

impl PartialEq for TopAlbum {
    fn eq(&self, other: &Self) -> bool {
        self.playcount == other.playcount
    }
}

impl Eq for TopAlbum {}

impl PartialOrd for TopAlbum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopAlbum {
    fn cmp(&self, other: &Self) -> Ordering {
        self.playcount.cmp(&other.playcount)
    }
}

// SQLITE EXPORT ==============================================================

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteExportable for TopAlbum {
    fn table_name() -> &'static str {
        "top_albums"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS top_albums (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            name      TEXT,
            mbid      TEXT,
            url       TEXT,
            artist    TEXT,
            playcount INTEGER NOT NULL,
            rank      INTEGER NOT NULL
        )"
    }

    fn insert_sql() -> &'static str {
        "INSERT INTO top_albums (name, mbid, url, artist, playcount, rank)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    }

    fn bind_and_execute(&self, stmt: &mut rusqlite::Statement<'_>) -> rusqlite::Result<usize> {
        let rank: u32 = self.attr.rank.parse().unwrap_or_default();
        stmt.execute(rusqlite::params![
            self.name,
            self.mbid,
            self.url,
            self.artist.name,
            self.playcount,
            rank,
        ])
    }
}

// SQLITE LOAD ================================================================

#[cfg(feature = "sqlite")]
impl crate::sqlite::SqliteLoadable for TopAlbum {
    fn select_sql() -> &'static str {
        // Columns: name(0) mbid(1) url(2) artist(3) playcount(4) rank(5)
        "SELECT name, mbid, url, artist, playcount, rank
         FROM top_albums
         ORDER BY rank ASC"
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            name: row.get(0)?,
            mbid: row.get(1)?,
            url: row.get(2)?,
            artist: crate::types::BaseObject {
                name: row.get(3)?,
                mbid: None,
                url: None,
            },
            playcount: row.get(4)?,
            attr: crate::types::RankAttr {
                rank: row.get::<_, u32>(5)?.to_string(),
            },
            // Fields not stored in the schema — reconstructed with defaults.
            image: vec![],
        })
    }
}

/// Top albums response wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopAlbums {
    /// List of top albums
    #[serde(deserialize_with = "vec_or_single")]
    pub album: Vec<TopAlbum>,
    /// Response metadata
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

/// Top-level top albums API response
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserTopAlbums {
    /// Top albums data
    pub topalbums: TopAlbums,
}
