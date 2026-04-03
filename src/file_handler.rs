use chrono::Local;
use csv::Writer;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Result, Write as _};

#[cfg(feature = "sqlite")]
use rusqlite::{Connection as SqliteConnection, OpenFlags};

use crate::types::TrackPlayInfo;

/// File format options for saving track data
#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum FileFormat {
    /// Save as JSON format with pretty printing
    Json,
    /// Save as CSV format with headers
    Csv,
    /// Save as NDJSON (Newline Delimited JSON) - one compact JSON object per line
    Ndjson,
}

/// Handler for file I/O operations (JSON and CSV)
#[derive(Debug)]
#[non_exhaustive]
pub struct FileHandler;

impl FileHandler {
    /// Save data to a file in the data directory.
    ///
    /// Files are saved to the `data/` directory (created if it doesn't exist) with a timestamp in the filename.
    ///
    /// # Arguments
    /// * `data` - Data to save (must implement Serialize)
    /// * `format` - File format to save as (`FileFormat::Json` for JSON or `FileFormat::Csv` for CSV)
    /// * `filename_prefix` - Prefix for the filename. The final filename will be `{prefix}_{timestamp}.{extension}`
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be opened or written to, or if the data directory cannot be created
    /// * `serde_json::Error` - If the JSON cannot be serialized
    ///
    /// # Returns
    /// * `Result<String>` - Full path to the saved file (e.g., `data/recent_tracks_20240101_120000.json`)
    pub fn save<T: Serialize>(
        data: &[T],
        format: &FileFormat,
        filename_prefix: &str,
    ) -> Result<String> {
        // Create data directory if it doesn't exist
        fs::create_dir_all("data")?;

        // Generate timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");

        // Create filename with timestamp
        let filename = format!(
            "data/{}_{}.{}",
            filename_prefix,
            timestamp,
            match format {
                FileFormat::Json => "json",
                FileFormat::Csv => "csv",
                FileFormat::Ndjson => "ndjson",
            }
        );

        match format {
            FileFormat::Json => {
                // Special case: if T is a HashMap with track info
                if std::any::type_name::<T>()
                    == std::any::type_name::<HashMap<String, TrackPlayInfo>>()
                    && let Some(single_item) = data.first()
                {
                    Self::save_single(single_item, &filename)?;
                    return Ok(filename);
                }
                Self::save_as_json(data, &filename)
            }
            FileFormat::Csv => Self::save_as_csv(data, &filename),
            FileFormat::Ndjson => Self::save_as_ndjson(data, &filename),
        }?;

        Ok(filename)
    }

