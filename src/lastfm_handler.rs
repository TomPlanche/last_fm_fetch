use crate::analytics::AnalysisHandler;
use crate::error::{LastFmError, LastFmErrorResponse, Result};
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{
    ApiRecentTrack, LovedTrack, RecentTrack, Timestamped, TopTrack, UserLovedTracks,
    UserRecentTracks, UserTopTracks,
};
use crate::url_builder::{QueryParams, Url};

use futures::future::join_all;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;

const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

const API_MAX_LIMIT: u32 = 1000;

const CHUNK_MULTIPLIER: u32 = 5;
const CHUNK_SIZE: u32 = API_MAX_LIMIT * CHUNK_MULTIPLIER;

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
    fn as_api_str(self) -> &'static str {
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

impl TrackContainer for UserTopTracks {
    type ApiTrackType = TopTrack;
    type StorageTrackType = TopTrack;

    fn total_tracks(&self) -> u32 {
        self.toptracks.attr.total
    }
    fn tracks(self) -> Vec<Self::ApiTrackType> {
        self.toptracks.track
    }
}

/// Represents a track's play count information
#[derive(Debug, Serialize)]
pub struct TrackPlayInfo {
    pub name: String,
    pub play_count: u32,
    pub artist: String,
    pub album: Option<String>,
    pub image_url: Option<String>,
    pub currently_playing: bool,
    pub date: Option<u32>,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct LastFMHandler {
    url: Url,
    base_options: QueryParams,
}

impl LastFMHandler {
    /// Creates a new `LastFMHandler` instance.
    ///
    /// # Arguments
    /// * `username` - The Last.fm username.
    ///
    /// # Panics
    /// Panics if the environment variable `LAST_FM_API_KEY` is not set.
    ///
    /// # Returns
    /// * `Self` - The created `LastFMHandler` instance.
    #[must_use]
    pub fn new(username: &str) -> Self {
        let mut base_options = QueryParams::new();
        base_options.insert("api_key".to_string(), env::var("LAST_FM_API_KEY").unwrap());
        base_options.insert("limit".to_string(), API_MAX_LIMIT.to_string());
        base_options.insert("format".to_string(), "json".to_string());
        base_options.insert("user".to_string(), username.to_string());

        let url = Url::new(BASE_URL);

        LastFMHandler { url, base_options }
    }

    /// Get loved tracks for a user.
    ///
    /// # Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// # Errors
    /// Returns an error if the API request fails.
    ///
    /// # Returns
    /// * `Result<Vec<LovedTrack>, Error>` - The fetched tracks.
    pub async fn get_user_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<LovedTrack>> {
        self.get_user_tracks::<UserLovedTracks>("user.getlovedtracks", limit.into(), None)
            .await
    }

    /// Get recent tracks for a user.
    ///
    /// # Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// # Errors
    /// Returns an error if the API request fails.
    ///
    /// # Returns
    /// * `Result<Vec<RecentTrack>, Error>` - The fetched tracks.
    pub async fn get_user_recent_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<RecentTrack>> {
        self.get_user_tracks::<UserRecentTracks>("user.getrecenttracks", limit.into(), None)
            .await
    }

    /// Get top tracks for a user.
    ///
    /// # Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all available top tracks.
    /// * `period` - Optional period filter
    ///   (`Period::Overall`, `Period::SevenDay`, `Period::OneMonth`,
    ///   `Period::ThreeMonth`, `Period::SixMonth`, `Period::TwelveMonth`)
    ///
    /// # Errors
    /// Returns an error if the API request fails.
    ///
    /// # Returns
    /// * `Result<Vec<TopTrack>>` - The fetched tracks.
    pub async fn get_user_top_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        period: Option<Period>,
    ) -> Result<Vec<TopTrack>> {
        let mut params = QueryParams::new();
        if let Some(p) = period {
            params.insert("period".to_string(), p.as_api_str().to_string());
        }

        self.get_user_tracks::<UserTopTracks>("user.gettoptracks", limit.into(), Some(params))
            .await
    }

