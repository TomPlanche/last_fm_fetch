//! Save recent scrobbles to disk, run incremental updates, reload files, and use local stats.
//!
//! Covers:
//!
//! - **`fetch_and_save`** / **`fetch_and_update`** — JSON, CSV, and NDJSON (same path for updates)
//! - **`FileHandler::load`** — JSON → `Vec<T>` → **`TrackList::from`** → `to_set()`, `top_artists()`, …
//! - **`FileHandler::load_ndjson`** — line-delimited JSON
//! - **`fetch_extended_and_save`**, loading **`RecentTrackExtended`**, extended aggregations
//! - **`analyze`** — `TrackStats` from the API without persisting first
//! - **`check_currently_playing`**
//! - **`on_progress`** callback
//! - **`top_tracks` / `top_artists` / `top_albums`** with **`Period`**, plus **`fetch_and_save`** for top tracks
//! - **`loved_tracks`** **`fetch_and_save`** / **`fetch_and_update`**
//!
//! Run from the **crate root** (next to this repo's `Cargo.toml`) so `data/` resolves correctly:
//!
//! ```bash
//! LAST_FM_API_KEY=your_key cargo run --example scrobbles_file_workflow
//! ```
//!
//! Optional: `LASTFM_USERNAME=yourname` (defaults to `tom_planche`).

use lastfm_client::api::Period;
use lastfm_client::file_handler::{FileFormat, FileHandler};
use lastfm_client::{
    Analyze, FetchAndSave, FetchAndUpdate, LastFmClient, LimitBuilder, RecentTrack,
    RecentTrackExtended, TrackList,
};

type DynResult = Result<(), Box<dyn std::error::Error>>;

fn username() -> String {
    std::env::var("LASTFM_USERNAME").unwrap_or_else(|_| "tom_planche".into())
}

async fn demo_json(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== JSON: save → incremental update → load → local aggregations ===\n");
    let json_path = client
        .recent_tracks(user)
        .limit(200)
        .fetch_and_save(FileFormat::Json, "scrobbles_demo")
        .await?;
    println!("Initial save: {json_path}");

    let added_json = client
        .recent_tracks(user)
        .fetch_and_update(&json_path)
        .await?;
    println!("fetch_and_update (JSON): {added_json} new scrobble(s)\n");

    let from_disk: Vec<RecentTrack> = FileHandler::load(&json_path)?;
    let tracks = TrackList::from(from_disk);
    println!(
        "Loaded {} scrobbles — unique tracks: {}, unique artists: {}",
        tracks.len(),
        tracks.unique_track_count(),
        tracks.unique_artist_count()
    );
    println!("Longest day streak (UTC dates): {} day(s)", tracks.streak());
    println!("Top 3 artists (local aggregation):");
    for a in tracks.top_artists().iter().take(3) {
        println!("  {a}");
    }
    println!();
    Ok(())
}

async fn demo_csv(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== CSV: save → fetch_and_update (append rows) ===\n");
    let csv_path = client
        .recent_tracks(user)
        .limit(50)
        .fetch_and_save(FileFormat::Csv, "scrobbles_demo_csv")
        .await?;
    println!("CSV save: {csv_path}");
    let added = client
        .recent_tracks(user)
        .fetch_and_update(&csv_path)
        .await?;
    println!("fetch_and_update (CSV): {added} new row(s)\n");
    Ok(())
}

async fn demo_ndjson(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== NDJSON: save → update (append) → load_ndjson ===\n");
    let nd_path = client
        .recent_tracks(user)
        .limit(100)
        .fetch_and_save(FileFormat::Ndjson, "scrobbles_demo_ndjson")
        .await?;
    println!("NDJSON save: {nd_path}");
    let added_nd = client
        .recent_tracks(user)
        .fetch_and_update(&nd_path)
        .await?;
    println!("fetch_and_update (NDJSON): {added_nd} new line(s)\n");

    let nd_vec: Vec<RecentTrack> = FileHandler::load_ndjson(&nd_path)?;
    let nd_list = TrackList::from(nd_vec);
    println!("NDJSON loaded {} rows; top track (local):", nd_list.len());
    if let Some(t) = nd_list.to_set().first() {
        println!("  {t}\n");
    }
    Ok(())
}

