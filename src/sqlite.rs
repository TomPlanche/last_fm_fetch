//! SQLite export support for Last.fm data types.
//!
//! This module is only available when the `sqlite` feature flag is enabled.
//!
//! # Example
//! ```ignore
//! use lastfm_client::LastFmClient;
//!
//! let client = LastFmClient::new("your_api_key")?;
//! let path = client
//!     .recent_tracks("username")
//!     .fetch_and_save_sqlite("recent_tracks")
//!     .await?;
//! println!("Saved to {path}");
//! ```

/// Trait for types that can be exported to a `SQLite` database.
///
/// Implementors declare their table schema and provide row-binding logic,
/// enabling generic bulk-insert operations across all data types.
pub trait SqliteExportable {
    /// Returns the name of the `SQLite` table for this type.
    fn table_name() -> &'static str;

    /// Returns the `CREATE TABLE IF NOT EXISTS` SQL statement for this type.
    fn create_table_sql() -> &'static str;

    /// Returns the `INSERT INTO` SQL statement for this type.
    fn insert_sql() -> &'static str;

    /// Binds this instance's fields to a prepared statement and executes it.
    ///
    /// # Errors
    /// Returns a `rusqlite::Error` if binding or execution fails.
    fn bind_and_execute(&self, stmt: &mut rusqlite::Statement<'_>) -> rusqlite::Result<usize>;
}
