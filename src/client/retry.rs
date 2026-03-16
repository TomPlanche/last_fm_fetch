use crate::client::HttpClient;
use crate::error::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    exponential: bool,
}

impl RetryPolicy {
    /// Create an exponential backoff retry policy
    ///
    /// Delays increase exponentially: `base_delay` * 2^attempt
    ///
    /// # Example
    /// ```
    /// use lastfm_client::client::RetryPolicy;
    ///
    /// let policy = RetryPolicy::exponential(3); // Max 3 retries
    /// ```
    #[must_use]
    pub const fn exponential(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            exponential: true,
        }
    }

    /// Create a linear backoff retry policy
    ///
    /// Delays increase linearly: `base_delay` * attempt
    ///
    /// # Example
    /// ```
    /// use lastfm_client::client::RetryPolicy;
    ///
    /// let policy = RetryPolicy::linear(3); // Max 3 retries
    /// ```
    #[must_use]
    pub const fn linear(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            exponential: false,
        }
    }

    /// Create a custom retry policy
    #[must_use]
    pub const fn custom(
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        exponential: bool,
    ) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
            exponential,
        }
    }

    /// Calculate backoff delay for a given attempt
    #[must_use]
    pub fn backoff(&self, attempt: u32) -> Duration {
        if self.exponential {
            let delay = self.base_delay * 2_u32.saturating_pow(attempt);
            delay.min(self.max_delay)
        } else {
            (self.base_delay * attempt).min(self.max_delay)
        }
    }

    /// Get maximum number of attempts
    #[must_use]
    pub const fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential(3)
    }
}

/// HTTP client wrapper that adds retry logic
pub struct RetryClient<C> {
    inner: C,
    policy: RetryPolicy,
}

impl<C: std::fmt::Debug> std::fmt::Debug for RetryClient<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetryClient")
            .field("inner", &self.inner)
            .field("policy", &self.policy)
            .finish()
    }
}

impl<C> RetryClient<C> {
    /// Create a new retry client wrapping an existing HTTP client
    pub const fn new(inner: C, policy: RetryPolicy) -> Self {
        Self { inner, policy }
    }

    /// Get a reference to the inner client
    pub const fn inner(&self) -> &C {
        &self.inner
    }

    /// Get a reference to the retry policy
    pub const fn policy(&self) -> &RetryPolicy {
        &self.policy
    }
}

#[async_trait]
impl<C: HttpClient + Send + Sync> HttpClient for RetryClient<C> {
    async fn get(&self, url: &str) -> Result<serde_json::Value> {
        let mut attempts = 0;

        loop {
            match self.inner.get(url).await {
                Ok(response) => return Ok(response),
                Err(e) if e.is_retryable() && attempts < self.policy.max_attempts => {
                    attempts += 1;

                    let delay = e
                        .retry_after()
                        .unwrap_or_else(|| self.policy.backoff(attempts));

                    // Log the retry attempt (in production, use tracing)
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Retrying request (attempt {}/{}) after {:?}...",
                        attempts, self.policy.max_attempts, delay
                    );

                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy::exponential(3);

        assert_eq!(policy.backoff(0), Duration::from_millis(100));
        assert_eq!(policy.backoff(1), Duration::from_millis(200));
        assert_eq!(policy.backoff(2), Duration::from_millis(400));
        assert_eq!(policy.backoff(3), Duration::from_millis(800));
        assert_eq!(policy.backoff(10), Duration::from_secs(30)); // Max delay
    }

    #[test]
    fn test_linear_backoff() {
        let policy = RetryPolicy::linear(3);

        assert_eq!(policy.backoff(1), Duration::from_secs(1));
        assert_eq!(policy.backoff(2), Duration::from_secs(2));
        assert_eq!(policy.backoff(3), Duration::from_secs(3));
        assert_eq!(policy.backoff(20), Duration::from_secs(10)); // Max delay
    }

    #[test]
    fn test_custom_policy() {
        let policy =
            RetryPolicy::custom(5, Duration::from_millis(500), Duration::from_secs(5), true);

        assert_eq!(policy.max_attempts(), 5);
        assert_eq!(policy.backoff(0), Duration::from_millis(500));
        assert_eq!(policy.backoff(1), Duration::from_millis(1000));
    }

    #[tokio::test]
    async fn test_retry_client_success() {
        use crate::client::MockClient;
        use serde_json::json;

        let mock = MockClient::new().with_response("test.method", json!({"success": true}));

        let retry_client = RetryClient::new(mock, RetryPolicy::exponential(3));

        let result = retry_client
            .get("http://example.com?method=test.method")
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts(), 3);
    }
}
