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

// Re-export API clients
pub use api::{
    LovedTracksClient, LovedTracksRequestBuilder, ProgressCallback, RecentTracksClient,
    RecentTracksRequestBuilder, TopTracksClient, TopTracksRequestBuilder,
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
