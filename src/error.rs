use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct LastFmErrorResponse {
    pub message: String,
    pub error: u32,
}

#[derive(Debug)]
pub enum LastFmError {
    /// Represents a Last.fm API error with code and message
    Api(LastFmErrorResponse),
    /// Represents HTTP/network errors
    Http(reqwest::Error),
    /// Represents JSON parsing errors
    Parse(serde_json::Error),
    /// Represents file I/O errors
    Io(std::io::Error),
    /// Represents missing environment variable errors
    MissingEnvVar(String),
    /// Represents other errors
    Other(String),
}

impl std::error::Error for LastFmError {}

impl fmt::Display for LastFmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LastFmError::Api(e) => write!(f, "Last.fm API error {}: {}", e.error, e.message),
            LastFmError::Http(e) => write!(f, "HTTP error: {e}"),
            LastFmError::Parse(e) => write!(f, "Parse error: {e}"),
            LastFmError::Io(e) => write!(f, "I/O error: {e}"),
            LastFmError::MissingEnvVar(var) => write!(
                f,
                "Missing required environment variable: {var}\n\
                 Please set it in your environment or .env file"
            ),
            LastFmError::Other(e) => write!(f, "Error: {e}"),
        }
    }
}

impl From<reqwest::Error> for LastFmError {
    fn from(err: reqwest::Error) -> Self {
        LastFmError::Http(err)
    }
}

impl From<serde_json::Error> for LastFmError {
    fn from(err: serde_json::Error) -> Self {
        LastFmError::Parse(err)
    }
}

impl From<std::io::Error> for LastFmError {
    fn from(err: std::io::Error) -> Self {
        LastFmError::Io(err)
    }
}

// Handle Box<dyn std::error::Error>
impl From<Box<dyn StdError>> for LastFmError {
    fn from(err: Box<dyn StdError>) -> Self {
        LastFmError::Other(err.to_string())
    }
}

/// Helper type for Result with `LastFmError`
pub type Result<T> = std::result::Result<T, LastFmError>;
