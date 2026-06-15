# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [4.0.2] - 2026-06-15

### Fixed

- **Single-item list responses from Last.fm no longer fail to parse.** Every `fetch()` /
  `fetch_extended()` issues an initial `limit=1` probe request to discover the total item
  count. Last.fm collapses single-element lists into a bare object instead of a one-element
  array, which broke deserialization with `invalid type: map, expected a sequence`. This was
  most visible via `fetch_extended_and_update_sqlite` against a fresh database (no prior
  timestamp, so the probe path always runs) and when the user was not currently playing a
  track. A new `vec_or_single` deserializer accepts both shapes and is applied to the list
  fields of `RecentTracks`, `RecentTracksExtended`, `LovedTracks`, `TopTracks`, `TopArtists`,
  and `TopAlbums`.

### Changed (internal)

- Resolved `clippy::pedantic` lints newly raised under Rust 1.96.0: `Duration::from_secs(60)`
  replaced with `Duration::from_mins(1)` (`http.rs`, `config.rs`), and
  `sort_unstable_by(|a, b| b.cmp(&a))` replaced with
  `sort_unstable_by_key(|b| std::cmp::Reverse(...))` (`tracks.rs`). No behaviour change.

## [4.0.0] - 2026-04-04

### Changed (breaking)

Builder methods `limit`, `unlimited`, `fetch_and_save`, `fetch_and_save_sqlite`,
`fetch_and_update`, `fetch_and_update_sqlite`, `analyze`, and `analyze_and_print` have moved
from concrete implementations on each builder to shared extension traits. You must import the
relevant trait to call these methods.

```rust
// Before (3.x)
use lastfm_client::LastFmClient;
client.recent_tracks("user").limit(50).fetch().await?;

// After (4.0)
use lastfm_client::{LastFmClient, LimitBuilder};
client.recent_tracks("user").limit(50).fetch().await?;
```

| Trait | Methods | Import |
|-------|---------|--------|
| `LimitBuilder` | `limit`, `unlimited` | `use lastfm_client::LimitBuilder;` |
| `FetchAndSave` | `fetch_and_save`, `fetch_and_save_sqlite` | `use lastfm_client::FetchAndSave;` |
| `FetchAndUpdate` | `fetch_and_update`, `fetch_and_update_sqlite` | `use lastfm_client::FetchAndUpdate;` |
| `Analyze` | `analyze`, `analyze_and_print` | `use lastfm_client::Analyze;` |

All four traits are re-exported from the crate root and from a new `prelude` module.
The recommended import is:

```rust
use lastfm_client::{LastFmClient, prelude::*};
```

**`XClient` intermediate types removed from the public API.**
`LovedTracksClient`, `RecentTracksClient`, `TopTracksClient`, `TopArtistsClient`,
and `TopAlbumsClient` are no longer exported from the crate root.
These types existed only as factory objects; use `LastFmClient` directly:

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

**`LastFmClient` simplified**: the struct now holds a single `Arc<dyn HttpClient>` plus
`Arc<Config>` instead of one sub-client per endpoint. The public builder methods
(`recent_tracks`, `loved_tracks`, etc.) are unchanged.

### Added

- `LimitBuilder` trait: single `limit_mut()` hook, default `limit(n)` and `unlimited()` — now
  implemented on all five builder types.
- `FetchAndSave` trait: new `latest_timestamp` hook (default `None`) lets timestamped builders
  write a sidecar file after saving without duplicating that logic per-builder.
- `FetchAndUpdate` trait: `fetch_and_update` default now handles NDJSON files in addition to
  JSON and CSV.
- `Analyze` trait: blanket impl — any builder implementing `FetchAndSave` whose item type
  implements `TrackAnalyzable` automatically gets `analyze` and `analyze_and_print`.
- `on_progress(callback)` builder method is now available on all five resource builders
  (`recent_tracks`, `loved_tracks`, `top_tracks`, `top_artists`, `top_albums`). Previously
  it existed only on `recent_tracks`.
