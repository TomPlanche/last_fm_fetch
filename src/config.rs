use crate::error::{LastFmError, Result};
use std::env;
use std::time::Duration;

/// Rate limiting configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RateLimit {
    /// Maximum number of requests allowed in the time window
    pub max_requests: u32,
    /// Time window for the rate limit
    pub per_duration: Duration,
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) api_key: String,
    pub(crate) user_agent: String,
    pub(crate) timeout: Duration,
    pub(crate) max_concurrent_requests: usize,
    pub(crate) retry_attempts: u32,
    pub(crate) rate_limit: Option<RateLimit>,
}

impl Config {
    /// Get the API key
    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the user agent
    #[must_use]
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Get the timeout duration
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get maximum concurrent requests
    #[must_use]
    pub const fn max_concurrent_requests(&self) -> usize {
        self.max_concurrent_requests
    }

    /// Get retry attempts
    #[must_use]
    pub const fn retry_attempts(&self) -> u32 {
        self.retry_attempts
    }

    /// Get rate limit configuration
    #[must_use]
    pub const fn rate_limit(&self) -> Option<&RateLimit> {
        self.rate_limit.as_ref()
    }
}

/// Configuration builder for creating a `Config` instance
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    api_key: Option<String>,
    user_agent: Option<String>,
    timeout: Option<Duration>,
    max_concurrent_requests: Option<usize>,
    retry_attempts: Option<u32>,
    rate_limit: Option<RateLimit>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the API key
    ///
    /// # Example
    /// ```
    /// use lastfm_client::ConfigBuilder;
    ///
    /// let builder = ConfigBuilder::new().api_key("my_api_key");
    /// ```
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Load API key from environment variable
    ///
    /// # Example
    /// ```no_run
    /// use lastfm_client::ConfigBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let builder = ConfigBuilder::new().from_env()?;
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    /// Returns an error if the `LAST_FM_API_KEY` environment variable is missing.
    pub fn from_env(mut self) -> Result<Self> {
        let api_key = env::var("LAST_FM_API_KEY")
            .map_err(|_| LastFmError::MissingEnvVar("LAST_FM_API_KEY".to_string()))?;
        self.api_key = Some(api_key);
        Ok(self)
    }

    /// Set the user agent
    ///
    /// If not set, defaults to `async_lastfm/VERSION`
    #[must_use]
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    /// Set the request timeout
    ///
    /// If not set, defaults to 30 seconds
    #[must_use]
    pub const fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set maximum concurrent requests
    ///
    /// If not set, defaults to 5
    #[must_use]
    pub const fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = Some(max);
        self
    }

    /// Set number of retry attempts
    ///
    /// If not set, defaults to 3
    #[must_use]
    pub const fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = Some(attempts);
        self
    }

    /// Set rate limiting
    ///
    /// # Example
    /// ```
    /// use lastfm_client::ConfigBuilder;
    /// use std::time::Duration;
    ///
    /// let builder = ConfigBuilder::new()
    ///     .api_key("key")
    ///     .rate_limit(5, Duration::from_secs(1)); // Max 5 requests per second
    /// ```
    #[must_use]
    pub const fn rate_limit(mut self, max_requests: u32, per_duration: Duration) -> Self {
        self.rate_limit = Some(RateLimit {
            max_requests,
            per_duration,
        });
        self
    }

    /// Build the configuration
    ///
    /// # Errors
    /// Returns an error if the API key is not set and cannot be loaded from environment
    pub fn build(self) -> Result<Config> {
        // Try to get API key from builder, then from environment
        let api_key = self.api_key.or_else(|| {
            env::var("LAST_FM_API_KEY").ok()
        }).ok_or_else(|| LastFmError::Config(
            "API key is required. Set it via .api_key() or LAST_FM_API_KEY environment variable".to_string()
        ))?;

        Ok(Config {
            api_key,
            user_agent: self
                .user_agent
                .unwrap_or_else(|| format!("async_lastfm/{}", env!("CARGO_PKG_VERSION"))),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            max_concurrent_requests: self.max_concurrent_requests.unwrap_or(5),
            retry_attempts: self.retry_attempts.unwrap_or(3),
            rate_limit: self.rate_limit,
        })
    }

    /// Build the configuration with defaults, trying to load API key from environment
    ///
    /// This is equivalent to `ConfigBuilder::new().build()` but more explicit about
    /// the default behavior.
    ///
    /// # Errors
    /// Returns an error if the API key is not set and cannot be loaded from environment
    pub fn build_with_defaults() -> Result<Config> {
        Self::new().build()
    }
}

/// Validates that all required environment variables are set
///
/// # Errors
/// Returns `LastFmError::MissingEnvVar` if any required environment variable is missing
pub fn validate_env_vars() -> Result<()> {
    const REQUIRED_ENV_VARS: &[&str] = &["LAST_FM_API_KEY"];

    let mut missing_vars = Vec::new();

    for var_name in REQUIRED_ENV_VARS {
        if env::var(var_name).is_err() {
            missing_vars.push(*var_name);
        }
    }

    if !missing_vars.is_empty() {
        return Err(LastFmError::MissingEnvVar(missing_vars.join(", ")));
    }

    Ok(())
}

/// Gets a required environment variable
///
/// # Errors
/// Returns `LastFmError::MissingEnvVar` if the environment variable is not set
pub fn get_required_env_var(var_name: &str) -> Result<String> {
    env::var(var_name).map_err(|_| LastFmError::MissingEnvVar(var_name.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .api_key("test_key")
            .user_agent("test_agent")
            .timeout(Duration::from_secs(60))
            .max_concurrent_requests(10)
            .retry_attempts(5)
            .build()
            .unwrap();

        assert_eq!(config.api_key(), "test_key");
        assert_eq!(config.user_agent(), "test_agent");
        assert_eq!(config.timeout(), Duration::from_secs(60));
        assert_eq!(config.max_concurrent_requests(), 10);
        assert_eq!(config.retry_attempts(), 5);
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = ConfigBuilder::new().api_key("test_key").build().unwrap();

        assert_eq!(config.api_key(), "test_key");
        assert!(config.user_agent().starts_with("async_lastfm/"));
        assert_eq!(config.timeout(), Duration::from_secs(30));
        assert_eq!(config.max_concurrent_requests(), 5);
        assert_eq!(config.retry_attempts(), 3);
    }

    #[test]
    fn test_config_builder_missing_api_key() {
        let result = ConfigBuilder::new().build();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LastFmError::Config(_)));
    }

    #[test]
    fn test_rate_limit() {
        let config = ConfigBuilder::new()
            .api_key("test_key")
            .rate_limit(10, Duration::from_secs(1))
            .build()
            .unwrap();

        let rate_limit = config.rate_limit().unwrap();
        assert_eq!(rate_limit.max_requests, 10);
        assert_eq!(rate_limit.per_duration, Duration::from_secs(1));
    }
}
