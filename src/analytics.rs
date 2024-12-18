use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, path::Path};

use serde::de::DeserializeOwned;

use crate::types::{LovedTrack, RecentTrack, Timestamped};

/// Trait for types that can be analyzed as tracks
#[allow(dead_code)]
pub trait TrackAnalyzable {
    /// Get the artist name from the track
    fn get_artist_name(&self) -> String;

    /// Get the track name from the track
    fn get_track_name(&self) -> String;

    /// Get the full track identifier (usually "artist - track")
    fn get_track_identifier(&self) -> String {
        format!("{} - {}", self.get_artist_name(), self.get_track_name())
    }
}

impl TrackAnalyzable for RecentTrack {
    fn get_artist_name(&self) -> String {
        self.artist.text.clone()
    }

    fn get_track_name(&self) -> String {
        self.name.clone()
    }
}

impl TrackAnalyzable for LovedTrack {
    fn get_artist_name(&self) -> String {
        self.artist.name.clone()
    }

    fn get_track_name(&self) -> String {
        self.name.clone()
    }
}

/// Represents statistics about tracks
#[derive(Debug)]
pub struct TrackStats {
    /// Total number of tracks
    pub total_tracks: usize,
    /// Map of artist names to play counts
    pub artist_play_counts: HashMap<String, usize>,
    /// Map of track names to play counts
    pub track_play_counts: HashMap<String, usize>,
    /// Map of tracks played less than threshold
    pub tracks_below_threshold: HashMap<String, usize>,
    /// Map of tracks played more than threshold
    pub tracks_above_threshold: HashMap<String, usize>,
    /// Most played artist
    pub most_played_artist: Option<(String, usize)>,
    /// Most played track
    pub most_played_track: Option<(String, usize)>,
}

pub struct AnalysisHandler;

impl AnalysisHandler {
    /// Analyze tracks from a JSON file
    ///
    /// # Arguments
    /// * `filename` - Path to the JSON file
    /// * `threshold` - Threshold for counting tracks with plays below this number
    ///
    /// # Returns
    /// * `Result<TrackStats, Box<dyn std::error::Error>>` - Analysis results
    pub fn analyze_file<T: DeserializeOwned + TrackAnalyzable>(
        file_path: &Path,
        threshold: usize,
    ) -> Result<TrackStats, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let tracks: Vec<T> = serde_json::from_reader(reader)?;