- `with_progress()` builder method (requires the new `progress` feature) renders a live
  terminal progress bar while fetching, backed by [`indicatif`](https://github.com/console-rs/indicatif).
  Enable the feature and call `.with_progress()` before `.fetch()`:
  ```toml
  lastfm-client = { version = "4", features = ["progress"] }
  ```
  ```rust
  let tracks = client.recent_tracks("user").with_progress().fetch().await?;
  ```
  The bar shows `{spinner} [{bar:40}] {pos}/{len} ({percent}%)` and finishes automatically.
- `full` feature that enables both `sqlite` and `progress` at once:
  ```toml
  lastfm-client = { version = "4", features = ["full"] }
  ```
- **`user.getInfo`**: `client.user_info("username").fetch()` returns `UserInfo` with scrobble count,
  real name, country, registration date, and subscriber status.
- **`client.user_exists("username")`**: convenience method returning `Ok(true/false)`;
  API error codes 6 and 7 map to `Ok(false)`, all other errors propagate.
- **`user.getTopTags`**: `client.top_tags("username").limit(n).fetch()` returns `Vec<UserTopTag>`.
  Maximum 50 tags; values above 50 are clamped automatically.
- **`user.getFriends`**: `client.friends("username")` builder with `.fetch_page()` (single page)
  and `.fetch_all()` (auto-paginated). Returns `FriendsPage` / `Vec<FriendProfile>`.
- **`user.getPersonalTags`**: `client.personal_tags("username", "tag")` builder with three terminal
  methods — `.fetch_tracks()`, `.fetch_artists()`, `.fetch_albums()` — returning the corresponding
  `PersonalTagged*Page` type.
- **Weekly chart endpoints** (four new methods on `LastFmClient`):
  - `client.weekly_chart_list("username").fetch()` → `Vec<WeeklyChartRange>`
  - `client.weekly_track_chart("username").range(&range).fetch()` → `Vec<WeeklyTrack>`
  - `client.weekly_artist_chart("username").range(&range).fetch()` → `Vec<WeeklyArtist>`
  - `client.weekly_album_chart("username").range(&range).fetch()` → `Vec<WeeklyAlbum>`
  - All three chart builders expose `.from(u32)` and `.to(u32)` for manual range selection
    in addition to the `.range(&WeeklyChartRange)` convenience setter.
- New public types: `UserInfo`, `UserTopTag`, `FriendProfile`, `FriendsPage`,
  `PersonalTaggedTrack`, `PersonalTaggedArtist`, `PersonalTaggedAlbum`,
  `PersonalTaggedTracksPage`, `PersonalTaggedArtistsPage`, `PersonalTaggedAlbumsPage`,
  `WeeklyChartRange`, `WeeklyTrack`, `WeeklyArtist`, `WeeklyAlbum`.

### Changed (internal)

- API method strings (`"user.getrecenttracks"`, `"user.getlovedtracks"`, etc.) moved from
  inline string literals to named constants in `src/api/constants.rs`.
- `src/api/recent_tracks.rs` split into a module directory (`recent_tracks/mod.rs`,
  `builder.rs`, `extended.rs`) for better organisation. Public API is unchanged.
- Duplicate `from`/`to` date-range guard extracted to a shared `validate_date_range` helper
  called by both `fetch()` and `fetch_extended()`. Behaviour is unchanged.
- `fetch_and_update` logic unified under the `FetchAndUpdate::fetch_since` trait hook.
- `LastFmClient::with_http` constructor simplified — no sub-client construction.
- All `XRequestBuilder::new()` methods are now `pub(crate)`, constructed directly
  by `LastFmClient`.
- `user_params(method, username, api_key)` helper extracted to `src/api/fetch_utils.rs` to
  eliminate repeated param setup across every `fetch()` method.
- `HttpClient` trait now requires `std::fmt::Debug` as a supertrait, enabling
  `Arc<dyn HttpClient>: Debug` and `#[derive(Debug)]` on simpler builder structs.

### Removed

Duplicate concrete implementations of `limit`, `unlimited`, `fetch_and_save`,
`fetch_and_save_sqlite`, `fetch_and_update`, `fetch_and_update_sqlite`, `analyze`, and
`analyze_and_print` have been removed from each individual builder.

- `FileHandler::append` — unused public method superseded by `append_or_create_csv`, `append_or_create_ndjson`, and `prepend_json`
- `AnalysisHandler::get_most_recent_timestamp` — unused public method superseded by the sidecar timestamp mechanism (v3.2.0)

## [3.9.0] - 2026-04-03

### Added

`SQLite` load support — a read-side companion to the existing export API.

#### New trait: `SqliteLoadable` (`src/sqlite.rs`, `sqlite` feature)

```rust
pub trait SqliteLoadable: Sized {
    fn select_sql() -> &'static str;
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>;
}
```

Implemented for all six persistable types: `RecentTrack`, `RecentTrackExtended`,
`LovedTrack`, `TopTrack`, `TopArtist`, `TopAlbum`.

**Note on field coverage**: SQLite schemas store a curated subset of fields.
Fields not persisted (e.g. `image`, `streamable`, human-readable date strings)
are reconstructed with empty/default values. All analysis methods (`to_set()`,
`top_artists()`, `by_date()`, `streak()`, etc.) work correctly on loaded data.

#### New method: `FileHandler::load_sqlite` (`sqlite` feature)

```rust
pub fn load_sqlite<T: SqliteLoadable>(file_path: &str) -> io::Result<TrackList<T>>
```

Loads all rows from a `.db` file produced by `fetch_and_save_sqlite` or
`fetch_and_update_sqlite`. Returns a `TrackList<T>` ready for analysis.

#### Exports

- `SqliteLoadable` is now re-exported from the crate root alongside `SqliteExportable`

## [3.8.0] - 2026-04-03

### Added

`impl TrackList<RecentTrackExtended>` — the same local aggregation and analysis helpers as
`TrackList<RecentTrack>` (v3.7), for data from `.fetch_extended()` / extended API responses.

#### New methods on `TrackList<RecentTrackExtended>`

| Method | Returns | Description |
|--------|---------|-------------|
| `to_set()` | `TrackList<ScoredTrack>` | Deduplicate by `(name, artist)` and count plays |
| `top_artists()` | `TrackList<ScoredArtist>` | Group by artist and count plays |
| `top_albums()` | `TrackList<ScoredAlbum>` | Group by `(album, artist)` and count plays; empty-album tracks excluded |
| `by_hour()` | `[u32; 24]` | Play counts per UTC hour (index = hour 0–23) |
| `by_date()` | `BTreeMap<NaiveDate, u32>` | Play counts per calendar date (UTC), sorted chronologically |
| `streak()` | `u32` | Longest consecutive listening-day streak |
| `without_now_playing()` | `TrackList<RecentTrackExtended>` | Copy of the list without the currently-playing track |
| `unique_artist_count()` | `usize` | Number of distinct artist names |
| `unique_track_count()` | `usize` | Number of distinct `(name, artist)` pairs |

All methods take `&self` (non-consuming) and are `#[must_use]`.

Implementation matches `RecentTrack` where possible; extended rows use `artist.name` /
`album.name` (`BaseObject`) instead of `artist.text` / `album.text` (`BaseMbidText`), and
the now-playing flag is read from `attr["nowplaying"]` when `attr` is a `HashMap`.

## [3.7.0] - 2026-04-03

### Added

Local aggregation and analysis methods on `TrackList<RecentTrack>` — a complement to the Top Tracks / Top Artists / Top Albums API endpoints for any custom date range.

#### New types

- **`ScoredTrack`** — a track with a locally-computed `play_count` and `rank`; output of `to_set()`
- **`ScoredArtist`** — an artist with a locally-computed `play_count` and `rank`; output of `top_artists()`
- **`ScoredAlbum`** — an album with a locally-computed `play_count` and `rank`; output of `top_albums()`

All three types implement `Display`, `Ord` (by `play_count`), `Eq`, `Serialize`, and `Clone`. They are re-exported from the crate root.

#### New methods on `TrackList<RecentTrack>`

| Method | Returns | Description |
|--------|---------|-------------|
| `to_set()` | `TrackList<ScoredTrack>` | Deduplicate by `(name, artist)` and count plays |
| `top_artists()` | `TrackList<ScoredArtist>` | Group by artist and count plays |
| `top_albums()` | `TrackList<ScoredAlbum>` | Group by `(album, artist)` and count plays; empty-album tracks excluded |
| `by_hour()` | `[u32; 24]` | Play counts per UTC hour (index = hour 0–23) |
| `by_date()` | `BTreeMap<NaiveDate, u32>` | Play counts per calendar date (UTC), sorted chronologically |
| `streak()` | `u32` | Longest consecutive listening-day streak |
| `without_now_playing()` | `TrackList<RecentTrack>` | Copy of the list without the currently-playing track |
| `unique_artist_count()` | `usize` | Number of distinct artist names |
| `unique_track_count()` | `usize` | Number of distinct `(name, artist)` pairs |

All methods take `&self` (non-consuming) and are `#[must_use]`.

## [3.6.0] - 2026-04-01

### Added

- **`TrackList<T>`** (`src/types/track_list.rs`): Newtype wrapper around `Vec<T>` returned by all `fetch()` and `fetch_extended()` methods. Implements `Display` (items printed in descending order — most recent first for time-stamped types, most played first for playcount types), `Deref`/`DerefMut` to `Vec<T>`, `From<Vec<T>>`, `From<TrackList<T>> for Vec<T>`, `IntoIterator` (by value, shared ref, and mutable ref), and `FromIterator`. All existing code using the result as a slice or vec continues to work unchanged via deref coercion.
- **`Ord` (and `PartialOrd`, `Eq`, `PartialEq`) for all resource types**:
  - `RecentTrack`, `RecentTrackExtended`: ordered by `date.uts`; a `None` date (now-playing track) sorts as the most recent.
  - `LovedTrack`: ordered by `date.uts`.
  - `TopTrack`, `TopArtist`, `TopAlbum`: ordered by `playcount`.
- **`TrackList` re-exported** from the crate root.

### Changed

- All public `fetch()` / `fetch_extended()` return types changed from `Result<Vec<T>>` to `Result<TrackList<T>>` (minor API change; `TrackList<T>` derefs to `Vec<T>` so call sites are unaffected in practice).

## [3.5.0] - 2026-03-17

### Added

- **`SqliteExportable` for `RecentTrackExtended`**: The extended recent track type now implements `SqliteExportable`, storing data in a `recent_tracks_extended` table. The schema includes all `recent_tracks` columns plus `mbid`, `artist_url`, and `album_url` (available from the `BaseObject` type returned by the extended API).
- **`fetch_extended_and_save_sqlite(prefix)`** on `RecentTracksRequestBuilder`: Fetches all extended recent tracks and saves them to a new timestamped `.db` file.
- **`fetch_extended_and_update_sqlite(db_path)`** on `RecentTracksRequestBuilder`: Reads `MAX(date_uts)` from the `recent_tracks_extended` table, fetches only newer tracks, and appends them. No sidecar file needed.

### SQLite schema addition

| Type | Table | Notable columns |
|------|-------|-----------------|
| `RecentTrackExtended` | `recent_tracks_extended` | `name`, `url`, `mbid`, `artist`, `artist_mbid`, `artist_url`, `album`, `album_mbid`, `album_url`, `date_uts` (NULL when now-playing), `loved` |

## [3.4.0] - 2026-03-16

### Added

- **`FileFormat::Ndjson`**: New export format that writes one compact JSON object per line (`.ndjson` extension)
- **`FileHandler::save_as_ndjson`** (private): Creates an NDJSON file from a slice
- **`FileHandler::append_or_create_ndjson`**: Appends items as new lines to an existing NDJSON file, or creates it if it does not exist; mirrors `append_or_create_csv`
- **`FileHandler::load_ndjson`**: Deserializes an NDJSON file line-by-line into `Vec<T>`
- **NDJSON support in incremental update flow**: `fetch_and_update` on `RecentTracksRequestBuilder` and `LovedTracksRequestBuilder` now detects `.ndjson` paths and appends new records (oldest-first), complementing the existing JSON-prepend and CSV-append strategies. The sidecar `.meta` timestamp mechanism works identically.
- **`append()` handles `.ndjson`**: The generic `FileHandler::append` method now recognises the `.ndjson` extension alongside `.json` and `.csv`

## [3.3.0] - 2026-03-16

### Added

- **SQLite export** (optional feature `sqlite`): Enable with `lastfm-client = { version = "3.3", features = ["sqlite"] }` in `Cargo.toml`
- **`SqliteExportable` trait** (`src/sqlite.rs`): Implemented for `RecentTrack`, `LovedTrack`, `TopTrack`, `TopArtist`, and `TopAlbum`; each declares its table schema and row-binding logic
- **`FileHandler::save_sqlite`**: Creates a timestamped `.db` file under `data/` and bulk-inserts all rows in a single transaction
- **`FileHandler::append_or_create_sqlite`**: Opens an existing database or creates a new one, creates the table if absent, and inserts rows; used by the incremental update flow
- **`FileHandler::read_sqlite_max_timestamp`**: Queries `MAX(date_uts)` from a table to determine the latest stored timestamp without loading all data
- **`fetch_and_save_sqlite(prefix)`** on all five API builders (`RecentTracksRequestBuilder`, `LovedTracksRequestBuilder`, `TopTracksRequestBuilder`, `TopArtistsRequestBuilder`, `TopAlbumsRequestBuilder`)
- **`fetch_and_update_sqlite(db_path)`** on `RecentTracksRequestBuilder` and `LovedTracksRequestBuilder`: reads `MAX(date_uts)` directly from the database (no sidecar file needed) and inserts only new rows

### SQLite schema

| Type | Table | Key columns |
|------|-------|-------------|
| `RecentTrack` | `recent_tracks` | `id PK`, `name`, `artist`, `album`, `date_uts` (nullable for now-playing), `loved` |
| `LovedTrack` | `loved_tracks` | `id PK`, `name`, `artist`, `date_uts` |
| `TopTrack` | `top_tracks` | `id PK`, `name`, `artist`, `playcount`, `rank` |
| `TopArtist` | `top_artists` | `id PK`, `name`, `playcount`, `rank` |
| `TopAlbum` | `top_albums` | `id PK`, `name`, `artist`, `playcount`, `rank` |

## [3.2.0] - 2026-03-16

### Added

- **Incremental file updates**: `RecentTracksRequestBuilder::fetch_and_update(file_path)` and `fetch_extended_and_update(file_path)` fetch only tracks newer than the most recent entry in an existing file, then write them. JSON files have new tracks prepended (newest-first); CSV files have new tracks appended. If the file does not exist it is created with a full fetch.
- **Incremental loved tracks updates**: `LovedTracksRequestBuilder::fetch_and_update(file_path)` does the same for loved tracks. Because the loved tracks API has no `from` timestamp filter, all tracks are fetched and already-present entries are filtered out by timestamp before writing.
- **CSV support in incremental updates**: Pass a `.csv` path to any `fetch_and_update` method to maintain a growing CSV file. New rows are appended without re-writing the whole file. The sidecar (`.meta`) is used as the timestamp source; the CSV slow-path scan is skipped because complex nested types do not round-trip through CSV reliably.
- **`FileHandler::append_or_create_csv`**: Creates a CSV file with headers on first call; appends rows without headers on subsequent calls.
- **Sidecar metadata file**: After each update, the latest Unix timestamp is written to `{file_path}.meta`. Subsequent calls read this tiny sidecar instead of deserializing the full data file, making repeated calls O(1) for timestamp lookup regardless of file size.
- **`FileHandler::load`**: Public method to deserialize an existing JSON file into `Vec<T>`.
- **`FileHandler::prepend_json`**: Prepends new items to the front of an existing JSON array file (or creates the file), preserving newest-first order.
- **`FileHandler::sidecar_path` / `read_sidecar_timestamp` / `write_sidecar_timestamp`**: Helpers for the sidecar metadata file.

## [3.1.0] - 2026-03-16

### Added

- **Progress callbacks**: `RecentTracksRequestBuilder::on_progress(callback)` accepts any `Fn(u32, u32) + Send + Sync + 'static` and fires with `(fetched, total)` after each batch of tracks is received (and once at `(0, total)` when the total is first known from the API)
- **`ProgressCallback` type alias**: `Arc<dyn Fn(u32, u32) + Send + Sync>` is now exported from the crate root for use in calling code
- Generic `fetch()` in `fetch_utils` accepts an optional `ProgressCallback`; all existing callers (`loved_tracks`, `top_tracks`, `top_artists`, `top_albums`) pass `None` and are unaffected

## [3.0.0] - 2026-02-07

### Added

- **TopArtistsClient**: Fetch top artists via `client.top_artists("username")` with builder pattern
- **TopAlbumsClient**: Fetch top albums via `client.top_albums("username")` with builder pattern
- **TopArtist type**: `TopArtist` struct for `user.gettopartists` responses
- **TopAlbum type**: `TopAlbum` struct with `artist` field (`BaseObject`) for `user.gettopalbums` responses
- **Unified `ResourceContainer` trait**: Single trait in `fetch_utils.rs` used across all resource types (`ItemType`, `total()`, `items()`)

### Removed

- **`lastfm_handler` module** (BREAKING): The entire deprecated v1.x API has been removed
  - `LastFMHandler` struct and all its methods (`get_user_recent_tracks`, `get_user_top_tracks`, etc.)
  - The duplicate private `ResourceContainer` trait that used different names (`ResourceType`, `total_resources()`, `resources()`)
  - `TrackLimit` enum from `lastfm_handler` (the one in `types::period` remains)
- All v1.x deprecation warnings are gone since the deprecated code no longer exists

### Changed

- **`TrackPlayInfo` moved** (BREAKING): From `lastfm_client::lastfm_handler::TrackPlayInfo` to `lastfm_client::types::TrackPlayInfo`
- Updated `file_handler.rs` to import `TrackPlayInfo` from `crate::types`
- Updated README: removed all v1.x documentation, added top artists/albums sections, added v2.x -> v3.0 migration table
- Updated CLAUDE.md: reflects current architecture without legacy references

### Migration from v2.x

| v2.x (removed) | v3.0 equivalent |
|---|---|
| `LastFMHandler::new("user")` | `LastFmClient::new()?` |
| `handler.get_user_recent_tracks(Some(100))` | `client.recent_tracks("user").limit(100).fetch().await?` |
| `handler.get_user_recent_tracks_between(from, to, false)` | `client.recent_tracks("user").between(from, to).fetch().await?` |
| `handler.get_user_top_tracks(Some(50), Some(Period::Week))` | `client.top_tracks("user").limit(50).period(Period::Week).fetch().await?` |
| `handler.get_user_loved_tracks(Some(100))` | `client.loved_tracks("user").limit(100).fetch().await?` |
| `lastfm_client::lastfm_handler::TrackPlayInfo` | `lastfm_client::types::TrackPlayInfo` |

## [2.0.0] - 2025-01-XX

### Added

#### New V2.0 API
- **LastFmClient**: New main client with builder pattern for configuration
- **RecentTracksClient**: Dedicated client for recent tracks with fluent API
- **Builder Pattern**: All API methods now support method chaining
  - `.limit(n)` - Limit number of tracks
  - `.unlimited()` - Fetch all available tracks
  - `.since(timestamp)` - Fetch tracks since a timestamp
  - `.between(from, to)` - Fetch tracks between two timestamps (validates to > from)
  - `.extended(bool)` - Include extended track information
  - `.fetch()` - Execute request and return results
  - `.fetch_extended()` - Execute request with extended information

#### HTTP Abstraction Layer
- **HttpClient trait**: Abstraction for HTTP communication enabling testability
- **ReqwestClient**: Production HTTP client implementation
- **MockClient**: Mock HTTP client for testing with predefined responses

#### Retry Logic
- **RetryClient**: Automatic retry wrapper for HTTP requests
- **RetryPolicy**: Configurable retry strategies
  - Exponential backoff (default): 100ms -> 200ms -> 400ms -> 800ms...
  - Linear backoff: 1s -> 2s -> 3s -> 4s...
  - Custom policies with configurable base delay and max delay
- Respects `is_retryable()` hints from error types
- Honors server-specified retry delays via `retry_after()`

#### Rate Limiting
- **RateLimiter**: Sliding window rate limiting to prevent API abuse
- **RateLimitedClient**: HTTP client wrapper with rate limiting
- Configurable max requests per time window
- Thread-safe implementation using `parking_lot` and `tokio::Semaphore`
- Automatic request pacing with async blocking

#### Enhanced Error Handling
- Rich error types with context and retry hints
- `LastFmError::Api`: Now includes method name, error code, and retryable flag
- **API error classification**: Automatic categorization of Last.fm error codes
  - Retryable: 8 (operation failed), 11 (service offline), 16 (temporary), 29 (rate limit)
  - Non-retryable: 6 (invalid params), 9 (invalid session), 10 (invalid key), etc.
- `LastFmError::RateLimited`: Includes optional retry_after duration
- Detection of Last.fm API quirk: errors returned with HTTP 200 status
- `.is_retryable()` method to check if error can be retried
- `.retry_after()` method to get suggested retry delay

#### Configuration
- **ConfigBuilder**: Builder pattern for client configuration
  - `.api_key()` - Set API key directly
  - `.from_env()` - Load from environment variables
  - `.user_agent()` - Custom user agent
  - `.timeout()` - Request timeout
  - `.max_concurrent_requests()` - Concurrency limit
  - `.retry_attempts()` - Number of retry attempts
  - `.rate_limit()` - Rate limiting configuration

#### Type System
- **Unified types**: Removed Api* prefix from all types (breaking change for internal use only)
- Enhanced deserializers handling both string and numeric JSON values
- **Period enum**: Type-safe time period representation
- **TrackLimit enum**: Explicit Limited/Unlimited distinction

#### Testing
- 62 tests (up from 7 in v1.1.0)
  - 34 unit tests in `src/`
  - 28 integration tests in `tests/integration_test.rs`
  - 13 doc tests
- Comprehensive integration test suite with mock HTTP responses
- Tests for retry logic, rate limiting, error handling, date validation
- Fast execution (< 100ms) using mock strategy

### Changed

#### Production Quality Improvements
- Replaced all `println!` debug statements with structured `tracing` logging
- Fixed unsafe `.unwrap()` calls with proper error handling
- Improved error messages with context
- Better separation of concerns with modular architecture

#### Internal Architecture
- Split monolithic `LastFMHandler` into focused clients
- New module structure:
  - `src/client/` - HTTP client implementations
  - `src/api/` - API-specific clients (recent_tracks, etc.)
  - `src/types/` - Type definitions organized by domain

#### Dependencies
- Added `async-trait = "0.1"` - For async trait methods
- Added `thiserror = "1.0"` - For ergonomic error definitions
- Added `parking_lot = "0.12"` - For efficient mutex in rate limiter
- Added `tracing = "0.1"` - For structured logging
- Removed `mockito` - No longer needed with new HttpClient trait
- Removed `tabular` - Unused dependency

### Deprecated

- All v1.x API methods via `LastFMHandler` (removed in v3.0.0)

### Removed

- `mockito` dependency (replaced by built-in MockClient)
- `tabular` dependency (unused)

### Fixed

- Race conditions in concurrent requests with proper mutex scoping
- Borrow checker issues in rate limiter by restructuring lock usage
- Type conversion overhead by unifying API and storage types
- Inconsistent error handling across different API methods
- Missing error context in API failures
- Date range validation prevents invalid API calls with clear error messages

## [1.1.0] - 2024-XX-XX

### Added
- Support for loved tracks fetching
- Support for top tracks fetching
- Extended track information support
- Time-based filtering with `from`/`to` timestamps
- Period-based filtering for top tracks
- Comprehensive `*_with_options` methods exposing all Last.fm API parameters
- Convenience methods for date range queries

### Changed
- Improved pagination handling
- Better error messages

## [1.0.0] - Initial Release

### Added
- Basic Last.fm API integration
- Recent tracks fetching
- Analytics functionality
- JSON and CSV export
- Error handling

---

## Versioning Policy

- **Major version** (X.0.0): Breaking API changes
- **Minor version** (x.X.0): New features, backward compatible
- **Patch version** (x.x.X): Bug fixes, backward compatible

## Links

- [Repository](https://github.com/TomPlanche/lastfm-client)
- [Documentation](https://docs.rs/lastfm-client)
- [Last.fm API](https://www.last.fm/api)
