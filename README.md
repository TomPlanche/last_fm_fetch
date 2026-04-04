# lastfm-client

A modern, async Rust library for fetching and analyzing Last.fm user data with ease.

## What's new in v4.0

Eight new `user.*` endpoints, a simplified `LastFmClient` that no longer exposes
intermediate client types, and extension traits that eliminate ~400 lines of
duplicated builder boilerplate.

### New endpoints

```rust
use lastfm_client::LastFmClient;

let client = LastFmClient::new()?;

// User profile
let info = client.user_info("username").fetch().await?;
println!("{} — {} scrobbles", info.name, info.play_count);

// Check existence
if client.user_exists("username").await? { /* … */ }

// Top tags (max 50)
let tags = client.top_tags("username").limit(20).fetch().await?;

// Friends (auto-paginated)
let friends = client.friends("username").fetch_all().await?;

// Personal tags — tracks, artists, or albums
let page = client.personal_tags("username", "shoegaze").fetch_tracks().await?;

// Weekly charts
let ranges = client.weekly_chart_list("username").fetch().await?;
if let Some(range) = ranges.first() {
    let tracks  = client.weekly_track_chart("username").range(range).fetch().await?;
    let artists = client.weekly_artist_chart("username").range(range).fetch().await?;
    let albums  = client.weekly_album_chart("username").range(range).fetch().await?;
}
```

### Simplified client API

The intermediate `XClient` factory types (`LovedTracksClient`, `RecentTracksClient`,
`TopTracksClient`, `TopArtistsClient`, `TopAlbumsClient`) have been removed from the
public API. Use `LastFmClient` directly:

```rust
// Before (3.x)
use lastfm_client::api::LovedTracksClient;
let client = LovedTracksClient::new(http, config);
let tracks = client.builder("user").limit(50).fetch().await?;

// After (4.0)
use lastfm_client::LastFmClient;
let client = LastFmClient::new()?;
let tracks = client.loved_tracks("user").limit(50).fetch().await?;
```

### Extension traits

Builder methods are now provided by shared extension traits instead of being duplicated on each
builder. This removes ~400 lines of boilerplate and makes the API consistent across all five
resource types.

You need to import the relevant trait to use the methods:

| Trait | Methods |
|-------|---------|
| `LimitBuilder` | `.limit(n)`, `.unlimited()` |
| `FetchAndSave` | `.fetch_and_save(format, prefix)`, `.fetch_and_save_sqlite(prefix)` |
| `FetchAndUpdate` | `.fetch_and_update(path)`, `.fetch_and_update_sqlite(path)` |
| `Analyze` | `.analyze(threshold)`, `.analyze_and_print(threshold)` |

```rust
use lastfm_client::{LastFmClient, prelude::*};

let client = LastFmClient::new()?;

// limit / fetch_and_save / fetch_and_update all require the trait in scope
let path = client
    .recent_tracks("username")
    .limit(500)
    .fetch_and_save(FileFormat::Json, "scrobbles")
    .await?;

let new_count = client
    .recent_tracks("username")
    .fetch_and_update(&path)
    .await?;

let stats = client
    .recent_tracks("username")
    .limit(1000)
    .analyze(5)
    .await?;
```

## What's new in v3.9

Load data back from a `SQLite` database produced by `fetch_and_save_sqlite` or
`fetch_and_update_sqlite`. The returned `TrackList<T>` supports all analysis
methods.

```rust
use lastfm_client::{file_handler::FileHandler, RecentTrack};

let tracks = FileHandler::load_sqlite::<RecentTrack>("data/recent_tracks.db")?;

let top     = tracks.to_set();         // TrackList<ScoredTrack>
let artists = tracks.top_artists();    // TrackList<ScoredArtist>
let streak  = tracks.streak();         // u32
```

Supported types: `RecentTrack`, `RecentTrackExtended`, `LovedTrack`, `TopTrack`,
`TopArtist`, `TopAlbum`. Requires the `sqlite` feature.

> **Note**: fields not stored in the schema (images, streamability flags,
> human-readable dates) are reconstructed with empty defaults. All analysis
> methods work correctly on loaded data.


## What's new in v3.8 and earlier

**`impl TrackList<RecentTrackExtended>`**: the same aggregation helpers as
`TrackList<RecentTrack>` — `to_set`, `top_artists`, `top_albums`, `by_hour`,
`by_date`, `streak`, `without_now_playing`, `unique_artist_count`, `unique_track_count`.
Use `.fetch_extended()` instead of `.fetch()`.

