use crate::analytics::AnalysisHandler;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::{LovedTrack, Timestamped, TrackLimit, UserLovedTracks};

use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

use super::fetch_utils::{ResourceContainer, fetch};

/// Client for fetching loved tracks
pub struct LovedTracksClient {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
}

impl fmt::Debug for LovedTracksClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LovedTracksClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl LovedTracksClient {
    /// Create a new loved tracks client
    pub fn new(http: Arc<dyn HttpClient>, config: Arc<Config>) -> Self {
        Self { http, config }
    }

    /// Create a builder for loved tracks requests
    pub fn builder(&self, username: impl Into<String>) -> LovedTracksRequestBuilder {
        LovedTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }
}

/// Builder for loved tracks requests
pub struct LovedTracksRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
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
    fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
        }
    }

    /// Set the maximum number of tracks to fetch
    ///
    /// # Arguments
    /// * `limit` - Maximum number of tracks to fetch. The Last.fm API supports fetching up to thousands of tracks.
    ///   If you need all tracks, use `unlimited()` instead.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Fetch all available tracks (no limit)
    #[must_use]
    pub const fn unlimited(mut self) -> Self {
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
        tracing::info!("Saving {} loved tracks to file", tracks.len());
        let filename = FileHandler::save(&tracks, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;
        if let Some(latest_ts) = tracks
            .first()
            .and_then(crate::types::Timestamped::get_timestamp)
        {
            FileHandler::write_sidecar_timestamp(&filename, latest_ts)
                .map_err(crate::error::LastFmError::Io)?;
        }
        Ok(filename)
    }

    /// Fetch tracks and save them to a new `SQLite` database file.
    ///
    /// # Arguments
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the database cannot be saved.
    ///
    /// # Returns
    /// * `Result<String>` - Path to the saved database file
    #[cfg(feature = "sqlite")]
    pub async fn fetch_and_save_sqlite(
        self,
        filename_prefix: &str,
    ) -> crate::error::Result<String> {
        let tracks = self.fetch().await?;
        tracing::info!("Saving {} loved tracks to SQLite", tracks.len());
        crate::file_handler::FileHandler::save_sqlite(&tracks, filename_prefix)
            .map_err(crate::error::LastFmError::Io)
    }

    /// Fetch only tracks newer than the most recent entry in an existing `SQLite` database and
    /// append them to it. If the database file does not exist, all tracks are fetched and
    /// the database is created.
    ///
    /// Because the loved tracks API does not support a `from` timestamp filter, all tracks
    /// are fetched and those already present (by `date_uts`) are filtered out before inserting.
    ///
    /// # Arguments
    /// * `db_path` - Path to the `SQLite` database file to update (or create)
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the database cannot be written.
    ///
    /// # Returns
    /// * `Result<usize>` - Number of new tracks inserted
    #[cfg(feature = "sqlite")]
    pub async fn fetch_and_update_sqlite(self, db_path: &str) -> crate::error::Result<usize> {
        let max_existing_ts = crate::file_handler::FileHandler::read_sqlite_max_timestamp(
            db_path,
            <LovedTrack as crate::sqlite::SqliteExportable>::table_name(),
        );

        let all_tracks = self.fetch().await?;

        let new_tracks: Vec<LovedTrack> = match max_existing_ts {
            Some(max_ts) => all_tracks
                .into_iter()
                .filter(|t| t.date.uts > max_ts)
                .collect(),
            None => all_tracks,
        };

        let count = new_tracks.len();

        if !new_tracks.is_empty() {
            crate::file_handler::FileHandler::append_or_create_sqlite(&new_tracks, db_path)
                .map_err(crate::error::LastFmError::Io)?;
        }

        Ok(count)
    }

    /// Fetch only tracks newer than the most recent entry in an existing JSON file and prepend
    /// them to it. If the file does not exist, all tracks are fetched and the file is created.
    ///
    /// Unlike `recent_tracks`, the loved tracks API does not support a `from` timestamp filter,
    /// so all tracks are fetched and those already present (by timestamp) are filtered out.
    ///
    /// # Arguments
    /// * `file_path` - Path to the JSON file to update (or create)
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the file
    /// cannot be read or written.
    ///
    /// # Returns
    /// * `Result<usize>` - Number of new tracks prepended
    pub async fn fetch_and_update(self, file_path: &str) -> Result<usize> {
        let is_csv = std::path::Path::new(file_path)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("csv"));

        let max_existing_ts = if let Some(ts) = FileHandler::read_sidecar_timestamp(file_path) {
            Some(ts)
        } else if !is_csv && std::path::Path::new(file_path).exists() {
            let existing: Vec<LovedTrack> =
                FileHandler::load(file_path).map_err(crate::error::LastFmError::Io)?;
            let ts = existing.iter().filter_map(Timestamped::get_timestamp).max();
            if let Some(t) = ts {
                FileHandler::write_sidecar_timestamp(file_path, t)
                    .map_err(crate::error::LastFmError::Io)?;
            }
            ts
        } else {
            None
        };

        let all_tracks = self.fetch().await?;

        let new_tracks: Vec<LovedTrack> = match max_existing_ts {
            Some(max_ts) => all_tracks
                .into_iter()
                .filter(|t| t.get_timestamp().is_some_and(|ts| ts > max_ts))
                .collect(),
            None => all_tracks,
        };

        let count = new_tracks.len();

        if !new_tracks.is_empty() {
            if let Some(latest_ts) = new_tracks
                .first()
                .and_then(crate::types::Timestamped::get_timestamp)
            {
                FileHandler::write_sidecar_timestamp(file_path, latest_ts)
                    .map_err(crate::error::LastFmError::Io)?;
            }
            if is_csv {
                FileHandler::append_or_create_csv(&new_tracks, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            } else {
                FileHandler::prepend_json(&new_tracks, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            }
        }

        Ok(count)
    }

    /// Analyze tracks and return statistics
    ///
    /// # Arguments
    /// * `threshold` - Minimum play count threshold. Tracks with fewer plays than this value will be
    ///   counted separately in `tracks_below_threshold`. For example, use 5 to identify
    ///   tracks played less than 5 times.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    ///
    /// # Returns
    /// * `Result<crate::analytics::TrackStats>` - Analysis results including play counts, most played tracks, etc.
    pub async fn analyze(self, threshold: usize) -> Result<crate::analytics::TrackStats> {
        let tracks = self.fetch().await?;
        Ok(AnalysisHandler::analyze_tracks(&tracks, threshold))
    }

    /// Analyze tracks and print statistics
    ///
    /// # Arguments
    /// * `threshold` - Minimum play count threshold. Tracks with fewer plays than this value will be
    ///   counted separately. For example, use 5 to identify tracks played less than 5 times.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn analyze_and_print(self, threshold: usize) -> Result<()> {
        let stats = self.analyze(threshold).await?;
        AnalysisHandler::print_analysis(&stats);
        Ok(())
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
            "user.getlovedtracks",
            limit,
            QueryParams::new(),
            None,
        )
        .await
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
