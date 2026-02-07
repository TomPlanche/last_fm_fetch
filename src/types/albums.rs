use std::fmt;

use crate::types::{BaseObject, BaseResponse, RankAttr, TrackImage};
use serde::{Deserialize, Serialize};

use crate::types::utils::u32_from_str;

/// An album from a user's top albums, ranked by play count
///
/// Retrieved from the `user.gettopalbums` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopAlbums {
    pub album: Vec<TopAlbum>,
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTopAlbums {
    pub topalbums: TopAlbums,
}
