use crate::analytics::AnalysisHandler;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::*;
use crate::url_builder::{QueryParams, Url};

use futures::future::join_all;
use reqwest::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::path::Path;

const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

const API_MAX_LIMIT: u32 = 1000;

const CHUNK_MULTIPLIER: u32 = 5;
const CHUNK_SIZE: u32 = API_MAX_LIMIT * CHUNK_MULTIPLIER;

#[derive(Debug, Clone, Copy)]
pub enum TrackLimit {
    Limited(u32),
    Unlimited,
}

impl From<Option<u32>> for TrackLimit {
    fn from(opt: Option<u32>) -> Self {
        match opt {
            Some(limit) => TrackLimit::Limited(limit),
            None => TrackLimit::Unlimited,
        }
    }
}

trait TrackContainer {
    type ApiTrackType;
    type StorageTrackType: From<Self::ApiTrackType>;

    fn total_tracks(&self) -> u32;
    fn tracks(self) -> Vec<Self::ApiTrackType>;
}

impl TrackContainer for UserLovedTracks {
    type ApiTrackType = LovedTrack; // No change needed for LovedTracks
    type StorageTrackType = LovedTrack; // No change needed for LovedTracks

    fn total_tracks(&self) -> u32 {
        self.lovedtracks.attr.total
    }
    fn tracks(self) -> Vec<Self::ApiTrackType> {
        self.lovedtracks.track
    }
}

impl TrackContainer for UserRecentTracks {
    type ApiTrackType = ApiRecentTrack;
    type StorageTrackType = RecentTrack;

    fn total_tracks(&self) -> u32 {
        self.recenttracks.attr.total
    }
    fn tracks(self) -> Vec<Self::ApiTrackType> {
        self.recenttracks.track
    }
}

#[derive(Debug)]
pub struct LastFMHandler {
    url: Url,
    base_options: QueryParams,
}

impl LastFMHandler {
    pub fn new(username: &str) -> Self {
        let mut base_options = QueryParams::new();
        base_options.insert("api_key".to_string(), env::var("LAST_FM_API_KEY").unwrap());
        base_options.insert("limit".to_string(), API_MAX_LIMIT.to_string());
        base_options.insert("format".to_string(), "json".to_string());
        base_options.insert("user".to_string(), username.to_string());

        let url = Url::new(BASE_URL);

        LastFMHandler { url, base_options }
    }

