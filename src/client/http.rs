use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::{LastFmError, LastFmErrorResponse, Result};

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
        let body_text = response.text().await?;

        if !status.is_success() {
            #[cfg(debug_assertions)]
            eprintln!(
                "HTTP error {} for URL: {}\nRaw body:\n{}",
                status, url, body_text
            );

            if let Ok(error) = serde_json::from_str::<LastFmErrorResponse>(&body_text) {
                return Err(LastFmError::Api {
                    method: "unknown".to_string(),
                    message: error.message,
                    error_code: error.error,
                    retryable: false,
                });
            }

            return Err(LastFmError::Other(format!(
                "HTTP {status} with non-JSON body"
            )));
        }

        match serde_json::from_str::<serde_json::Value>(&body_text) {
            Ok(json) => Ok(json),
            Err(err) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "JSON parse failed for URL: {}\nError: {}\nBody:\n{}",
                    url, err, body_text
                );
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
            .ok_or_else(|| {
                LastFmError::Other("No method parameter in mock URL".to_string())
            })?;

        self.responses
            .get(&method)
            .cloned()
            .ok_or_else(|| LastFmError::Other(format!("No mock response for method: {method}")))
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