        Ok(Self::analyze_tracks(&tracks, threshold))
    }

    /// Analyze a vector of tracks
    ///
    /// # Arguments
    /// * `tracks` - Vector of tracks to analyze
    /// * `threshold` - Threshold for counting tracks with plays below this number
    ///
    /// # Returns
    /// * `TrackStats` - Analysis results
    pub fn analyze_tracks<T: TrackAnalyzable>(tracks: &[T], threshold: usize) -> TrackStats {
        let mut artist_play_counts: HashMap<String, usize> = HashMap::new();
        let mut track_play_counts: HashMap<String, usize> = HashMap::new();

        // Count plays for each artist and track
        for track in tracks {
            let artist_name = track.get_artist_name();
            let track_identifier = track.get_track_identifier();

            *artist_play_counts.entry(artist_name).or_insert(0) += 1;
            *track_play_counts.entry(track_identifier).or_insert(0) += 1;
        }

        // Find most played artist and track
        let most_played_artist = artist_play_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(name, &count)| (name.clone(), count));

        let most_played_track = track_play_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(name, &count)| (name.clone(), count));

        // Find tracks played less than threshold
        let tracks_below_threshold: HashMap<String, usize> = track_play_counts
            .iter()
            .filter(|(_, &count)| count < threshold)
            .map(|(name, &count)| (name.clone(), count))
            .collect();

        // Find tracks played more than threshold
        let tracks_above_threshold: HashMap<String, usize> = track_play_counts
            .iter()
            .filter(|(_, &count)| count >= threshold)
            .map(|(name, &count)| (name.clone(), count))
            .collect();

        TrackStats {
            total_tracks: tracks.len(),
            artist_play_counts,
            track_play_counts,
            tracks_below_threshold,
            tracks_above_threshold,
            most_played_artist,
            most_played_track,
        }
    }

    /// Print analysis results in a formatted way
    ///
    /// # Arguments
    /// * `stats` - TrackStats to print
    pub fn print_analysis(stats: &TrackStats) {
        println!("=== Track Analysis ===");
        println!("Total tracks: {}", stats.total_tracks);

        if let Some((artist, count)) = &stats.most_played_artist {
            println!("\nMost played artist: {} ({} plays)", artist, count);
        }

        if let Some((track, count)) = &stats.most_played_track {
            println!("Most played track: {} ({} plays)", track, count);
        }

        println!("\nTop 10 Artists:");
        let mut artists: Vec<_> = stats.artist_play_counts.iter().collect();
        artists.sort_by(|a, b| b.1.cmp(a.1));
        for (artist, count) in artists.iter().take(10) {
            println!("  {} - {} plays", artist, count);
        }

        println!("\nTop 10 Tracks:");
        let mut tracks: Vec<_> = stats.track_play_counts.iter().collect();
        tracks.sort_by(|a, b| b.1.cmp(a.1));
        for (track, count) in tracks.iter().take(10) {
            println!("  {} - {} plays", track, count);
        }

        println!(
            "\nTracks below threshold: {}",
            stats.tracks_below_threshold.len()
        );

        println!(
            "\nTracks above threshold: {}",
            stats.tracks_above_threshold.len()
        );
    }

    ///
    /// # `get_most_recent_timestamp`
    /// Get the most recent timestamp from a JSON file.
    ///
    /// ## Arguments
    /// * `file_path` - Path to the JSON file
    ///
    /// ## Returns
    /// * `Option<u32>` - Most recent timestamp
    pub fn get_most_recent_timestamp<T: DeserializeOwned + Timestamped>(
        file_path: &Path,
    ) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let tracks: Vec<T> = serde_json::from_reader(reader)?;

        Ok(tracks
            .iter()
            .filter_map(|track| track.get_timestamp())
            .max())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BaseMbidText, BaseObject, Date, Streamable};

    fn create_recent_track(artist: &str, name: &str) -> RecentTrack {
        RecentTrack {
            artist: BaseMbidText {
                mbid: String::new(),
                text: artist.to_string(),
            },
            streamable: false,
            image: Vec::new(),
            album: BaseMbidText {
                mbid: String::new(),
                text: String::new(),
            },
            attr: None,
            date: None,
            name: name.to_string(),
        }
    }

    fn create_loved_track(artist: &str, name: &str) -> LovedTrack {
        LovedTrack {
            artist: BaseObject {
                mbid: String::new(),
                url: String::new(),
                name: artist.to_string(),
            },
            date: Date {
                uts: 0,
                text: String::new(),
            },
            image: Vec::new(),
            streamable: Streamable {
                fulltrack: String::new(),
                text: String::new(),
            },
            name: name.to_string(),
            mbid: String::new(),
            url: String::new(),
        }
    }

    #[test]
    fn test_analyze_recent_tracks() {
        let tracks = vec![
            create_recent_track("Artist1", "Song1"),
            create_recent_track("Artist1", "Song1"),
            create_recent_track("Artist1", "Song2"),
            create_recent_track("Artist2", "Song3"),
        ];

        let stats = AnalysisHandler::analyze_tracks(&tracks, 2);

        assert_eq!(stats.total_tracks, 4);
        assert_eq!(stats.artist_play_counts["Artist1"], 3);
        assert_eq!(stats.artist_play_counts["Artist2"], 1);
        assert_eq!(stats.track_play_counts["Artist1 - Song1"], 2);
        assert_eq!(stats.most_played_artist, Some(("Artist1".to_string(), 3)));
    }

    #[test]
    fn test_analyze_loved_tracks() {
        let tracks = vec![
            create_loved_track("Artist1", "Song1"),
            create_loved_track("Artist1", "Song1"),
            create_loved_track("Artist1", "Song2"),
            create_loved_track("Artist2", "Song3"),
        ];

        let stats = AnalysisHandler::analyze_tracks(&tracks, 2);

        assert_eq!(stats.total_tracks, 4);
        assert_eq!(stats.artist_play_counts["Artist1"], 3);
        assert_eq!(stats.artist_play_counts["Artist2"], 1);
        assert_eq!(stats.track_play_counts["Artist1 - Song1"], 2);
        assert_eq!(stats.most_played_artist, Some(("Artist1".to_string(), 3)));
    }
}
