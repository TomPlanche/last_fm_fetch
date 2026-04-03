use crate::api::builder_ext::{FetchAndSave, LimitBuilder};
use crate::api::constants::METHOD_TOP_ARTISTS;
use crate::api::fetch_utils::{Period, ProgressCallback, ResourceContainer, fetch};
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{TopArtist, TrackLimit, TrackList, UserTopArtists};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

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
    progress_callback: Option<ProgressCallback>,
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
    pub async fn fetch(self) -> Result<TrackList<TopArtist>> {
        let mut params = QueryParams::new();

        if let Some(period) = self.period {
            params.insert("period".to_string(), period.as_api_str().to_string());
        }

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_artists::<UserTopArtists>(limit, params)
            .await
            .map(TrackList::from)
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
            METHOD_TOP_ARTISTS,
            limit,
            additional_params,
            self.progress_callback.as_ref(),
        )
        .await
    }
}

impl LimitBuilder for TopArtistsRequestBuilder {
    fn limit_mut(&mut self) -> &mut Option<u32> {
        &mut self.limit
    }
}

impl FetchAndSave for TopArtistsRequestBuilder {
    type Item = TopArtist;

    fn resource_label() -> &'static str {
        "top artists"
    }

    async fn do_fetch(self) -> crate::error::Result<Vec<Self::Item>> {
        Ok(Vec::from(self.fetch().await?))
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
