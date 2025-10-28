use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{TopTrack, TrackLimit, UserTopTracks};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::fetch_utils::{fetch_tracks, TrackContainer};

/// Period options for Last.fm time range filters
#[derive(Debug, Clone, Copy)]
pub enum Period {
    Overall,
    Week,
    Month,
    ThreeMonth,
    SixMonth,
    TwelveMonth,
}

impl Period {
    #[must_use]
    pub fn as_api_str(self) -> &'static str {
        match self {
            Period::Overall => "overall",
            Period::Week => "7day",
            Period::Month => "1month",
            Period::ThreeMonth => "3month",
            Period::SixMonth => "6month",
            Period::TwelveMonth => "12month",
        }
    }
}

/// Client for fetching top tracks
pub struct TopTracksClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl TopTracksClient {
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for top tracks requests
    pub fn builder(&self, username: impl Into<String>) -> TopTracksRequestBuilder {
        TopTracksRequestBuilder::new(
            self.http.clone(),
            self.config.clone(),
            username.into(),
        )
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

    /// Set the time period for top tracks
    #[must_use]
    pub fn period(mut self, period: Period) -> Self {
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
        let filename = FileHandler::save(&tracks, &format, filename_prefix).map_err(crate::error::LastFmError::Io)?;
        Ok(filename)
    }

    async fn fetch_tracks<T>(
        &self,
        limit: TrackLimit,
        additional_params: QueryParams,
    ) -> Result<Vec<TopTrack>>
    where
        T: DeserializeOwned + TrackContainer<TrackType = TopTrack>,
    {
        fetch_tracks::<TopTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            "user.gettoptracks",
            limit,
            additional_params,
        )
        .await
    }
}

impl TrackContainer for UserTopTracks {
    type TrackType = TopTrack;

    fn total_tracks(&self) -> u32 {
        self.toptracks.attr.total
    }

    fn tracks(self) -> Vec<Self::TrackType> {
        self.toptracks.track
    }
}
