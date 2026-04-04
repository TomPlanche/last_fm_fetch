//! Demonstrates fetching loved tracks.

use lastfm_client::LastFmClient;
use lastfm_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Create client
    let client = LastFmClient::new()?;

    println!("Fetching loved tracks...");

    // Fetch first 50 loved tracks
    let loved_tracks = client.loved_tracks("tom_planche").limit(50).fetch().await?;

    println!("Found {} loved tracks", loved_tracks.len());

    // Show first few loved tracks with readable dates
    for (i, track) in loved_tracks.iter().take(5).enumerate() {
        let date = chrono::DateTime::from_timestamp(i64::from(track.date.uts), 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S");

        println!(
            "{}. {} - {} (loved on {})",
            i + 1,
            track.artist.name,
            track.name,
            date
        );
    }

    // Fetch all loved tracks (unlimited)
    println!("\nFetching all loved tracks...");
    let all_loved_tracks = client
        .loved_tracks("tom_planche")
        .unlimited()
        .fetch()
        .await?;

    println!("Total loved tracks: {}", all_loved_tracks.len());

    // Compare with recent tracks
    println!("\nFetching recent tracks for comparison...");
    let recent_tracks = client
        .recent_tracks("tom_planche")
        .limit(50)
        .fetch()
        .await?;

    println!("Recent tracks: {}", recent_tracks.len());

    Ok(())
}
