use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{LovedTrack, TrackLimit, UserLovedTracks};

use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::fetch_utils::{fetch_tracks, TrackContainer};

/// Client for fetching loved tracks
pub struct LovedTracksClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl LovedTracksClient {
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for loved tracks requests
    pub fn builder(&self, username: impl Into<String>) -> LovedTracksRequestBuilder {
        LovedTracksRequestBuilder::new(
            self.http.clone(),
            self.config.clone(),
            username.into(),
        )
    }
}

/// Builder for loved tracks requests
pub struct LovedTracksRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
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

    async fn fetch_tracks<T>(
        &self,
        limit: TrackLimit,
    ) -> Result<Vec<LovedTrack>>
    where
        T: DeserializeOwned + TrackContainer<TrackType = LovedTrack>,
    {
        use crate::url_builder::QueryParams;
        
        fetch_tracks::<LovedTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.getlovedtracks",
            limit,
            QueryParams::new(),
        )
        .await
    }

}

impl TrackContainer for UserLovedTracks {
    type TrackType = LovedTrack;

    fn total_tracks(&self) -> u32 {
        self.lovedtracks.attr.total
    }

    fn tracks(self) -> Vec<Self::TrackType> {
        self.lovedtracks.track
    }
}
