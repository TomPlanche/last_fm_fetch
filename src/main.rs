#[path = "file_handler.rs"]
mod file_handler;

#[path = "lastfm_handler.rs"]
mod lastfm_handler;

#[path = "types.rs"]
mod types;

#[path = "url_builder.rs"]
mod url_builder;

use dotenv::dotenv;
use file_handler::{FileFormat, FileHandler};
use lastfm_handler::TrackLimit;
use reqwest::Error;
// use tabular::{Row, Table};
use url_builder::Url;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    println!("Creating base URL");
    let base_url = Url::new("https://ws.audioscrobbler.com/2.0/");
    println!("Base URL created: {}", base_url.build());

    let handler = lastfm_handler::LastFMHandler::new(base_url, "tom_planche");

    let all_recent_tracks = handler
        .get_user_recent_tracks(TrackLimit::Unlimited)
        .await?;

    match FileHandler::save(&all_recent_tracks, FileFormat::JSON, "recent_tracks") {
        Ok(filename) => println!("Successfully saved tracks to {}", filename),
        Err(e) => eprintln!("Error saving tracks: {}", e),
    }

    Ok(())
}
