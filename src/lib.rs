// Modular structure
pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod types;

// Utility modules
#[path = "analytics.rs"]
pub mod analytics;

#[path = "file_handler.rs"]
pub mod file_handler;

#[path = "lastfm_handler.rs"]
pub mod lastfm_handler;

#[path = "url_builder.rs"]
pub mod url_builder;

// Public API re-exports
pub use client::LastFmClient;
pub use config::{Config, ConfigBuilder, RateLimit};
pub use error::{LastFmError, Result};
pub use types::{Period, TrackLimit};

// Re-export API clients
pub use api::{
    LovedTracksClient, LovedTracksRequestBuilder, RecentTracksClient, RecentTracksRequestBuilder,
    TopTracksClient, TopTracksRequestBuilder,
};

// Re-export commonly used types
pub use types::{
    LovedTrack, RecentTrack, RecentTrackExtended, TopTrack, UserLovedTracks, UserRecentTracks,
    UserRecentTracksExtended, UserTopAlbums, UserTopArtists, UserTopTracks,
};
