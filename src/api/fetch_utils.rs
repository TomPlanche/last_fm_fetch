use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::TrackLimit;
use crate::url_builder::{QueryParams, Url};

use futures::future::join_all;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::constants::{API_MAX_LIMIT, BASE_URL, CHUNK_MULTIPLIER, CHUNK_SIZE};

/// Trait for containers that hold track data
pub trait TrackContainer {
    type TrackType;

    fn total_tracks(&self) -> u32;
    fn tracks(self) -> Vec<Self::TrackType>;
}

/// Generic function to fetch tracks with pagination
pub async fn fetch_tracks<T, R>(
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    method: &str,
    limit: TrackLimit,
    additional_params: QueryParams,
) -> Result<Vec<T>>
where
    R: DeserializeOwned + TrackContainer<TrackType = T>,
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

        let response: R = fetch_json(&http, &single_params).await?;
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

                let http = http.clone();
                async move {
                    let response: R = fetch_json(&http, &call_params).await?;
                    Ok::<Vec<T>, crate::error::LastFmError>(
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

async fn fetch_json<T: DeserializeOwned>(
    http: &Arc<dyn HttpClient>,
    params: &QueryParams,
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