async fn demo_extended(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== Extended JSON: save → load → TrackList<RecentTrackExtended> stats ===\n");
    let ext_path = client
        .recent_tracks(user)
        .limit(80)
        .fetch_extended_and_save(FileFormat::Json, "scrobbles_extended_demo")
        .await?;
    println!("Extended save: {ext_path}");
    let ext_vec: Vec<RecentTrackExtended> = FileHandler::load(&ext_path)?;
    let ext = TrackList::from(ext_vec);
    println!(
        "Extended: {} rows, by_hour peak UTC hour index: {:?}",
        ext.len(),
        ext.by_hour()
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| *c)
            .map(|(h, c)| (h, *c))
    );
    println!();
    Ok(())
}

async fn demo_analyze_and_now_playing(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== analyze (API → TrackStats, no file) ===\n");
    let stats = client.recent_tracks(user).limit(400).analyze(3).await?;
    println!(
        "Sample: {} scrobbles, most played track: {:?}\n",
        stats.total_tracks, stats.most_played_track
    );

    println!("=== check_currently_playing ===\n");
    match client.recent_tracks(user).check_currently_playing().await? {
        Some(t) => println!(
            "Now playing: {} — {}\n",
            t.artist.text.as_deref().unwrap_or(""),
            t.name.as_deref().unwrap_or("")
        ),
        None => println!("Nothing marked as now playing.\n"),
    }
    Ok(())
}

async fn demo_progress_sample(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== on_progress (batched fetches) ===\n");
    let _ = client
        .recent_tracks(user)
        .limit(250)
        .on_progress(|fetched, total| {
            eprintln!("  progress: {fetched}/{total}");
        })
        .fetch()
        .await?;
    println!("(progress lines on stderr)\n");
    Ok(())
}

async fn demo_top_charts(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== Top tracks (Period::Month) — fetch_and_save JSON ===\n");
    let top_path = client
        .top_tracks(user)
        .period(Period::Month)
        .limit(30)
        .fetch_and_save(FileFormat::Json, "top_tracks_demo")
        .await?;
    println!("Top tracks saved to {top_path}\n");

    println!("=== Top artists & albums (fetch only) ===\n");
    let artists = client
        .top_artists(user)
        .period(Period::Month)
        .limit(5)
        .fetch()
        .await?;
    let albums = client
        .top_albums(user)
        .period(Period::Month)
        .limit(5)
        .fetch()
        .await?;
    println!("Top artists (API): {}", artists.len());
    println!("Top albums (API): {}\n", albums.len());
    Ok(())
}

async fn demo_loved(client: &LastFmClient, user: &str) -> DynResult {
    println!("=== Loved tracks: save → fetch_and_update ===\n");
    let loved_path = client
        .loved_tracks(user)
        .limit(80)
        .fetch_and_save(FileFormat::Json, "loved_demo")
        .await?;
    println!("Loved save: {loved_path}");
    let loved_new = client
        .loved_tracks(user)
        .fetch_and_update(&loved_path)
        .await?;
    println!(
        "Loved fetch_and_update: {loved_new} new (API has no since= filter; client filters)\n"
    );
    Ok(())
}

#[tokio::main]
async fn main() -> DynResult {
    dotenv::dotenv().ok();
    let client = LastFmClient::new()?;
    let user = username();

    demo_json(&client, &user).await?;
    demo_csv(&client, &user).await?;
    demo_ndjson(&client, &user).await?;
    demo_extended(&client, &user).await?;
    demo_analyze_and_now_playing(&client, &user).await?;
    demo_progress_sample(&client, &user).await?;
    demo_top_charts(&client, &user).await?;
    demo_loved(&client, &user).await?;

    println!("Done. Files are under ./data/ — run from the crate root.");
    Ok(())
}
