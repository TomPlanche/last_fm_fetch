# 🎵 async_lastfm

A trivial, small async Rust library for fetching and analyzing Last.fm user data with ease.

## 🚀 Features

### Data Fetching
- **Async API Integration**: Modern asynchronous Last.fm API communication
- **Flexible Track Fetching**: Get recent tracks and loved tracks with configurable limits
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

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
