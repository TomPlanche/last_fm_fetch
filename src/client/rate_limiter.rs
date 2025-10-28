use crate::client::HttpClient;
use crate::error::Result;
use async_trait::async_trait;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Rate limiter using sliding window algorithm
pub struct RateLimiter {
    max_requests: u32,
    per_duration: Duration,
    semaphore: Arc<Semaphore>,
    window_start: Arc<Mutex<Instant>>,
    request_count: Arc<Mutex<u32>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed
    /// * `per_duration` - Time window for the rate limit
    ///
    /// # Example
    /// ```
    /// use lastfm_client::client::RateLimiter;
    /// use std::time::Duration;
    ///
    /// // Allow max 5 requests per second
    /// let limiter = RateLimiter::new(5, Duration::from_secs(1));
    /// ```
    #[must_use]
    pub fn new(max_requests: u32, per_duration: Duration) -> Self {
        Self {
            max_requests,
            per_duration,
            semaphore: Arc::new(Semaphore::new(max_requests as usize)),
            window_start: Arc::new(Mutex::new(Instant::now())),
            request_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Acquire permission to make a request
    ///
    /// This will block until a slot is available according to the rate limit
    ///
    /// # Panics
    /// Panics if acquiring a semaphore permit fails (should be unreachable under normal operation).
    pub async fn acquire(&self) {
        // Acquire a permit from the semaphore
        let _permit = self.semaphore.acquire().await.unwrap();

        let sleep_time = {
            let mut count = self.request_count.lock();
            let mut window = self.window_start.lock();

            let elapsed = window.elapsed();

            // If the window has expired, reset it
            if elapsed >= self.per_duration {
                *window = Instant::now();
                *count = 0;
            }

            // If we've hit the limit, return the sleep time
            if *count >= self.max_requests {
                let sleep_time = self.per_duration.saturating_sub(elapsed);
                Some(sleep_time)
            } else {
                *count += 1;
                None
            }
        }; // Locks are dropped here

        // Sleep outside the lock if needed
        if let Some(duration) = sleep_time {
            if duration > Duration::ZERO {
                #[cfg(debug_assertions)]
                eprintln!("Rate limit reached, sleeping for {duration:?}");

                tokio::time::sleep(duration).await;
            }

            // Reset the window after sleeping
            *self.window_start.lock() = Instant::now();
            *self.request_count.lock() = 1; // Count this request
        }
    }

    /// Get the maximum requests per duration
    #[must_use]
    pub fn max_requests(&self) -> u32 {
        self.max_requests
    }

    /// Get the time duration for the rate limit
    #[must_use]
    pub fn per_duration(&self) -> Duration {
        self.per_duration
    }

    /// Get the current request count in the current window
    #[must_use]
    pub fn current_count(&self) -> u32 {
        *self.request_count.lock()
    }
}

/// HTTP client wrapper that adds rate limiting
pub struct RateLimitedClient<C> {
    inner: C,
    limiter: Arc<RateLimiter>,
}

impl<C> RateLimitedClient<C> {
    /// Create a new rate-limited client wrapping an existing HTTP client
    #[must_use]
    pub fn new(inner: C, limiter: Arc<RateLimiter>) -> Self {
        Self { inner, limiter }
    }

    /// Get a reference to the inner client
    #[must_use]
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Get a reference to the rate limiter
    #[must_use]
    pub fn limiter(&self) -> &RateLimiter {
        &self.limiter
    }
}

#[async_trait]
impl<C: HttpClient + Send + Sync> HttpClient for RateLimitedClient<C> {
    async fn get(&self, url: &str) -> Result<serde_json::Value> {
        self.limiter.acquire().await;
        self.inner.get(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(2, Duration::from_millis(100));

        let start = Instant::now();

        // First two requests should be immediate
        limiter.acquire().await;
        limiter.acquire().await;

        let first_two = start.elapsed();
        assert!(first_two < Duration::from_millis(50));

        // Third request should wait for window to reset
        limiter.acquire().await;

        let all_three = start.elapsed();
        assert!(all_three >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limiter_window_reset() {
        let limiter = RateLimiter::new(1, Duration::from_millis(50));

        limiter.acquire().await;
        assert_eq!(limiter.current_count(), 1);

        // Wait for window to reset
        tokio::time::sleep(Duration::from_millis(60)).await;

        limiter.acquire().await;
        // Count should be 1 again after reset
        assert_eq!(limiter.current_count(), 1);
    }

    #[tokio::test]
    async fn test_rate_limited_client() {
        use crate::client::MockClient;
        use serde_json::json;

        let mock = MockClient::new().with_response("test.method", json!({"success": true}));

        let limiter = Arc::new(RateLimiter::new(5, Duration::from_secs(1)));
        let rate_limited = RateLimitedClient::new(mock, limiter);

        let result = rate_limited
            .get("http://example.com?method=test.method")
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limiter_properties() {
        let limiter = RateLimiter::new(10, Duration::from_secs(2));

        assert_eq!(limiter.max_requests(), 10);
        assert_eq!(limiter.per_duration(), Duration::from_secs(2));
    }
}