    /// Save data to a JSON file.
    ///
    /// # Arguments
    /// * `data` - Data to save
    /// * `filename` - Filename to save as
    #[allow(dead_code)]
    fn save_as_json<T: Serialize>(data: &[T], filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let mut file = File::create(filename)?;

        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Save data to a CSV file.
    ///
    /// # Arguments
    /// * `data` - Data to save
    /// * `filename` - Filename to save as
    fn save_as_csv<T: Serialize>(data: &[T], filename: &str) -> Result<()> {
        let mut writer = Writer::from_path(filename)?;

        for item in data {
            writer.serialize(item)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Save data to an NDJSON file - one compact JSON object per line.
    ///
    /// # Arguments
    /// * `data` - Data to save
    /// * `filename` - Filename to save as
    fn save_as_ndjson<T: Serialize>(data: &[T], filename: &str) -> Result<()> {
        let mut file = File::create(filename)?;
        for item in data {
            let line = serde_json::to_string(item)?;
            file.write_all(line.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    /// Append items to an existing NDJSON file as new lines.
    ///
    /// # Arguments
    /// * `data` - Data to append
    /// * `file_path` - Path to the target file
    fn append_ndjson_lines<T: Serialize>(data: &[T], file_path: &str) -> Result<()> {
        let mut file = OpenOptions::new().append(true).open(file_path)?;
        for item in data {
            let line = serde_json::to_string(item)?;
            file.write_all(line.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    /// Load existing NDJSON data from a file - one item per line.
    ///
    /// # Arguments
    /// * `file_path` - Path to the NDJSON file to read
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be opened
    /// * `serde_json::Error` - If a line cannot be deserialized into `T`
    pub fn load_ndjson<T: serde::de::DeserializeOwned>(file_path: &str) -> Result<Vec<T>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut items = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            let item: T = serde_json::from_str(&line)?;
            items.push(item);
        }
        Ok(items)
    }

    /// Append new items to an existing NDJSON file, or create it if it does not exist.
    ///
    /// # Arguments
    /// * `new_data` - New items to append
    /// * `file_path` - Path to the target NDJSON file
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be opened or written
    pub fn append_or_create_ndjson<T: Serialize>(new_data: &[T], file_path: &str) -> Result<()> {
        if std::path::Path::new(file_path).exists() {
            Self::append_ndjson_lines(new_data, file_path)
        } else {
            if let Some(parent) = std::path::Path::new(file_path).parent() {
                fs::create_dir_all(parent)?;
            }
            Self::save_as_ndjson(new_data, file_path)
        }
    }

    /// Append data to an existing file.
    ///
    /// # Arguments
    /// * `data` - Data to append
    /// * `file_path` - Path to the file to append to
    ///
    /// # Returns
    /// * `Result<String>` - Path of the updated file
    ///
    /// Append data to an existing file.
    ///
    /// # Arguments
    /// * `data` - Data to append
    /// * `file_path` - Path to the file to append to
    ///
    /// # Errors
    /// * `std::io::Error` - If an I/O error occurs
    ///
    /// # Returns
    /// * `Result<String>` - Path of the updated file
    #[allow(dead_code)]
    pub fn append<T: Serialize + for<'de> serde::Deserialize<'de> + Clone>(
        data: &[T],
        file_path: &str,
    ) -> Result<String> {
        // Determine file format from extension
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);

        let format = match ext.as_deref() {
            Some("json") => FileFormat::Json,
            Some("csv") => FileFormat::Csv,
            Some("ndjson") => FileFormat::Ndjson,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Unsupported file format",
                ));
            }
        };

        match format {
            FileFormat::Json => {
                // For JSON, we need to read the existing data, combine it, and write it back
                let file = File::open(file_path)?;
                let mut existing_data: Vec<T> = serde_json::from_reader(file)?;

                existing_data.extend(data.iter().cloned());

                Self::save_as_json(&existing_data, file_path)?;
            }
            FileFormat::Csv => {
                // For CSV, we can simply append to the file
                let mut writer =
                    Writer::from_writer(OpenOptions::new().append(true).open(file_path)?);

                for item in data {
                    writer.serialize(item)?;
                }
                writer.flush()?;
            }
            FileFormat::Ndjson => {
                Self::append_ndjson_lines(data, file_path)?;
            }
        }

        Ok(file_path.to_string())
    }

    /// Save a single item to a JSON file
    ///
    /// # Errors
    /// * `std::io::Error` - If there was an error reading or writing the file
    /// * `serde_json::Error` - If there was an error serializing the data
    ///
    /// # Arguments
    /// * `data` - Data to save
    /// * `filename` - Filename to save as
    pub fn save_single<T: Serialize>(data: &T, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load existing JSON data from a file.
    ///
    /// # Arguments
    /// * `file_path` - Path to the JSON file to read
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be opened
    /// * `serde_json::Error` - If the JSON cannot be deserialized into `Vec<T>`
    pub fn load<T: serde::de::DeserializeOwned>(file_path: &str) -> Result<Vec<T>> {
        let file = File::open(file_path)?;
        let data: Vec<T> = serde_json::from_reader(file)?;
        Ok(data)
    }

    /// Return the path of the sidecar metadata file for `file_path`.
    ///
    /// The sidecar stores the latest known Unix timestamp so subsequent update calls do not
    /// need to deserialize the full data file.
    #[must_use]
    pub fn sidecar_path(file_path: &str) -> String {
        format!("{file_path}.meta")
    }

    /// Read the latest timestamp from a sidecar metadata file.
    ///
    /// Returns `None` if the sidecar does not exist or cannot be parsed.
    #[must_use]
    pub fn read_sidecar_timestamp(file_path: &str) -> Option<u32> {
        fs::read_to_string(Self::sidecar_path(file_path))
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    /// Write a timestamp to the sidecar metadata file associated with `file_path`.
    ///
    /// # Errors
    /// * `std::io::Error` - If the sidecar file cannot be written
    pub fn write_sidecar_timestamp(file_path: &str, timestamp: u32) -> Result<()> {
        fs::write(Self::sidecar_path(file_path), timestamp.to_string())
    }

    /// Append new items to an existing CSV file, or create it with headers if it does not exist.
    ///
    /// When appending to an existing file the header row is omitted so it is not duplicated.
    ///
    /// # Arguments
    /// * `new_data` - New items to append
    /// * `file_path` - Path to the target CSV file
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be opened or written
    /// * `csv::Error` - If serialization fails
    pub fn append_or_create_csv<T: Serialize>(new_data: &[T], file_path: &str) -> Result<()> {
        if std::path::Path::new(file_path).exists() {
            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(OpenOptions::new().append(true).open(file_path)?);
            for item in new_data {
                writer.serialize(item)?;
            }
            writer.flush()?;
        } else {
            if let Some(parent) = std::path::Path::new(file_path).parent() {
                fs::create_dir_all(parent)?;
            }
            Self::save_as_csv(new_data, file_path)?;
        }
        Ok(())
    }

    /// Save data to a new `SQLite` database file.
    ///
    /// Creates a timestamped `.db` file under `data/`. All rows are inserted in a single
    /// transaction for performance.
    ///
    /// # Arguments
    /// * `data` - Data to save (must implement `SqliteExportable`)
    /// * `filename_prefix` - Prefix for the generated filename
    ///
    /// # Errors
    /// * `std::io::Error` - If the data directory cannot be created or the database cannot be opened or written
    ///
    /// # Returns
    /// * `Result<String>` - Full path to the saved database file (e.g., `data/recent_tracks_20240101_120000.db`)
    #[cfg(feature = "sqlite")]
    pub fn save_sqlite<T: crate::sqlite::SqliteExportable>(
        data: &[T],
        filename_prefix: &str,
    ) -> Result<String> {
        fs::create_dir_all("data")?;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("data/{filename_prefix}_{timestamp}.db");

        let mut conn =
            SqliteConnection::open(&filename).map_err(|e| std::io::Error::other(e.to_string()))?;

        conn.execute_batch(T::create_table_sql())
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        {
            let mut stmt = tx
                .prepare(T::insert_sql())
                .map_err(|e| std::io::Error::other(e.to_string()))?;

            for item in data {
                item.bind_and_execute(&mut stmt)
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        Ok(filename)
    }

    /// Append new items to an existing `SQLite` database, or create it if it does not exist.
    ///
    /// Opens the database at `file_path`, creates the table if it does not already exist,
    /// and inserts all rows in a single transaction.
    ///
    /// # Arguments
    /// * `data` - Data to insert
    /// * `file_path` - Path to the target `.db` file
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be opened or the data cannot be written
    #[cfg(feature = "sqlite")]
    pub fn append_or_create_sqlite<T: crate::sqlite::SqliteExportable>(
        data: &[T],
        file_path: &str,
    ) -> Result<()> {
        if let Some(parent) = std::path::Path::new(file_path).parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let mut conn =
            SqliteConnection::open(file_path).map_err(|e| std::io::Error::other(e.to_string()))?;

        conn.execute_batch(T::create_table_sql())
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        {
            let mut stmt = tx
                .prepare(T::insert_sql())
                .map_err(|e| std::io::Error::other(e.to_string()))?;

            for item in data {
                item.bind_and_execute(&mut stmt)
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        Ok(())
    }

    /// Load all rows from a `SQLite` database into a [`crate::types::TrackList`].
    ///
    /// Opens the database at `file_path` and runs `T::select_sql()`, mapping
    /// each row with `T::from_row`. The returned `TrackList<T>` supports all
    /// analysis methods (`to_set()`, `top_artists()`, `by_date()`, etc.).
    ///
    /// Fields that are not persisted in the database schema (such as `image`,
    /// `streamable`, and human-readable date strings) are reconstructed with
    /// empty/default values. See [`crate::sqlite::SqliteLoadable`] for details.
    ///
    /// # Arguments
    /// * `file_path` - Path to the `.db` file produced by `fetch_and_save_sqlite`
    ///   or `fetch_and_update_sqlite`. Relative paths are resolved from the **process
    ///   current working directory** (for `cargo run`, that is normally the package
    ///   root where `Cargo.toml` lives, not `target/release/`).
    ///
    /// # Errors
    /// * `std::io::Error` - If the database cannot be opened or the query fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// use lastfm_client::{file_handler::FileHandler, RecentTrack};
    ///
    /// let tracks = FileHandler::load_sqlite::<RecentTrack>("data/recent_tracks.db")?;
    /// let top = tracks.to_set();        // TrackList<ScoredTrack>
    /// let artists = tracks.top_artists(); // TrackList<ScoredArtist>
    /// println!("Streak: {} day(s)", tracks.streak());
    /// ```
    #[cfg(feature = "sqlite")]
    pub fn load_sqlite<T: crate::sqlite::SqliteLoadable>(
        file_path: &str,
    ) -> std::io::Result<crate::types::TrackList<T>> {
        let path = std::path::Path::new(file_path);
        if !path.is_file() {
            let cwd = std::env::current_dir()
                .map_or_else(|_| "<unavailable>".to_string(), |p| p.display().to_string());
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("SQLite database not found at {file_path:?} (resolved from cwd {cwd:?})"),
            ));
        }

        let conn = SqliteConnection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        let mut stmt = conn
            .prepare(T::select_sql())
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| T::from_row(row))
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let items: rusqlite::Result<Vec<T>> = rows.collect();
        let items = items.map_err(|e| std::io::Error::other(e.to_string()))?;

        Ok(crate::types::TrackList::from(items))
    }

    /// Query the maximum `date_uts` value stored in a `SQLite` table.
    ///
    /// Used by the update flow to determine the latest timestamp already present in the
    /// database, so only newer records need to be fetched from the API.
    ///
    /// Returns `None` if the file does not exist, the table is empty, or the query fails.
    ///
    /// # Arguments
    /// * `file_path` - Path to the `.db` file
    /// * `table_name` - Name of the table to query
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn read_sqlite_max_timestamp(file_path: &str, table_name: &str) -> Option<u32> {
        if !std::path::Path::new(file_path).exists() {
            return None;
        }
        let conn = SqliteConnection::open(file_path).ok()?;
        conn.query_row(
            &format!("SELECT MAX(date_uts) FROM {table_name}"),
            [],
            |row| row.get::<_, Option<u32>>(0),
        )
        .ok()
        .flatten()
    }

    /// Prepend new items to an existing JSON file, or create the file if it does not exist.
    ///
    /// New items are placed before existing items so the result remains sorted newest-first,
    /// which matches the order returned by the Last.fm API.
    ///
    /// # Arguments
    /// * `new_data` - New items to prepend
    /// * `file_path` - Path to the target JSON file
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be read or written
    /// * `serde_json::Error` - If serialization or deserialization fails
    pub fn prepend_json<T: Serialize + serde::de::DeserializeOwned + Clone>(
        new_data: &[T],
        file_path: &str,
    ) -> Result<()> {
        let existing: Vec<T> = if std::path::Path::new(file_path).exists() {
            Self::load(file_path)?
        } else {
            // Ensure the parent directory exists before creating the file
            if let Some(parent) = std::path::Path::new(file_path).parent() {
                fs::create_dir_all(parent)?;
            }
            vec![]
        };

        let mut combined = new_data.to_vec();
        combined.extend(existing);
        Self::save_as_json(&combined, file_path)
    }
}
