# lastfm-client

A modern, async Rust library for fetching and analyzing Last.fm user data with ease.

## What's new in v3.0

- **Breaking**: Removed the deprecated `lastfm_handler` module (`LastFMHandler` and all v1.x methods)
- **Top Artists**: Fetch top artists via `client.top_artists("username")`
- **Top Albums**: Fetch top albums via `client.top_albums("username")`
- Unified `ResourceContainer` trait across all resource types

If you were still using the v1.x API, see the [migration section](#migrating-from-v2x) below.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lastfm-client = "3.2"
```

## Features

- **Builder Pattern**: Fluent, discoverable API design
- **Automatic Retries**: Configurable retry logic with exponential or linear backoff
- **Rate Limiting**: Prevent API abuse with built-in rate limiting
- **Enhanced Error Handling**: Rich error types with retry hints and context
- **Testable**: HTTP abstraction layer for easy mocking
- **Type Safe**: Leverages Rust's type system for compile-time guarantees

### Data Fetching
- **Async API Integration**: Modern asynchronous Last.fm API communication
- **Flexible Fetching**: Get recent tracks, loved tracks, top tracks, top artists, and top albums with configurable limits
- **Advanced Filtering**: Time-based filtering (`from`/`to` timestamps) and period-based filtering for top resources
- **Extended Data Support**: Fetch extended track information with additional artist details
- **Efficient Pagination**: Smart handling of Last.fm's pagination system with chunked concurrent requests

### Analytics
- **Comprehensive Statistics**: Total play counts, artist-level analytics, track-level analytics, most played artists/tracks, play count thresholds
- **Custom Analysis**: Extensible analysis framework with the `TrackAnalyzable` trait

### Data Export
- **Multiple Formats**: Export data in JSON and CSV formats
- **Timestamp-based Filenames**: Automatic file naming with timestamps
- **Organized Storage**: Structured data directory management
- **Incremental Updates**: `fetch_and_update` writes only new entries to an existing file (prepend for JSON, append for CSV); a sidecar metadata file keeps repeated calls fast regardless of file size

## Configuration

Create a `.env` file in your project root:

```env
LAST_FM_API_KEY=your_api_key_here
```

## Usage

### Quick Start

```rust
use lastfm_client::LastFmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with defaults (loads API key from LAST_FM_API_KEY env var)
    let client = LastFmClient::new()?;

    // Fetch recent tracks with builder pattern
    let tracks = client
        .recent_tracks("username")
        .limit(50)
        .fetch()
        .await?;

    println!("Fetched {} tracks", tracks.len());
    Ok(())
}
```

### Custom Configuration

```rust
use lastfm_client::LastFmClient;
use std::time::Duration;

let client = LastFmClient::builder()
    .api_key("your_api_key")
    .user_agent("MyApp/1.0")
    .timeout(Duration::from_secs(60))
    .max_concurrent_requests(10)
    .retry_attempts(5)
    .rate_limit(10, Duration::from_secs(1))  // 10 requests per second
    .build_client()?;
```

### Fetching Recent Tracks

```rust
// Limited tracks
let tracks = client
    .recent_tracks("username")
    .limit(100)
    .fetch()
    .await?;

// All available tracks
let all_tracks = client
    .recent_tracks("username")
    .unlimited()
    .fetch()
    .await?;

// Tracks from specific date
let since_timestamp = 1704067200; // Jan 1, 2024
let recent = client
    .recent_tracks("username")
    .since(since_timestamp)
    .fetch()
    .await?;

// Tracks between two dates
let from = 1704067200; // Jan 1, 2024
let to = 1706745600;   // Feb 1, 2024
let tracks = client
    .recent_tracks("username")
    .between(from, to)
    .fetch()
    .await?;

// Extended track information (includes full artist details)
let extended_tracks = client
    .recent_tracks("username")
    .limit(50)
    .extended()
    .fetch_extended()
    .await?;

// Check if user is currently playing
let currently_playing = client
    .recent_tracks("username")
    .check_currently_playing()
    .await?;

// Analyze tracks and get statistics
let stats = client
    .recent_tracks("username")
    .limit(100)
    .analyze(5)
    .await?;

// Fetch and save to file
let filename = client
    .recent_tracks("username")
    .limit(50)
    .fetch_and_save(FileFormat::Json, "my_tracks")
    .await?;

