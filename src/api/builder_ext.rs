//! Extension traits shared across all API request builders.
//!
//! Import these traits to access the `limit`, `unlimited`, `fetch_and_save`,
//! `fetch_and_save_sqlite`, `fetch_and_update`, `fetch_and_update_sqlite`,
//! `analyze`, and `analyze_and_print` methods on any builder type.

use crate::file_handler::{FileFormat, FileHandler};
use crate::types::Timestamped;

/// Extension trait providing `limit` and `unlimited` builder methods.
///
/// Import this trait to set a fetch limit on any request builder.
///
/// # Example
/// ```rust,ignore
/// use lastfm_client::{LastFmClient, prelude::*};
///
/// let tracks = client.top_tracks("username")
///     .limit(50)
///     .fetch()
///     .await?;
/// ```
pub trait LimitBuilder: Sized {
    /// Returns a mutable reference to the builder's `limit` field.
    ///
    /// Implement this to opt-in to the default `limit` and `unlimited` methods.
    #[doc(hidden)]
    fn limit_mut(&mut self) -> &mut Option<u32>;

    /// Set the maximum number of items to fetch.
    ///
    /// # Arguments
    /// * `n` - Maximum number of items. The Last.fm API supports up to thousands of items.
    ///   Use `unlimited()` to fetch everything.
    #[must_use]
    fn limit(mut self, n: u32) -> Self {
        *self.limit_mut() = Some(n);

        self
    }

    /// Fetch all available items (no limit).
    #[must_use]
    fn unlimited(mut self) -> Self {
        *self.limit_mut() = None;

        self
    }
}

/// Extension trait providing `fetch_and_save` and `fetch_and_save_sqlite` for request builders.
#[allow(async_fn_in_trait)]
///
/// Import this trait to save fetched data directly to a file or `SQLite` database.
///
/// # Example
/// ```rust,ignore
/// use lastfm_client::{LastFmClient, FetchAndSave};
///
/// let path = client.top_tracks("username")
///     .fetch_and_save(FileFormat::Json, "top_tracks")
///     .await?;
/// ```
pub trait FetchAndSave: Sized {
    /// The item type produced by this builder.
    type Item: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;

    /// A human-readable label used in log messages (e.g. `"top tracks"`).
    fn resource_label() -> &'static str;

    /// Return the most recent timestamp from the given items, used to write a sidecar file
    /// after saving. Return `None` (the default) if the item type has no timestamp.
    fn latest_timestamp(_items: &[Self::Item]) -> Option<u32> {
        None
    }

    /// Execute the underlying fetch and return items.
    ///
    /// This internal method is called by the provided `fetch_and_save` default implementations.
    /// Use the builder's own `fetch` method for direct access.
    #[doc(hidden)]
    async fn do_fetch(self) -> crate::error::Result<Vec<Self::Item>>;

    /// Fetch items and save them to a file.
    ///
    /// # Arguments
    /// * `format` - The file format to save the items in
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the file
    /// cannot be saved.
    ///
    /// # Returns
    /// * `Result<String>` - The filename of the saved file
    async fn fetch_and_save(
        self,
        format: FileFormat,
        filename_prefix: &str,
    ) -> crate::error::Result<String> {
        let items = self.do_fetch().await?;
        tracing::info!("Saving {} {} to file", items.len(), Self::resource_label());

        let filename = FileHandler::save(&items, &format, filename_prefix)
            .map_err(crate::error::LastFmError::Io)?;

        if let Some(latest_ts) = Self::latest_timestamp(&items) {
            FileHandler::write_sidecar_timestamp(&filename, latest_ts)
                .map_err(crate::error::LastFmError::Io)?;
        }

        Ok(filename)
    }

    /// Fetch items and save them to a new `SQLite` database file.
    ///
    /// # Arguments
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the
    /// database cannot be saved.
    ///
    /// # Returns
    /// * `Result<String>` - Path to the saved database file
    #[cfg(feature = "sqlite")]
    async fn fetch_and_save_sqlite(self, filename_prefix: &str) -> crate::error::Result<String>
    where
        Self::Item: crate::sqlite::SqliteExportable,
    {
        let items = self.do_fetch().await?;

        tracing::info!(
            "Saving {} {} to SQLite",
            items.len(),
            Self::resource_label()
        );

        FileHandler::save_sqlite(&items, filename_prefix).map_err(crate::error::LastFmError::Io)
    }
}

/// Extension trait providing `fetch_and_update` and `fetch_and_update_sqlite` for request
/// builders whose items carry a timestamp.
///
/// Implementations decide how to apply the timestamp - either as a server-side API filter
/// (e.g. `since`) or as an in-memory filter after fetching everything.
#[allow(async_fn_in_trait)]
pub trait FetchAndUpdate: Sized {
    /// The item type produced by this builder.
    type Item: serde::Serialize
        + serde::de::DeserializeOwned
        + Timestamped
        + Clone
        + Send
        + Sync
        + 'static;

