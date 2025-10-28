# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-01-XX

### Added

#### New V2.0 API
- **LastFmClient**: New main client with builder pattern for configuration
- **RecentTracksClient**: Dedicated client for recent tracks with fluent API
- **Builder Pattern**: All API methods now support method chaining
  - `.limit(n)` - Limit number of tracks
  - `.unlimited()` - Fetch all available tracks
  - `.since(timestamp)` - Fetch tracks since a timestamp
  - `.between(from, to)` - Fetch tracks between two timestamps
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
  - Exponential backoff (default): 100ms → 200ms → 400ms → 800ms...
  - Linear backoff: 1s → 2s → 3s → 4s...
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
- `LastFmError::RateLimited`: Includes optional retry_after duration
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

#### Documentation
- Comprehensive examples in `examples/new_api_demo.rs`
- Advanced features demo in `examples/advanced_features.rs`
- Updated README with v2.0 API documentation
- Migration guide for v1.x → v2.0
- Detailed PHASE2_SUMMARY.md documenting the refactoring process

#### Testing
- 35 tests (up from 7 in v1.1.0) - 400% increase
- Mock HTTP client for unit testing
- Tests for retry logic, rate limiting, error handling
- Tests for custom deserializers and type conversions

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
- Renamed `types.rs` to `legacy_types.rs` (internal, no API change)

#### Dependencies
- Added `async-trait = "0.1"` - For async trait methods
- Added `thiserror = "1.0"` - For ergonomic error definitions
- Added `parking_lot = "0.12"` - For efficient mutex in rate limiter
- Added `tracing = "0.1"` - For structured logging
- Removed `mockito` - No longer needed with new HttpClient trait
- Removed `tabular` - Unused dependency

### Deprecated

Nothing deprecated. The v1.x API remains fully supported.

### Removed

- `mockito` dependency (replaced by built-in MockClient)
- `tabular` dependency (unused)

### Fixed

- Race conditions in concurrent requests with proper mutex scoping
- Borrow checker issues in rate limiter by restructuring lock usage
- Type conversion overhead by unifying API and storage types
- Inconsistent error handling across different API methods
- Missing error context in API failures

### Security

- Rate limiting prevents accidental API abuse
- Proper timeout handling prevents hanging requests
- Structured logging replaces debug print statements

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

## Migration Guide

### From v1.x to v2.0

**Good news**: v2.0 is 100% backward compatible! You don't need to change anything.

However, we recommend migrating to the new API for better ergonomics:

#### Before (v1.x)
```rust
let handler = LastFMHandler::new("username")?;
let tracks = handler.get_user_recent_tracks_with_options(
    Some(50),
    Some(from),
    Some(to),
    true
).await?;
```

#### After (v2.0)
```rust
let client = LastFmClient::builder().from_env()?.build()?;
let tracks = client
    .recent_tracks("username")
    .limit(50)
    .between(from, to)
    .fetch()
    .await?;
```

See `MIGRATION.md` for detailed migration instructions.

---

## Versioning Policy

- **Major version** (X.0.0): Breaking API changes
- **Minor version** (x.X.0): New features, backward compatible
- **Patch version** (x.x.X): Bug fixes, backward compatible

## Support

- v2.x: Active development, full support
- v1.x: Maintenance mode, security updates only

## Links

- [Repository](https://github.com/tom_planche/lastfm-client)
- [Documentation](https://docs.rs/lastfm-client)
- [Last.fm API](https://www.last.fm/api)
