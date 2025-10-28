/// Base URL for the Last.fm API
pub const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

/// Maximum number of tracks that can be fetched in a single API request
pub const API_MAX_LIMIT: u32 = 1000;

/// Multiplier for chunking large requests
pub const CHUNK_MULTIPLIER: u32 = 5;

/// Size of each chunk for pagination
pub const CHUNK_SIZE: u32 = API_MAX_LIMIT * CHUNK_MULTIPLIER;
