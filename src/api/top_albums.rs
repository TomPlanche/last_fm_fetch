use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{TopAlbum, TrackLimit, UserTopAlbums};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::fetch_utils::{Period, ResourceContainer, fetch};

/// Client for fetching top albums
pub struct TopAlbumsClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl TopAlbumsClient {
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for top albums requests
    pub fn builder(&self, username: impl Into<String>) -> TopAlbumsRequestBuilder {
        TopAlbumsRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }
}

/// Builder for top albums requests
pub struct TopAlbumsRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
    period: Option<Period>,
}

impl TopAlbumsRequestBuilder {
    fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
            period: None,
        }
    }

    /// Set the maximum number of albums to fetch
    ///
    /// # Arguments
    /// * `limit` - Maximum number of albums to fetch. The Last.fm API supports fetching up to thousands of albums.
    ///   If you need all albums, use `unlimited()` instead.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Fetch all available albums (no limit)
    #[must_use]
    pub fn unlimited(mut self) -> Self {
        self.limit = None;
        self
    }

    /// Set the time period for top albums
    ///
    /// # Arguments
    /// * `period` - The time range to calculate top albums over. Use `Period::Overall` for all-time,
    ///   `Period::Week` for last 7 days, `Period::Month` for last 30 days, etc.
    ///   If not set, defaults to the Last.fm API's default behavior (typically overall).
    #[must_use]
    pub fn period(mut self, period: Period) -> Self {
        self.period = Some(period);
        self
    }

    /// Fetch the albums
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<TopAlbum>> {
        let mut params = QueryParams::new();

        if let Some(period) = self.period {
            params.insert("period".to_string(), period.as_api_str().to_string());
        }

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_albums::<UserTopAlbums>(limit, params).await
    }

    /// Fetch albums and save them to a file
    ///
    /// # Arguments
    /// * `format` - The file format to save the albums in
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, response cannot be parsed, or file cannot be saved.
    ///
    /// # Returns
    /// * `Result<String>` - The filename of the saved file
    pub async fn fetch_and_save(self, format: FileFormat, filename_prefix: &str) -> Result<String> {
        let albums = self.fetch().await?;
        tracing::info!("Saving {} top albums to file", albums.len());
        let filename = FileHandler::save(&albums, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;
        Ok(filename)
    }

    async fn fetch_albums<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<TopAlbum>>
    where
        T: DeserializeOwned + ResourceContainer<ItemType = TopAlbum>,
    {
        fetch::<TopAlbum, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.gettopalbums",
            limit,
            additional_params,
        )
        .await
    }
}

impl ResourceContainer for UserTopAlbums {
    type ItemType = TopAlbum;

    fn total(&self) -> u32 {
        self.topalbums.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.topalbums.album
    }
}