    ///
    /// # get_user_loved_tracks
    /// Get loved tracks for a user.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Result<Vec<LovedTrack>, Error>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<LovedTrack>, Error> {
        self.get_user_tracks::<UserLovedTracks>("user.getlovedtracks", limit.into(), None)
            .await
    }

    ///
    /// # get_user_recent_tracks
    /// Get recent tracks for a user.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Result<Vec<RecentTrack>, Error>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_recent_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<RecentTrack>, Error> {
        self.get_user_tracks::<UserRecentTracks>("user.getrecenttracks", limit.into(), None)
            .await
    }

    ///
    /// # get_user_tracks
    /// Get tracks for a user.
    ///
    /// ## Arguments
    /// * `method` - The method to call.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Result<Vec<T::TrackType>, Error>` - The fetched tracks.
    async fn get_user_tracks<T: DeserializeOwned + TrackContainer>(
        &self,
        method: &str,
        limit: TrackLimit,
        additional_params: Option<QueryParams>,
    ) -> Result<Vec<T::StorageTrackType>, Error> {
        let mut params = self.base_options.clone();
        if let Some(additional_params) = additional_params {
            params.extend(additional_params);
        }

        // Make an initial request to get the total number of tracks
        let mut base_params: QueryParams = HashMap::new();
        base_params.insert("limit".to_string(), "1".to_string());
        base_params.insert("page".to_string(), "1".to_string());
        base_params.extend(params.clone());

        let initial_response: T = self.fetch(method, &base_params).await?;
        let total_tracks = initial_response.total_tracks();

        let final_limit = match limit {
            TrackLimit::Limited(l) => l.min(total_tracks),
            TrackLimit::Unlimited => total_tracks,
        };

        println!("Need to fetch {} tracks", final_limit);

        if final_limit <= API_MAX_LIMIT {
            // If we need less than the API limit, just make a single request
            let mut base_params: QueryParams = HashMap::new();
            base_params.insert("limit".to_string(), final_limit.to_string());
            base_params.insert("page".to_string(), "1".to_string());
            base_params.extend(params);

            let response: T = self.fetch(method, &base_params).await?;
            return Ok(response
                .tracks()
                .into_iter()
                .take(final_limit as usize)
                .map(T::StorageTrackType::from)
                .collect());
        }

        let chunk_nb = (final_limit as f32 / CHUNK_SIZE as f32).ceil() as u32;

        let mut all_tracks = Vec::new();

        // Process chunks sequentially
        for chunk_index in 0..chunk_nb {
            println!("Processing chunk {}/{}", chunk_index + 1, chunk_nb);
            let chunk_params = params.clone();

            // Calculate how many API calls we need for this chunk
            let chunk_api_calls = if chunk_index == chunk_nb - 1 {
                // Last chunk
                final_limit % CHUNK_SIZE / API_MAX_LIMIT + 1
            } else {
                CHUNK_SIZE / API_MAX_LIMIT
            };

            // Create futures for concurrent API calls within this chunk
            let api_call_futures: Vec<_> = (0..chunk_api_calls)
                .map(|call_index| {
                    let mut call_params = chunk_params.clone();
                    let call_limit =
                        (final_limit - chunk_index * CHUNK_SIZE - call_index * API_MAX_LIMIT)
                            .min(API_MAX_LIMIT);

                    let page = chunk_index * CHUNK_SIZE / API_MAX_LIMIT + call_index + 1;

                    call_params.insert("limit".to_string(), call_limit.to_string());
                    call_params.insert("page".to_string(), page.to_string());

                    async move {
                        let response: T = self.fetch(method, &call_params).await?;
                        Ok::<_, Error>(
                            response
                                .tracks()
                                .into_iter()
                                .take(call_limit as usize)
                                .map(T::StorageTrackType::from)
                                .collect::<Vec<_>>(),
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

    ///
    /// # fetch
    /// Fetch data from the LastFM API.
    ///
    /// ## Arguments
    /// * `method` - The method to call.
    /// * `params` - The parameters to pass to the API.
    ///
    /// ## Returns
    /// * `Result<T, Error>` - The fetched data.
    async fn fetch<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &QueryParams,
    ) -> Result<T, Error> {
        let mut final_params = self.base_options.clone();
        final_params.insert("method".to_string(), method.to_string());
        final_params.extend(params.clone());

        let base_url = self.url.clone().add_args(final_params).build();

        let response = reqwest::get(&base_url).await?;
        let parsed_response = response.json::<T>().await?;

        Ok(parsed_response)
    }

    ///
    /// # get_and_save_recent_tracks
    /// Get and save recent tracks to a file.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    /// * `format` - The file format to save the tracks in.
    ///
    /// ## Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the saved file.
    #[allow(dead_code)]
    pub async fn get_and_save_recent_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        format: FileFormat,
        filename_prefix: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tracks = self.get_user_recent_tracks(limit).await?;

        println!("Saving {} tracks to file", tracks.len());

        let filename = FileHandler::save(&tracks, format, filename_prefix)?;
        Ok(filename)
    }

    ///
    /// # get_and_save_loved_tracks
    /// Get and save loved tracks to a file.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    /// * `format` - The file format to save the tracks in.
    ///
    /// ## Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the saved file.
    #[allow(dead_code)]
    pub async fn get_and_save_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        format: FileFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tracks = self.get_user_loved_tracks(limit).await?;
        let filename = FileHandler::save(&tracks, format, "loved_tracks")?;
        Ok(filename)
    }

    ///
    /// # `get_user_recent_tracks_since`
    /// Get recent tracks for a user since a given timestamp.
    ///
    /// ## Arguments
    /// * `timestamp` - The timestamp to fetch tracks since.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Vec<RecentTrack>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_recent_tracks_since(
        &self,
        timestamp: u32,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<RecentTrack>, Error> {
        let mut params = QueryParams::new();
        params.insert("from".to_string(), timestamp.to_string());

        self.get_user_tracks::<UserRecentTracks>("user.getrecenttracks", limit.into(), Some(params))
            .await
    }

    ///
    /// # `get_user_loved_tracks_since`
    /// Get loved tracks for a user since a given timestamp.
    ///
    /// ## Arguments
    /// * `timestamp` - The timestamp to fetch tracks since.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Vec<LovedTrack>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_loved_tracks_since(
        &self,
        timestamp: u32,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<LovedTrack>, Error> {
        let tracks = self.get_user_loved_tracks(limit).await?;

        Ok(tracks
            .into_iter()
            .filter(|track| track.date.uts > timestamp)
            .collect())
    }

    ///
    /// # `update_tracks_file`
    /// Update a tracks file with new tracks.
    ///
    /// ## Arguments
    /// * `file_path` - Path to the file to update.
    /// * `fetch_since` - Function to fetch tracks since a given timestamp.
    ///
    /// ## Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the updated file.
    #[allow(dead_code)]
    pub async fn update_tracks_file<T: DeserializeOwned + Serialize + Timestamped>(
        &self,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Get the most recent timestamp from the file
        let last_timestamp =
            AnalysisHandler::get_most_recent_timestamp::<T>(file_path)?.unwrap_or(0);

        // Find the recent tracks in the file
        let recent_tracks = self
            .get_user_recent_tracks_since(last_timestamp, None)
            .await?;

        let file_path_str = file_path.to_str().unwrap();

        // // Append the new tracks to the file
        let updated_file = FileHandler::append(&recent_tracks, file_path_str)?;

        Ok(updated_file)
    }
}