    /// Get tracks for a user.
    ///
    /// # Arguments
    /// * `method` - The method to call.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// # Returns
    /// * `Result<Vec<T::TrackType>, Error>` - The fetched tracks.
    async fn get_user_tracks<T: DeserializeOwned + TrackContainer>(
        &self,
        method: &str,
        limit: TrackLimit,
        additional_params: Option<QueryParams>,
    ) -> Result<Vec<T::StorageTrackType>> {
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

        println!("Need to fetch {final_limit} tracks");

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

        let chunk_nb = final_limit.div_ceil(CHUNK_SIZE);

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
                        Ok::<_, LastFmError>(
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

    /// Fetch data from the `LastFM` API.
    ///
    /// # Arguments
    /// * `method` - The method to call.
    /// * `params` - The parameters to pass to the API.
    ///
    /// # Returns
    /// * `Result<T, Error>` - The fetched data.
    async fn fetch<T: DeserializeOwned>(&self, method: &str, params: &QueryParams) -> Result<T> {
        let mut final_params = self.base_options.clone();
        final_params.insert("method".to_string(), method.to_string());
        final_params.extend(params.clone());

        let base_url = self.url.clone().add_args(final_params).build();

        let response = reqwest::get(&base_url).await?;

        // Check if the response is an error
        if !response.status().is_success() {
            let error: LastFmErrorResponse = response.json().await?;
            return Err(LastFmError::Api(error));
        }

        // Try to parse the successful response
        let parsed_response = response.json::<T>().await?;
        Ok(parsed_response)
    }

    /// Get and save recent tracks to a file.
    ///
    /// # Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    /// * `format` - The file format to save the tracks in.
    ///
    /// # Errors
    /// * `LastFmError::Api` - If the API returns an error.
    /// * `LastFmError::Io` - If there is an error saving the file.
    ///
    /// # Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the saved file.
    pub async fn get_and_save_recent_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        format: FileFormat,
        filename_prefix: &str,
    ) -> Result<String> {
        let tracks = self.get_user_recent_tracks(limit).await?;
        println!("Saving {} tracks to file", tracks.len());
        let filename =
            FileHandler::save(&tracks, &format, filename_prefix).map_err(LastFmError::Io)?;
        Ok(filename)
    }

    /// Get and save loved tracks to a file.
    ///
    /// # Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    /// * `format` - The file format to save the tracks in.
    ///
    /// # Errors
    /// * `FileError` - If there was an error reading or writing the file
    /// * `InvalidUtf8` - If the file path is not valid UTF-8
    ///
    /// # Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the saved file.
    pub async fn get_and_save_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        format: FileFormat,
    ) -> Result<String> {
        let tracks = self.get_user_loved_tracks(limit).await?;
        let filename =
            FileHandler::save(&tracks, &format, "loved_tracks").map_err(LastFmError::Io)?;
        Ok(filename)
    }

    /// Get recent tracks for a user since a given timestamp.
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp to fetch tracks since.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// # Errors
    /// * `FileError` - If there was an error reading or writing the file
    /// * `InvalidUtf8` - If the file path is not valid UTF-8
    ///
    /// # Returns
    /// * `Vec<RecentTrack>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_recent_tracks_since(
        &self,
        timestamp: i64,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<RecentTrack>> {
        let mut params = QueryParams::new();
        params.insert("from".to_string(), timestamp.to_string());

        self.get_user_tracks::<UserRecentTracks>("user.getrecenttracks", limit.into(), Some(params))
            .await
    }

    /// Get loved tracks for a user since a given timestamp.
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp to fetch tracks since.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// # Errors
    /// * `FileError` - If there was an error reading or writing the file
    /// * `InvalidUtf8` - If the file path is not valid UTF-8
    ///
    /// # Returns
    /// * `Vec<LovedTrack>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_loved_tracks_since(
        &self,
        timestamp: u32,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<LovedTrack>> {
        let tracks = self.get_user_loved_tracks(limit).await?;

        Ok(tracks
            .into_iter()
            .filter(|track| track.date.uts > timestamp)
            .collect())
    }

    /// Update a tracks file with new tracks.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to update.
    /// * `fetch_since` - Function to fetch tracks since a given timestamp.
    ///
    /// # Errors
    /// * `FileError` - If there was an error reading or writing the file
    ///
    /// # Panics
    /// * If the file path is not valid UTF-8
    ///
    /// # Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the updated file.
    #[allow(dead_code)]
    pub async fn update_tracks_file<T: DeserializeOwned + Serialize + Timestamped>(
        &self,
        file_path: &Path,
    ) -> Result<String> {
        // Get the most recent timestamp from the file
        let last_timestamp =
            AnalysisHandler::get_most_recent_timestamp::<T>(file_path)?.unwrap_or(0);

        // Find the recent tracks in the file
        let recent_tracks = self
            .get_user_recent_tracks_since(last_timestamp, None)
            .await?;

        let file_path_str = file_path.to_str().unwrap();

        // Append the new tracks to the file
        let updated_file = FileHandler::append(&recent_tracks, file_path_str)?;

        Ok(updated_file)
    }

