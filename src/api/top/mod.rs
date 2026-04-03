//! Top-charts resource clients (top tracks, top artists, top albums).

mod albums;
mod artists;
mod tracks;

pub use albums::{TopAlbumsClient, TopAlbumsRequestBuilder};
pub use artists::{TopArtistsClient, TopArtistsRequestBuilder};
pub use tracks::{TopTracksClient, TopTracksRequestBuilder};