    /// Fetch items that are newer than `max_ts`, or all items if `None`.
    ///
    /// Each builder implements this to decide whether to apply the filter on the API side
    /// (more efficient) or in memory after fetching.
    async fn fetch_since(self, max_ts: Option<u32>) -> crate::error::Result<Vec<Self::Item>>;

    /// Fetch only items newer than the most recent entry in an existing file and prepend them
    /// to it. Creates the file if it does not exist.
    ///
    /// The latest timestamp is read from a sidecar file, falling back to scanning the JSON
    /// file itself. CSV and NDJSON files rely exclusively on the sidecar.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the file
    /// cannot be read or written.
    ///
    /// # Returns
    /// * `Result<usize>` - Number of new items prepended
    async fn fetch_and_update(self, file_path: &str) -> crate::error::Result<usize> {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        let is_csv = ext.as_deref() == Some("csv");
        let is_ndjson = ext.as_deref() == Some("ndjson");

        let max_ts = if let Some(ts) = FileHandler::read_sidecar_timestamp(file_path) {
            Some(ts)
        } else if !is_csv && !is_ndjson && std::path::Path::new(file_path).exists() {
            let existing: Vec<Self::Item> =
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

        let new_items = self.fetch_since(max_ts).await?;
        let count = new_items.len();

        if !new_items.is_empty() {
            if let Some(latest_ts) = new_items.first().and_then(Timestamped::get_timestamp) {
                FileHandler::write_sidecar_timestamp(file_path, latest_ts)
                    .map_err(crate::error::LastFmError::Io)?;
            }

            if is_csv {
                FileHandler::append_or_create_csv(&new_items, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            } else if is_ndjson {
                FileHandler::append_or_create_ndjson(&new_items, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            } else {
                FileHandler::prepend_json(&new_items, file_path)
                    .map_err(crate::error::LastFmError::Io)?;
            }
        }

        Ok(count)
    }

    /// Fetch only items newer than the most recent entry in an existing `SQLite` database and
    /// append them to it. Creates the database if it does not exist.
    ///
    /// The latest timestamp is determined by querying `MAX(date_uts)` directly from the
    /// database - no sidecar file is needed.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the response cannot be parsed, or the
    /// database cannot be written.
    ///
    /// # Returns
    /// * `Result<usize>` - Number of new items inserted
    #[cfg(feature = "sqlite")]
    async fn fetch_and_update_sqlite(self, db_path: &str) -> crate::error::Result<usize>
    where
        Self::Item: crate::sqlite::SqliteExportable,
    {
        let max_ts = FileHandler::read_sqlite_max_timestamp(
            db_path,
            <Self::Item as crate::sqlite::SqliteExportable>::table_name(),
        );

        let new_items = self.fetch_since(max_ts).await?;
        let count = new_items.len();

        if !new_items.is_empty() {
            FileHandler::append_or_create_sqlite(&new_items, db_path)
                .map_err(crate::error::LastFmError::Io)?;
        }

        Ok(count)
    }
}

/// Extension trait providing `analyze` and `analyze_and_print` for request builders.
///
/// A blanket implementation is provided for every builder that implements [`FetchAndSave`]
/// whose item type implements [`crate::analytics::TrackAnalyzable`]. Import this trait to
/// call `.analyze(threshold)` and `.analyze_and_print(threshold)` on any qualifying builder.
///
/// # Example
/// ```rust,ignore
/// use lastfm_client::{LastFmClient, Analyze};
///
/// let stats = client.recent_tracks("username")
///     .analyze(5)
///     .await?;
/// ```
#[allow(async_fn_in_trait)]
pub trait Analyze: Sized {
    /// The item type produced by this builder.
    type Item: crate::analytics::TrackAnalyzable;

    /// Execute the underlying fetch and return items for analysis.
    #[doc(hidden)]
    async fn do_fetch_for_analyze(self) -> crate::error::Result<Vec<Self::Item>>;

    /// Fetch items and return play-count statistics.
    ///
    /// # Arguments
    /// * `threshold` - Tracks with fewer plays than this are counted in
    ///   `tracks_below_threshold`.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    async fn analyze(self, threshold: usize) -> crate::error::Result<crate::analytics::TrackStats> {
        let items = self.do_fetch_for_analyze().await?;

        Ok(crate::analytics::AnalysisHandler::analyze_tracks(
            &items, threshold,
        ))
    }

    /// Fetch items, compute statistics, and print them to stdout.
    ///
    /// # Arguments
    /// * `threshold` - Tracks with fewer plays than this are counted in
    ///   `tracks_below_threshold`.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    async fn analyze_and_print(self, threshold: usize) -> crate::error::Result<()> {
        let stats = self.analyze(threshold).await?;

        crate::analytics::AnalysisHandler::print_analysis(&stats);

        Ok(())
    }
}

impl<T> Analyze for T
where
    T: FetchAndSave,
    T::Item: crate::analytics::TrackAnalyzable,
{
    type Item = T::Item;

    async fn do_fetch_for_analyze(self) -> crate::error::Result<Vec<Self::Item>> {
        self.do_fetch().await
    }
}
