use chrono::Local;
use csv::Writer;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Result, prelude::*};

#[cfg(feature = "sqlite")]
use rusqlite::Connection as SqliteConnection;

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
        let format = if std::path::Path::new(file_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            FileFormat::Json
        } else if std::path::Path::new(file_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
        {
            FileFormat::Csv
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported file format",
            ));
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
