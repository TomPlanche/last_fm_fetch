use crate::api::{
    LovedTracksClient, RecentTracksClient, TopAlbumsClient, TopArtistsClient, TopTracksClient,
};
use crate::client::{
    HttpClient, RateLimitedClient, RateLimiter, ReqwestClient, RetryClient, RetryPolicy,
};
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
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    recent_tracks_client: RecentTracksClient,
    loved_tracks_client: LovedTracksClient,
    top_tracks_client: TopTracksClient,
    top_artists_client: TopArtistsClient,
    top_albums_client: TopAlbumsClient,
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

    /// Create a new `LastFmClient` with default configuration
    ///
    /// This will automatically try to load the API key from the `LAST_FM_API_KEY`
    /// environment variable. All other settings use sensible defaults.
    ///
    /// # Example
    /// ```no_run
    /// use lastfm_client::LastFmClient;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = LastFmClient::new()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns an error if the API key is not set and cannot be loaded from environment
    pub fn new() -> Result<Self> {
        let config = ConfigBuilder::build_with_defaults()?;
        Ok(Self::from_config(config))
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
        let recent_tracks_client = RecentTracksClient::new(http.clone(), config.clone());
        let loved_tracks_client = LovedTracksClient::new(http.clone(), config.clone());
        let top_tracks_client = TopTracksClient::new(http.clone(), config.clone());
        let top_artists_client = TopArtistsClient::new(http.clone(), config.clone());
        let top_albums_client = TopAlbumsClient::new(http.clone(), config.clone());

        Self {
            http,
            config,
            recent_tracks_client,
            loved_tracks_client,
            top_tracks_client,
            top_artists_client,
            top_albums_client,
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
        let recent_tracks_client = RecentTracksClient::new(http.clone(), config.clone());
        let loved_tracks_client = LovedTracksClient::new(http.clone(), config.clone());
        let top_tracks_client = TopTracksClient::new(http.clone(), config.clone());
        let top_artists_client = TopArtistsClient::new(http.clone(), config.clone());
        let top_albums_client = TopAlbumsClient::new(http.clone(), config.clone());

        Self {
            http,
            config,
            recent_tracks_client,
            loved_tracks_client,
            top_tracks_client,
            top_artists_client,
            top_albums_client,
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
    pub fn recent_tracks(
        &self,
        username: impl Into<String>,
    ) -> crate::api::RecentTracksRequestBuilder {
        self.recent_tracks_client.builder(username)
    }

    /// Get a builder for loved tracks requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tracks = client
    ///     .loved_tracks("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn loved_tracks(
        &self,
        username: impl Into<String>,
    ) -> crate::api::LovedTracksRequestBuilder {
        self.loved_tracks_client.builder(username)
    }

    /// Get a builder for top tracks requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tracks = client
    ///     .top_tracks("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn top_tracks(&self, username: impl Into<String>) -> crate::api::TopTracksRequestBuilder {
        self.top_tracks_client.builder(username)
    }

    /// Get a builder for top artists requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let artists = client
    ///     .top_artists("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn top_artists(&self, username: impl Into<String>) -> crate::api::TopArtistsRequestBuilder {
        self.top_artists_client.builder(username)
    }

    /// Get a builder for top albums requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let albums = client
    ///     .top_albums("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn top_albums(&self, username: impl Into<String>) -> crate::api::TopAlbumsRequestBuilder {
        self.top_albums_client.builder(username)
    }

    /// Check if a Last.fm user exists
    ///
    /// # Arguments
    /// * `username` - The Last.fm username to check
    ///
    /// # Returns
    /// * `Ok(true)` - User exists
    /// * `Ok(false)` - User does not exist
    /// * `Err` - Network error or other API error
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// if client.user_exists("rj").await? {
    ///     println!("User exists!");
    /// } else {
    ///     println!("User not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns an error if the request fails due to network issues or other API errors
    /// (not including "user not found" which returns `Ok(false)`)
    pub async fn user_exists(&self, username: impl Into<String>) -> Result<bool> {
        use crate::api::constants::BASE_URL;
        use crate::error::LastFmError;
        use crate::url_builder::{QueryParams, Url};

        let username = username.into();
        let mut params = QueryParams::new();
        params.insert("method".to_string(), "user.getinfo".to_string());
        params.insert("user".to_string(), username);
        params.insert("api_key".to_string(), self.config.api_key().to_string());
        params.insert("format".to_string(), "json".to_string());

        let url = Url::new(BASE_URL).add_args(params).build();

        match self.http.get(&url).await {
            Ok(_) => Ok(true),
            Err(LastFmError::Api { error_code, .. }) if error_code == 6 || error_code == 7 => {
                // Error code 6: Invalid parameters (user not found)
                // Error code 7: Invalid resource specified (user not found)
                Ok(false)
            }
            Err(e) => Err(e),
        }
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
        let config = ConfigBuilder::new().api_key("test_key").build().unwrap();

        let client = LastFmClient::from_config(config);
        assert_eq!(client.config().api_key(), "test_key");
    }

    #[test]
    fn test_client_with_mock() {
        let config = ConfigBuilder::new().api_key("test_key").build().unwrap();

        let mock = MockClient::new();
        let http = Arc::new(mock);
        let client = LastFmClient::with_http(config, http);
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

    #[tokio::test]
    async fn test_user_exists_returns_true() {
        use serde_json::json;

        let config = ConfigBuilder::new().api_key("test_key").build().unwrap();

        let mock = MockClient::new().with_response(
            "user.getinfo",
            json!({
                "user": {
                    "name": "rj",
                    "realname": "Richard Jones",
                    "url": "https://www.last.fm/user/rj"
                }
            }),
        );

        let client = LastFmClient::with_http(config, Arc::new(mock));
        let result = client.user_exists("rj").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_user_exists_returns_false_for_error_6() {
        use serde_json::json;

        let config = ConfigBuilder::new().api_key("test_key").build().unwrap();

        // Mock returns error code 6 (Invalid parameters / user not found)
        let mock = MockClient::new().with_response(
            "user.getinfo",
            json!({
                "error": 6,
                "message": "User not found"
            }),
        );

        let client = LastFmClient::with_http(config, Arc::new(mock));
        let result = client.user_exists("nonexistentuser").await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_user_exists_returns_false_for_error_7() {
        use serde_json::json;

        let config = ConfigBuilder::new().api_key("test_key").build().unwrap();

        // Mock returns error code 7 (Invalid resource specified)
        let mock = MockClient::new().with_response(
            "user.getinfo",
            json!({
                "error": 7,
                "message": "Invalid resource specified"
            }),
        );

        let client = LastFmClient::with_http(config, Arc::new(mock));
        let result = client.user_exists("invaliduser").await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_user_exists_propagates_other_api_errors() {
        use crate::error::LastFmError;
        use serde_json::json;

        let config = ConfigBuilder::new().api_key("test_key").build().unwrap();

        // Mock returns error code 10 (Invalid API key)
        let mock = MockClient::new().with_response(
            "user.getinfo",
            json!({
                "error": 10,
                "message": "Invalid API key"
            }),
        );

        let client = LastFmClient::with_http(config, Arc::new(mock));
        let result = client.user_exists("someuser").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, LastFmError::Api { error_code: 10, .. }));
    }
}