```rust
let extended = client
    .recent_tracks("username")
    .between(two_weeks_ago.timestamp(), now.timestamp())
    .fetch_extended()
    .await?;

let top_tracks  = extended.to_set();        // TrackList<ScoredTrack>
let top_artists = extended.top_artists();   // TrackList<ScoredArtist>
let top_albums  = extended.top_albums();    // TrackList<ScoredAlbum>
let hours       = extended.by_hour();       // [u32; 24]
let streak      = extended.streak();        // u32
let clean       = extended.without_now_playing();
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lastfm-client = "4.0"
```

### Optional Features

| Feature | Description |
|---------|-------------|
| `sqlite`   | SQLite export via `rusqlite` (bundled SQLite, no system dependency required) |
| `progress` | Built-in terminal progress bar via `indicatif` — adds `.with_progress()` to all builders |
| `full`     | Enables both `sqlite` and `progress` |

```toml
[dependencies]
# All optional features
lastfm-client = { version = "4.0", features = ["full"] }

# Individual features
lastfm-client = { version = "4.0", features = ["sqlite", "progress"] }
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
- **Multiple Formats**: Export data in JSON, NDJSON, CSV, and SQLite formats
- **Timestamp-based Filenames**: Automatic file naming with timestamps
- **Organized Storage**: Structured data directory management
- **Incremental Updates**: `fetch_and_update` writes only new entries to an existing file (prepend for JSON, append for CSV and NDJSON); a sidecar metadata file keeps repeated calls fast regardless of file size
- **SQLite Export** (`sqlite` feature): Export to a queryable `.db` file; incremental updates use `MAX(date_uts)` directly from the database with no sidecar needed

## Configuration

Create a `.env` file in your project root:

```env
LAST_FM_API_KEY=your_api_key_here
```

## Examples

Runnable programs live in [`examples/`](examples/). Set `LAST_FM_API_KEY` (and optionally `LASTFM_USERNAME`). Run from the **crate root** so relative paths like `data/` match the library’s file helpers.

| Example | Command | What it shows |
|---------|---------|----------------|
| `scrobbles_file_workflow` | `cargo run --example scrobbles_file_workflow` | `fetch_and_save` / `fetch_and_update` (JSON, CSV, NDJSON), `FileHandler::load` / `load_ndjson`, `TrackList` aggregations, extended saves, `analyze`, `check_currently_playing`, `on_progress`, top charts + loved tracks |
| `scrobbles_sqlite` | `cargo run --example scrobbles_sqlite --features sqlite` | `fetch_and_save_sqlite`, `fetch_and_update_sqlite`, `FileHandler::load_sqlite` |
| `with_progress` | `cargo run --example with_progress --features progress` | `with_progress()` terminal progress bar |
| `new_api_demo` | `cargo run --example new_api_demo` | Basic recent-track builders |
| `advanced_features` | `cargo run --example advanced_features` | Retry, rate limiting, config |
| `loved_tracks_demo` | `cargo run --example loved_tracks_demo` | Loved + recent tracks |
| `check_user_exists` | `cargo run --example check_user_exists` | `user_exists` |
| `validate_env` | `cargo run --example validate_env` | `validate_env_vars` |

## Usage

### Quick Start

```rust
use lastfm_client::{LastFmClient, prelude::*};

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

// Incremental update (NDJSON): new records are appended as lines to the .ndjson file.
// A sidecar (data/scrobbles.ndjson.meta) tracks the latest timestamp.
let new_count = client
    .recent_tracks("username")
    .fetch_and_update("data/scrobbles.ndjson")
    .await?;
println!("{new_count} new scrobbles");

// Same for extended tracks (JSON or CSV path works)
let new_count = client
    .recent_tracks("username")
    .fetch_extended_and_update("data/scrobbles_extended.json")
    .await?;

// SQLite export (requires `sqlite` feature)
let db_path = client
    .recent_tracks("username")
    .unlimited()
    .fetch_and_save_sqlite("recent_tracks")
    .await?;
println!("Saved to {db_path}");

// SQLite incremental update - reads MAX(date_uts) from the DB, no sidecar file needed
let new_count = client
    .recent_tracks("username")
    .fetch_and_update_sqlite("data/scrobbles.db")
    .await?;
