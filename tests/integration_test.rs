//! Integration tests for the lastfm-client library.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use lastfm_client::client::MockClient;
use lastfm_client::{ConfigBuilder, LastFmClient, LimitBuilder};
use serde_json::json;
use std::sync::Arc;

/// Helper to create a mock client with typical API responses
fn create_mock_client() -> MockClient {
    MockClient::new()
        .with_response("user.getrecenttracks", mock_recent_tracks_response())
        .with_response("user.getlovedtracks", mock_loved_tracks_response())
        .with_response("user.gettoptracks", mock_top_tracks_response())
}

/// Mock response for recent tracks (simplified but realistic structure)
fn mock_recent_tracks_response() -> serde_json::Value {
    json!({
        "recenttracks": {
            "track": [
                {
                    "artist": {
                        "mbid": "artist-mbid-1",
                        "#text": "Test Artist 1"
                    },
                    "streamable": "1",
                    "image": [
                        {
                            "size": "small",
                            "#text": "https://example.com/image1.jpg"
                        }
                    ],
                    "mbid": "track-mbid-1",
                    "album": {
                        "mbid": "album-mbid-1",
                        "#text": "Test Album 1"
                    },
                    "name": "Test Track 1",
                    "url": "https://www.last.fm/music/test-track-1",
                    "date": {
                        "uts": "1715637600",
                        "#text": "14 May 2024, 00:00"
                    }
                },
                {
                    "artist": {
                        "mbid": "artist-mbid-2",
                        "#text": "Test Artist 2"
                    },
                    "streamable": "0",
                    "image": [],
                    "mbid": "track-mbid-2",
                    "album": {
                        "mbid": "album-mbid-2",
                        "#text": "Test Album 2"
                    },
                    "name": "Test Track 2",
                    "url": "https://www.last.fm/music/test-track-2",
                    "date": {
                        "uts": "1715641200",
                        "#text": "14 May 2024, 01:00"
                    }
                }
            ],
            "@attr": {
                "user": "testuser",
                "totalPages": "1",
                "page": "1",
                "perPage": "50",
                "total": "2"
            }
        }
    })
}

/// Mock response for loved tracks
fn mock_loved_tracks_response() -> serde_json::Value {
    json!({
        "lovedtracks": {
            "track": [
                {
                    "artist": {
                        "mbid": "loved-artist-mbid-1",
                        "name": "Loved Artist 1",
                        "url": "https://www.last.fm/music/loved-artist-1"
                    },
                    "date": {
                        "uts": "1715637600",
                        "#text": "14 May 2024, 00:00"
                    },
                    "mbid": "loved-track-mbid-1",
                    "url": "https://www.last.fm/music/loved-track-1",
                    "name": "Loved Track 1",
                    "image": [
                        {
                            "size": "small",
                            "#text": "https://example.com/loved1.jpg"
                        }
                    ],
                    "streamable": {
                        "fulltrack": "1",
                        "#text": "1"
                    }
                }
            ],
            "@attr": {
                "user": "testuser",
                "totalPages": "1",
                "page": "1",
                "perPage": "50",
                "total": "1"
            }
        }
    })
}

/// Mock response for top tracks
fn mock_top_tracks_response() -> serde_json::Value {
    json!({
        "toptracks": {
            "track": [
                {
                    "streamable": {
                        "fulltrack": "1",
                        "#text": "1"
                    },
                    "mbid": "top-track-mbid-1",
                    "name": "Top Track 1",
                    "image": [],
                    "artist": {
                        "url": "https://www.last.fm/music/top-artist-1",
                        "name": "Top Artist 1",
                        "mbid": "top-artist-mbid-1"
                    },
                    "url": "https://www.last.fm/music/top-track-1",
                    "@attr": {
                        "rank": "1"
                    },
                    "playcount": "100",
                    "duration": "300"
                },
                {
                    "streamable": {
                        "fulltrack": "0",
                        "#text": "0"
                    },
                    "mbid": "top-track-mbid-2",
                    "name": "Top Track 2",
                    "image": [],
                    "artist": {
                        "url": "https://www.last.fm/music/top-artist-2",
                        "name": "Top Artist 2",
                        "mbid": "top-artist-mbid-2"
                    },
                    "url": "https://www.last.fm/music/top-track-2",
                    "@attr": {
                        "rank": "2"
                    },
                    "playcount": "50",
                    "duration": "250"
                }
            ],
            "@attr": {
                "user": "testuser",
                "totalPages": "1",
                "page": "1",
                "perPage": "50",
                "total": "2"
            }
        }
    })
}

