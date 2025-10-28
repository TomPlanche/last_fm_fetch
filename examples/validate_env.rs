use lastfm_client::LastFmClient;

/// Example demonstrating environment variable validation
///
/// This example shows how the library now handles missing environment variables
/// gracefully with clear error messages instead of panicking.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Validate all required environment variables upfront
    println!("Validating required environment variables...");
    match lastfm_client::config::validate_env_vars() {
        Ok(()) => println!("✓ All required environment variables are set"),
        Err(e) => {
            eprintln!("✗ Configuration error: {e}");
            std::process::exit(1);
        }
    }

    // Now we can safely create the client
    // If the API key is missing, we'll get a clear error message
    let username = "tom_planche";
    match LastFmClient::new() {
        Ok(client) => {
            println!("✓ LastFmClient created successfully for user: {username}");
            println!("Client is ready to make API calls!");

            // Example: Try to get recent tracks using the new API
            match client.recent_tracks(username).limit(5).fetch().await {
                Ok(tracks) => {
                    println!("\nFetched {} recent tracks:", tracks.len());
                    for (i, track) in tracks.iter().enumerate() {
                        println!("  {}. {} - {}", i + 1, track.name, track.artist.text);
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching tracks: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to create client: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}
