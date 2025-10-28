use lastfm_client::api::{LovedTracksClient, RecentTracksClient};
use lastfm_client::client::ReqwestClient;
use lastfm_client::config::ConfigBuilder;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Create configuration
    let config = Arc::new(ConfigBuilder::new().from_env()?.build()?);
    
    // Create HTTP client
    let http_client = Arc::new(ReqwestClient::new());
    
    // Create loved tracks client
    let loved_tracks_client = LovedTracksClient::new(http_client.clone(), config.clone());
    
    // Create recent tracks client for comparison
    let recent_tracks_client = RecentTracksClient::new(http_client, config);

    println!("Fetching loved tracks...");
    
    // Fetch first 50 loved tracks
    let loved_tracks = loved_tracks_client
        .builder("tom_planche")
        .limit(50)
        .fetch()
        .await?;
    
    println!("Found {} loved tracks", loved_tracks.len());
    
    // Show first few loved tracks with readable dates
    for (i, track) in loved_tracks.iter().take(5).enumerate() {
        let date = chrono::DateTime::from_timestamp(i64::from(track.date.uts), 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S");
        
        println!("{}. {} - {} (loved on {})", 
            i + 1, 
            track.artist.name, 
            track.name,
            date
        );
    }
    
    // Fetch all loved tracks (unlimited)
    println!("\nFetching all loved tracks...");
    let all_loved_tracks = loved_tracks_client
        .builder("tom_planche")
        .unlimited()
        .fetch()
        .await?;
    
    println!("Total loved tracks: {}", all_loved_tracks.len());
    
    // Compare with recent tracks
    println!("\nFetching recent tracks for comparison...");
    let recent_tracks = recent_tracks_client
        .builder("tom_planche")
        .limit(50)
        .fetch()
        .await?;
    
    println!("Recent tracks: {}", recent_tracks.len());
    
    Ok(())
}
