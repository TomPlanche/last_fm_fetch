#[path = "analytics.rs"]
mod analytics;

#[path = "file_handler.rs"]
mod file_handler;

#[path = "lastfm_handler.rs"]
mod lastfm_handler;

#[path = "types.rs"]
mod types;

#[path = "url_builder.rs"]
mod url_builder;

use analytics::{AnalysisHandler, TrackStats};
use dotenv::dotenv;
use file_handler::{FileFormat, FileHandler};
use lastfm_handler::TrackLimit;
use reqwest::Error;
use std::path::Path;
use types::RecentTrack;
// use tabular::{Row, Table};
use url_builder::Url;

/// Error type for application-specific errors
#[derive(Debug)]
pub enum AppError {
    /// Represents an error during API connection
    ApiError(String),
    /// Represents an error during file operations
    FileError(std::io::Error),
    /// Represents an error during data parsing
    ParseError(serde_json::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::ApiError(msg) => write!(f, "API Error: {}", msg),
            AppError::FileError(e) => write!(f, "File Error: {}", e),
            AppError::ParseError(e) => write!(f, "Parse Error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    println!("Creating base URL");
    let base_url = Url::new("https://ws.audioscrobbler.com/2.0/");
    println!("Base URL created: {}", base_url.build());

    let handler = lastfm_handler::LastFMHandler::new(base_url, "tom_planche");

    let all_recent_tracks = handler
        .get_user_recent_tracks(TrackLimit::Limited(30))
        .await?;

    match FileHandler::save(&all_recent_tracks, FileFormat::JSON, "recent_tracks") {
        Ok(filename) => {
            println!("Successfully saved tracks to {}", filename);

            let file_path = Path::new(&filename);

            println!("\nAnalyzing recent tracks...");
            let stats: TrackStats =
                AnalysisHandler::analyze_file::<RecentTrack>(file_path, 100).unwrap();

            AnalysisHandler::print_analysis(&stats);
        }
        Err(e) => eprintln!("Error saving tracks: {}", e),
    }

    Ok(())
}
