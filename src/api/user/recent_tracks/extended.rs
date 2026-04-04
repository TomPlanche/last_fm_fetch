//! Extended-track methods on `RecentTracksRequestBuilder`.

use crate::api::constants::METHOD_RECENT_TRACKS;
use crate::api::fetch_utils::{ResourceContainer, fetch};
use crate::error::Result;
use crate::file_handler::FileHandler;
use crate::types::{
    RecentTrackExtended, Timestamped, TrackLimit, TrackList, UserRecentTracksExtended,
};
use crate::url_builder::QueryParams;

use serde::de::DeserializeOwned;

use super::builder::{RecentTracksRequestBuilder, validate_date_range};

impl RecentTracksRequestBuilder {
    /// Fetch tracks with extended information.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The HTTP request fails or the response cannot be parsed
    /// - The date range is invalid (to <= from when both timestamps are set)
    pub async fn fetch_extended(self) -> Result<TrackList<RecentTrackExtended>> {
        validate_date_range(self.from, self.to)?;

        let mut params = self.build_params();
        params.insert("extended".to_string(), "1".to_string());

        let limit = self
            .limit
            .map_or(TrackLimit::Unlimited, TrackLimit::Limited);

        fetch_tracks_extended::<UserRecentTracksExtended>(&self, limit, params)
            .await
            .map(TrackList::from)
    }

    /// Fetch tracks with extended information and save them to a file.
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
    pub async fn fetch_extended_and_save(
        self,
        format: crate::file_handler::FileFormat,
        filename_prefix: &str,
    ) -> Result<String> {
        let tracks = self.fetch_extended().await?;
        tracing::info!("Saving {} recent tracks (extended) to file", tracks.len());

        let filename = FileHandler::save(&tracks, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;

        if let Some(latest_ts) = tracks.first().and_then(Timestamped::get_timestamp) {
            FileHandler::write_sidecar_timestamp(&filename, latest_ts)
                .map_err(crate::error::LastFmError::Io)?;
        }
        Ok(filename)
    }

    /// Fetch only extended tracks newer than the most recent entry in an existing file and
    /// prepend them to it. If the file does not exist, all tracks are fetched and the file is
    /// created.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to update (or create)
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the file
    /// cannot be read or written.
    ///
    /// # Returns
    /// * `Result<usize>` - Number of new tracks prepended
    pub async fn fetch_extended_and_update(self, file_path: &str) -> Result<usize> {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        let is_csv = ext.as_deref() == Some("csv");
        let is_ndjson = ext.as_deref() == Some("ndjson");

        let since_timestamp = if let Some(ts) = FileHandler::read_sidecar_timestamp(file_path) {
            Some(ts)
        } else if !is_csv && !is_ndjson && std::path::Path::new(file_path).exists() {
            let existing: Vec<RecentTrackExtended> =
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

        let builder = match since_timestamp {
            Some(ts) => self.since(i64::from(ts) + 1),
            None => self,
        };

        let new_tracks = builder.fetch_extended().await?;
        let count = new_tracks.len();

        if !new_tracks.is_empty() {
            if let Some(latest_ts) = new_tracks.first().and_then(Timestamped::get_timestamp) {
                FileHandler::write_sidecar_timestamp(file_path, latest_ts)
                    .map_err(crate::error::LastFmError::Io)?;
            }

            if is_csv {
                FileHandler::append_or_create_csv(&new_tracks, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            } else if is_ndjson {
                FileHandler::append_or_create_ndjson(&new_tracks, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            } else {
                FileHandler::prepend_json(&new_tracks, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            }
        }

        Ok(count)
    }

    /// Fetch extended tracks and save them to a new `SQLite` database file.
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
    pub async fn fetch_extended_and_save_sqlite(
        self,
        filename_prefix: &str,
    ) -> crate::error::Result<String> {
        let tracks = self.fetch_extended().await?;

        tracing::info!("Saving {} extended recent tracks to SQLite", tracks.len());

        crate::file_handler::FileHandler::save_sqlite(&tracks, filename_prefix)
            .map_err(crate::error::LastFmError::Io)
    }

    /// Fetch only extended tracks newer than the most recent entry in an existing `SQLite` database
    /// and append them to it. If the database file does not exist, all tracks are fetched and
    /// the database is created.
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
    pub async fn fetch_extended_and_update_sqlite(
        self,
        db_path: &str,
    ) -> crate::error::Result<usize> {
        let since_timestamp = crate::file_handler::FileHandler::read_sqlite_max_timestamp(
            db_path,
            <RecentTrackExtended as crate::sqlite::SqliteExportable>::table_name(),
        );

        let builder = match since_timestamp {
            Some(ts) => self.since(i64::from(ts) + 1),
            None => self,
        };

        let new_tracks = builder.fetch_extended().await?;
        let count = new_tracks.len();

        if !new_tracks.is_empty() {
            crate::file_handler::FileHandler::append_or_create_sqlite(&new_tracks, db_path)
                .map_err(crate::error::LastFmError::Io)?;
        }

        Ok(count)
    }
}

async fn fetch_tracks_extended<T>(
    builder: &RecentTracksRequestBuilder,
    limit: TrackLimit,
    additional_params: QueryParams,
) -> Result<Vec<RecentTrackExtended>>
where
    T: DeserializeOwned + ResourceContainer<ItemType = RecentTrackExtended>,
{
    fetch::<RecentTrackExtended, T>(
        builder.http.clone(),
        builder.config.clone(),
        builder.username.clone(),
        METHOD_RECENT_TRACKS,
        limit,
        additional_params,
        builder.progress_callback.as_ref(),
    )
    .await
}