println!("{new_count} new scrobbles inserted");
```

### SQLite Export (optional feature)

Enable the `sqlite` feature in `Cargo.toml`:

```toml
lastfm-client = { version = "4.0", features = ["sqlite"] }
```

All five resource types support `fetch_and_save_sqlite(prefix)`. Recent tracks and loved tracks additionally support `fetch_and_update_sqlite(db_path)` for incremental updates. Extended recent tracks have their own pair of methods: `fetch_extended_and_save_sqlite(prefix)` and `fetch_extended_and_update_sqlite(db_path)`. The databases can be queried with any SQLite tool:

```bash
sqlite3 data/recent_tracks_20240101_120000.db \
  "SELECT artist, COUNT(*) AS plays FROM recent_tracks GROUP BY artist ORDER BY plays DESC LIMIT 10"
```

Schema overview:

| Resource | Table | Notable columns |
|----------|-------|-----------------|
| `RecentTrack` | `recent_tracks` | `name`, `artist`, `album`, `date_uts` (NULL when now-playing) |
| `RecentTrackExtended` | `recent_tracks_extended` | `name`, `url`, `mbid`, `artist`, `artist_url`, `album`, `album_url`, `date_uts` (NULL when now-playing) |
| `LovedTrack` | `loved_tracks` | `name`, `artist`, `date_uts` |
| `TopTrack` | `top_tracks` | `name`, `artist`, `playcount`, `rank` |
| `TopArtist` | `top_artists` | `name`, `playcount`, `rank` |
| `TopAlbum` | `top_albums` | `name`, `artist`, `playcount`, `rank` |

### Progress Tracking

**Built-in progress bar** (requires the `progress` feature):

```toml
lastfm-client = { version = "5", features = ["progress"] }
```

```rust
// One call — indicatif renders the bar automatically
let tracks = client
    .recent_tracks("username")
    .unlimited()
    .with_progress()
    .fetch()
    .await?;
```

**Custom callback** — `.on_progress(callback)` fires after the total is known
(`fetched = 0`) and then after each batch (~5000 tracks):

```rust
let filename = client
    .recent_tracks("username")
    .unlimited()
    .on_progress(|fetched, total| println!("{fetched}/{total}"))
    .fetch_extended_and_save(FileFormat::Json, "all_tracks")
    .await?;
println!("Saved to {filename}");
```

### Aggregating Recent Tracks (custom periods)

The Top Tracks / Top Artists / Top Albums API only supports fixed periods (`7day`, `1month`, etc.). Use these methods on any fetched `TrackList<RecentTrack>` **or** `TrackList<RecentTrackExtended>` to compute the same views for **any date range**:

```rust
use chrono::{Duration, Utc};

let now = Utc::now();
let two_weeks_ago = now - Duration::weeks(2);

// Works with .fetch() → TrackList<RecentTrack>
let recent = client
    .recent_tracks("username")
    .between(two_weeks_ago.timestamp(), now.timestamp())
    .fetch()
    .await?;

// Also works with .fetch_extended() → TrackList<RecentTrackExtended>
let extended = client
    .recent_tracks("username")
    .between(two_weeks_ago.timestamp(), now.timestamp())
    .fetch_extended()
    .await?;

// All methods below are available on both types:

// Top tracks — equivalent to user.gettoptracks for a custom period
let top_tracks = recent.to_set();
println!("{top_tracks}"); // sorted by play count

// Top artists
let top_artists = extended.top_artists();
for artist in &top_artists {
    println!("{artist}"); // "#1 Radiohead (42 plays)"
}

// Top albums (tracks without album info are excluded)
let top_albums = recent.top_albums();

// Listening clock — plays per UTC hour (index 0 = midnight)
let hours = recent.by_hour();
let (peak_hour, peak_count) = hours
    .iter()
    .enumerate()
    .max_by_key(|(_, &c)| c)
    .map(|(h, &c)| (h, c))
    .unwrap_or((0, 0));
println!("Most active at {peak_hour}:00 UTC ({peak_count} plays)");

// Plays per calendar date — useful for heatmaps
let by_date = extended.by_date(); // BTreeMap<NaiveDate, u32>

// Longest consecutive listening-day streak
let streak = recent.streak();
println!("Best streak: {streak} day(s)");

// Remove currently-playing track before processing
let scrobbles = extended.without_now_playing();

// Distinct counts
println!(
    "{} unique artists, {} unique tracks",
    recent.unique_artist_count(),
    recent.unique_track_count(),
);
```

### Fetching Loved Tracks

```rust
// Limited loved tracks
let loved_tracks = client
    .loved_tracks("username")
    .limit(50)
    .fetch()
    .await?;

