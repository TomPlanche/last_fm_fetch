use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{TopArtist, TrackLimit, UserTopArtists};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

use super::fetch_utils::{Period, ResourceContainer, fetch};

/// Client for fetching top artists
pub struct TopArtistsClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl fmt::Debug for TopArtistsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TopArtistsClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl TopArtistsClient {
    /// Create a new top artists client
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for top artists requests
    pub fn builder(&self, username: impl Into<String>) -> TopArtistsRequestBuilder {
        TopArtistsRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }
}

/// Builder for top artists requests
pub struct TopArtistsRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
    period: Option<Period>,
}

impl fmt::Debug for TopArtistsRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TopArtistsRequestBuilder")
            .field("username", &self.username)
            .field("limit", &self.limit)
            .field("period", &self.period)
            .finish_non_exhaustive()
    }
}

impl TopArtistsRequestBuilder {
    fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
            period: None,
        }
    }

    /// Set the maximum number of artists to fetch
    ///
    /// # Arguments
    /// * `limit` - Maximum number of artists to fetch. The Last.fm API supports fetching up to thousands of artists.
    ///   If you need all artists, use `unlimited()` instead.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Fetch all available artists (no limit)
    #[must_use]
    pub const fn unlimited(mut self) -> Self {
        self.limit = None;
        self
    }

    /// Set the time period for top artists
    ///
    /// # Arguments
    /// * `period` - The time range to calculate top artists over. Use `Period::Overall` for all-time,
    ///   `Period::Week` for last 7 days, `Period::Month` for last 30 days, etc.
    ///   If not set, defaults to the Last.fm API's default behavior (typically overall).
    #[must_use]
    pub const fn period(mut self, period: Period) -> Self {
        self.period = Some(period);
        self
    }

    /// Fetch the artists
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<TopArtist>> {
        let mut params = QueryParams::new();

        if let Some(period) = self.period {
            params.insert("period".to_string(), period.as_api_str().to_string());
        }

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_artists::<UserTopArtists>(limit, params).await
    }

    /// Fetch artists and save them to a file
    ///
    /// # Arguments
    /// * `format` - The file format to save the artists in
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, response cannot be parsed, or file cannot be saved.
    ///
    /// # Returns
    /// * `Result<String>` - The filename of the saved file
    pub async fn fetch_and_save(self, format: FileFormat, filename_prefix: &str) -> Result<String> {
        let artists = self.fetch().await?;
        tracing::info!("Saving {} top artists to file", artists.len());
        let filename = FileHandler::save(&artists, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;
        Ok(filename)
    }

    /// Fetch artists and save them to a new `SQLite` database file.
    ///
    /// # Arguments
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the database cannot be saved.
    ///
    /// # Returns
    /// * `Result<String>` - Path to the saved database file
    #[cfg(feature = "sqlite")]
    pub async fn fetch_and_save_sqlite(self, filename_prefix: &str) -> Result<String> {
        let artists = self.fetch().await?;
        tracing::info!("Saving {} top artists to SQLite", artists.len());
        crate::file_handler::FileHandler::save_sqlite(&artists, filename_prefix)
            .map_err(crate::error::LastFmError::Io)
    }

    async fn fetch_artists<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<TopArtist>>
    where
        T: DeserializeOwned + ResourceContainer<ItemType = TopArtist>,
    {
        fetch::<TopArtist, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.gettopartists",
            limit,
            additional_params,
            None,
        )
        .await
    }
}

impl ResourceContainer for UserTopArtists {
    type ItemType = TopArtist;

    fn total(&self) -> u32 {
        self.topartists.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.topartists.artist
    }
}
