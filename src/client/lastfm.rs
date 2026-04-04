use crate::api::{
    FriendsRequestBuilder, LovedTracksRequestBuilder, PersonalTagsRequestBuilder,
    RecentTracksRequestBuilder, TopAlbumsRequestBuilder, TopArtistsRequestBuilder,
    TopTagsRequestBuilder, TopTracksRequestBuilder, UserInfoRequestBuilder,
    WeeklyAlbumChartRequestBuilder, WeeklyArtistChartRequestBuilder, WeeklyChartListRequestBuilder,
    WeeklyTrackChartRequestBuilder,
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
    config: Arc<Config>,
    http: Arc<dyn HttpClient>,
}

impl std::fmt::Debug for LastFmClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LastFmClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
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
        let retry_policy = RetryPolicy::exponential(config.retry_attempts());
        let http: Arc<dyn HttpClient> = if let Some(rate_limit_config) = config.rate_limit() {
            let retry_client = RetryClient::new(base_client, retry_policy);

            let limiter = Arc::new(RateLimiter::new(
                rate_limit_config.max_requests,
                rate_limit_config.per_duration,
            ));
            Arc::new(RateLimitedClient::new(retry_client, limiter))
        } else {
            Arc::new(RetryClient::new(base_client, retry_policy))
        };

        Self {
            config: Arc::new(config),
            http,
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
        Self {
            config: Arc::new(config),
            http,
        }
    }

    /// Get a builder for recent tracks requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::{LastFmClient, prelude::*};
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tracks = client
    ///     .recent_tracks("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn recent_tracks(&self, username: impl Into<String>) -> RecentTracksRequestBuilder {
        RecentTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for loved tracks requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::{LastFmClient, prelude::*};
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tracks = client
    ///     .loved_tracks("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn loved_tracks(&self, username: impl Into<String>) -> LovedTracksRequestBuilder {
        LovedTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for top tracks requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::{LastFmClient, prelude::*};
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tracks = client
    ///     .top_tracks("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn top_tracks(&self, username: impl Into<String>) -> TopTracksRequestBuilder {
        TopTracksRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for top artists requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::{LastFmClient, prelude::*};
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let artists = client
    ///     .top_artists("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn top_artists(&self, username: impl Into<String>) -> TopArtistsRequestBuilder {
        TopArtistsRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for top albums requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::{LastFmClient, prelude::*};
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let albums = client
    ///     .top_albums("username")
    ///     .limit(100)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn top_albums(&self, username: impl Into<String>) -> TopAlbumsRequestBuilder {
        TopAlbumsRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for top tags requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tags = client
    ///     .top_tags("username")
    ///     .limit(20)
    ///     .fetch()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn top_tags(&self, username: impl Into<String>) -> TopTagsRequestBuilder {
        TopTagsRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for `user.getWeeklyChartList` requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let ranges = client.weekly_chart_list("username").fetch().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn weekly_chart_list(&self, username: impl Into<String>) -> WeeklyChartListRequestBuilder {
        WeeklyChartListRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for `user.getWeeklyTrackChart` requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let ranges = client.weekly_chart_list("username").fetch().await?;
    /// if let Some(range) = ranges.first() {
    ///     let tracks = client.weekly_track_chart("username").range(range).fetch().await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn weekly_track_chart(
        &self,
        username: impl Into<String>,
    ) -> WeeklyTrackChartRequestBuilder {
        WeeklyTrackChartRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for `user.getWeeklyArtistChart` requests
    pub fn weekly_artist_chart(
        &self,
        username: impl Into<String>,
    ) -> WeeklyArtistChartRequestBuilder {
        WeeklyArtistChartRequestBuilder::new(
            self.http.clone(),
            self.config.clone(),
            username.into(),
        )
    }

    /// Get a builder for `user.getWeeklyAlbumChart` requests
    pub fn weekly_album_chart(
        &self,
        username: impl Into<String>,
    ) -> WeeklyAlbumChartRequestBuilder {
        WeeklyAlbumChartRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for friends requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let friends = client.friends("rj").fetch_all().await?;
    /// for friend in &friends {
    ///     println!("{}", friend.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn friends(&self, username: impl Into<String>) -> FriendsRequestBuilder {
        FriendsRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Get a builder for personal tags requests
    ///
    /// Returns tracks, artists, or albums that the user has tagged with the given tag.
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let page = client.personal_tags("rj", "rock").fetch_tracks().await?;
    /// for track in &page.tracks {
    ///     println!("{} - {}", track.artist_name, track.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn personal_tags(
        &self,
        username: impl Into<String>,
        tag: impl Into<String>,
    ) -> PersonalTagsRequestBuilder {
        PersonalTagsRequestBuilder::new(
            self.http.clone(),
            self.config.clone(),
            username.into(),
            tag.into(),
        )
    }

    /// Get a builder for user profile requests
    ///
    /// # Example
    /// ```no_run
    /// # use lastfm_client::LastFmClient;
    /// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let info = client.user_info("rj").fetch().await?;
    /// println!("{} has {} scrobbles", info.name, info.play_count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_info(&self, username: impl Into<String>) -> UserInfoRequestBuilder {
        UserInfoRequestBuilder::new(self.http.clone(), self.config.clone(), username.into())
    }

    /// Check if a Last.fm user exists
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
        use crate::error::LastFmError;

        match self.user_info(username).fetch().await {
            Ok(_) => Ok(true),
            Err(LastFmError::Api { error_code, .. }) if error_code == 6 || error_code == 7 => {
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
#[allow(clippy::unwrap_used)]
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
                    "url": "https://www.last.fm/user/rj",
                    "country": "UK",
                    "age": "0",
                    "gender": "m",
                    "subscriber": "0",
                    "playcount": "12345",
                    "playlists": "0",
                    "registered": { "unixtime": "1104874958", "#text": "2005-01-05 00:00" }
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
