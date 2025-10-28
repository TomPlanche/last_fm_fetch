use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{RecentTrack, RecentTrackExtended, TrackLimit, UserRecentTracks, UserRecentTracksExtended};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::fetch_utils::{fetch_tracks, TrackContainer};

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
        fetch_tracks::<RecentTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.getrecenttracks",
            limit,
            additional_params,
        )
        .await
    }

    async fn fetch_tracks_extended<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<RecentTrackExtended>>
    where
        T: DeserializeOwned + TrackContainer<TrackType = RecentTrackExtended>,
    {
        fetch_tracks::<RecentTrackExtended, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.getrecenttracks",
            limit,
            additional_params,
        )
        .await
    }

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
