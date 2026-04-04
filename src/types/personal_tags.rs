use serde::Deserialize;

use crate::types::TrackImage;
use crate::types::utils::u32_from_str;

// ── Shared attr ───────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct PersonalTagsAttr {
    #[serde(deserialize_with = "u32_from_str")]
    pub(crate) total: u32,
    #[serde(deserialize_with = "u32_from_str")]
    pub(crate) page: u32,
    #[serde(rename = "totalPages", deserialize_with = "u32_from_str")]
    pub(crate) total_pages: u32,
    #[serde(rename = "perPage", deserialize_with = "u32_from_str")]
    pub(crate) per_page: u32,
}

// ── PersonalTaggedTracks ──────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct PersonalTaggedTracksResponse {
    taggings: PersonalTaggedTracksRaw,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedTracksRaw {
    tracks: PersonalTaggedTracksInner,
    #[serde(rename = "@attr")]
    pub(crate) attr: PersonalTagsAttr,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedTracksInner {
    track: Vec<PersonalTaggedTrackRaw>,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedTrackRaw {
    name: String,
    mbid: String,
    url: String,
    artist: PersonalTaggedArtistRef,
    image: Vec<TrackImage>,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedArtistRef {
    name: String,
    mbid: String,
    url: String,
}

/// A track tagged with a personal tag.
///
/// Returned by [`user.getPersonalTags`](https://www.last.fm/api/show/user.getPersonalTags)
/// when `taggingtype=track`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PersonalTaggedTrack {
    /// Track name
    pub name: String,
    /// `MusicBrainz` track identifier (may be empty)
    pub mbid: String,
    /// Last.fm URL for this track
    pub url: String,
    /// Artist name
    pub artist_name: String,
    /// `MusicBrainz` artist identifier (may be empty)
    pub artist_mbid: String,
    /// Last.fm artist URL
    pub artist_url: String,
    /// Track images in various sizes
    pub images: Vec<TrackImage>,
}

impl From<PersonalTaggedTrackRaw> for PersonalTaggedTrack {
    fn from(r: PersonalTaggedTrackRaw) -> Self {
        Self {
            name: r.name,
            mbid: r.mbid,
            url: r.url,
            artist_name: r.artist.name,
            artist_mbid: r.artist.mbid,
            artist_url: r.artist.url,
            images: r.image,
        }
    }
}

/// Paginated result from `user.getPersonalTags` with `taggingtype=track`
#[derive(Debug)]
#[non_exhaustive]
pub struct PersonalTaggedTracksPage {
    /// Tracks tagged with this tag
    pub tracks: Vec<PersonalTaggedTrack>,
    /// Total number of tagged tracks
    pub total: u32,
    /// Current page (1-indexed)
    pub page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Results per page
    pub per_page: u32,
}

impl From<PersonalTaggedTracksResponse> for PersonalTaggedTracksPage {
    fn from(r: PersonalTaggedTracksResponse) -> Self {
        Self {
            total: r.taggings.attr.total,
            page: r.taggings.attr.page,
            total_pages: r.taggings.attr.total_pages,
            per_page: r.taggings.attr.per_page,
            tracks: r
                .taggings
                .tracks
                .track
                .into_iter()
                .map(PersonalTaggedTrack::from)
                .collect(),
        }
    }
}

// ── PersonalTaggedArtists ─────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct PersonalTaggedArtistsResponse {
    taggings: PersonalTaggedArtistsRaw,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedArtistsRaw {
    artists: PersonalTaggedArtistsInner,
    #[serde(rename = "@attr")]
    pub(crate) attr: PersonalTagsAttr,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedArtistsInner {
    artist: Vec<PersonalTaggedArtistRaw>,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedArtistRaw {
    name: String,
    mbid: String,
    url: String,
    image: Vec<TrackImage>,
}

/// An artist tagged with a personal tag.
///
/// Returned by [`user.getPersonalTags`](https://www.last.fm/api/show/user.getPersonalTags)
/// when `taggingtype=artist`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PersonalTaggedArtist {
    /// Artist name
    pub name: String,
    /// `MusicBrainz` artist identifier (may be empty)
    pub mbid: String,
    /// Last.fm URL for this artist
    pub url: String,
    /// Artist images in various sizes
    pub images: Vec<TrackImage>,
}

impl From<PersonalTaggedArtistRaw> for PersonalTaggedArtist {
    fn from(r: PersonalTaggedArtistRaw) -> Self {
        Self {
            name: r.name,
            mbid: r.mbid,
            url: r.url,
            images: r.image,
        }
    }
}

/// Paginated result from `user.getPersonalTags` with `taggingtype=artist`
#[derive(Debug)]
#[non_exhaustive]
pub struct PersonalTaggedArtistsPage {
    /// Artists tagged with this tag
    pub artists: Vec<PersonalTaggedArtist>,
    /// Total number of tagged artists
    pub total: u32,
    /// Current page (1-indexed)
    pub page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Results per page
    pub per_page: u32,
}

impl From<PersonalTaggedArtistsResponse> for PersonalTaggedArtistsPage {
    fn from(r: PersonalTaggedArtistsResponse) -> Self {
        Self {
            total: r.taggings.attr.total,
            page: r.taggings.attr.page,
            total_pages: r.taggings.attr.total_pages,
            per_page: r.taggings.attr.per_page,
            artists: r
                .taggings
                .artists
                .artist
                .into_iter()
                .map(PersonalTaggedArtist::from)
                .collect(),
        }
    }
}

// ── PersonalTaggedAlbums ──────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct PersonalTaggedAlbumsResponse {
    taggings: PersonalTaggedAlbumsRaw,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedAlbumsRaw {
    albums: PersonalTaggedAlbumsInner,
    #[serde(rename = "@attr")]
    pub(crate) attr: PersonalTagsAttr,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedAlbumsInner {
    album: Vec<PersonalTaggedAlbumRaw>,
}

#[derive(Deserialize, Debug)]
struct PersonalTaggedAlbumRaw {
    name: String,
    mbid: String,
    url: String,
    artist: PersonalTaggedArtistRef,
    image: Vec<TrackImage>,
}

/// An album tagged with a personal tag.
///
/// Returned by [`user.getPersonalTags`](https://www.last.fm/api/show/user.getPersonalTags)
/// when `taggingtype=album`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PersonalTaggedAlbum {
    /// Album name
    pub name: String,
    /// `MusicBrainz` album identifier (may be empty)
    pub mbid: String,
    /// Last.fm URL for this album
    pub url: String,
    /// Artist name
    pub artist_name: String,
    /// `MusicBrainz` artist identifier (may be empty)
    pub artist_mbid: String,
    /// Last.fm artist URL
    pub artist_url: String,
    /// Album images in various sizes
    pub images: Vec<TrackImage>,
}

impl From<PersonalTaggedAlbumRaw> for PersonalTaggedAlbum {
    fn from(r: PersonalTaggedAlbumRaw) -> Self {
        Self {
            name: r.name,
            mbid: r.mbid,
            url: r.url,
            artist_name: r.artist.name,
            artist_mbid: r.artist.mbid,
            artist_url: r.artist.url,
            images: r.image,
        }
    }
}

/// Paginated result from `user.getPersonalTags` with `taggingtype=album`
#[derive(Debug)]
#[non_exhaustive]
pub struct PersonalTaggedAlbumsPage {
    /// Albums tagged with this tag
    pub albums: Vec<PersonalTaggedAlbum>,
    /// Total number of tagged albums
    pub total: u32,
    /// Current page (1-indexed)
    pub page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Results per page
    pub per_page: u32,
}

impl From<PersonalTaggedAlbumsResponse> for PersonalTaggedAlbumsPage {
    fn from(r: PersonalTaggedAlbumsResponse) -> Self {
        Self {
            total: r.taggings.attr.total,
            page: r.taggings.attr.page,
            total_pages: r.taggings.attr.total_pages,
            per_page: r.taggings.attr.per_page,
            albums: r
                .taggings
                .albums
                .album
                .into_iter()
                .map(PersonalTaggedAlbum::from)
                .collect(),
        }
    }
}
