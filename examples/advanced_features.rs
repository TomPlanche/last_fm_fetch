use lastfm_client::LastFmClient;
use std::time::Duration;

/// Example demonstrating advanced v2.0 features
///
/// This shows retry logic, rate limiting, and the builder pattern.
#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    println!("=== Advanced Features Demo ===\n");

    // Example 1: Client with custom retry configuration
    println!("Example 1: Custom retry configuration");
    let client_with_retry = LastFmClient::builder()
        .from_env()?
        .retry_attempts(5) // Try up to 5 times
        .build()
        .map(LastFmClient::from_config)?;

    println!("✓ Client created with 5 retry attempts");
    println!("  - Exponential backoff: 100ms, 200ms, 400ms, 800ms, 1.6s");
    // Use the client by reading its configuration
    println!(
        "  - Config retry attempts: {}",
        client_with_retry.config().retry_attempts()
    );
    println!();

    // Example 2: Client with rate limiting
    println!("Example 2: Rate limiting configuration");
    let client_with_rate_limit = LastFmClient::builder()
        .from_env()?
        .rate_limit(5, Duration::from_secs(1)) // Max 5 requests per second
        .build()
        .map(LastFmClient::from_config)?;

    println!("✓ Client created with rate limiting");
    println!("  - Max 5 requests per second");
    // Use the client by reading and printing rate limit configuration
    if let Some(rl) = client_with_rate_limit.config().rate_limit() {
        println!(
            "  - Config rate limit: {} per {:?}",
            rl.max_requests, rl.per_duration
        );
    }
    println!();

    // Example 3: Full configuration
    println!("Example 3: Complete configuration");
    let client = LastFmClient::builder()
        .from_env()?
        .timeout(Duration::from_secs(60))
        .max_concurrent_requests(10)
        .retry_attempts(3)
        .rate_limit(10, Duration::from_secs(1))
        .user_agent("MyApp/1.0")
        .build()
        .map(LastFmClient::from_config)?;

    println!("✓ Client created with full configuration:");
    println!("  - Timeout: 60 seconds");
    println!("  - Max concurrent: 10 requests");
    println!("  - Retry: 3 attempts with exponential backoff");
    println!("  - Rate limit: 10 requests per second");
    println!("  - User agent: MyApp/1.0");
    println!();

    // Example 4: Using the configured client
    println!("Example 4: Fetching with retry + rate limiting");
    match client.recent_tracks("tom_planche").limit(20).fetch().await {
        Ok(tracks) => {
            println!("✓ Successfully fetched {} tracks", tracks.len());
            println!("  (Retry logic ensures failed requests are retried)");
            println!("  (Rate limiting prevents overwhelming the API)");

            // Show first few tracks
            for (i, track) in tracks.iter().take(3).enumerate() {
                println!("  {}. {} - {}", i + 1, track.name, track.artist.text);
            }
        }
        Err(e) => {
            if e.is_retryable() {
                println!("✗ Retryable error occurred: {e}");
                if let Some(delay) = e.retry_after() {
                    println!("  Suggested retry after: {delay:?}");
                }
            } else {
                println!("✗ Non-retryable error: {e}");
            }
        }
    }

    println!();

    // Example 5: Error handling with retry information
    println!("Example 5: Advanced error handling");
    println!("The new error types provide rich information:");
    println!("  - is_retryable(): Check if you should retry");
    println!("  - retry_after(): Get suggested retry delay");
    println!("  - Automatic retry with exponential backoff");
    println!();

    // Example 6: Date range with rate limiting
    println!("Example 6: Large request with automatic rate limiting");
    let one_week_ago = (chrono::Utc::now() - chrono::Duration::days(7)).timestamp();
    let now = chrono::Utc::now().timestamp();

    println!("Fetching all tracks from last 7 days...");
    println!("(Rate limiter will automatically pace requests)");

    match client
        .recent_tracks("tom_planche")
        .between(one_week_ago, now)
        .fetch()
        .await
    {
        Ok(tracks) => {
            println!("✓ Fetched {} tracks from last week", tracks.len());
            println!("  (Multiple API calls were automatically rate-limited)");
        }
        Err(e) => {
            println!("✗ Error: {e}");
        }
    }

    println!();
    println!("=== Demo Complete ===");
    println!();
    println!("Key takeaways:");
    println!("  1. Builder pattern provides clean, flexible configuration");
    println!("  2. Automatic retry with exponential backoff");
    println!("  3. Rate limiting prevents API abuse");
    println!("  4. Rich error types with retry hints");
    println!("  5. All features work seamlessly together");

    Ok(())
}