// Track progress during a long fetch
let filename = client
    .recent_tracks("username")
    .unlimited()
    .on_progress(|fetched, total| {
        println!("{fetched}/{total} tracks fetched");
    })
    .fetch_extended_and_save(FileFormat::Json, "all_tracks")
    .await?;

// Incremental update: only fetch tracks newer than the last entry in the file.
// On the first call the file is created with a full fetch. Subsequent calls are
// fast because the latest timestamp is read from a small sidecar file instead
// of deserializing the entire data file.
let new_count = client
    .recent_tracks("username")
    .on_progress(|fetched, total| println!("{fetched}/{total}"))
    .fetch_and_update("data/scrobbles.json")
    .await?;
println!("{new_count} new scrobbles");

// Incremental update (CSV): same logic, but new rows are appended to the CSV.
// A sidecar (data/scrobbles.csv.meta) tracks the latest timestamp.
let new_count = client
    .recent_tracks("username")
    .fetch_and_update("data/scrobbles.csv")
    .await?;
println!("{new_count} new scrobbles");

// Same for extended tracks (JSON or CSV path works)
let new_count = client
    .recent_tracks("username")
    .fetch_extended_and_update("data/scrobbles_extended.json")
    .await?;
```

### Progress Tracking

For large libraries, `.on_progress()` fires after the total is known (`fetched = 0`) and then after each batch (~5000 tracks):

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(0);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner} [{elapsed_precise}] [{bar:40}] {pos}/{len} tracks ({eta})")?
        .progress_chars("##-"),
);

let pb_clone = pb.clone();
let filename = client
    .recent_tracks("username")
    .unlimited()
    .on_progress(move |fetched, total| {
        if pb_clone.length() == Some(0) {
            pb_clone.set_length(u64::from(total));
        }
        pb_clone.set_position(u64::from(fetched));
    })
    .fetch_extended_and_save(FileFormat::Json, "all_tracks")
    .await?;

pb.finish_and_clear();
println!("Saved to {filename}");
```

### Fetching Loved Tracks

```rust
// Limited loved tracks
let loved_tracks = client
    .loved_tracks("username")
    .limit(50)
    .fetch()
    .await?;

// All loved tracks
let all_loved = client
    .loved_tracks("username")
    .unlimited()
    .fetch()
    .await?;

// Analyze loved tracks
let stats = client
    .loved_tracks("username")
    .analyze(1)
    .await?;

// Fetch and save loved tracks
let filename = client
    .loved_tracks("username")
    .fetch_and_save(FileFormat::Json, "loved_tracks")
    .await?;

// Incremental update for loved tracks
let new_count = client
    .loved_tracks("username")
    .fetch_and_update("data/loved_tracks.json")
    .await?;
println!("{new_count} newly loved tracks");
```

### Fetching Top Tracks

```rust
use lastfm_client::api::Period;

// Top tracks with period filter
let top_tracks = client
    .top_tracks("username")
    .limit(50)
    .period(Period::ThreeMonth)
    .fetch()
    .await?;

// All-time top tracks
let all_time_top = client
    .top_tracks("username")
    .unlimited()
    .period(Period::Overall)
    .fetch()
    .await?;

// Fetch and save top tracks
let filename = client
    .top_tracks("username")
    .limit(100)
    .period(Period::Month)
    .fetch_and_save(FileFormat::Json, "monthly_top")
    .await?;
```

### Fetching Top Artists

```rust
use lastfm_client::api::Period;

let top_artists = client
    .top_artists("username")
    .limit(25)
    .period(Period::SixMonth)
    .fetch()
    .await?;
```

### Fetching Top Albums

```rust
use lastfm_client::api::Period;

let top_albums = client
    .top_albums("username")
    .limit(25)
    .period(Period::TwelveMonth)
    .fetch()
    .await?;
```

### Error Handling with Retry Hints

```rust
use lastfm_client::error::LastFmError;

match client.recent_tracks("username").limit(50).fetch().await {
    Ok(tracks) => println!("Success: {} tracks", tracks.len()),
    Err(e) => {
        if e.is_retryable() {
            if let Some(retry_after) = e.retry_after() {
                println!("Rate limited. Retry after {:?}", retry_after);
                tokio::time::sleep(retry_after).await;
                // Retry the request...
            }
        } else {
            eprintln!("Non-retryable error: {}", e);
        }
    }
}
```