    /// Export play counts for the last X songs with additional track information
    ///
    /// # Arguments
    /// * `limit` - Number of recent tracks to analyze
    /// * `file_path` - Path to the file to save the play counts to
    ///
    /// # Errors
    /// * `FileError` - If there was an error reading or writing the file
    ///
    /// # Returns
    /// * `Result<String>` - Path to the saved JSON file containing play counts
    pub async fn export_recent_play_counts(&self, limit: impl Into<TrackLimit>) -> Result<String> {
        // Get recent tracks
        let tracks = self.get_user_recent_tracks(limit.into()).await?;

        // Count plays and collect track info
        let mut play_counts: HashMap<String, TrackPlayInfo> = HashMap::new();

        for track in tracks {
            let entry = play_counts
                .entry(track.name.clone())
                .or_insert(TrackPlayInfo {
                    name: track.name.clone(),
                    play_count: 0,
                    artist: track.artist.text.clone(),
                    album: Some(track.album.text.clone()),
                    image_url: track
                        .image
                        .iter()
                        .find(|img| img.size == "large")
                        .map(|img| img.text.clone())
                        .or_else(|| track.image.first().map(|img| img.text.clone())),
                    currently_playing: track.attr.is_some_and(|attr| attr.nowplaying == "true"),
                    date: track.date.map(|date| date.uts),
                    url: track.url,
                });

            entry.play_count += 1;
        }

        // Convert HashMap values into a Vec
        let play_counts_vec: Vec<TrackPlayInfo> = play_counts.into_values().collect();

        // Save to file
        let filename = FileHandler::save(&[play_counts_vec], &FileFormat::Json, "play_counts")
            .map_err(LastFmError::Io)?;

        Ok(filename)
    }

    /// Update or create a file with play counts for the last X songs with additional track information
    ///
    /// # Arguments
    /// * `limit` - Number of recent tracks to analyze
    /// * `file_path` - Path to the file to update/create
    ///
    /// # Errors
    /// * `LastFmError::Api` - If the API returns an error
    /// * `LastFmError::Io` - If there is an error reading or writing the file
    ///
    /// # Returns
    /// * `Result<String>` - Path to the updated/created JSON file containing play counts
    pub async fn update_recent_play_counts(
        &self,
        limit: impl Into<TrackLimit>,
        file_path: &str,
    ) -> Result<String> {
        // Get recent tracks
        let tracks = self.get_user_recent_tracks(limit.into()).await?;

        // Count plays and collect track info
        let mut play_counts: HashMap<String, TrackPlayInfo> = HashMap::new();

        for track in tracks {
            let entry = play_counts
                .entry(track.name.clone())
                .or_insert(TrackPlayInfo {
                    name: track.name.clone(),
                    play_count: 0,
                    artist: track.artist.text.clone(),
                    album: Some(track.album.text.clone()),
                    image_url: track
                        .image
                        .iter()
                        .find(|img| img.size == "extralarge") // Best size for album art
                        .map(|img| img.text.clone())
                        .or_else(|| track.image.first().map(|img| img.text.clone())),
                    currently_playing: track
                        .attr
                        .as_ref()
                        .is_some_and(|val| val.nowplaying == "true"),
                    date: track.date.map(|date| date.uts),
                    url: track.url,
                });

            entry.play_count += 1;
        }

        // Convert HashMap values into a Vec
        let play_counts_vec: Vec<TrackPlayInfo> = play_counts.into_values().collect();

        // Create the file (overwriting if it exists)
        let file = File::create(file_path).map_err(LastFmError::Io)?;
        serde_json::to_writer_pretty(file, &play_counts_vec).map_err(LastFmError::Parse)?;

        Ok(file_path.to_string())
    }

    /// Check if the user is currently playing a track
    ///
    /// # Errors
    /// * `LastFmError` - If there was an error communicating with Last.fm
    ///
    /// # Returns
    /// * `Result<Option<RecentTrack>>` - The currently playing track if any
    pub async fn is_currently_playing(&self) -> Result<Option<RecentTrack>> {
        let mut params = QueryParams::new();
        params.insert("limit".to_string(), "1".to_string());

        let tracks = self
            .get_user_tracks::<UserRecentTracks>(
                "user.getrecenttracks",
                TrackLimit::Limited(1),
                Some(params),
            )
            .await?;

        // Check if the first track has the "now playing" attribute
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

    /// Update a file with the currently playing track information
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to update
    ///
    /// # Errors
    /// * `LastFmError::Api` - If the API returns an error
    /// * `LastFmError::Io` - If there is an error reading or writing the file
    /// * `LastFmError::Parse` - If there is an error parsing the JSON
    ///
    /// # Returns
    /// * `Result<Option<RecentTrack>>` - The currently playing track if any
    pub async fn update_currently_listening(&self, file_path: &str) -> Result<Option<RecentTrack>> {
        let current_track = self.is_currently_playing().await?;

        // Create or overwrite the file
        let file = File::create(file_path).map_err(LastFmError::Io)?;

        if let Some(track) = &current_track {
            serde_json::to_writer_pretty(file, track).map_err(LastFmError::Parse)?;
        } else {
            // Write an empty object when no track is playing
            serde_json::to_writer_pretty(file, &serde_json::json!({}))
                .map_err(LastFmError::Parse)?;
        }

        Ok(current_track)
    }
}
