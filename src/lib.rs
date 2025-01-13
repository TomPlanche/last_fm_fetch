use serde::Serialize;

#[path = "analytics.rs"]
pub mod analytics;

#[path = "file_handler.rs"]
pub mod file_handler;

#[path = "lastfm_handler.rs"]
pub mod lastfm_handler;

#[path = "types.rs"]
pub mod types;

#[path = "url_builder.rs"]
pub mod url_builder;

#[derive(Serialize)]
pub enum LastFmError {
    NoTracks,
}
