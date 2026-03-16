use crate::analytics::AnalysisHandler;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{LovedTrack, TrackLimit, UserLovedTracks};

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

use super::fetch_utils::{ResourceContainer, fetch};

/// Client for fetching loved tracks
pub struct LovedTracksClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl fmt::Debug for LovedTracksClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LovedTracksClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl LovedTracksClient {
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for loved tracks requests
    pub fn builder(&self, username: impl Into<String>) -> LovedTracksRequestBuilder {
        LovedTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }
}

/// Builder for loved tracks requests
pub struct LovedTracksRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
}

impl fmt::Debug for LovedTracksRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LovedTracksRequestBuilder")
            .field("username", &self.username)
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}

impl LovedTracksRequestBuilder {
    fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
        }
    }

    /// Set the maximum number of tracks to fetch
    ///
    /// # Arguments
    /// * `limit` - Maximum number of tracks to fetch. The Last.fm API supports fetching up to thousands of tracks.
    ///   If you need all tracks, use `unlimited()` instead.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Fetch all available tracks (no limit)
    #[must_use]
    pub const fn unlimited(mut self) -> Self {
        self.limit = None;
        self
    }

    /// Fetch the tracks
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<LovedTrack>> {
        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_tracks::<UserLovedTracks>(limit).await
    }

    /// Fetch tracks and save them to a file
    ///
    /// # Arguments
    /// * `format` - The file format to save the tracks in
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, response cannot be parsed, or file cannot be saved.
    ///
    /// # Returns
    /// * `Result<String>` - The filename of the saved file
    pub async fn fetch_and_save(self, format: FileFormat, filename_prefix: &str) -> Result<String> {
        let tracks = self.fetch().await?;
        tracing::info!("Saving {} loved tracks to file", tracks.len());
        let filename = FileHandler::save(&tracks, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;
        Ok(filename)
    }

    /// Analyze tracks and return statistics
    ///
    /// # Arguments
    /// * `threshold` - Minimum play count threshold. Tracks with fewer plays than this value will be
    ///   counted separately in `tracks_below_threshold`. For example, use 5 to identify
    ///   tracks played less than 5 times.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    ///
    /// # Returns
    /// * `Result<crate::analytics::TrackStats>` - Analysis results including play counts, most played tracks, etc.
    pub async fn analyze(self, threshold: usize) -> Result<crate::analytics::TrackStats> {
        let tracks = self.fetch().await?;
        Ok(AnalysisHandler::analyze_tracks(&tracks, threshold))
    }

    /// Analyze tracks and print statistics
    ///
    /// # Arguments
    /// * `threshold` - Minimum play count threshold. Tracks with fewer plays than this value will be
    ///   counted separately. For example, use 5 to identify tracks played less than 5 times.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn analyze_and_print(self, threshold: usize) -> Result<()> {
        let stats = self.analyze(threshold).await?;
        AnalysisHandler::print_analysis(&stats);
        Ok(())
    }

    async fn fetch_tracks<T>(&self, limit: TrackLimit) -> Result<Vec<LovedTrack>>
    where
        T: DeserializeOwned + ResourceContainer<ItemType = LovedTrack>,
    {
        use crate::url_builder::QueryParams;

        fetch::<LovedTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.getlovedtracks",
            limit,
            QueryParams::new(),
            None,
        )
        .await
    }
}

impl ResourceContainer for UserLovedTracks {
    type ItemType = LovedTrack;

    fn total(&self) -> u32 {
        self.lovedtracks.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.lovedtracks.track
    }
}
