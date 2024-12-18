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

use analytics::AnalysisHandler;
use dotenv::dotenv;
use lastfm_handler::TrackLimit;
use reqwest::Error;
use std::path::Path;
use types::RecentTrack;
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

    // match FileHandler::save(&all_recent_tracks, FileFormat::JSON, "recent_tracks") {
    //     Ok(filename) => {
    //         println!("Successfully saved tracks to {}", filename);

    //         let file_path = Path::new(&filename);

    //         println!("\nAnalyzing recent tracks...");
    //         let stats: TrackStats =
    //             AnalysisHandler::analyze_file::<RecentTrack>(file_path, 100).unwrap();

    //         AnalysisHandler::print_analysis(&stats);
    //     }
    //     Err(e) => eprintln!("Error saving tracks: {}", e),
    // }

    // let filename = "data/recent_tracks_20241204_232653.json";

    // let stats = AnalysisHandler::analyze_file::<RecentTrack>(Path::new(&filename), 10).unwrap();
    // AnalysisHandler::print_analysis(&stats);

    let recent_tracks_file = Path::new("data/recent_tracks_20241204_232653.json");

    match handler
        .update_tracks_file::<RecentTrack>(recent_tracks_file)
        .await
    {
        Ok(file) => {
            println!("Successfully updated tracks file: {file:?}");

            let stats = AnalysisHandler::analyze_file::<RecentTrack>(Path::new(&file), 10).unwrap();
            AnalysisHandler::print_analysis(&stats);
        }
        Err(e) => eprintln!("Error updating tracks file: {e}"),
    }

    Ok(())
}
// 100436
