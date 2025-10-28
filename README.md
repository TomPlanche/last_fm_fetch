# lastfm-client

A modern, async Rust library for fetching and analyzing Last.fm user data with ease.

**Version 2.0** introduces a brand new builder-pattern API with retry logic, rate limiting, and improved ergonomics, while maintaining 100% backward compatibility with the 1.x API.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lastfm-client = "2.0"
```

## Features

### V2.0 API (New!)
- **Builder Pattern**: Fluent, discoverable API design
- **Automatic Retries**: Configurable retry logic with exponential or linear backoff
- **Rate Limiting**: Prevent API abuse with built-in rate limiting
- **Enhanced Error Handling**: Rich error types with retry hints and context
- **Testable**: HTTP abstraction layer for easy mocking
- **Type Safe**: Leverages Rust's type system for compile-time guarantees

### Data Fetching
- **Async API Integration**: Modern asynchronous Last.fm API communication
- **Flexible Track Fetching**: Get recent tracks, loved tracks, and top tracks with configurable limits
- **Advanced Filtering**: Time-based filtering (`from`/`to` timestamps) and period-based filtering for top tracks
- **Extended Data Support**: Fetch extended track information with additional artist details
- **Efficient Pagination**: Smart handling of Last.fm's pagination system with chunked concurrent requests

### Analytics
- **Comprehensive Statistics**:
  - Total play counts
  - Artist-level analytics
  - Track-level analytics
  - Most played artists/tracks
  - Play count thresholds
- **Custom Analysis**: Extensible analysis framework with the `TrackAnalyzable` trait

### Data Export
- **Multiple Formats**: Export data in JSON and CSV formats
- **Timestamp-based Filenames**: Automatic file naming with timestamps
- **Organized Storage**: Structured data directory management

### Error Handling
- **Robust Error Types**: Custom error handling for API and file operations
- **Graceful Failure Recovery**: Proper handling of API and parsing errors

## Configuration

Create a `.env` file in your project root:

```env
LAST_FM_API_KEY=your_api_key_here
```

## Usage

Choose between the **v2.0 API** (recommended for new projects) or the **v1.x API** (fully supported for existing projects).

---

## V2.0 API (Recommended)

### Quick Start

```rust
use lastfm_client::LastFmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client from environment variables
    let client = LastFmClient::builder()
        .from_env()?
        .build_client()?;

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

