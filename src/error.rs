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
    /// Access details via the struct fields: `method`, `message`, `error_code`, `retryable`
    #[error("api request failed")]
    Api {
        method: String,
        message: String,
        error_code: u32,
        retryable: bool,
    },

    /// Represents rate limiting error
    /// Access `retry_after` via the struct field
    #[error("rate limit exceeded")]
    RateLimited { retry_after: Option<Duration> },

    /// Represents HTTP/network errors
    /// Access source error via `Error::source()`
    #[error("network error")]
    Network(#[from] reqwest::Error),

    /// Represents JSON parsing errors
    /// Access source error via `Error::source()`
    #[error("failed to parse response")]
    Parse(#[from] serde_json::Error),

    /// Represents file I/O errors
    /// Access source error via `Error::source()`
    #[error("file operation failed")]
    Io(#[from] std::io::Error),

    /// Represents CSV errors
    /// Access source error via `Error::source()`
    #[error("csv operation failed")]
    Csv(#[from] csv::Error),

    /// Represents missing environment variable errors
    #[error("missing required environment variable")]
    MissingEnvVar(String),

    /// Represents configuration errors
    #[error("configuration error")]
    Config(String),

    /// Represents HTTP response errors (non-success status codes)
    #[error("http error")]
    Http {
        status: u16,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents URL parsing errors
    #[error("invalid url")]
    Url {
        #[source]
        source: url::ParseError,
    },
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

    /// Get the API method name if this is an API error
    #[must_use]
    pub fn api_method(&self) -> Option<&str> {
        match self {
            LastFmError::Api { method, .. } => Some(method),
            _ => None,
        }
    }

    /// Get the API error code if this is an API error
    #[must_use]
    pub fn api_error_code(&self) -> Option<u32> {
        match self {
            LastFmError::Api { error_code, .. } => Some(*error_code),
            _ => None,
        }
    }

    /// Get the API error message if this is an API error
    #[must_use]
    pub fn api_message(&self) -> Option<&str> {
        match self {
            LastFmError::Api { message, .. } => Some(message),
            _ => None,
        }
    }

    /// Get the environment variable name if this is a missing env var error
    #[must_use]
    pub fn env_var_name(&self) -> Option<&str> {
        match self {
            LastFmError::MissingEnvVar(name) => Some(name),
            _ => None,
        }
    }

    /// Get the HTTP status code if this is an HTTP error
    #[must_use]
    pub fn http_status(&self) -> Option<u16> {
        match self {
            LastFmError::Http { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl From<url::ParseError> for LastFmError {
    fn from(err: url::ParseError) -> Self {
        LastFmError::Url { source: err }
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
        assert_eq!(display, "missing required environment variable");
    }

    #[test]
    fn test_api_error_accessors() {
        let error = LastFmError::Api {
            method: "user.getrecenttracks".to_string(),
            message: "Invalid API key".to_string(),
            error_code: 10,
            retryable: false,
        };

        assert_eq!(error.api_method(), Some("user.getrecenttracks"));
        assert_eq!(error.api_error_code(), Some(10));
        assert_eq!(error.api_message(), Some("Invalid API key"));

        // Test that non-API errors return None
        let parse_error = LastFmError::Parse(serde_json::from_str::<()>("invalid").unwrap_err());
        assert_eq!(parse_error.api_method(), None);
        assert_eq!(parse_error.api_error_code(), None);
        assert_eq!(parse_error.api_message(), None);
    }

    #[test]
    fn test_env_var_accessor() {
        let error = LastFmError::MissingEnvVar("LAST_FM_API_KEY".to_string());
        assert_eq!(error.env_var_name(), Some("LAST_FM_API_KEY"));

        let api_error = LastFmError::Api {
            method: "test".to_string(),
            message: "Error".to_string(),
            error_code: 10,
            retryable: false,
        };
        assert_eq!(api_error.env_var_name(), None);
    }

    #[test]
    fn test_http_error() {
        let error = LastFmError::Http {
            status: 404,
            source: None,
        };
        assert_eq!(error.http_status(), Some(404));
        assert_eq!(format!("{error}"), "http error");
    }

    #[test]
    fn test_display_messages_format() {
        // All Display messages should be lowercase and concise
        assert_eq!(
            format!(
                "{}",
                LastFmError::Api {
                    method: "test".to_string(),
                    message: "msg".to_string(),
                    error_code: 1,
                    retryable: false
                }
            ),
            "api request failed"
        );
        assert_eq!(
            format!("{}", LastFmError::RateLimited { retry_after: None }),
            "rate limit exceeded"
        );
        assert_eq!(
            format!("{}", LastFmError::MissingEnvVar("TEST".to_string())),
            "missing required environment variable"
        );
        assert_eq!(
            format!("{}", LastFmError::Config("test".to_string())),
            "configuration error"
        );
    }
}
