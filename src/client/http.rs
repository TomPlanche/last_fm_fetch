use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

use crate::error::{LastFmError, LastFmErrorResponse, Result};

/// Determine if a Last.fm API error code is retryable
///
/// Based on Last.fm API documentation:
/// - <https://www.last.fm/api/errorcodes>
/// - <https://lastfm-docs.github.io/api-docs/codes>
fn is_api_error_retryable(error_code: u32) -> bool {
    matches!(
        error_code,
        8  // Operation failed (temporary server issue)
        | 11  // Service Offline (temporary maintenance)
        | 16  // Temporary error processing request
        | 29 // Rate limit exceeded
    )
}

/// Extract the Last.fm API method name from a URL
///
/// Parses the URL query parameters to find the "method" parameter.
/// Returns "unknown" if the method cannot be extracted.
fn extract_method_from_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|url| {
            url.query_pairs()
                .find(|(key, _)| key == "method")
                .map(|(_, value)| value.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// HTTP client abstraction for making API requests
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Perform a GET request and return the response as JSON
    async fn get(&self, url: &str) -> Result<serde_json::Value>;
}

/// Production HTTP client using reqwest
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    #[must_use]
    /// # Panics
    /// Panics if the underlying HTTP client cannot be constructed.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("failed to build reqwest client"),
        }
    }

    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn get(&self, url: &str) -> Result<serde_json::Value> {
        let response = self.client.get(url).send().await?;
        let status = response.status();

        // Check for rate limiting via HTTP 429
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Try to extract Retry-After header
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(Duration::from_secs);

            return Err(LastFmError::RateLimited { retry_after });
        }

        let body_text = response.text().await?;

        if !status.is_success() {
            #[cfg(debug_assertions)]
            eprintln!("HTTP error {status} for URL: {url}\nRaw body:\n{body_text}");

            // Try to parse as Last.fm API error response
            if let Ok(error) = serde_json::from_str::<LastFmErrorResponse>(&body_text) {
                let method = extract_method_from_url(url);
                let retryable = is_api_error_retryable(error.error);

                // Special handling for rate limit error code 29
                if error.error == 29 {
                    return Err(LastFmError::RateLimited {
                        retry_after: Some(Duration::from_secs(60)), // Default 1 minute
                    });
                }

                return Err(LastFmError::Api {
                    method,
                    message: error.message,
                    error_code: error.error,
                    retryable,
                });
            }

            return Err(LastFmError::Other(format!(
                "HTTP {status} with non-JSON body"
            )));
        }

        match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json) => {
                // Check if the JSON contains an error field (Last.fm returns errors with HTTP 200)
                if json.get("error").is_some()
                    && let Ok(error) = serde_json::from_value::<LastFmErrorResponse>(json.clone())
                {
                    let method = extract_method_from_url(url);
                    let retryable = is_api_error_retryable(error.error);

                    // Special handling for rate limit error code 29
                    if error.error == 29 {
                        return Err(LastFmError::RateLimited {
                            retry_after: Some(Duration::from_secs(60)),
                        });
                    }

                    return Err(LastFmError::Api {
                        method,
                        message: error.message,
                        error_code: error.error,
                        retryable,
                    });
                }

                Ok(json)
            }
            Err(err) => {
                #[cfg(debug_assertions)]
                eprintln!("JSON parse failed for URL: {url}\nError: {err}\nBody:\n{body_text}");
                Err(err.into())
            }
        }
    }
}

/// Mock HTTP client for testing
#[derive(Debug, Clone)]
pub struct MockClient {
    responses: HashMap<String, serde_json::Value>,
}

impl MockClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_response(mut self, method: &str, data: serde_json::Value) -> Self {
        self.responses.insert(method.to_string(), data);
        self
    }
}

impl Default for MockClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClient for MockClient {
    async fn get(&self, url: &str) -> Result<serde_json::Value> {
        // Extract method from URL query parameters
        let url_obj = url::Url::parse(url)
            .map_err(|e| LastFmError::Other(format!("Invalid URL in mock client: {e}")))?;

        let method = url_obj
            .query_pairs()
            .find(|(key, _)| key == "method")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| LastFmError::Other("No method parameter in mock URL".to_string()))?;

        let json =
            self.responses.get(&method).cloned().ok_or_else(|| {
                LastFmError::Other(format!("No mock response for method: {method}"))
            })?;

        // Check if the JSON contains an error field (Last.fm returns errors with HTTP 200)
        if json.get("error").is_some()
            && let Ok(error) = serde_json::from_value::<LastFmErrorResponse>(json.clone())
        {
            let retryable = is_api_error_retryable(error.error);

            // Special handling for rate limit error code 29
            if error.error == 29 {
                return Err(LastFmError::RateLimited {
                    retry_after: Some(Duration::from_secs(60)),
                });
            }

            return Err(LastFmError::Api {
                method: method.clone(),
                message: error.message,
                error_code: error.error,
                retryable,
            });
        }

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mock_client_with_response() {
        let mock = MockClient::new().with_response(
            "user.getrecenttracks",
            json!({
                "recenttracks": {
                    "track": [],
                    "@attr": {
                        "user": "test",
                        "totalPages": "0",
                        "page": "1",
                        "perPage": "50",
                        "total": "0"
                    }
                }
            }),
        );

        let response = mock
            .get("http://example.com?method=user.getrecenttracks")
            .await
            .unwrap();

        assert!(response.is_object());
        assert!(response["recenttracks"].is_object());
    }

    #[tokio::test]
    async fn test_mock_client_missing_method() {
        let mock = MockClient::new();

        let result = mock
            .get("http://example.com?method=user.getrecenttracks")
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LastFmError::Other(_)));
    }

    #[tokio::test]
    async fn test_mock_client_invalid_url() {
        let mock = MockClient::new();

        let result = mock.get("not a valid url").await;

        assert!(result.is_err());
    }
}
