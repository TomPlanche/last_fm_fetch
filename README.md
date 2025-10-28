# 🎵 async_lastfm

A trivial, small async Rust library for fetching and analyzing Last.fm user data with ease.

## 🚀 Features

### Data Fetching
- **Async API Integration**: Modern asynchronous Last.fm API communication
- **Flexible Track Fetching**: Get recent tracks, loved tracks, and top tracks with configurable limits
- **Advanced Filtering**: Time-based filtering (`from`/`to` timestamps) and period-based filtering for top tracks
- **Extended Data Support**: Fetch extended track information with additional artist details
- **Efficient Pagination**: Smart handling of Last.fm's pagination system
- **Rate Limit Aware**: Built-in handling of API rate limits

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

## 🔧 Configuration

Create a `.env` file in your project root:

```env
LAST_FM_API_KEY=your_api_key_here
```

## 🎮 Usage

### Basic Example

```rust
use async_lastfm::{LastFMHandler, TrackLimit, Url};

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
use async_lastfm::file_handler::FileFormat;
use async_lastfm::lastfm_handler::{LastFMHandler, TrackLimit};
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
use async_lastfm::{AnalysisHandler, FileHandler, FileFormat};

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
use async_lastfm::{LastFMHandler, TrackLimit};

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
use async_lastfm::LastFMHandler;
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
use async_lastfm::{LastFMHandler, Period, TrackLimit};

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
use async_lastfm::{LastFMHandler, TrackLimit};

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

## 📚 API Methods Reference

### Recent Tracks Methods

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

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
