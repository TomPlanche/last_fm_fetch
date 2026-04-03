use crate::api::builder_ext::{FetchAndSave, LimitBuilder};
use crate::api::constants::METHOD_TOP_ALBUMS;
use crate::api::fetch_utils::{Period, ProgressCallback, ResourceContainer, fetch};
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{TopAlbum, TrackLimit, TrackList, UserTopAlbums};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

/// Client for fetching top albums
pub struct TopAlbumsClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl fmt::Debug for TopAlbumsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TopAlbumsClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl TopAlbumsClient {
    /// Create a new top albums client
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
    progress_callback: Option<ProgressCallback>,
}

impl fmt::Debug for TopAlbumsRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TopAlbumsRequestBuilder")
            .field("username", &self.username)
            .field("limit", &self.limit)
            .field("period", &self.period)
            .finish_non_exhaustive()
    }
}

impl TopAlbumsRequestBuilder {
    fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
            period: None,
            progress_callback: None,
        }
    }

    /// Register a progress callback invoked with `(fetched, total)` after each batch.
    #[must_use]
    pub fn on_progress(mut self, callback: impl Fn(u32, u32) + Send + Sync + 'static) -> Self {
        self.progress_callback = Some(Arc::new(callback));
        self
    }

    /// Display a terminal progress bar while fetching (requires `progress` feature).
    #[cfg(feature = "progress")]
    #[must_use]
    pub fn with_progress(self) -> Self {
        self.on_progress(crate::api::progress::make_progress_callback())
    }

    /// Set the time period for top albums
    ///
    /// # Arguments
    /// * `period` - The time range to calculate top albums over. Use `Period::Overall` for all-time,
    ///   `Period::Week` for last 7 days, `Period::Month` for last 30 days, etc.
    ///   If not set, defaults to the Last.fm API's default behavior (typically overall).
    #[must_use]
    pub const fn period(mut self, period: Period) -> Self {
        self.period = Some(period);
        self
    }

    /// Fetch the albums
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<TrackList<TopAlbum>> {
        let mut params = QueryParams::new();

        if let Some(period) = self.period {
            params.insert("period".to_string(), period.as_api_str().to_string());
        }

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_albums::<UserTopAlbums>(limit, params)
            .await
            .map(TrackList::from)
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
            METHOD_TOP_ALBUMS,
            limit,
            additional_params,
            self.progress_callback.as_ref(),
        )
        .await
    }
}

impl LimitBuilder for TopAlbumsRequestBuilder {
    fn limit_mut(&mut self) -> &mut Option<u32> {
        &mut self.limit
    }
}

impl FetchAndSave for TopAlbumsRequestBuilder {
    type Item = TopAlbum;

    fn resource_label() -> &'static str {
        "top albums"
    }

    async fn do_fetch(self) -> crate::error::Result<Vec<Self::Item>> {
        Ok(Vec::from(self.fetch().await?))
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
