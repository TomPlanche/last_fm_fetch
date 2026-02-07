use std::fmt;

use crate::types::{BaseResponse, RankAttr, TrackImage};
use serde::{Deserialize, Serialize};

use crate::types::utils::{bool_from_str, u32_from_str};

/// An artist from a user's top artists, ranked by play count
///
/// Retrieved from the `user.gettopartists` API endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopArtists {
    pub artist: Vec<TopArtist>,
    #[serde(rename = "@attr")]
    pub attr: BaseResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTopArtists {
    pub topartists: TopArtists,
}
