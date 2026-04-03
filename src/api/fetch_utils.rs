use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::TrackLimit;
use crate::url_builder::{QueryParams, Url};

use futures::future::join_all;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::constants::{API_MAX_LIMIT, BASE_URL, CHUNK_MULTIPLIER, CHUNK_SIZE};

/// Callback invoked with `(fetched, total)` after each batch of tracks is received.
pub type ProgressCallback = Arc<dyn Fn(u32, u32) + Send + Sync>;

/// Period options for Last.fm time range filters
///
/// These periods define the time range for calculating top tracks.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Period {
    /// All-time top tracks (no time limit)
    Overall,
    /// Top tracks from the last 7 days
    Week,
    /// Top tracks from the last month (30 days)
    Month,
    /// Top tracks from the last 3 months (90 days)
    ThreeMonth,
    /// Top tracks from the last 6 months (180 days)
    SixMonth,
    /// Top tracks from the last 12 months (365 days)
    TwelveMonth,
}

impl Period {
    /// Convert to the Last.fm API string representation
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::Overall => "overall",
            Self::Week => "7day",
            Self::Month => "1month",
            Self::ThreeMonth => "3month",
            Self::SixMonth => "6month",
            Self::TwelveMonth => "12month",
        }
    }
}

/// Trait for containers that hold Last.fm resources (tracks, artists, etc.)
pub trait ResourceContainer {
    /// The type of items contained in this resource
    type ItemType;

    /// Get the total number of items available
    fn total(&self) -> u32;
    /// Extract the items from this container
    fn items(self) -> Vec<Self::ItemType>;
}

/// Generic function to fetch things with pagination
pub(in crate::api) async fn fetch<T, R>(
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    method: &str,
    limit: TrackLimit,
    additional_params: QueryParams,
    on_progress: Option<&ProgressCallback>,
) -> Result<Vec<T>>
where
    R: DeserializeOwned + ResourceContainer<ItemType = T>,
{
    let mut base_params = QueryParams::new();
    base_params.insert("api_key".to_string(), config.api_key().to_string());
    base_params.insert("method".to_string(), method.to_string());
    base_params.insert("user".to_string(), username);
    base_params.insert("format".to_string(), "json".to_string());
    base_params.extend(additional_params);

    // Make an initial request to get the total number of tracks
    let mut initial_params = base_params.clone();
    initial_params.insert("limit".to_string(), "1".to_string());
    initial_params.insert("page".to_string(), "1".to_string());

    let initial_response: R = fetch_json(&http, &initial_params).await?;
    let total_tracks = initial_response.total();

    let final_limit = match limit {
        TrackLimit::Limited(l) => l.min(total_tracks),
        TrackLimit::Unlimited => total_tracks,
    };

    if final_limit == 0 {
        return Ok(Vec::new());
    }

    // Report initial state (total known, nothing fetched yet)
    if let Some(cb) = on_progress {
        cb(0, final_limit);
    }

    if final_limit <= API_MAX_LIMIT {
        // If we need less than the API limit, just make a single request
        let mut single_params = base_params;
        single_params.insert("limit".to_string(), final_limit.to_string());
        single_params.insert("page".to_string(), "1".to_string());

        let response: R = fetch_json(&http, &single_params).await?;
        let items: Vec<T> = response
            .items()
            .into_iter()
            .take(final_limit as usize)
            .collect();

        if let Some(cb) = on_progress {
            #[allow(clippy::cast_possible_truncation)]
            cb(items.len() as u32, final_limit);
        }

        return Ok(items);
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

                let http = http.clone();
                async move {
                    let response: R = fetch_json(&http, &call_params).await?;
                    Ok::<Vec<T>, crate::error::LastFmError>(
                        response
                            .items()
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

        if let Some(cb) = on_progress {
            #[allow(clippy::cast_possible_truncation)]
            cb(all_tracks.len() as u32, final_limit);
        }
    }

    Ok(all_tracks)
}

async fn fetch_json<T: DeserializeOwned>(
    http: &Arc<dyn HttpClient>,
    params: &QueryParams,
) -> Result<T> {
    let url = Url::new(BASE_URL).add_args(params.clone()).build();
    let response = http.get(&url).await?;

    match serde_json::from_value::<T>(response) {
        Ok(parsed) => Ok(parsed),
        Err(err) => {
            #[cfg(debug_assertions)]
            eprintln!("Deserialization failed: {err}\nURL: {url}");
            Err(err.into())
        }
    }
}
