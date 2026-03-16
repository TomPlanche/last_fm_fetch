use std::fmt;

use crate::types::{BaseObject, BaseResponse, RankAttr, TrackImage};
use serde::{Deserialize, Serialize};

use crate::types::utils::u32_from_str;

/// An album from a user's top albums, ranked by play count
///
/// Retrieved from the `user.gettopalbums` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopAlbum {
    /// Album name
    pub name: String,
    /// Artist information
    pub artist: BaseObject,
    /// `MusicBrainz` album identifier (may be empty string)
    pub mbid: String,
    /// Last.fm URL for this album
    pub url: String,
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
            self.attr.rank, self.name, self.artist.name, self.playcount
        )
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
            name      TEXT    NOT NULL,
            mbid      TEXT    NOT NULL,
            url       TEXT    NOT NULL,
            artist    TEXT    NOT NULL,
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

/// Top albums response wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopAlbums {
    /// List of top albums
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
