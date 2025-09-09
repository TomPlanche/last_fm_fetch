use chrono::Local;
use csv::Writer;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, Result};

use crate::lastfm_handler::TrackPlayInfo;

#[allow(dead_code)]
pub enum FileFormat {
    Json,
    Csv,
}

pub struct FileHandler;

impl FileHandler {
    /// Save data to a file in the data directory.
    ///
    /// # Arguments
    /// * `data` - Data to save
    /// * `format` - File format to save as
    /// * `filename_prefix` - Prefix for the filename
    ///
    /// # Errors
    /// * `std::io::Error` - If the file cannot be opened or written to
    /// * `serde_json::Error` - If the JSON cannot be serialized
    ///
    /// # Returns
    /// * `Result<String>` - Filename of the saved file
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
                {
                    if let Some(single_item) = data.first() {
                        Self::save_single(single_item, &filename)?;
                        return Ok(filename);
                    }
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
}
