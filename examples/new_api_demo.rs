use lastfm_client::LastFmClient;

/// Example demonstrating the new v2.0 API with builder pattern
///
/// This example shows how to use the new fluent API to fetch recent tracks.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Create client from environment variables
    let client = LastFmClient::builder()
        .from_env()?
        .build()
        .map(LastFmClient::from_config)?;

    println!("✓ LastFmClient created successfully!");
    println!();

    // Example 1: Fetch limited number of recent tracks
    println!("Example 1: Fetching last 10 tracks...");
    match client
        .recent_tracks("tom_planche")
        .limit(10)
        .fetch()
        .await
    {
        Ok(tracks) => {
            println!("✓ Fetched {} tracks", tracks.len());
            for (i, track) in tracks.iter().enumerate().take(5) {
                println!("  {}. {} - {}", i + 1, track.name, track.artist.text);
            }
        }
        Err(e) => {
            eprintln!("✗ Error: {e}");
        }
    }

    println!();

    // Example 2: Fetch recent tracks with a safe upper bound
    println!("Example 2: Fetching up to 200 recent tracks...");
    match client
        .recent_tracks("tom_planche")
        .limit(200)
        .fetch()
        .await
    {
        Ok(tracks) => {
            println!("✓ Fetched {} tracks (capped)", tracks.len());
        }
        Err(e) => {
            eprintln!("✗ Error: {e}");
        }
    }

    println!();

    // Example 3: Fetch tracks from a specific time range
    println!("Example 3: Fetching tracks from last 7 days...");
    let one_week_ago = (chrono::Utc::now() - chrono::Duration::days(7)).timestamp();
    let now = chrono::Utc::now().timestamp();

    match client
        .recent_tracks("tom_planche")
        .between(one_week_ago, now)
        .fetch()
        .await
    {
        Ok(tracks) => {
            println!("✓ Tracks in last 7 days: {}", tracks.len());
        }
        Err(e) => {
            eprintln!("✗ Error: {e}");
        }
    }

    println!();

    // Example 4: Fetch extended track information
    println!("Example 4: Fetching tracks with extended info...");
    match client
        .recent_tracks("tom_planche")
        .limit(5)
        .extended(true)
        .fetch_extended()
        .await
    {
        Ok(tracks) => {
            println!("✓ Fetched {} extended tracks", tracks.len());
            for track in &tracks {
                println!("  - {} by {} (Album: {})",
                    track.name,
                    track.artist.name,
                    track.album.name
                );
            }
        }
        Err(e) => {
            eprintln!("✗ Error: {e}");
        }
    }

    println!();
    println!("✓ All examples completed!");

    Ok(())
}
