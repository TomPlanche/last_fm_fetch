//! Flat CSV rows for Last.fm resource types.
//!
//! Generic `csv::Writer::serialize` cannot handle nested structs (`BaseMbidText`, `Streamable`,
//! `Vec<TrackImage>`, …). These helpers write explicit column layouts for each supported type.
//!
//! Dispatch uses a `serde_json` round-trip so we can detect the element type without `unsafe`
//! (this crate forbids `unsafe_code`).

use std::io::{self, ErrorKind, Result};

use csv::Writer;
use serde::Serialize;

use crate::types::{
    LovedTrack, RecentTrack, RecentTrackExtended, ScoredAlbum, ScoredArtist, ScoredTrack, TopAlbum,
    TopArtist, TopTrack,
};

#[inline]
fn csv_err(e: &csv::Error) -> io::Error {
    io::Error::other(e.to_string())
}

fn json_value<T: Serialize>(data: &[T]) -> Result<serde_json::Value> {
    serde_json::to_value(data).map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))
}

pub(crate) fn save_as_csv_dispatch<T: Serialize>(data: &[T], filename: &str) -> Result<()> {
    let v = json_value(data)?;

    if let Ok(tracks) = serde_json::from_value::<Vec<RecentTrack>>(v.clone()) {
        return write_recent_tracks_csv(&tracks, filename, true);
    }

    if let Ok(tracks) = serde_json::from_value::<Vec<RecentTrackExtended>>(v.clone()) {
        return write_recent_tracks_extended_csv(&tracks, filename, true);
    }

    if let Ok(tracks) = serde_json::from_value::<Vec<LovedTrack>>(v.clone()) {
        return write_loved_tracks_csv(&tracks, filename, true);
    }

    if let Ok(tracks) = serde_json::from_value::<Vec<TopTrack>>(v.clone()) {
        return write_top_tracks_csv(&tracks, filename, true);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<TopArtist>>(v.clone()) {
        return write_top_artists_csv(&rows, filename, true);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<TopAlbum>>(v.clone()) {
        return write_top_albums_csv(&rows, filename, true);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<ScoredTrack>>(v.clone()) {
        return write_scored_tracks_csv(&rows, filename, true);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<ScoredArtist>>(v.clone()) {
        return write_scored_artists_csv(&rows, filename, true);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<ScoredAlbum>>(v) {
        return write_scored_albums_csv(&rows, filename, true);
    }

    save_as_csv_serde(data, filename)
}

pub(crate) fn append_csv_rows_dispatch<T: Serialize>(data: &[T], file_path: &str) -> Result<()> {
    let v = json_value(data)?;

    if let Ok(tracks) = serde_json::from_value::<Vec<RecentTrack>>(v.clone()) {
        return write_recent_tracks_csv(&tracks, file_path, false);
    }

    if let Ok(tracks) = serde_json::from_value::<Vec<RecentTrackExtended>>(v.clone()) {
        return write_recent_tracks_extended_csv(&tracks, file_path, false);
    }

    if let Ok(tracks) = serde_json::from_value::<Vec<LovedTrack>>(v.clone()) {
        return write_loved_tracks_csv(&tracks, file_path, false);
    }

    if let Ok(tracks) = serde_json::from_value::<Vec<TopTrack>>(v.clone()) {
        return write_top_tracks_csv(&tracks, file_path, false);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<TopArtist>>(v.clone()) {
        return write_top_artists_csv(&rows, file_path, false);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<TopAlbum>>(v.clone()) {
        return write_top_albums_csv(&rows, file_path, false);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<ScoredTrack>>(v.clone()) {
        return write_scored_tracks_csv(&rows, file_path, false);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<ScoredArtist>>(v.clone()) {
        return write_scored_artists_csv(&rows, file_path, false);
    }

    if let Ok(rows) = serde_json::from_value::<Vec<ScoredAlbum>>(v) {
        return write_scored_albums_csv(&rows, file_path, false);
    }

    append_csv_rows_serde(data, file_path)
}

fn save_as_csv_serde<T: Serialize>(data: &[T], filename: &str) -> Result<()> {
    let mut writer = Writer::from_path(filename)?;
    for item in data {
        writer.serialize(item).map_err(|e| csv_err(&e))?;
    }
    writer.flush()
}

fn append_csv_rows_serde<T: Serialize>(data: &[T], file_path: &str) -> Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(std::fs::OpenOptions::new().append(true).open(file_path)?);
    for item in data {
        writer.serialize(item).map_err(|e| csv_err(&e))?;
    }
    writer.flush()
}

fn write_recent_tracks_csv(
    tracks: &[RecentTrack],
    path: &str,
    include_headers: bool,
) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record([
            "name",
            "artist",
            "artist_mbid",
            "album",
            "album_mbid",
            "track_mbid",
            "url",
            "streamable",
            "date_uts",
            "date_text",
            "now_playing",
        ])
        .map_err(|e| csv_err(&e))?;
    }

    for t in tracks {
        let date_uts = t
            .date
            .as_ref()
            .map(|d| d.uts.to_string())
            .unwrap_or_default();
        let streamable = if t.streamable { "true" } else { "false" };
        let now_playing = t.attr.as_ref().map_or("false", |a| a.nowplaying.as_str());
        w.write_record([
            t.name.as_str(),
            t.artist.text.as_str(),
            t.artist.mbid.as_str(),
            t.album.text.as_str(),
            t.album.mbid.as_str(),
            t.mbid.as_str(),
            t.url.as_str(),
            streamable,
            date_uts.as_str(),
            t.date.as_ref().map_or("", |d| d.text.as_str()),
            now_playing,
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_recent_tracks_extended_csv(
    tracks: &[RecentTrackExtended],
    path: &str,
    include_headers: bool,
) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record([
            "name",
            "track_mbid",
            "url",
            "streamable",
            "artist_name",
            "artist_mbid",
            "artist_url",
            "album_name",
            "album_mbid",
            "album_url",
            "date_uts",
            "date_text",
            "now_playing",
        ])
        .map_err(|e| csv_err(&e))?;
    }

    for t in tracks {
        let date_uts = t
            .date
            .as_ref()
            .map(|d| d.uts.to_string())
            .unwrap_or_default();
        let streamable = if t.streamable { "true" } else { "false" };
        let now_playing = t
            .attr
            .as_ref()
            .and_then(|m| m.get("nowplaying"))
            .map_or("false", std::string::String::as_str);
        w.write_record([
            t.name.as_str(),
            t.mbid.as_str(),
            t.url.as_str(),
            streamable,
            t.artist.name.as_str(),
            t.artist.mbid.as_str(),
            t.artist.url.as_str(),
            t.album.name.as_str(),
            t.album.mbid.as_str(),
            t.album.url.as_str(),
            date_uts.as_str(),
            t.date.as_ref().map_or("", |d| d.text.as_str()),
            now_playing,
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_loved_tracks_csv(tracks: &[LovedTrack], path: &str, include_headers: bool) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record([
            "name",
            "mbid",
            "url",
            "artist_name",
            "artist_mbid",
            "artist_url",
            "date_uts",
            "date_text",
            "streamable_fulltrack",
            "streamable_text",
        ])
        .map_err(|e| csv_err(&e))?;
    }

    for t in tracks {
        let date_uts = t.date.uts.to_string();
        w.write_record([
            t.name.as_str(),
            t.mbid.as_str(),
            t.url.as_str(),
            t.artist.name.as_str(),
            t.artist.mbid.as_str(),
            t.artist.url.as_str(),
            date_uts.as_str(),
            t.date.text.as_str(),
            t.streamable.fulltrack.as_str(),
            t.streamable.text.as_str(),
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_top_tracks_csv(tracks: &[TopTrack], path: &str, include_headers: bool) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record([
            "name",
            "mbid",
            "url",
            "duration",
            "playcount",
            "rank",
            "artist_name",
            "artist_mbid",
            "artist_url",
            "streamable_fulltrack",
            "streamable_text",
        ])
        .map_err(|e| csv_err(&e))?;
    }

    for t in tracks {
        let duration = t.duration.to_string();
        let playcount = t.playcount.to_string();
        w.write_record([
            t.name.as_str(),
            t.mbid.as_str(),
            t.url.as_str(),
            duration.as_str(),
            playcount.as_str(),
            t.attr.rank.as_str(),
            t.artist.name.as_str(),
            t.artist.mbid.as_str(),
            t.artist.url.as_str(),
            t.streamable.fulltrack.as_str(),
            t.streamable.text.as_str(),
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_top_artists_csv(rows: &[TopArtist], path: &str, include_headers: bool) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record(["name", "mbid", "url", "playcount", "streamable", "rank"])
            .map_err(|e| csv_err(&e))?;
    }

    for t in rows {
        let playcount = t.playcount.to_string();
        let streamable = if t.streamable { "true" } else { "false" };
        w.write_record([
            t.name.as_str(),
            t.mbid.as_str(),
            t.url.as_str(),
            playcount.as_str(),
            streamable,
            t.attr.rank.as_str(),
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_top_albums_csv(rows: &[TopAlbum], path: &str, include_headers: bool) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record([
            "name",
            "mbid",
            "url",
            "playcount",
            "rank",
            "artist_name",
            "artist_mbid",
            "artist_url",
        ])
        .map_err(|e| csv_err(&e))?;
    }

    for t in rows {
        let playcount = t.playcount.to_string();
        w.write_record([
            t.name.as_str(),
            t.mbid.as_str(),
            t.url.as_str(),
            playcount.as_str(),
            t.attr.rank.as_str(),
            t.artist.name.as_str(),
            t.artist.mbid.as_str(),
            t.artist.url.as_str(),
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_scored_tracks_csv(rows: &[ScoredTrack], path: &str, include_headers: bool) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record([
            "name",
            "artist",
            "artist_mbid",
            "album",
            "mbid",
            "url",
            "play_count",
            "rank",
        ])
        .map_err(|e| csv_err(&e))?;
    }

    for t in rows {
        let play_count = t.play_count.to_string();
        let rank = t.rank.to_string();
        w.write_record([
            t.name.as_str(),
            t.artist.as_str(),
            t.artist_mbid.as_str(),
            t.album.as_str(),
            t.mbid.as_str(),
            t.url.as_str(),
            play_count.as_str(),
            rank.as_str(),
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_scored_artists_csv(
    rows: &[ScoredArtist],
    path: &str,
    include_headers: bool,
) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record(["name", "mbid", "play_count", "rank"])
            .map_err(|e| csv_err(&e))?;
    }

    for t in rows {
        let play_count = t.play_count.to_string();
        let rank = t.rank.to_string();
        w.write_record([
            t.name.as_str(),
            t.mbid.as_str(),
            play_count.as_str(),
            rank.as_str(),
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

fn write_scored_albums_csv(rows: &[ScoredAlbum], path: &str, include_headers: bool) -> Result<()> {
    let mut w = if include_headers {
        Writer::from_path(path).map_err(|e| csv_err(&e))?
    } else {
        csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(std::fs::OpenOptions::new().append(true).open(path)?)
    };

    if include_headers {
        w.write_record(["name", "mbid", "artist", "play_count", "rank"])
            .map_err(|e| csv_err(&e))?;
    }

    for t in rows {
        let play_count = t.play_count.to_string();
        let rank = t.rank.to_string();
        w.write_record([
            t.name.as_str(),
            t.mbid.as_str(),
            t.artist.as_str(),
            play_count.as_str(),
            rank.as_str(),
        ])
        .map_err(|e| csv_err(&e))?;
    }
    w.flush()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use crate::types::BaseMbidText;

    #[test]
    fn recent_track_csv_roundtrip_header() {
        let t = RecentTrack {
            artist: BaseMbidText {
                mbid: "a1".into(),
                text: "Artist".into(),
            },
            streamable: false,
            image: vec![],
            album: BaseMbidText {
                mbid: "b1".into(),
                text: "Album".into(),
            },
            attr: None,
            date: None,
            name: "Song".into(),
            mbid: String::new(),
            url: "https://x".into(),
        };
        let dir = std::env::temp_dir();
        let path = dir.join("lastfm_client_test_recent.csv");
        let path_s = path.to_str().expect("temp path must be UTF-8");
        let _ = std::fs::remove_file(path_s);
        write_recent_tracks_csv(std::slice::from_ref(&t), path_s, true).expect("csv write");
        let content = std::fs::read_to_string(path_s).expect("read csv");
        assert!(content.contains("name,artist,artist_mbid"));
        assert!(content.contains("Song"));
        assert!(content.contains("Artist"));
        let _ = std::fs::remove_file(path_s);
    }
}
