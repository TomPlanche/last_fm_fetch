use chrono::Local;
use csv::Writer;
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, Result};

#[allow(dead_code)]
pub enum FileFormat {
    Json,
    Csv,
}

pub struct FileHandler;

impl FileHandler {
    ///
    /// # save
    /// Save data to a file in the data directory.
    ///
    /// ## Arguments
    /// * `data` - Data to save
    /// * `format` - File format to save as
    /// * `filename_prefix` - Prefix for the filename
    ///
    /// ## Returns
    /// * `Result<String>` - Filename of the saved file
    pub fn save<T: Serialize>(
        data: &[T],
        format: FileFormat,
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
            FileFormat::Json => Self::save_as_json(data, &filename),
            FileFormat::Csv => Self::save_as_csv(data, &filename),
        }?;

        Ok(filename)
    }

    ///
    /// # save_as_json
    /// Save data to a JSON file.
    ///
    /// ## Arguments
    /// * `data` - Data to save
    /// * `filename` - Filename to save as
    #[allow(dead_code)]
    fn save_as_json<T: Serialize>(data: &[T], filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        let mut file = File::create(filename)?;

        file.write_all(json.as_bytes())?;

        Ok(())
    }

    ///
    /// # save_as_csv
    /// Save data to a CSV file.
    ///
    /// ## Arguments
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

    ///
    /// # append
    /// Append data to an existing file.
    ///
    /// ## Arguments
    /// * `data` - Data to append
    /// * `file_path` - Path to the file to append to
    ///
    /// ## Returns
    /// * `Result<String>` - Path of the updated file
    ///
    /// # append
    /// Append data to an existing file.
    ///
    /// ## Arguments
    /// * `data` - Data to append
    /// * `file_path` - Path to the file to append to
    ///
    /// ## Returns
    /// * `Result<String>` - Path of the updated file
    #[allow(dead_code)]
    pub fn append<T: Serialize + for<'de> serde::Deserialize<'de> + Clone>(
        data: &[T],
        file_path: &str,
    ) -> Result<String> {
        // Determine file format from extension
        let format = if file_path.ends_with(".json") {
            FileFormat::Json
        } else if file_path.ends_with(".csv") {
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
}
