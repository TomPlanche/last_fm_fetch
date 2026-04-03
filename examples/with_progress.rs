//! Demonstrates the built-in terminal progress bar (requires `progress` feature).
//!
//! Run with:
//!   `cargo run --example with_progress --features progress`

use lastfm_client::{LastFmClient, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let client = LastFmClient::builder()
        .from_env()?
        .build()
        .map(LastFmClient::from_config)?;

    let username = "tom_planche";

    // Recent tracks with a progress bar
    println!("Fetching recent tracks for {username}...");
    let tracks = client
        .recent_tracks(username)
        .limit(200)
        .with_progress()
        .fetch()
        .await?;
    println!("Fetched {} recent tracks.", tracks.len());
    println!();

    // Top artists with a progress bar
    println!("Fetching top artists for {username}...");
    let artists = client
        .top_artists(username)
        .limit(50)
        .with_progress()
        .fetch()
        .await?;
    println!("Fetched {} top artists.", artists.len());

    Ok(())
}
