//! Async Rust client for the Last.fm API.
//!
//! Provides typed access to user listening data including recent tracks,
//! loved tracks, top tracks, top artists, and top albums.

/// API client modules for each Last.fm resource type
pub mod api;
/// HTTP client infrastructure (trait, retry, rate limiting)
pub mod client;
/// Configuration types and builder
pub mod config;
/// Error types and result alias
pub mod error;
/// Data types for Last.fm API responses
pub mod types;

/// Track analysis and statistics
#[path = "analytics.rs"]
pub mod analytics;

/// Flat CSV layouts for nested Last.fm types (internal)
mod csv_export;

/// File I/O for saving and loading track data
#[path = "file_handler.rs"]
pub mod file_handler;

/// URL construction utilities
#[path = "url_builder.rs"]
pub mod url_builder;

/// SQLite export support (requires `sqlite` feature)
#[cfg(feature = "sqlite")]
#[path = "sqlite.rs"]
pub mod sqlite;

// Public API re-exports
pub use client::LastFmClient;
pub use config::{Config, ConfigBuilder, RateLimit};
pub use error::{LastFmError, Result};
pub use types::{Period, TrackLimit, TrackList};

// Re-export extension traits
pub use api::{Analyze, FetchAndSave, FetchAndUpdate, LimitBuilder};

/// Convenience re-exports for the most common traits and types.
///
/// Import with `use lastfm_client::prelude::*;` to bring all extension traits
/// into scope without listing them individually.
///
/// # Example
/// ```no_run
/// use lastfm_client::{LastFmClient, prelude::*};
///
/// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
/// let tracks = client.recent_tracks("username").limit(100).fetch().await?;
/// # Ok(())
/// # }
/// ```
pub mod prelude {
    pub use crate::api::{Analyze, FetchAndSave, FetchAndUpdate, LimitBuilder};
}

// Re-export API clients
pub use api::{
    FriendsRequestBuilder, LovedTracksRequestBuilder, PersonalTagsRequestBuilder, ProgressCallback,
    RecentTracksRequestBuilder, TopAlbumsRequestBuilder, TopArtistsRequestBuilder,
    TopTagsRequestBuilder, TopTracksRequestBuilder, UserInfoRequestBuilder,
    WeeklyAlbumChartRequestBuilder, WeeklyArtistChartRequestBuilder, WeeklyChartListRequestBuilder,
    WeeklyTrackChartRequestBuilder,
};
pub use types::{
    FriendProfile, FriendsPage, PersonalTaggedAlbum, PersonalTaggedAlbumsPage,
    PersonalTaggedArtist, PersonalTaggedArtistsPage, PersonalTaggedTrack, PersonalTaggedTracksPage,
    UserInfo, UserTopTag, WeeklyAlbum, WeeklyArtist, WeeklyChartRange, WeeklyTrack,
};

// Re-export SQLite traits when feature is enabled
#[cfg(feature = "sqlite")]
pub use sqlite::{SqliteExportable, SqliteLoadable};

// Re-export commonly used types
pub use types::{
    LovedTrack, RecentTrack, RecentTrackExtended, ScoredAlbum, ScoredArtist, ScoredTrack, TopTrack,
    UserLovedTracks, UserRecentTracks, UserRecentTracksExtended, UserTopAlbums, UserTopArtists,
    UserTopTracks,
};
