use chrono::Local;
use csv::Writer;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{prelude::*, Result};

pub enum FileFormat {
    JSON,
    CSV,
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
                FileFormat::JSON => "json",
                FileFormat::CSV => "csv",
            }
        );

        match format {
            FileFormat::JSON => Self::save_as_json(data, &filename),
            FileFormat::CSV => Self::save_as_csv(data, &filename),
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
}
