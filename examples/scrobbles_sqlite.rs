//! `SQLite` workflow for scrobbles: create/update a `.db`, then **`FileHandler::load_sqlite`**.
//!
//! Requires the **`sqlite`** feature (bundled `rusqlite`, no system `SQLite` needed).
//!
//! Run from the **crate root** so `data/` paths match your working directory:
//!
//! ```bash
//! LAST_FM_API_KEY=your_key cargo run --example scrobbles_sqlite --features sqlite
//! ```
//!
//! Optional: `LASTFM_USERNAME=yourname` (defaults to `tom_planche`).

use lastfm_client::file_handler::FileHandler;
use lastfm_client::{
    FetchAndSave, FetchAndUpdate, LastFmClient, LimitBuilder, RecentTrack, TrackList,
};

fn username() -> String {
    std::env::var("LASTFM_USERNAME").unwrap_or_else(|_| "tom_planche".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let client = LastFmClient::new()?;
    let user = username();

    println!("=== SQLite: fetch_and_save_sqlite ===\n");
    let db_path = client
        .recent_tracks(&user)
        .limit(150)
        .fetch_and_save_sqlite("scrobbles_demo_sqlite")
        .await?;
    println!("Created: {db_path}\n");

    println!("=== SQLite: fetch_and_update_sqlite (uses MAX(date_uts) in DB) ===\n");
    let inserted = client
        .recent_tracks(&user)
        .fetch_and_update_sqlite(&db_path)
        .await?;
    println!("Inserted {inserted} new row(s)\n");

    println!("=== load_sqlite → TrackList → local aggregations ===\n");
    let tracks: TrackList<RecentTrack> = FileHandler::load_sqlite(&db_path)?;
    println!("Loaded {} rows from DB", tracks.len());
    println!("Unique tracks: {}", tracks.unique_track_count());
    println!("Top 3 (local to_set):");
    for t in tracks.to_set().iter().take(3) {
        println!("  {t}");
    }

    Ok(())
}
