#[path = "types.rs"]
mod types;

#[path = "url_builder.rs"]
mod url_builder;

#[path = "lastfm_handler.rs"]
mod lastfm_handler;

use dotenv::dotenv;
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

    let loved_tracks = handler.get_user_loved_tracks(Some(10_000)).await?;

    println!("loved_tracks length: {}", loved_tracks.len());

    Ok(())
}
