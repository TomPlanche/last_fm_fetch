use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{RecentTrack, RecentTrackExtended, TrackLimit, UserRecentTracks, UserRecentTracksExtended};
use crate::url_builder::{QueryParams, Url};

use futures::future::join_all;
use serde::de::DeserializeOwned;
use std::sync::Arc;

const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";
const API_MAX_LIMIT: u32 = 1000;
const CHUNK_MULTIPLIER: u32 = 5;
const CHUNK_SIZE: u32 = API_MAX_LIMIT * CHUNK_MULTIPLIER;

/// Client for fetching recent tracks
pub struct RecentTracksClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl RecentTracksClient {
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for recent tracks requests
    pub fn builder(&self, username: impl Into<String>) -> RecentTracksRequestBuilder {
        RecentTracksRequestBuilder::new(
            self.http.clone(),
            self.config.clone(),
            username.into(),
        )
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
        }
    }

    /// Set the maximum number of tracks to fetch
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Fetch all available tracks (no limit)
    #[must_use]
    pub fn unlimited(mut self) -> Self {
        self.limit = None;
        self
    }

    /// Fetch tracks from this timestamp onwards
    #[must_use]
    pub fn since(mut self, timestamp: i64) -> Self {
        self.from = Some(timestamp);
        self
    }

    /// Fetch tracks between two timestamps
    #[must_use]
    pub fn between(mut self, from: i64, to: i64) -> Self {
        self.from = Some(from);
        self.to = Some(to);
        self
    }

    /// Fetch extended track information
    #[must_use]
    pub fn extended(mut self, extended: bool) -> Self {
        self.extended = extended;
        self
    }

    /// Fetch the tracks
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<RecentTrack>> {
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
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch_extended(self) -> Result<Vec<RecentTrackExtended>> {
        let mut params = self.build_params();
        params.insert("extended".to_string(), "1".to_string());

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_tracks_extended::<UserRecentTracksExtended>(limit, params).await
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
        T: DeserializeOwned + TrackContainer<TrackType = RecentTrack>,
    {
        let mut base_params = QueryParams::new();
        base_params.insert("api_key".to_string(), self.config.api_key().to_string());
        base_params.insert("method".to_string(), "user.getrecenttracks".to_string());
        base_params.insert("user".to_string(), self.username.clone());
        base_params.insert("format".to_string(), "json".to_string());
        base_params.extend(additional_params);

        // Make an initial request to get the total number of tracks
        let mut initial_params = base_params.clone();
        initial_params.insert("limit".to_string(), "1".to_string());
        initial_params.insert("page".to_string(), "1".to_string());

        let initial_response: T = self.fetch_json(&initial_params).await?;
        let total_tracks = initial_response.total_tracks();

        let final_limit = match limit {
            TrackLimit::Limited(l) => l.min(total_tracks),
            TrackLimit::Unlimited => total_tracks,
        };

        if final_limit == 0 {
            return Ok(Vec::new());
        }

        if final_limit <= API_MAX_LIMIT {
            // If we need less than the API limit, just make a single request
            let mut single_params = base_params;
            single_params.insert("limit".to_string(), final_limit.to_string());
            single_params.insert("page".to_string(), "1".to_string());

            let response: T = self.fetch_json(&single_params).await?;
            return Ok(response
                .tracks()
                .into_iter()
                .take(final_limit as usize)
                .collect());
        }

        // Handle pagination with chunking
        let chunk_nb = final_limit.div_ceil(CHUNK_SIZE);
        let mut all_tracks = Vec::new();

        // Process chunks sequentially
        for chunk_index in 0..chunk_nb {
            let chunk_params = base_params.clone();

            // Calculate how many API calls we need for this chunk
            let chunk_api_calls = if chunk_index == chunk_nb - 1 {
                // Last chunk
                (final_limit % CHUNK_SIZE).div_ceil(API_MAX_LIMIT).max(1)
            } else {
                CHUNK_MULTIPLIER
            };

            // Create futures for concurrent API calls within this chunk
            let api_call_futures: Vec<_> = (0..chunk_api_calls)
                .map(|call_index| {
                    let mut call_params = chunk_params.clone();
                    let call_limit =
                        (final_limit - chunk_index * CHUNK_SIZE - call_index * API_MAX_LIMIT)
                            .min(API_MAX_LIMIT);

                    let page = chunk_index * CHUNK_MULTIPLIER + call_index + 1;

                    call_params.insert("limit".to_string(), call_limit.to_string());
                    call_params.insert("page".to_string(), page.to_string());

                    let http = self.http.clone();
                    async move {
                        let response: T = self.fetch_json_with_http(&call_params, http).await?;
                        Ok::<Vec<RecentTrack>, crate::error::LastFmError>(
                            response
                                .tracks()
                                .into_iter()
                                .take(call_limit as usize)
                                                .collect(),
                        )
                    }
                })
                .collect();

            // Process all API calls in this chunk concurrently
            let chunk_results = join_all(api_call_futures).await;

            // Collect results from this chunk
            for result in chunk_results {
                all_tracks.extend(result?);
            }
        }

        Ok(all_tracks)
    }

    async fn fetch_tracks_extended<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<RecentTrackExtended>>
    where
        T: DeserializeOwned + TrackContainer<TrackType = RecentTrackExtended>,
    {
        let mut base_params = QueryParams::new();
        base_params.insert("api_key".to_string(), self.config.api_key().to_string());
        base_params.insert("method".to_string(), "user.getrecenttracks".to_string());
        base_params.insert("user".to_string(), self.username.clone());
        base_params.insert("format".to_string(), "json".to_string());
        base_params.extend(additional_params);

        // Make an initial request to get the total number of tracks
        let mut initial_params = base_params.clone();
        initial_params.insert("limit".to_string(), "1".to_string());
        initial_params.insert("page".to_string(), "1".to_string());

        let initial_response: T = self.fetch_json(&initial_params).await?;
        let total_tracks = initial_response.total_tracks();

        let final_limit = match limit {
            TrackLimit::Limited(l) => l.min(total_tracks),
            TrackLimit::Unlimited => total_tracks,
        };

        if final_limit == 0 {
            return Ok(Vec::new());
        }

        if final_limit <= API_MAX_LIMIT {
            let mut single_params = base_params;
            single_params.insert("limit".to_string(), final_limit.to_string());
            single_params.insert("page".to_string(), "1".to_string());

            let response: T = self.fetch_json(&single_params).await?;
            return Ok(response
                .tracks()
                .into_iter()
                .take(final_limit as usize)
                .collect());
        }

        // Handle pagination (similar logic as fetch_tracks but for extended)
        let chunk_nb = final_limit.div_ceil(CHUNK_SIZE);
        let mut all_tracks = Vec::new();

        for chunk_index in 0..chunk_nb {
            let chunk_params = base_params.clone();
            let chunk_api_calls = if chunk_index == chunk_nb - 1 {
                (final_limit % CHUNK_SIZE).div_ceil(API_MAX_LIMIT).max(1)
            } else {
                CHUNK_MULTIPLIER
            };

            let api_call_futures: Vec<_> = (0..chunk_api_calls)
                .map(|call_index| {
                    let mut call_params = chunk_params.clone();
                    let call_limit =
                        (final_limit - chunk_index * CHUNK_SIZE - call_index * API_MAX_LIMIT)
                            .min(API_MAX_LIMIT);

                    let page = chunk_index * CHUNK_MULTIPLIER + call_index + 1;

                    call_params.insert("limit".to_string(), call_limit.to_string());
                    call_params.insert("page".to_string(), page.to_string());

                    let http = self.http.clone();
                    async move {
                        let response: T = self.fetch_json_with_http(&call_params, http).await?;
                        Ok::<Vec<RecentTrackExtended>, crate::error::LastFmError>(
                            response
                                .tracks()
                                .into_iter()
                                .take(call_limit as usize)
                                                .collect(),
                        )
                    }
                })
                .collect();

            let chunk_results = join_all(api_call_futures).await;

            for result in chunk_results {
                all_tracks.extend(result?);
            }
        }

        Ok(all_tracks)
    }

    async fn fetch_json<T: DeserializeOwned>(&self, params: &QueryParams) -> Result<T> {
        self.fetch_json_with_http(params, self.http.clone()).await
    }

    async fn fetch_json_with_http<T: DeserializeOwned>(
        &self,
        params: &QueryParams,
        http: Arc<dyn HttpClient>,
    ) -> Result<T> {
        let url = Url::new(BASE_URL).add_args(params.clone()).build();
        let response = http.get(&url).await?;

        match serde_json::from_value::<T>(response.clone()) {
            Ok(parsed) => Ok(parsed),
            Err(err) => {
                #[cfg(debug_assertions)]
                {
                    eprintln!(
                        "Deserialization failed: {err}\nURL: {url}\nRaw JSON:\n{}",
                        serde_json::to_string_pretty(&response).unwrap_or_default()
                    );
                }
                Err(err.into())
            }
        }
    }
}

// Trait for containers - simplified since we no longer need conversions!
trait TrackContainer {
    type TrackType;

    fn total_tracks(&self) -> u32;
    fn tracks(self) -> Vec<Self::TrackType>;
}

impl TrackContainer for UserRecentTracks {
    type TrackType = RecentTrack;

    fn total_tracks(&self) -> u32 {
        self.recenttracks.attr.total
    }

    fn tracks(self) -> Vec<Self::TrackType> {
        self.recenttracks.track
    }
}

impl TrackContainer for UserRecentTracksExtended {
    type TrackType = RecentTrackExtended;

    fn total_tracks(&self) -> u32 {
        self.recenttracks.attr.total
    }

    fn tracks(self) -> Vec<Self::TrackType> {
        self.recenttracks.track
    }
}
