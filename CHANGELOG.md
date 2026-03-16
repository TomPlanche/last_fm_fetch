# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