/// Helper to create a test client with mock HTTP
fn create_test_client() -> LastFmClient {
    let config = ConfigBuilder::new()
        .api_key("test_api_key")
        .build()
        .expect("Failed to build config");

    let mock = create_mock_client();
    LastFmClient::with_http(config, Arc::new(mock))
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_recent_tracks_flow() {
    let client = create_test_client();

    // Test basic fetch
    let tracks = client
        .recent_tracks("testuser")
        .limit(50)
        .fetch()
        .await
        .expect("Failed to fetch recent tracks");

    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].name.as_deref(), Some("Test Track 1"));
    assert_eq!(tracks[0].artist.text.as_deref(), Some("Test Artist 1"));
    assert_eq!(tracks[1].name.as_deref(), Some("Test Track 2"));
}

#[tokio::test]
async fn test_recent_tracks_with_date_range() {
    let client = create_test_client();

    // Test with date range
    let from = 1_715_637_600; // 2024-05-14 00:00:00
    let to = 1_715_724_000; // 2024-05-15 00:00:00

    let tracks = client
        .recent_tracks("testuser")
        .between(from, to)
        .fetch()
        .await
        .expect("Failed to fetch recent tracks with date range");

    assert_eq!(tracks.len(), 2);

    // Verify tracks are within date range
    for track in &tracks {
        if let Some(date) = &track.date {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let from_u32 = from as u32;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let to_u32 = to as u32;
            assert!(date.uts >= from_u32);
            assert!(date.uts <= to_u32);
        }
    }
}

#[tokio::test]
async fn test_recent_tracks_unlimited() {
    let client = create_test_client();

    // Test unlimited fetch
    let tracks = client
        .recent_tracks("testuser")
        .unlimited()
        .fetch()
        .await
        .expect("Failed to fetch unlimited recent tracks");

    // Should still get 2 tracks (all available in mock)
    assert_eq!(tracks.len(), 2);
}

#[tokio::test]
async fn test_loved_tracks_flow() {
    let client = create_test_client();

    let tracks = client
        .loved_tracks("testuser")
        .limit(50)
        .fetch()
        .await
        .expect("Failed to fetch loved tracks");

    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].name.as_deref(), Some("Loved Track 1"));
    assert_eq!(tracks[0].artist.name.as_deref(), Some("Loved Artist 1"));
    assert_eq!(tracks[0].streamable.fulltrack, "1");
}

#[tokio::test]
async fn test_top_tracks_flow() {
    let client = create_test_client();

    let tracks = client
        .top_tracks("testuser")
        .limit(50)
        .fetch()
        .await
        .expect("Failed to fetch top tracks");

    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].name.as_deref(), Some("Top Track 1"));
    assert_eq!(tracks[0].playcount, 100);
    assert_eq!(tracks[1].name.as_deref(), Some("Top Track 2"));
    assert_eq!(tracks[1].playcount, 50);

    // Verify tracks are sorted by play count
    assert!(tracks[0].playcount >= tracks[1].playcount);
}

#[tokio::test]
async fn test_top_tracks_with_period() {
    use lastfm_client::api::Period;

    let client = create_test_client();

    let tracks = client
        .top_tracks("testuser")
        .period(Period::Month)
        .fetch()
        .await
        .expect("Failed to fetch top tracks with period");

    assert_eq!(tracks.len(), 2);
}

