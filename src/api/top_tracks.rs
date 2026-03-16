use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{TopTrack, TrackLimit, UserTopTracks};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

use super::fetch_utils::{Period, ResourceContainer, fetch};

/// Client for fetching top tracks
pub struct TopTracksClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl fmt::Debug for TopTracksClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TopTracksClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl TopTracksClient {
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for top tracks requests
    pub fn builder(&self, username: impl Into<String>) -> TopTracksRequestBuilder {
        TopTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }
}

/// Builder for top tracks requests
pub struct TopTracksRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
    period: Option<Period>,
}

impl fmt::Debug for TopTracksRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TopTracksRequestBuilder")
            .field("username", &self.username)
            .field("limit", &self.limit)
            .field("period", &self.period)
            .finish_non_exhaustive()
    }
}

impl TopTracksRequestBuilder {
    fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
            period: None,
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

    /// Set the time period for top tracks
    ///
    /// # Arguments
    /// * `period` - The time range to calculate top tracks over. Use `Period::Overall` for all-time,
    ///   `Period::Week` for last 7 days, `Period::Month` for last 30 days, etc.
    ///   If not set, defaults to the Last.fm API's default behavior (typically overall).
    ///
    /// # Example
    /// ```ignore
    /// use lastfm_client::api::Period;
    ///
    /// let tracks = client.top_tracks("username")
    ///     .period(Period::Month)
    ///     .limit(50)
    ///     .fetch()
    ///     .await?;
    /// ```
    #[must_use]
    pub const fn period(mut self, period: Period) -> Self {
        self.period = Some(period);
        self
    }

    /// Fetch the tracks
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<TopTrack>> {
        let mut params = QueryParams::new();

        if let Some(period) = self.period {
            params.insert("period".to_string(), period.as_api_str().to_string());
        }

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_tracks::<UserTopTracks>(limit, params).await
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
        tracing::info!("Saving {} top tracks to file", tracks.len());
        let filename = FileHandler::save(&tracks, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;
        Ok(filename)
    }

    async fn fetch_tracks<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<TopTrack>>
    where
        T: DeserializeOwned + ResourceContainer<ItemType = TopTrack>,
    {
        fetch::<TopTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.gettoptracks",
            limit,
            additional_params,
            None,
        )
        .await
    }
}

impl ResourceContainer for UserTopTracks {
    type ItemType = TopTrack;

    fn total(&self) -> u32 {
        self.toptracks.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.toptracks.track
    }
}