// With progress tracking
let all_loved = client
    .loved_tracks("username")
    .unlimited()
    .on_progress(|fetched, total| println!("{fetched}/{total}"))
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
    .fetch()                 // Execute and get TrackList<RecentTrack>
    .fetch_extended()        // Execute and get TrackList<RecentTrackExtended>
    .check_currently_playing() // Check if currently playing
    .analyze(usize)          // Analyze tracks and get statistics
    .analyze_and_print(usize) // Analyze and print statistics
    .fetch_and_save(format, prefix) // Fetch and save to a new timestamped file
    .fetch_extended_and_save(format, prefix) // Fetch extended and save
    .fetch_and_update(file_path) // Fetch only new tracks and prepend to file -> usize
    .fetch_extended_and_update(file_path) // Same for extended tracks -> usize
    // sqlite feature only:
    .fetch_and_save_sqlite(prefix)                  // Fetch and save to a new .db file -> String
    .fetch_and_update_sqlite(db_path)               // Fetch only new tracks and insert into .db -> usize
    .fetch_extended_and_save_sqlite(prefix)         // Fetch extended and save to a new .db file -> String
    .fetch_extended_and_update_sqlite(db_path)      // Fetch only new extended tracks and insert -> usize
    // On a fetched TrackList<RecentTrack>:
    // (all methods take &self and return new values — non-consuming)
    // recent.to_set()               -> TrackList<ScoredTrack>   (deduplicated with play counts)
    // recent.top_artists()          -> TrackList<ScoredArtist>  (grouped by artist)
    // recent.top_albums()           -> TrackList<ScoredAlbum>   (grouped by album+artist)
    // recent.by_hour()              -> [u32; 24]                (plays per UTC hour)
    // recent.by_date()              -> BTreeMap<NaiveDate, u32> (plays per calendar date)
    // recent.streak()               -> u32                      (longest consecutive-day streak)
    // recent.without_now_playing()  -> TrackList<RecentTrack>   (drop currently-playing track)
    // recent.unique_artist_count()  -> usize
    // recent.unique_track_count()   -> usize
```

### Loved Tracks

```rust
client.loved_tracks("username")
    .limit(u32)              // Limit number of tracks
    .unlimited()             // Fetch all available tracks
    .on_progress(|fetched, total| { ... }) // Progress callback (fetched, total)
    .fetch()                 // Execute and get TrackList<LovedTrack>
    .analyze(usize)          // Analyze tracks and get statistics
    .analyze_and_print(usize) // Analyze and print statistics
    .fetch_and_save(format, prefix) // Fetch and save to a new timestamped file
    .fetch_and_update(file_path) // Fetch only new tracks and prepend to file -> usize
    // sqlite feature only:
    .fetch_and_save_sqlite(prefix)        // Fetch and save to a new .db file -> String
    .fetch_and_update_sqlite(db_path)     // Fetch only new tracks and insert into .db -> usize
```

### Top Tracks

```rust
client.top_tracks("username")
    .limit(u32)              // Limit number of tracks
    .unlimited()             // Fetch all available tracks
    .period(Period)          // Time period filter
    .on_progress(|fetched, total| { ... }) // Progress callback (fetched, total)
    .fetch()                 // Execute and get TrackList<TopTrack>
    .fetch_and_save(format, prefix)    // Fetch and save to file
    .fetch_and_save_sqlite(prefix)     // sqlite feature only: save to .db file
```

### Top Artists

```rust
client.top_artists("username")
    .limit(u32)              // Limit number of artists
    .unlimited()             // Fetch all available artists
    .period(Period)          // Time period filter
    .on_progress(|fetched, total| { ... }) // Progress callback (fetched, total)
    .fetch()                 // Execute and get TrackList<TopArtist>
    .fetch_and_save(format, prefix)    // Fetch and save to file
    .fetch_and_save_sqlite(prefix)     // sqlite feature only: save to .db file
```

### Top Albums

```rust
client.top_albums("username")
    .limit(u32)              // Limit number of albums
    .unlimited()             // Fetch all available albums
    .period(Period)          // Time period filter
    .on_progress(|fetched, total| { ... }) // Progress callback (fetched, total)
    .fetch()                 // Execute and get TrackList<TopAlbum>
    .fetch_and_save(format, prefix)    // Fetch and save to file
    .fetch_and_save_sqlite(prefix)     // sqlite feature only: save to .db file