---

### Friendly error messages (Display vs Debug)

If you see output like `MissingEnvVar("LAST_FM_API_KEY")`, the error is being printed with Debug formatting (`{:?}`) somewhere. This library implements friendly Display messages (via `#[error("...")]`), so prefer Display (`{}`) when printing errors.

Use an explicit `main` error handler to guarantee Display formatting:

```rust
use dotenvy::dotenv;
use lastfm_client::LastFmClient;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err}"); // Display, not Debug
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client = LastFmClient::builder()
        .from_env()? // Missing LAST_FM_API_KEY -> friendly message via Display
        .build_client()?;

    let tracks = client.recent_tracks("username").limit(50).fetch().await?;
    println!("Fetched {} tracks", tracks.len());
    Ok(())
}
```

Tips:
- Use `eprintln!("{}", err)` or `eprintln!("Error: {err}")` (Display), avoid `{:?}`/`{:#?}` (Debug).
- If you keep `fn main() -> Result<...>`, your runtime may show Debug output on failure. The explicit handler above guarantees Display.
- This applies to all errors from this library, including configuration errors like missing `LAST_FM_API_KEY`.

---

## API Methods Reference

### Client Creation

```rust
use lastfm_client::LastFmClient;

// Simple: with defaults (loads API key from LAST_FM_API_KEY env var)
let client = LastFmClient::new()?;

// With custom configuration
let client = LastFmClient::builder()
    .api_key("your_key")
    .retry_attempts(5)
    .rate_limit(10, Duration::from_secs(1))
    .build_client()?;
```

### Recent Tracks

```rust
client.recent_tracks("username")
    .limit(u32)              // Limit number of tracks
    .unlimited()             // Fetch all available tracks
    .since(i64)              // Tracks since timestamp
    .between(i64, i64)       // Tracks between two timestamps
    .extended()              // Include extended info
    .on_progress(|fetched, total| { ... }) // Progress callback (fetched, total)
    .fetch()                 // Execute and get Vec<RecentTrack>
    .fetch_extended()        // Execute and get Vec<RecentTrackExtended>
    .check_currently_playing() // Check if currently playing
    .analyze(usize)          // Analyze tracks and get statistics
    .analyze_and_print(usize) // Analyze and print statistics
    .fetch_and_save(format, prefix) // Fetch and save to a new timestamped file
    .fetch_extended_and_save(format, prefix) // Fetch extended and save
    .fetch_and_update(file_path) // Fetch only new tracks and prepend to file -> usize
    .fetch_extended_and_update(file_path) // Same for extended tracks -> usize
```

### Loved Tracks

```rust
client.loved_tracks("username")
    .limit(u32)              // Limit number of tracks
    .unlimited()             // Fetch all available tracks
    .fetch()                 // Execute and get Vec<LovedTrack>
    .analyze(usize)          // Analyze tracks and get statistics
    .analyze_and_print(usize) // Analyze and print statistics
    .fetch_and_save(format, prefix) // Fetch and save to a new timestamped file
    .fetch_and_update(file_path) // Fetch only new tracks and prepend to file -> usize
```

### Top Tracks

```rust
client.top_tracks("username")
    .limit(u32)              // Limit number of tracks
    .unlimited()             // Fetch all available tracks
    .period(Period)          // Time period filter
    .fetch()                 // Execute and get Vec<TopTrack>
    .fetch_and_save(format, prefix) // Fetch and save to file
```

### Top Artists

```rust
client.top_artists("username")
    .limit(u32)              // Limit number of artists
    .unlimited()             // Fetch all available artists
    .period(Period)          // Time period filter
    .fetch()                 // Execute and get Vec<TopArtist>
    .fetch_and_save(format, prefix) // Fetch and save to file
```

### Top Albums

```rust
client.top_albums("username")
    .limit(u32)              // Limit number of albums
    .unlimited()             // Fetch all available albums
    .period(Period)          // Time period filter
    .fetch()                 // Execute and get Vec<TopAlbum>
    .fetch_and_save(format, prefix) // Fetch and save to file
```

### Available Period Options

- `Period::Overall` - All time
- `Period::Week` - Last 7 days
- `Period::Month` - Last month
- `Period::ThreeMonth` - Last 3 months
- `Period::SixMonth` - Last 6 months
- `Period::TwelveMonth` - Last 12 months

### Parameter Types

