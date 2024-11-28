use crate::types::*;
use crate::url_builder::{QueryParams, Url};

use reqwest::Error;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::env;

const API_MAX_LIMIT: u32 = 1000;

const CHUNK_MULTIPLIER: u32 = 5;
const CHUNK_SIZE: u32 = API_MAX_LIMIT * CHUNK_MULTIPLIER;

#[derive(Debug)]
pub struct LastFMHandler {
    url: Url,
    base_options: QueryParams,
}

impl LastFMHandler {
    pub fn new(url: Url, username: &str) -> Self {
        let mut base_options = QueryParams::new();
        base_options.insert("api_key".to_string(), env::var("LAST_FM_API_KEY").unwrap());
        base_options.insert("limit".to_string(), API_MAX_LIMIT.to_string());
        base_options.insert("format".to_string(), "json".to_string());
        base_options.insert("user".to_string(), username.to_string());

        LastFMHandler { url, base_options }
    }

    pub async fn get_user_loved_tracks(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<LovedTrack>, Error> {
        let final_limit = limit.unwrap_or(API_MAX_LIMIT);

        let final_limit_str = final_limit.to_string();

        let mut base_params: QueryParams = HashMap::new();
        base_params.insert("limit".to_string(), final_limit_str);

        let response: UserLovedTracks = self.fetch("user.getlovedtracks", &base_params).await?;

        Ok(response.lovedtracks.track)
    }

    pub async fn get_user_recent_tracks(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<RecentTrack>, Error> {
        let mut all_tracks: Vec<RecentTrack> = Vec::new();

        let final_limit = limit.unwrap_or(API_MAX_LIMIT);

        if final_limit > API_MAX_LIMIT {
            let needed_chunks = ((final_limit / CHUNK_SIZE) as f32).floor() as u32;

            println!("Needed chunks: {}", needed_chunks);

            for i in 0..needed_chunks {
                let mut all_fetches = Vec::new();

                println!("looping through chunks {}", i);

                for j in 0..CHUNK_MULTIPLIER {
                    println!("looping through chunk multiplier {}", j);

                    let chunk_offset = i * CHUNK_MULTIPLIER + (j + 1);
                    let final_limit_str = API_MAX_LIMIT.to_string();
                    let final_offset_str = chunk_offset.to_string();

                    // Create params inside this iteration to ensure it lives long enough
                    let mut params = self.base_options.clone();
                    params.insert("limit".to_string(), final_limit_str);
                    params.insert("page".to_string(), final_offset_str);

                    // Use async block to extend the lifetime of params
                    let fetch = async move {
                        self.fetch::<UserRecentTracks>("user.getrecenttracks", &params)
                            .await
                    };
                    all_fetches.push(fetch);
                }

                // Await all fetches and collect results
                let chunk_results = futures::future::join_all(all_fetches).await;

                // Process and extend all_tracks with the results
                for result in chunk_results {
                    // Handle potential errors and add tracks
                    match result {
                        Ok(tracks) => all_tracks.extend(tracks.recenttracks.track),
                        Err(e) => return Err(e), // Or handle errors as appropriate
                    }
                }
            }

            // Handle remainder
            let remainder = final_limit % CHUNK_SIZE;
            println!("Remainder: {}", remainder);
            let needed_calls = (remainder as f32 / API_MAX_LIMIT as f32).ceil() as u32;

            let mut all_fetches = Vec::new();

            for i in 0..needed_calls {
                let final_limit_str = API_MAX_LIMIT.to_string();
                let final_offset_str = (CHUNK_MULTIPLIER * needed_chunks + i + 1).to_string();

                let mut params = self.base_options.clone();
                params.insert("limit".to_string(), final_limit_str);
                params.insert("page".to_string(), final_offset_str);

                let fetch = async move {
                    self.fetch::<UserRecentTracks>("user.getrecenttracks", &params)
                        .await
                };
                all_fetches.push(fetch);
            }

            let chunk_results = futures::future::join_all(all_fetches).await;

            for result in chunk_results {
                match result {
                    Ok(tracks) => all_tracks.extend(tracks.recenttracks.track),
                    Err(e) => return Err(e),
                }
            }
        } else {
            let mut base_params: QueryParams = HashMap::new();
            let final_limit_str = final_limit.to_string();

            base_params.insert("limit".to_string(), final_limit_str);

            let response: UserRecentTracks =
                self.fetch("user.getrecenttracks", &base_params).await?;

            all_tracks.extend(response.recenttracks.track);
        }

        // trunc the vector to the final limit
        let final_tracks = all_tracks.into_iter().take(final_limit as usize).collect();

        Ok(final_tracks)
    }

    async fn fetch<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &QueryParams,
    ) -> Result<T, Error> {
        let mut final_params = self.base_options.clone();
        final_params.insert("method".to_string(), method.to_string());
        final_params.extend(params.clone());

        let base_url = self.url.clone().add_args(final_params).build();

        println!("Fetching: {}", base_url);

        let response = reqwest::get(&base_url).await?;

        let parsed_response = response.json::<T>().await?;

        Ok(parsed_response)
    }

    async fn test_fetch(
        &self,
        method: &str,
        params: &QueryParams,
    ) -> Result<UserRecentTracks, Error> {
        let mut final_params = self.base_options.clone();
        final_params.insert("method".to_string(), method.to_string());
        final_params.extend(params.clone());

        let base_url = self.url.clone().add_args(final_params).build();

        println!("[TEST] Fetching: {}", base_url);

        let a: UserRecentTracks = UserRecentTracks {
            recenttracks: RecentTracks {
                track: vec![],
                attr: BaseResponse {
                    user: "tom".to_string(),
                    total: 0,
                    total_pages: 0,
                    page: 0,
                    per_page: 0,
                },
            },
        };

        Ok(a)
    }
}
