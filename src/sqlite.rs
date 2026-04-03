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

/// Trait for types that can be loaded from a `SQLite` database.
///
/// This is the read-side companion to [`SqliteExportable`]. Implementors
/// declare a SELECT query and a row-to-value mapping function, enabling
/// [`crate::file_handler::FileHandler::load_sqlite`] to work generically
/// across all data types.
///
/// # Field coverage
///
/// `SQLite` schemas store a curated subset of each type's fields (see the
/// `SqliteExportable` schema for each type). Fields that are **not** stored
/// (e.g. `image`, `streamable`, human-readable date strings) are
/// reconstructed with sensible empty/default values on load. The loaded
/// data is fully usable for analysis — all aggregation methods such as
/// `to_set()`, `top_artists()`, `by_date()`, etc. work correctly.
pub trait SqliteLoadable: Sized {
    /// Returns the SELECT SQL that fetches all rows for this type.
    ///
    /// The column order in the query must match the index expectations of
    /// [`Self::from_row`].
    fn select_sql() -> &'static str;

    /// Construct an instance from a database row.
    ///
    /// Column indices must correspond to the order defined by
    /// [`Self::select_sql`].
    ///
    /// # Errors
    /// Returns a [`rusqlite::Error`] if a column is missing or has an
    /// unexpected type.
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>;
}