- **`limit`**: `impl Into<TrackLimit>` - Use `Some(n)` for limited results, `None` or `TrackLimit::Unlimited` for all
- **`from`/`to`**: `Option<i64>` - Unix timestamps in seconds
- **`extended`**: `bool` - Whether to fetch extended track information
- **`period`**: `Option<Period>` - Time period filter (Week, Month, ThreeMonth, etc.)
- **`format`**: `FileFormat` - `FileFormat::Json` or `FileFormat::Csv`

## Testing

Run the test suite:

```bash
cargo test
```

The library includes extensive test coverage with mock HTTP clients for reliable testing.

## Advanced Features

### Retry Logic

Configure automatic retries with exponential or linear backoff:

```rust
use lastfm_client::{LastFmClient, client::retry::RetryPolicy};
use std::time::Duration;

// Exponential backoff: 100ms -> 200ms -> 400ms -> 800ms
let client = LastFmClient::builder()
    .api_key("your_key")
    .retry_attempts(5)
    .build_client()?;

// Custom retry policy
let policy = RetryPolicy::exponential(3)
    .with_base_delay(Duration::from_millis(200))
    .with_max_delay(Duration::from_secs(10));
```

### Rate Limiting

Prevent API throttling with sliding window rate limiting:

```rust
use std::time::Duration;

let client = LastFmClient::builder()
    .api_key("your_key")
    .rate_limit(10, Duration::from_secs(1))  // 10 requests per second
    .build_client()?;
```

### Testing with Mocks

Use mock HTTP clients for testing:

```rust
use lastfm_client::client::http::MockClient;
use std::collections::HashMap;

let mut responses = HashMap::new();
responses.insert(
    "test_url".to_string(),
    serde_json::json!({"recenttracks": {"track": []}}),
);

let mock_client = MockClient::new(responses);
// Use mock_client in your tests
```

## Architecture

```
src/
├── client/
│   ├── lastfm.rs            # Main LastFmClient entry point
│   ├── http.rs              # HTTP abstraction (trait + implementations)
│   ├── retry.rs             # Retry logic with backoff strategies
│   └── rate_limiter.rs      # Rate limiting with sliding window
├── api/
│   ├── fetch_utils.rs       # Generic pagination with ResourceContainer trait
│   ├── recent_tracks.rs     # RecentTracksClient with builder pattern
│   ├── loved_tracks.rs      # LovedTracksClient with builder pattern
│   ├── top_tracks.rs        # TopTracksClient with builder pattern
│   ├── top_artists.rs       # TopArtistsClient with builder pattern
│   └── top_albums.rs        # TopAlbumsClient with builder pattern
├── types/
│   ├── tracks.rs            # Track type definitions
│   ├── artists.rs           # Artist type definitions
│   ├── albums.rs            # Album type definitions
│   └── period.rs            # Period and TrackLimit enums
├── config.rs                # Configuration with builder
├── error.rs                 # Rich error types with retry hints
├── analytics.rs             # TrackAnalyzable trait + AnalysisHandler
└── file_handler.rs          # JSON/CSV export
```

### Key Design Principles

- **Trait-based HTTP abstraction**: Easy to test with mocks
- **Builder patterns**: Fluent, discoverable APIs
- **Type safety**: Leverages Rust's type system
- **Zero-cost abstractions**: No runtime overhead

## Migrating from v2.x

v3.0 removes the `lastfm_handler` module (`LastFMHandler`) that was deprecated since v2.0. If you were using it:

| v2.x (removed) | v3.0 equivalent |
|---|---|
| `LastFMHandler::new("user")` | `LastFmClient::new()?` |
| `handler.get_user_recent_tracks(Some(100))` | `client.recent_tracks("user").limit(100).fetch().await?` |
| `handler.get_user_recent_tracks_between(from, to, false)` | `client.recent_tracks("user").between(from, to).fetch().await?` |
| `handler.get_user_top_tracks(Some(50), Some(Period::Week))` | `client.top_tracks("user").limit(50).period(Period::Week).fetch().await?` |
| `handler.get_user_loved_tracks(Some(100))` | `client.loved_tracks("user").limit(100).fetch().await?` |

`TrackPlayInfo` has moved from `lastfm_client::lastfm_handler::TrackPlayInfo` to `lastfm_client::types::TrackPlayInfo`.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built with Rust and powered by the Last.fm API.
