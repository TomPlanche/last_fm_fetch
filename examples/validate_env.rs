use async_lastfm::config;
use async_lastfm::lastfm_handler::LastFMHandler;

/// Example demonstrating environment variable validation
///
/// This example shows how the library now handles missing environment variables
/// gracefully with clear error messages instead of panicking.
#[tokio::main]
async fn main() {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Validate all required environment variables upfront
    println!("Validating required environment variables...");
    match config::validate_env_vars() {
        Ok(()) => println!("✓ All required environment variables are set"),
        Err(e) => {
            eprintln!("✗ Configuration error: {e}");
            std::process::exit(1);
        }
    }

    // Now we can safely create the handler
    // If the API key is missing, we'll get a clear error message
    let username = "example_user";
    match LastFMHandler::new(username) {
        Ok(handler) => {
            println!("✓ LastFMHandler created successfully for user: {username}");
            println!("Handler is ready to make API calls!");

            // Example: Try to get recent tracks
            match handler.get_user_recent_tracks(Some(5)).await {
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
            eprintln!("✗ Failed to create LastFMHandler: {e}");
            std::process::exit(1);
        }
    }
}
