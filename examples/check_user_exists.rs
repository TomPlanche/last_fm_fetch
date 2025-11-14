use lastfm_client::LastFmClient;

/// Example demonstrating user existence validation
///
/// This example shows how to check if a Last.fm user exists before
/// attempting to fetch their data, which is useful for:
/// - Validating user input
/// - Handling user-not-found errors gracefully
/// - Avoiding unnecessary API calls
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Create the client
    let client = LastFmClient::new()?;

    println!("Last.fm User Existence Checker\n");

    // Test with known existing users
    let existing_users = ["rj", "tom_planche"];
    for username in &existing_users {
        match client.user_exists(*username).await {
            Ok(true) => println!("User '{username}' exists"),
            Ok(false) => println!("User '{username}' does not exist"),
            Err(e) => eprintln!("Error checking user '{username}': {e}"),
        }
    }

    println!();

    // Test with a likely non-existent user
    let nonexistent_user = "this_user_definitely_does_not_exist_12345";
    match client.user_exists(nonexistent_user).await {
        Ok(true) => println!("User '{nonexistent_user}' exists"),
        Ok(false) => println!("User '{nonexistent_user}' does not exist (as expected)"),
        Err(e) => eprintln!("Error checking user '{nonexistent_user}': {e}"),
    }

    println!("\nPractical example: Validate before fetching data");

    // Practical use case: check if user exists before fetching their tracks
    let username_to_check = "tom_planche";
    match client.user_exists(username_to_check).await {
        Ok(true) => {
            println!("User '{username_to_check}' exists! Fetching recent tracks...");
            match client.recent_tracks(username_to_check).limit(5).fetch().await {
                Ok(tracks) => {
                    println!("\nFetched {} recent tracks:", tracks.len());
                    for (i, track) in tracks.iter().enumerate() {
                        println!("  {}. {} - {}", i + 1, track.name, track.artist.text);
                    }
                }
                Err(e) => eprintln!("Error fetching tracks: {e}"),
            }
        }
        Ok(false) => {
            println!("User '{username_to_check}' does not exist. Skipping data fetch.");
        }
        Err(e) => {
            eprintln!("Error checking if user exists: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}
