/// Extension traits shared across all API request builders
pub mod builder_ext;
/// API constants (base URL, limits, chunk sizes)
pub mod constants;
mod fetch_utils;
mod loved_tracks;
#[cfg(feature = "progress")]
mod progress;
mod recent_tracks;
mod top;

pub use builder_ext::{Analyze, FetchAndSave, FetchAndUpdate, LimitBuilder};
pub use fetch_utils::{Period, ProgressCallback, ResourceContainer};
pub use loved_tracks::{LovedTracksClient, LovedTracksRequestBuilder};
pub use recent_tracks::{RecentTracksClient, RecentTracksRequestBuilder};
pub use top::{TopAlbumsClient, TopAlbumsRequestBuilder};
pub use top::{TopArtistsClient, TopArtistsRequestBuilder};
pub use top::{TopTracksClient, TopTracksRequestBuilder};