### Advanced Configuration

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
    .extended(true)
    .fetch_extended()
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
        .from_env()? // Missing LAST_FM_API_KEY → friendly message via Display
        .build_client()?;

    let tracks = client.recent_tracks("username").limit(50).fetch().await?;
    println!("Fetched {} tracks", tracks.len());
    Ok(())
}
```

Tips:
- Use `eprintln!("{}", err)` or `eprintln!("Error: {err}")` (Display), avoid `{:?}`/`{:#?}` (Debug).
- If you keep `fn main() -> Result<…>`, your runtime may show Debug output on failure. The explicit handler above guarantees Display.
- This applies to all errors from this library, including configuration errors like missing `LAST_FM_API_KEY`.

---

## V1.x API (Legacy, Fully Supported)

### Basic Example

```rust
use lastfm_client::{LastFMHandler, TrackLimit, Url};

#[tokio::main]
async fn main() {
    // Create a new handler
    let base_url = Url::new("https://ws.audioscrobbler.com/2.0/");
    let handler = LastFMHandler::new(base_url, "username");

    // Fetch recent tracks
    let recent_tracks = handler
        .get_user_recent_tracks(TrackLimit::Limited(50))
        .await
        .unwrap();

    // Analyze the tracks
    let stats = AnalysisHandler::analyze_tracks(&recent_tracks, 10);
    AnalysisHandler::print_analysis(&stats);
}
```

### Fetching & Saving Example

```rust
use lastfm_client::file_handler::FileFormat;
use lastfm_client::lastfm_handler::{LastFMHandler, TrackLimit};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Create a new handler for user "tom_planche"
    let handler = LastFMHandler::new("tom_planche");

    // Fetch all tracks and save them to a JSON file named "all_scrobbles"
    let filename = handler
        .get_and_save_recent_tracks(TrackLimit::Unlimited, FileFormat::Json, "all_scrobbles")
        .await;

    match filename {
        Ok(filename) => println!("File saved as: {}", filename),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

This example shows how to:
- Load environment variables (including your Last.fm API key)
- Create a handler for a specific Last.fm user
- Fetch all scrobbled tracks (using `TrackLimit::Unlimited`)
- Save them to a JSON file with a custom name prefix
- Handle potential errors during the process

### Analytics Example

```rust
use lastfm_client::{AnalysisHandler, FileHandler, FileFormat};

// Save and analyze tracks
let filename = handler
    .get_and_save_recent_tracks(TrackLimit::Limited(100), FileFormat::JSON)
    .await?;

let stats = AnalysisHandler::analyze_file::<RecentTrack>(Path::new(&filename), 10)?;
AnalysisHandler::print_analysis(&stats);
```

### Advanced Fetching with Options

The library provides comprehensive `*_with_options` methods that expose all available Last.fm API parameters:

#### Recent Tracks with Options

```rust
use lastfm_client::{LastFMHandler, TrackLimit};

let handler = LastFMHandler::new("username").unwrap();

// Get last 50 tracks (basic usage)
let tracks = handler
    .get_user_recent_tracks_with_options(Some(50), None, None, false)
    .await?;

// Get tracks from the last week
let one_week_ago = (Utc::now() - Duration::days(7)).timestamp();
let tracks = handler
    .get_user_recent_tracks_with_options(None, Some(one_week_ago), None, false)
    .await?;

// Get tracks between two dates with extended info
let tracks = handler
    .get_user_recent_tracks_with_options(None, Some(start), Some(end), true)
    .await?;

// Get extended track information (alternative method)
let extended_tracks = handler
    .get_user_recent_tracks_extended(Some(100), None, None)
    .await?;
```

#### Recent Tracks Between Dates

Convenience methods for fetching all tracks within a specific time range:

```rust
use lastfm_client::LastFMHandler;
use chrono::{Utc, Duration, TimeZone};

let handler = LastFMHandler::new("username").unwrap();

// Get all tracks from January 2024
let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap().timestamp();
let end = Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap().timestamp();
let tracks = handler
    .get_user_recent_tracks_between(start, end, false)
    .await?;

// Get all tracks from last week with extended info
let one_week_ago = (Utc::now() - Duration::days(7)).timestamp();
let now = Utc::now().timestamp();
let tracks = handler
    .get_user_recent_tracks_between(one_week_ago, now, true)
    .await?;

// Get all tracks between dates with extended information
let tracks = handler
    .get_user_recent_tracks_between_extended(start, end)
    .await?;
```

#### Top Tracks with Options

```rust
use lastfm_client::{LastFMHandler, Period, TrackLimit};

let handler = LastFMHandler::new("username").unwrap();

// Get all-time top 50 tracks
let tracks = handler
    .get_user_top_tracks_with_options(Some(50), None)
    .await?;

// Get top tracks from the last week
let tracks = handler
    .get_user_top_tracks_with_options(None, Some(Period::Week))
    .await?;

// Get top 100 tracks from the last 3 months
let tracks = handler
    .get_user_top_tracks_with_options(Some(100), Some(Period::ThreeMonth))
    .await?;
```

#### Loved Tracks with Options

```rust
use lastfm_client::{LastFMHandler, TrackLimit};

let handler = LastFMHandler::new("username").unwrap();

// Get all loved tracks
let tracks = handler
    .get_user_loved_tracks_with_options(None)
    .await?;

// Get first 100 loved tracks
let tracks = handler
    .get_user_loved_tracks_with_options(Some(100))
    .await?;
```

### Available Period Options

When using `get_user_top_tracks_with_options`, you can filter by these time periods:

- `Period::Overall` - All time (default if None)
- `Period::Week` - Last 7 days
- `Period::Month` - Last month
- `Period::ThreeMonth` - Last 3 months
- `Period::SixMonth` - Last 6 months
- `Period::TwelveMonth` - Last 12 months

## Migration Guide (v1.x → v2.0)

The v2.0 API is **completely optional** and backward compatible. You can migrate gradually or continue using the v1.x API indefinitely.

### Before (v1.x)
```rust
// Multiple methods for different use cases
let handler = LastFMHandler::new("username")?;

// Limited tracks
handler.get_user_recent_tracks(Some(100))?;

// With date filtering
handler.get_user_recent_tracks_with_options(Some(50), Some(from), Some(to), true)?;

// Extended information
handler.get_user_recent_tracks_extended(Some(100), None, None)?;

// Between dates
handler.get_user_recent_tracks_between(from, to, false)?;
```

### After (v2.0)
```rust
// One method with builder pattern
let client = LastFmClient::builder().from_env()?.build_client()?;

// Limited tracks
client.recent_tracks("username").limit(100).fetch().await?;

// With date filtering
client.recent_tracks("username").limit(50).between(from, to).fetch().await?;

// Extended information
client.recent_tracks("username").limit(100).fetch_extended().await?;

// Between dates
client.recent_tracks("username").between(from, to).fetch().await?;
```

### Key Benefits of v2.0

1. **Simpler API**: One method instead of 6+ variants
2. **Discoverable**: Builder pattern makes options obvious
3. **Automatic Retries**: Built-in exponential backoff
4. **Rate Limiting**: Prevent API throttling
5. **Better Errors**: Rich error types with retry hints
6. **Testable**: Mock HTTP client for testing

---

## API Methods Reference

### V2.0 API

#### Client Creation

```rust
// From environment variables (.env file)
let client = LastFmClient::builder().from_env()?.build()?;

// With custom configuration
let client = LastFmClient::builder()
    .api_key("your_key")
    .retry_attempts(5)
    .rate_limit(10, Duration::from_secs(1))
    .build_client()?;
```

#### Recent Tracks Builder

```rust
client.recent_tracks("username")
    .limit(u32)              // Limit number of tracks
    .unlimited()             // Fetch all available tracks
    .since(i64)              // Tracks since timestamp
    .between(i64, i64)       // Tracks between two timestamps
    .extended(bool)          // Include extended info
    .fetch()                 // Execute and get Vec<RecentTrack>
    .fetch_extended()        // Execute and get Vec<RecentTrackExtended>
```

### V1.x API

#### Recent Tracks Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `get_user_recent_tracks` | `limit` | `Vec<RecentTrack>` | Simple method to fetch recent tracks |
| `get_user_recent_tracks_with_options` | `limit`, `from`, `to`, `extended` | `Vec<RecentTrack>` | Full control over all API parameters |
| `get_user_recent_tracks_extended` | `limit`, `from`, `to` | `Vec<RecentTrackExtended>` | Fetch recent tracks with extended info |
| `get_user_recent_tracks_since` | `from`, `to`, `limit` | `Vec<RecentTrack>` | Fetch tracks since a timestamp |
| `get_user_recent_tracks_between` | `from`, `to`, `extended` | `Vec<RecentTrack>` | Fetch all tracks between two dates |
| `get_user_recent_tracks_between_extended` | `from`, `to` | `Vec<RecentTrackExtended>` | Fetch all tracks between dates with extended info |

### Top Tracks Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `get_user_top_tracks` | `limit`, `period` | `Vec<TopTrack>` | Simple method to fetch top tracks |
| `get_user_top_tracks_with_options` | `limit`, `period` | `Vec<TopTrack>` | Full control over all API parameters |

### Loved Tracks Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `get_user_loved_tracks` | `limit` | `Vec<LovedTrack>` | Simple method to fetch loved tracks |
| `get_user_loved_tracks_with_options` | `limit` | `Vec<LovedTrack>` | Full control over all API parameters |
| `get_user_loved_tracks_since` | `timestamp`, `limit` | `Vec<LovedTrack>` | Fetch loved tracks since a timestamp |

### Helper Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `get_and_save_recent_tracks` | `limit`, `format`, `filename_prefix` | `Result<String>` | Fetch and save recent tracks to file |
| `get_and_save_loved_tracks` | `limit`, `format` | `Result<String>` | Fetch and save loved tracks to file |
| `export_recent_play_counts` | `limit` | `Result<String>` | Export play counts for recent tracks |
| `update_recent_play_counts` | `limit`, `file_path` | `Result<String>` | Update play counts in existing file |
| `is_currently_playing` | - | `Result<Option<RecentTrack>>` | Check if user is currently playing |
| `update_currently_listening` | `file_path` | `Result<Option<RecentTrack>>` | Update currently listening file |

### Parameter Types

- **`limit`**: `impl Into<TrackLimit>` - Use `Some(n)` for limited tracks, `None` or `TrackLimit::Unlimited` for all
- **`from`/`to`**: `Option<i64>` - Unix timestamps in seconds
- **`extended`**: `bool` - Whether to fetch extended track information
- **`period`**: `Option<Period>` - Time period filter (Week, Month, ThreeMonth, etc.)
- **`format`**: `FileFormat` - `FileFormat::Json` or `FileFormat::Csv`

## Testing

Run the test suite:

```bash
cargo test
```

The v2.0 API includes extensive test coverage with mock HTTP clients for reliable testing.

## Advanced Features (v2.0)

### Retry Logic

Configure automatic retries with exponential or linear backoff:

```rust
use lastfm_client::{LastFmClient, client::retry::RetryPolicy};
use std::time::Duration;

// Exponential backoff: 100ms → 200ms → 400ms → 800ms
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

## Architecture (v2.0)

The v2.0 API is built with a modular, testable architecture:

```
src/
├── client/
│   ├── client.rs           # Main LastFmClient entry point
│   ├── http.rs             # HTTP abstraction (trait + implementations)
│   ├── retry.rs            # Retry logic with backoff strategies
│   └── rate_limiter.rs     # Rate limiting with sliding window
├── api/
│   └── recent_tracks.rs    # RecentTracksClient with builder pattern
├── types/
│   ├── tracks.rs           # Track type definitions
│   └── period.rs           # Period and TrackLimit enums
├── config.rs               # Configuration with builder
└── error.rs                # Rich error types with retry hints
```

### Key Design Principles

- **Trait-based HTTP abstraction**: Easy to test with mocks
- **Builder patterns**: Fluent, discoverable APIs
- **Type safety**: Leverages Rust's type system
- **Zero-cost abstractions**: No runtime overhead
- **Backward compatibility**: v1.x API remains fully functional

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built with Rust and powered by the Last.fm API.
