//! `RecentTracksRequestBuilder` — fluent builder for recent-tracks requests.

use crate::api::builder_ext::{FetchAndSave, FetchAndUpdate, LimitBuilder};
use crate::api::constants::METHOD_RECENT_TRACKS;
use crate::api::fetch_utils::{ProgressCallback, ResourceContainer, fetch};
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{RecentTrack, Timestamped, TrackLimit, TrackList, UserRecentTracks};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

/// Validate that `to` is strictly greater than `from` when both are present.
///
/// # Errors
/// Returns `LastFmError::Config` if `to <= from`.
pub(in crate::api::user::recent_tracks) fn validate_date_range(
    from: Option<i64>,
    to: Option<i64>,
) -> crate::error::Result<()> {
    if let (Some(from), Some(to)) = (from, to)
        && to <= from
    {
        return Err(crate::error::LastFmError::Config(format!(
            "Invalid date range: 'to' timestamp ({to}) must be greater than 'from' timestamp ({from})"
        )));
    }
    Ok(())
}

/// Builder for recent tracks requests.
pub struct RecentTracksRequestBuilder {
    pub(in crate::api::user::recent_tracks) http: Arc<dyn HttpClient>,
    pub(in crate::api::user::recent_tracks) config: Arc<Config>,
    pub(in crate::api::user::recent_tracks) username: String,
    pub(in crate::api::user::recent_tracks) limit: Option<u32>,
    pub(in crate::api::user::recent_tracks) from: Option<i64>,
    pub(in crate::api::user::recent_tracks) to: Option<i64>,
    pub(in crate::api::user::recent_tracks) progress_callback: Option<ProgressCallback>,
}

impl fmt::Debug for RecentTracksRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecentTracksRequestBuilder")
            .field("username", &self.username)
            .field("limit", &self.limit)
            .field("from", &self.from)
            .field("to", &self.to)
            .finish_non_exhaustive()
    }
}

impl RecentTracksRequestBuilder {
    /// Create a new builder.
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
            from: None,
            to: None,
            progress_callback: None,
        }
    }

    /// Fetch tracks from this timestamp onwards.
    ///
    /// # Arguments
    /// * `timestamp` - Unix timestamp in seconds (not milliseconds) since January 1, 1970 UTC
    ///
    /// # Example
    /// ```ignore
    /// // Fetch tracks since January 1, 2024 00:00:00 UTC
    /// let tracks = client.recent_tracks()
    ///     .builder("username")
    ///     .since(1704067200)
    ///     .fetch()
    ///     .await?;
    /// ```
    #[must_use]
    pub const fn since(mut self, timestamp: i64) -> Self {
        self.from = Some(timestamp);

        self
    }

    /// Fetch tracks between two timestamps.
    ///
    /// # Arguments
    /// * `from` - Start Unix timestamp in seconds (not milliseconds) since January 1, 1970 UTC
    /// * `to` - End Unix timestamp in seconds (not milliseconds) since January 1, 1970 UTC
    ///
    /// # Example
    /// ```ignore
    /// // Fetch tracks between January 1, 2024 and February 1, 2024 (UTC)
    /// let tracks = client.recent_tracks()
    ///     .builder("username")
    ///     .between(1704067200, 1706745600)
    ///     .fetch()
    ///     .await?;
    /// ```
    #[must_use]
    pub const fn between(mut self, from: i64, to: i64) -> Self {
        self.from = Some(from);
        self.to = Some(to);

        self
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

    /// Fetch the tracks.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The HTTP request fails or the response cannot be parsed
    /// - The date range is invalid (to <= from when both timestamps are set)
    pub async fn fetch(self) -> Result<TrackList<RecentTrack>> {
        validate_date_range(self.from, self.to)?;

        let params = self.build_params();

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_tracks::<UserRecentTracks>(limit, params)
            .await
            .map(TrackList::from)
    }

    /// Check if the user is currently playing a track.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    ///
    /// # Returns
    /// * `Result<Option<RecentTrack>>` - The currently playing track if any
    pub async fn check_currently_playing(self) -> Result<Option<RecentTrack>> {
        let tracks = self.limit(1).fetch().await?;

        Ok(tracks.first().and_then(|track| {
            if track
                .attr
                .as_ref()
                .is_some_and(|val| val.nowplaying == "true")
            {
                Some(track.clone())
            } else {
                None
            }
        }))
    }

    /// Build the base query parameters from the builder state.
    pub(in crate::api::user::recent_tracks) fn build_params(&self) -> QueryParams {
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
        T: DeserializeOwned + ResourceContainer<ItemType = RecentTrack>,
    {
        fetch::<RecentTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            METHOD_RECENT_TRACKS,
            limit,
            additional_params,
            self.progress_callback.as_ref(),
        )
        .await
    }
}

impl LimitBuilder for RecentTracksRequestBuilder {
    fn limit_mut(&mut self) -> &mut Option<u32> {
        &mut self.limit
    }
}

impl FetchAndSave for RecentTracksRequestBuilder {
    type Item = RecentTrack;

    fn resource_label() -> &'static str {
        "recent tracks"
    }

    fn latest_timestamp(items: &[Self::Item]) -> Option<u32> {
        items.first().and_then(Timestamped::get_timestamp)
    }

    async fn do_fetch(self) -> crate::error::Result<Vec<Self::Item>> {
        Ok(Vec::from(self.fetch().await?))
    }
}

impl FetchAndUpdate for RecentTracksRequestBuilder {
    type Item = RecentTrack;

    /// Fetch recent tracks newer than `max_ts` using the API-side `from` filter.
    async fn fetch_since(self, max_ts: Option<u32>) -> crate::error::Result<Vec<Self::Item>> {
        let builder = match max_ts {
            Some(ts) => self.since(i64::from(ts) + 1),
            None => self,
        };
        Ok(Vec::from(builder.fetch().await?))
    }
}