```

### User Info

```rust
client.user_info("username")
    .fetch()    // Execute and get UserInfo

// Convenience check
client.user_exists("username").await? // -> bool
```

### Top Tags

```rust
client.top_tags("username")
    .limit(u32)    // 1–50; values above 50 are clamped to 50 (API cap)
    .fetch()       // Execute and get Vec<UserTopTag>
```

### Friends

```rust
client.friends("username")
    .limit(u32)          // Results per page (default 50, max 200)
    .page(u32)           // Page number (1-indexed)
    .recent_tracks(bool) // Include recent track info for each friend
    .fetch_page()        // Execute and get FriendsPage
    .fetch_all()         // Auto-paginate and get Vec<FriendProfile>
```

### Personal Tags

```rust
client.personal_tags("username", "tag")
    .limit(u32)        // Results per page (default 50, max 1000)
    .page(u32)         // Page number (1-indexed)
    .fetch_tracks()    // Execute and get PersonalTaggedTracksPage
    .fetch_artists()   // Execute and get PersonalTaggedArtistsPage
    .fetch_albums()    // Execute and get PersonalTaggedAlbumsPage
```

### Weekly Charts

```rust
client.weekly_chart_list("username")
    .fetch()    // Execute and get Vec<WeeklyChartRange>

client.weekly_track_chart("username")
    .from(u32)                 // Start Unix timestamp (optional)
    .to(u32)                   // End Unix timestamp (optional)
    .range(&WeeklyChartRange)  // Set both from + to from a range value
    .fetch()                   // Execute and get Vec<WeeklyTrack>

// Same builder interface for:
client.weekly_artist_chart("username") // -> Vec<WeeklyArtist>
client.weekly_album_chart("username")  // -> Vec<WeeklyAlbum>
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
- **`format`**: `FileFormat` - `FileFormat::Json`, `FileFormat::Csv`, or `FileFormat::Ndjson`

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
│   ├── lastfm.rs            # LastFmClient — single http + config, all builder methods
│   ├── http.rs              # HttpClient trait + ReqwestClient + MockClient
│   ├── retry.rs             # RetryClient + RetryPolicy (exponential/linear backoff)
│   └── rate_limiter.rs      # RateLimiter + RateLimitedClient (sliding window)
├── api/
│   ├── builder_ext.rs       # Shared extension traits (LimitBuilder, FetchAndSave, …)
│   ├── constants.rs         # API method name constants
│   ├── fetch_utils.rs       # Generic pagination, ResourceContainer, user_params helper
│   └── user/
│       ├── recent_tracks/
│       │   ├── mod.rs       # Re-exports + ResourceContainer impls
│       │   ├── builder.rs   # RecentTracksRequestBuilder + trait impls
│       │   └── extended.rs  # fetch_extended / fetch_extended_and_* methods
│       ├── loved_tracks.rs  # LovedTracksRequestBuilder + trait impls
│       ├── top/
│       │   ├── tracks.rs    # TopTracksRequestBuilder
│       │   ├── artists.rs   # TopArtistsRequestBuilder
│       │   ├── albums.rs    # TopAlbumsRequestBuilder
│       │   └── tags.rs      # TopTagsRequestBuilder
│       ├── info.rs          # UserInfoRequestBuilder
│       ├── friends.rs       # FriendsRequestBuilder
│       ├── personal_tags.rs # PersonalTagsRequestBuilder
│       └── weekly/
│           └── charts.rs    # Weekly chart builders (list, track, artist, album)
├── types/
│   ├── tracks.rs            # RecentTrack, LovedTrack, TopTrack, scored types
│   ├── artists.rs           # TopArtist
│   ├── albums.rs            # TopAlbum
│   ├── tags.rs              # UserTopTag
│   ├── friends.rs           # FriendProfile, FriendsPage
│   ├── personal_tags.rs     # PersonalTagged* types and pages
│   ├── weekly.rs            # WeeklyChartRange, WeeklyTrack/Artist/Album
│   ├── user_info.rs         # UserInfo
│   ├── track_list.rs        # TrackList<T> newtype with Display + Deref
│   └── period.rs            # Period and TrackLimit enums
├── config.rs                # Config + ConfigBuilder
├── error.rs                 # LastFmError with retry hints
├── analytics.rs             # TrackAnalyzable trait + AnalysisHandler
├── file_handler.rs          # JSON/NDJSON/CSV/SQLite export
└── sqlite.rs                # SqliteExportable trait (sqlite feature only)
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
