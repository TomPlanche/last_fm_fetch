//! `RecentTracksClient` — entry point for recent-tracks requests.

use crate::client::HttpClient;
use crate::config::Config;

use std::fmt;
use std::sync::Arc;

use super::builder::RecentTracksRequestBuilder;

/// Client for fetching recent tracks.
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
    /// Create a new recent tracks client.
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for recent tracks requests.
    pub fn builder(&self, username: impl Into<String>) -> RecentTracksRequestBuilder {
        RecentTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }
}