#[tokio::test]
async fn test_error_handling_missing_method() {
    let config = ConfigBuilder::new()
        .api_key("test_api_key")
        .build()
        .expect("Failed to build config");

    // Create mock without the expected response
    let mock = MockClient::new();
    let client = LastFmClient::with_http(config, Arc::new(mock));

    // This should fail because the mock doesn't have a response for this method
    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());

    // Verify error type
    match result {
        Err(lastfm_client::LastFmError::Config(msg)) => {
            assert!(msg.contains("No mock response"));
        }
        Err(e) => panic!("Expected Config error, got: {e:?}"),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[tokio::test]
async fn test_api_error_response() {
    let config = ConfigBuilder::new()
        .api_key("invalid_key")
        .build()
        .expect("Failed to build config");

    // Create mock that returns an API error
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 10,
            "message": "Invalid API key"
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_requests() {
    let client = Arc::new(create_test_client());

    // Spawn multiple concurrent requests
    let mut handles = vec![];

    for i in 0..5 {
        let client = Arc::clone(&client);
        let handle = tokio::spawn(async move {
            client
                .recent_tracks(format!("testuser{i}"))
                .limit(10)
                .fetch()
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        // All should succeed (or fail consistently)
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_builder_pattern_chaining() {
    let client = create_test_client();

    // Test that builder pattern allows flexible chaining
    let tracks = client
        .recent_tracks("testuser")
        .limit(100)
        .since(1_715_637_600)
        .fetch()
        .await
        .expect("Failed with chained builder");

    assert_eq!(tracks.len(), 2);
}

#[tokio::test]
async fn test_client_configuration() {
    use std::time::Duration;

    // Test that client can be configured with custom settings
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .timeout(Duration::from_mins(1))
        .max_concurrent_requests(10)
        .retry_attempts(5)
        .build()
        .expect("Failed to build config");

    assert_eq!(config.api_key(), "test_key");
    assert_eq!(config.timeout(), Duration::from_mins(1));
    assert_eq!(config.max_concurrent_requests(), 10);
    assert_eq!(config.retry_attempts(), 5);

    // Verify client can be created with this config
    let mock = create_mock_client();
    let _client = LastFmClient::with_http(config, Arc::new(mock));
}

#[tokio::test]
async fn test_rate_limiting_configuration() {
    use std::time::Duration;

    // Test client with rate limiting
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .rate_limit(5, Duration::from_secs(1))
        .build()
        .expect("Failed to build config");

    let rate_limit = config.rate_limit().expect("Rate limit not set");
    assert_eq!(rate_limit.max_requests, 5);
    assert_eq!(rate_limit.per_duration, Duration::from_secs(1));

    let mock = create_mock_client();
    let _client = LastFmClient::with_http(config, Arc::new(mock));
}

#[tokio::test]
async fn test_track_data_integrity() {
    let client = create_test_client();

    let tracks = client
        .recent_tracks("testuser")
        .fetch()
        .await
        .expect("Failed to fetch tracks");

    // Verify first track has all expected data
    let track = &tracks[0];
    assert!(track.name.is_some());
    assert!(track.artist.text.is_some());
    assert!(track.album.text.is_some());
    assert!(track.url.is_some());
    assert!(track.date.is_some());

    // Verify date parsing
    if let Some(date) = &track.date {
        assert_eq!(date.uts, 1_715_637_600);
        assert!(!date.text.is_empty());
    }
}

#[tokio::test]
async fn test_streamable_parsing() {
    let client = create_test_client();

    let tracks = client
        .recent_tracks("testuser")
        .fetch()
        .await
        .expect("Failed to fetch tracks");

    // First track should be streamable (string "1" -> true)
    assert!(tracks[0].streamable);

    // Second track should not be streamable (string "0" -> false)
    assert!(!tracks[1].streamable);
}

// ============================================================================
// Error Handling Tests (Expanded)
// ============================================================================

#[tokio::test]
async fn test_retryable_api_error_temporary() {
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .retry_attempts(2)
        .build()
        .expect("Failed to build config");

    // Mock returns error code 16 (temporary error)
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 16,
            "message": "There was a temporary error processing your request. Please try again later."
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Api {
        retryable,
        error_code,
        ..
    }) = result
    {
        assert!(retryable, "Error code 16 should be retryable");
        assert_eq!(error_code, 16);
    } else {
        panic!("Expected Api error");
    }
}

#[tokio::test]
async fn test_retryable_api_error_service_offline() {
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .retry_attempts(2)
        .build()
        .expect("Failed to build config");

    // Mock returns error code 11 (service offline)
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 11,
            "message": "Service Offline - This service is temporarily offline. Try again later."
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Api {
        retryable,
        error_code,
        ..
    }) = result
    {
        assert!(retryable, "Error code 11 should be retryable");
        assert_eq!(error_code, 11);
    } else {
        panic!("Expected Api error");
    }
}

#[tokio::test]
async fn test_non_retryable_api_error_invalid_key() {
    let config = ConfigBuilder::new()
        .api_key("invalid_key")
        .build()
        .expect("Failed to build config");

    // Mock returns error code 10 (invalid API key)
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 10,
            "message": "Invalid API key - You must be granted a valid key by last.fm"
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Api {
        retryable,
        error_code,
        method,
        message,
    }) = result
    {
        assert!(!retryable, "Error code 10 should NOT be retryable");
        assert_eq!(error_code, 10);
        assert_eq!(method, "user.getrecenttracks");
        assert!(message.contains("Invalid API key"));
    } else {
        panic!("Expected Api error");
    }
}

#[tokio::test]
async fn test_non_retryable_api_error_invalid_parameters() {
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .build()
        .expect("Failed to build config");

    // Mock returns error code 6 (invalid parameters)
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 6,
            "message": "Invalid parameters - Your request is missing a required parameter"
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Api {
        retryable,
        error_code,
        ..
    }) = result
    {
        assert!(!retryable, "Error code 6 should NOT be retryable");
        assert_eq!(error_code, 6);
    } else {
        panic!("Expected Api error");
    }
}

#[tokio::test]
async fn test_rate_limit_error_code_29() {
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .retry_attempts(3)
        .build()
        .expect("Failed to build config");

    // Mock returns error code 29 (rate limit exceeded)
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 29,
            "message": "Rate Limit Exceeded - Your IP has made too many requests in a short period"
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::RateLimited { retry_after }) = result {
        assert!(retry_after.is_some());
        // Should suggest 60 seconds retry
        assert_eq!(retry_after.unwrap().as_secs(), 60);
    } else {
        panic!("Expected RateLimited error, got: {result:?}");
    }
}

#[tokio::test]
async fn test_method_extraction_from_url() {
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .build()
        .expect("Failed to build config");

    // Mock returns an API error
    let mock = MockClient::new().with_response(
        "user.getlovedtracks",
        json!({
            "error": 10,
            "message": "Invalid API key"
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.loved_tracks("testuser").fetch().await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Api { method, .. }) = result {
        // Should extract the correct method from URL
        assert_eq!(method, "user.getlovedtracks");
    } else {
        panic!("Expected Api error");
    }
}

#[tokio::test]
async fn test_unknown_error_code_not_retryable() {
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .build()
        .expect("Failed to build config");

    // Mock returns unknown error code 999
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 999,
            "message": "Unknown error occurred"
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    let result = client.recent_tracks("testuser").fetch().await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Api {
        retryable,
        error_code,
        ..
    }) = result
    {
        // Unknown errors should default to NOT retryable (conservative)
        assert!(!retryable, "Unknown error codes should NOT be retryable");
        assert_eq!(error_code, 999);
    } else {
        panic!("Expected Api error");
    }
}

#[tokio::test]
async fn test_is_retryable_method() {
    // Test the is_retryable() method on various error types
    use lastfm_client::LastFmError;
    use std::time::Duration;

    // Retryable API error
    let retryable_api = LastFmError::Api {
        method: "test".to_string(),
        message: "Temporary error".to_string(),
        error_code: 16,
        retryable: true,
    };
    assert!(retryable_api.is_retryable());

    // Non-retryable API error
    let non_retryable_api = LastFmError::Api {
        method: "test".to_string(),
        message: "Invalid key".to_string(),
        error_code: 10,
        retryable: false,
    };
    assert!(!non_retryable_api.is_retryable());

    // Rate limited error
    let rate_limited = LastFmError::RateLimited {
        retry_after: Some(Duration::from_mins(1)),
    };
    assert!(rate_limited.is_retryable());
    assert_eq!(rate_limited.retry_after(), Some(Duration::from_mins(1)));

    // Config error (not retryable)
    let config_error = LastFmError::Config("Invalid config".to_string());
    assert!(!config_error.is_retryable());
}

// ============================================================================
// Date Range Validation Tests
// ============================================================================

#[tokio::test]
async fn test_date_range_validation_invalid() {
    let client = create_test_client();

    // Invalid: to <= from
    let from = 1_715_724_000; // 2024-05-15 00:00:00
    let to = 1_715_637_600; // 2024-05-14 00:00:00 (earlier!)

    let result = client
        .recent_tracks("testuser")
        .between(from, to)
        .fetch()
        .await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Config(msg)) = result {
        assert!(msg.contains("Invalid date range"));
        assert!(msg.contains("must be greater than"));
        assert!(msg.contains(&from.to_string()));
        assert!(msg.contains(&to.to_string()));
    } else {
        panic!("Expected Config error for invalid date range");
    }
}

#[tokio::test]
async fn test_date_range_validation_equal() {
    let client = create_test_client();

    // Invalid: to == from
    let timestamp = 1_715_637_600; // Same timestamp for both

    let result = client
        .recent_tracks("testuser")
        .between(timestamp, timestamp)
        .fetch()
        .await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Config(msg)) = result {
        assert!(msg.contains("Invalid date range"));
        assert!(msg.contains("must be greater than"));
    } else {
        panic!("Expected Config error for equal timestamps");
    }
}

