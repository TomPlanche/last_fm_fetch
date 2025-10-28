use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct LastFmErrorResponse {
    pub message: String,
    pub error: u32,
}

#[derive(Debug, Error)]
pub enum LastFmError {
    /// Represents a Last.fm API error with code and message
    #[error("API request failed: {method} - {message} (code: {error_code})")]
    Api {
        method: String,
        message: String,
        error_code: u32,
        retryable: bool,
    },

    /// Represents rate limiting error
    #[error("Rate limit exceeded. Retry after {retry_after:?}")]
    RateLimited {
        retry_after: Option<Duration>,
    },

    /// Represents HTTP/network errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Represents JSON parsing errors
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    /// Represents file I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Represents CSV errors
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Represents missing environment variable errors
    #[error("Missing required environment variable: {0}\nPlease set it in your environment or .env file")]
    MissingEnvVar(String),

    /// Represents configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Represents other errors
    #[error("{0}")]
    Other(String),
}

impl LastFmError {
    /// Check if this error is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            LastFmError::Api { retryable, .. } => *retryable,
            LastFmError::RateLimited { .. } | LastFmError::Network(_) => true,
            _ => false,
        }
    }

    /// Get the retry delay if specified
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            LastFmError::RateLimited { retry_after } => *retry_after,
            _ => None,
        }
    }
}

// Handle Box<dyn std::error::Error> for compatibility with existing code
impl From<Box<dyn std::error::Error>> for LastFmError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        LastFmError::Other(err.to_string())
    }
}

/// Helper type for Result with `LastFmError`
pub type Result<T> = std::result::Result<T, LastFmError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        let api_error = LastFmError::Api {
            method: "test.method".to_string(),
            message: "Temporary error".to_string(),
            error_code: 500,
            retryable: true,
        };
        assert!(api_error.is_retryable());

        let non_retryable = LastFmError::Api {
            method: "test.method".to_string(),
            message: "Invalid API key".to_string(),
            error_code: 10,
            retryable: false,
        };
        assert!(!non_retryable.is_retryable());

        let rate_limited = LastFmError::RateLimited {
            retry_after: Some(Duration::from_secs(5)),
        };
        assert!(rate_limited.is_retryable());

        let parse_error = LastFmError::Parse(serde_json::from_str::<()>("invalid").unwrap_err());
        assert!(!parse_error.is_retryable());
    }

    #[test]
    fn test_rate_limit_retry_after() {
        let error = LastFmError::RateLimited {
            retry_after: Some(Duration::from_secs(5)),
        };
        assert_eq!(error.retry_after(), Some(Duration::from_secs(5)));

        let api_error = LastFmError::Api {
            method: "test".to_string(),
            message: "Error".to_string(),
            error_code: 500,
            retryable: true,
        };
        assert_eq!(api_error.retry_after(), None);
    }

    #[test]
    fn test_error_display() {
        let error = LastFmError::MissingEnvVar("LAST_FM_API_KEY".to_string());
        let display = format!("{error}");
        assert!(display.contains("LAST_FM_API_KEY"));
        assert!(display.contains("Please set it"));
    }
}
