use crate::analytics::AnalysisHandler;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{
    RecentTrack, RecentTrackExtended, TrackLimit, UserRecentTracks, UserRecentTracksExtended,
};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

use super::fetch_utils::{ProgressCallback, ResourceContainer, fetch};

/// Client for fetching recent tracks
pub struct RecentTracksClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl fmt::Debug for RecentTracksClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecentTracksClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl RecentTracksClient {
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for recent tracks requests
    pub fn builder(&self, username: impl Into<String>) -> RecentTracksRequestBuilder {
        RecentTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }
}

/// Builder for recent tracks requests
pub struct RecentTracksRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
    from: Option<i64>,
    to: Option<i64>,
    extended: bool,
    progress_callback: Option<ProgressCallback>,
}

impl fmt::Debug for RecentTracksRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecentTracksRequestBuilder")
            .field("username", &self.username)
            .field("limit", &self.limit)
            .field("from", &self.from)
            .field("to", &self.to)
            .field("extended", &self.extended)
            .finish_non_exhaustive()
    }
}

impl RecentTracksRequestBuilder {
    fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
            from: None,
            to: None,
            extended: false,
            progress_callback: None,
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

    /// Fetch tracks from this timestamp onwards
    ///
    /// # Arguments
    /// * `timestamp` - Unix timestamp in seconds (not milliseconds) since January 1, 1970 UTC
    ///
    /// # Example
    /// ```ignore
    /// // Fetch tracks since January 1, 2024 00:00:00 UTC
    /// let tracks = client.recent_tracks()
    ///     .builder("username")
    ///     .since(1704067200)
    ///     .fetch()
    ///     .await?;
    /// ```
    #[must_use]
    pub const fn since(mut self, timestamp: i64) -> Self {
        self.from = Some(timestamp);
        self
    }

    /// Fetch tracks between two timestamps
    ///
    /// # Arguments
    /// * `from` - Start Unix timestamp in seconds (not milliseconds) since January 1, 1970 UTC
    /// * `to` - End Unix timestamp in seconds (not milliseconds) since January 1, 1970 UTC
    ///
    /// # Example
    /// ```ignore
    /// // Fetch tracks between January 1, 2024 and February 1, 2024 (UTC)
    /// let tracks = client.recent_tracks()
    ///     .builder("username")
    ///     .between(1704067200, 1706745600)
    ///     .fetch()
    ///     .await?;
    /// ```
    #[must_use]
    pub const fn between(mut self, from: i64, to: i64) -> Self {
        self.from = Some(from);
        self.to = Some(to);
        self
    }

    /// Fetch extended track information
    #[must_use]
    pub const fn extended(mut self, extended: bool) -> Self {
        self.extended = extended;
        self
    }

    /// Register a progress callback invoked with `(fetched, total)` after each batch.
    #[must_use]
    pub fn on_progress(mut self, callback: impl Fn(u32, u32) + Send + Sync + 'static) -> Self {
        self.progress_callback = Some(Arc::new(callback));
        self
    }

    /// Fetch the tracks
    ///
    /// # Errors
    /// Returns an error if:
    /// - The HTTP request fails or the response cannot be parsed
    /// - The date range is invalid (to <= from when both timestamps are set)
    pub async fn fetch(self) -> Result<Vec<RecentTrack>> {
        // Validate date range if both from and to are set
        if let (Some(from), Some(to)) = (self.from, self.to)
            && to <= from
        {
            return Err(crate::error::LastFmError::Config(format!(
                "Invalid date range: 'to' timestamp ({to}) must be greater than 'from' timestamp ({from})"
            )));
        }

        let mut params = self.build_params();

        if self.extended {
            params.insert("extended".to_string(), "1".to_string());
        }

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_tracks::<UserRecentTracks>(limit, params).await
    }

    /// Fetch tracks with extended information
    ///
    /// # Errors
    /// Returns an error if:
    /// - The HTTP request fails or the response cannot be parsed
    /// - The date range is invalid (to <= from when both timestamps are set)
    pub async fn fetch_extended(self) -> Result<Vec<RecentTrackExtended>> {
        // Validate date range if both from and to are set
        if let (Some(from), Some(to)) = (self.from, self.to)
            && to <= from
        {
            return Err(crate::error::LastFmError::Config(format!(
                "Invalid date range: 'to' timestamp ({to}) must be greater than 'from' timestamp ({from})"
            )));
        }

        let mut params = self.build_params();
        params.insert("extended".to_string(), "1".to_string());

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_tracks_extended::<UserRecentTracksExtended>(limit, params)
            .await
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
        tracing::info!("Saving {} recent tracks to file", tracks.len());
        let filename = FileHandler::save(&tracks, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;
        Ok(filename)
    }

    /// Fetch tracks with extended information and save them to a file
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
    pub async fn fetch_extended_and_save(
        self,
        format: FileFormat,
        filename_prefix: &str,
    ) -> Result<String> {
        let tracks = self.fetch_extended().await?;
        tracing::info!("Saving {} recent tracks (extended) to file", tracks.len());
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

    /// Check if the user is currently playing a track
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    ///
    /// # Returns
    /// * `Result<Option<RecentTrack>>` - The currently playing track if any
    pub async fn check_currently_playing(self) -> Result<Option<RecentTrack>> {
        let tracks = self.limit(1).fetch().await?;

        // Check if the first track has the "now playing" attribute
        Ok(tracks.first().and_then(|track| {
            if track
                .attr
                .as_ref()
                .is_some_and(|val| val.nowplaying == "true")
            {
                Some(track.clone())
            } else {
                None
            }
        }))
    }

    fn build_params(&self) -> QueryParams {
        let mut params = QueryParams::new();

        if let Some(from_timestamp) = self.from {
            params.insert("from".to_string(), from_timestamp.to_string());
        }

        if let Some(to_timestamp) = self.to {
            params.insert("to".to_string(), to_timestamp.to_string());
        }

        params
    }

    async fn fetch_tracks<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<RecentTrack>>
    where
        T: DeserializeOwned + ResourceContainer<ItemType = RecentTrack>,
    {
        fetch::<RecentTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.getrecenttracks",
            limit,
            additional_params,
            self.progress_callback.as_ref(),
        )
        .await
    }

    async fn fetch_tracks_extended<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<RecentTrackExtended>>
    where
        T: DeserializeOwned + ResourceContainer<ItemType = RecentTrackExtended>,
    {
        fetch::<RecentTrackExtended, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.getrecenttracks",
            limit,
            additional_params,
            self.progress_callback.as_ref(),
        )
        .await
    }
}

impl ResourceContainer for UserRecentTracks {
    type ItemType = RecentTrack;

    fn total(&self) -> u32 {
        self.recenttracks.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.recenttracks.track
    }
}

impl ResourceContainer for UserRecentTracksExtended {
    type ItemType = RecentTrackExtended;

    fn total(&self) -> u32 {
        self.recenttracks.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.recenttracks.track
    }
}