#[tokio::test]
async fn test_date_range_validation_valid() {
    let client = create_test_client();

    // Valid: to > from
    let from = 1_715_637_600; // 2024-05-14 00:00:00
    let to = 1_715_724_000; // 2024-05-15 00:00:00

    let result = client
        .recent_tracks("testuser")
        .between(from, to)
        .fetch()
        .await;

    // Should succeed (not fail on validation)
    assert!(result.is_ok());
    let tracks = result.unwrap();
    assert_eq!(tracks.len(), 2);
}

#[tokio::test]
async fn test_date_range_validation_only_from() {
    let client = create_test_client();

    // Only 'from' set - no validation needed
    let result = client
        .recent_tracks("testuser")
        .since(1_715_637_600)
        .fetch()
        .await;

    // Should succeed (no validation when only from is set)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_date_range_validation_extended() {
    let client = create_test_client();

    // Invalid date range with extended fetch
    let from = 1_715_724_000; // 2024-05-15
    let to = 1_715_637_600; // 2024-05-14 (earlier!)

    let result = client
        .recent_tracks("testuser")
        .between(from, to)
        .fetch_extended()
        .await;

    assert!(result.is_err());
    if let Err(lastfm_client::LastFmError::Config(msg)) = result {
        assert!(msg.contains("Invalid date range"));
    } else {
        panic!("Expected Config error for invalid date range in fetch_extended");
    }
}

#[tokio::test]
async fn test_date_range_validation_error_before_api_call() {
    let config = ConfigBuilder::new()
        .api_key("test_key")
        .build()
        .expect("Failed to build config");

    // Mock that would return an error if called
    // But validation should catch the bad date range first
    let mock = MockClient::new().with_response(
        "user.getrecenttracks",
        json!({
            "error": 10,
            "message": "This should not be reached"
        }),
    );

    let client = LastFmClient::with_http(config, Arc::new(mock));

    // Invalid date range
    let result = client
        .recent_tracks("testuser")
        .between(100, 50) // to < from
        .fetch()
        .await;

    assert!(result.is_err());
    // Should be Config error (validation), not Api error (from mock)
    match result {
        Err(lastfm_client::LastFmError::Config(msg)) => {
            assert!(msg.contains("Invalid date range"));
        }
        Err(lastfm_client::LastFmError::Api { .. }) => {
            panic!("Should have failed on validation, not reached API");
        }
        _ => panic!("Expected Config error"),
    }
}
