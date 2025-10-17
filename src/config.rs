use crate::error::{LastFmError, Result};
use std::env;

/// Required environment variables for the application
const REQUIRED_ENV_VARS: &[&str] = &["LAST_FM_API_KEY"];

/// Validates that all required environment variables are set
///
/// # Errors
/// Returns `LastFmError::MissingEnvVar` if any required environment variable is missing
///
/// # Returns
/// * `Result<()>` - Ok if all required environment variables are present
pub fn validate_env_vars() -> Result<()> {
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
/// # Arguments
/// * `var_name` - The name of the environment variable to retrieve
///
/// # Errors
/// Returns `LastFmError::MissingEnvVar` if the environment variable is not set
///
/// # Returns
/// * `Result<String>` - The value of the environment variable
pub fn get_required_env_var(var_name: &str) -> Result<String> {
    env::var(var_name).map_err(|_| LastFmError::MissingEnvVar(var_name.to_string()))
}
