use crate::api::RecentTracksClient;
use crate::client::{HttpClient, RateLimitedClient, RateLimiter, ReqwestClient, RetryClient, RetryPolicy};
use crate::config::{Config, ConfigBuilder};
use crate::error::Result;
use std::sync::Arc;

/// Main Last.fm API client
///
/// This is the entry point for interacting with the Last.fm API using the new v2.0 API.
///
/// # Example
/// ```
/// use lastfm_client::LastFmClient;
/// use std::time::Duration;
///
/// // Create client with custom configuration
/// let client = LastFmClient::builder()
///     .api_key("your_api_key")
///     .timeout(Duration::from_secs(60))
///     .max_concurrent_requests(10)
///     .build()
///     .unwrap();
///
/// // Use client.recent_tracks() to fetch data
/// ```
pub struct LastFmClient {
    config: Arc<Config>,
    recent_tracks_client: RecentTracksClient,
}

impl LastFmClient {
    /// Create a new configuration builder
    ///
    /// This is the recommended way to create a `LastFmClient`.
    ///
    /// # Example
    /// ```no_run
    /// use lastfm_client::LastFmClient;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LastFmClient::builder()
    ///     .api_key("your_api_key")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Create a new `LastFmClient` from a configuration
    ///
    /// This automatically sets up retry logic and rate limiting based on the configuration.
    /// Most users should use `builder()` instead.
    #[must_use]
    pub fn from_config(config: Config) -> Self {
        // Create base HTTP client
        let base_client = ReqwestClient::new();

        // Build the HTTP client with retry and rate limiting
        let http: Arc<dyn HttpClient> = if let Some(rate_limit_config) = config.rate_limit() {
            // With rate limiting
            let retry_policy = RetryPolicy::exponential(config.retry_attempts());
            let retry_client = RetryClient::new(base_client, retry_policy);

            let limiter = Arc::new(RateLimiter::new(
                rate_limit_config.max_requests,
                rate_limit_config.per_duration,
            ));
            Arc::new(RateLimitedClient::new(retry_client, limiter))
        } else {
            // Without rate limiting, just retry
            let retry_policy = RetryPolicy::exponential(config.retry_attempts());
            Arc::new(RetryClient::new(base_client, retry_policy))
        };

        let config = Arc::new(config);
        let recent_tracks_client = RecentTracksClient::new(http, config.clone());

        Self {
            config,
            recent_tracks_client,
        }
    }

    /// Create a new `LastFmClient` with a custom HTTP client
    ///
    /// This is primarily useful for testing with a mock HTTP client.
    ///
    /// # Example
    /// ```
    /// use lastfm_client::{LastFmClient, Config, ConfigBuilder};
    /// use lastfm_client::client::MockClient;
    /// use std::sync::Arc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigBuilder::new()
    ///     .api_key("test_key")
    ///     .build()?;
    ///
    /// let mock = MockClient::new();
    /// let client = LastFmClient::with_http(config, Arc::new(mock));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_http(config: Config, http: Arc<dyn HttpClient>) -> Self {
        let config = Arc::new(config);
        let recent_tracks_client = RecentTracksClient::new(http, config.clone());

        Self {
            config,
            recent_tracks_client,
        }
    }

    /// Get a builder for recent tracks requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tracks = client
    ///     .recent_tracks("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn recent_tracks(&self, username: impl Into<String>) -> crate::api::RecentTracksRequestBuilder {
        self.recent_tracks_client.builder(username)
    }

    /// Get a reference to the configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}

// Convenience: allow building the client directly from the ConfigBuilder
impl ConfigBuilder {
    /// Build a `LastFmClient` directly from this builder
    ///
    /// This is equivalent to calling `build().map(LastFmClient::from_config)`.
    ///
    /// # Errors
    /// Returns an error if the builder is missing required fields (e.g., API key).
    pub fn build_client(self) -> Result<LastFmClient> {
        self.build().map(LastFmClient::from_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockClient;

    #[test]
    fn test_client_from_config() {
        let config = ConfigBuilder::new()
            .api_key("test_key")
            .build()
            .unwrap();

        let client = LastFmClient::from_config(config);
        assert_eq!(client.config().api_key(), "test_key");
    }

    #[test]
    fn test_client_with_mock() {
        let config = ConfigBuilder::new()
            .api_key("test_key")
            .build()
            .unwrap();

        let mock = MockClient::new();
        let client = LastFmClient::with_http(config, Arc::new(mock));
        assert_eq!(client.config().api_key(), "test_key");
    }

    #[test]
    fn test_builder() {
        let client = LastFmClient::builder()
            .api_key("test_key")
            .build()
            .map(LastFmClient::from_config)
            .unwrap();

        assert_eq!(client.config().api_key(), "test_key");
    }
}


