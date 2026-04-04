use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{LovedTrack, Timestamped, TrackLimit, TrackList, UserLovedTracks};

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

use crate::api::builder_ext::{FetchAndSave, FetchAndUpdate, LimitBuilder};
use crate::api::constants::METHOD_LOVED_TRACKS;
use crate::api::fetch_utils::{ProgressCallback, ResourceContainer, fetch};

/// Builder for loved tracks requests
pub struct LovedTracksRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
    progress_callback: Option<ProgressCallback>,
}

impl fmt::Debug for LovedTracksRequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LovedTracksRequestBuilder")
            .field("username", &self.username)
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}

impl LovedTracksRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
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

    /// Fetch the tracks
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<TrackList<LovedTrack>> {
        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        self.fetch_tracks::<UserLovedTracks>(limit)
            .await
            .map(TrackList::from)
    }

    async fn fetch_tracks<T>(&self, limit: TrackLimit) -> Result<Vec<LovedTrack>>
    where
        T: DeserializeOwned + ResourceContainer<ItemType = LovedTrack>,
    {
        use crate::url_builder::QueryParams;

        fetch::<LovedTrack, T>(
            self.http.clone(),
            self.config.clone(),
            self.username.clone(),
            METHOD_LOVED_TRACKS,
            limit,
            QueryParams::new(),
            self.progress_callback.as_ref(),
        )
        .await
    }
}

impl LimitBuilder for LovedTracksRequestBuilder {
    fn limit_mut(&mut self) -> &mut Option<u32> {
        &mut self.limit
    }
}

impl FetchAndSave for LovedTracksRequestBuilder {
    type Item = LovedTrack;

    fn resource_label() -> &'static str {
        "loved tracks"
    }

    fn latest_timestamp(items: &[Self::Item]) -> Option<u32> {
        items.first().and_then(Timestamped::get_timestamp)
    }

    async fn do_fetch(self) -> crate::error::Result<Vec<Self::Item>> {
        Ok(Vec::from(self.fetch().await?))
    }
}

impl FetchAndUpdate for LovedTracksRequestBuilder {
    type Item = LovedTrack;

    /// Fetch all loved tracks and return only those newer than `max_ts`.
    ///
    /// Because the loved tracks API does not support a `from` timestamp filter, all tracks
    /// are fetched and those already present (by timestamp) are filtered out in memory.
    async fn fetch_since(self, max_ts: Option<u32>) -> crate::error::Result<Vec<Self::Item>> {
        let all_tracks: Vec<LovedTrack> = Vec::from(self.fetch().await?);
        Ok(match max_ts {
            Some(cutoff) => all_tracks
                .into_iter()
                .filter(|t| t.get_timestamp().is_some_and(|ts| ts > cutoff))
                .collect(),
            None => all_tracks,
        })
    }
}

impl ResourceContainer for UserLovedTracks {
    type ItemType = LovedTrack;

    fn total(&self) -> u32 {
        self.lovedtracks.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.lovedtracks.track
    }
}
