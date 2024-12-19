use crate::analytics::AnalysisHandler;
use crate::file_handler::{FileFormat, FileHandler};
use crate::types::*;
use crate::url_builder::{QueryParams, Url};

use futures::future::join_all;
use reqwest::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::path::Path;

const API_MAX_LIMIT: u32 = 1000;

const CHUNK_MULTIPLIER: u32 = 5;
const CHUNK_SIZE: u32 = API_MAX_LIMIT * CHUNK_MULTIPLIER;

#[derive(Debug, Clone, Copy)]
pub enum TrackLimit {
    Limited(u32),
    Unlimited,
}

impl From<Option<u32>> for TrackLimit {
    fn from(opt: Option<u32>) -> Self {
        match opt {
            Some(limit) => TrackLimit::Limited(limit),
            None => TrackLimit::Unlimited,
        }
    }
}

trait TrackContainer {
    type ApiTrackType;
    type StorageTrackType: From<Self::ApiTrackType>;

    fn total_tracks(&self) -> u32;
    fn tracks(self) -> Vec<Self::ApiTrackType>;
}

impl TrackContainer for UserLovedTracks {
    type ApiTrackType = LovedTrack; // No change needed for LovedTracks
    type StorageTrackType = LovedTrack; // No change needed for LovedTracks

    fn total_tracks(&self) -> u32 {
        self.lovedtracks.attr.total
    }
    fn tracks(self) -> Vec<Self::ApiTrackType> {
        self.lovedtracks.track
    }
}

impl TrackContainer for UserRecentTracks {
    type ApiTrackType = ApiRecentTrack;
    type StorageTrackType = RecentTrack;

    fn total_tracks(&self) -> u32 {
        self.recenttracks.attr.total
    }
    fn tracks(self) -> Vec<Self::ApiTrackType> {
        self.recenttracks.track
    }
}

#[derive(Debug)]
pub struct LastFMHandler {
    url: Url,
    base_options: QueryParams,
}

impl LastFMHandler {
    pub fn new(url: Url, username: &str) -> Self {
        let mut base_options = QueryParams::new();
        base_options.insert("api_key".to_string(), env::var("LAST_FM_API_KEY").unwrap());
        base_options.insert("limit".to_string(), API_MAX_LIMIT.to_string());
        base_options.insert("format".to_string(), "json".to_string());
        base_options.insert("user".to_string(), username.to_string());

        LastFMHandler { url, base_options }
    }

    ///
    /// # get_user_loved_tracks
    /// Get loved tracks for a user.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Result<Vec<LovedTrack>, Error>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<LovedTrack>, Error> {
        self.get_user_tracks::<UserLovedTracks>("user.getlovedtracks", limit.into(), None)
            .await
    }

    ///
    /// # get_user_recent_tracks
    /// Get recent tracks for a user.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Result<Vec<RecentTrack>, Error>` - The fetched tracks.
    #[allow(dead_code)]
    pub async fn get_user_recent_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<RecentTrack>, Error> {
        self.get_user_tracks::<UserRecentTracks>("user.getrecenttracks", limit.into(), None)
            .await
    }

    ///
    /// # get_user_tracks
    /// Get tracks for a user.
    ///
    /// ## Arguments
    /// * `method` - The method to call.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Result<Vec<T::TrackType>, Error>` - The fetched tracks.
    async fn get_user_tracks<T: DeserializeOwned + TrackContainer>(
        &self,
        method: &str,
        limit: TrackLimit,
        additional_params: Option<QueryParams>,
    ) -> Result<Vec<T::StorageTrackType>, Error> {
        let mut params = self.base_options.clone();
        if let Some(additional_params) = additional_params {
            params.extend(additional_params);
        }

        // Make an initial request to get the total number of tracks
        let mut base_params: QueryParams = HashMap::new();
        base_params.insert("limit".to_string(), "1".to_string());
        base_params.insert("page".to_string(), "1".to_string());
        base_params.extend(params.clone());

        let initial_response: T = self.fetch(method, &base_params).await?;
        let total_tracks = initial_response.total_tracks();

        let final_limit = match limit {
            TrackLimit::Limited(l) => l.min(total_tracks),
            TrackLimit::Unlimited => total_tracks,
        };

        println!("Need to fetch {} tracks", final_limit);

        if final_limit <= API_MAX_LIMIT {
            // If we need less than the API limit, just make a single request
            let mut base_params: QueryParams = HashMap::new();
            base_params.insert("limit".to_string(), final_limit.to_string());
            base_params.insert("page".to_string(), "1".to_string());
            base_params.extend(params);

            let response: T = self.fetch(method, &base_params).await?;
            return Ok(response
                .tracks()
                .into_iter()
                .take(final_limit as usize)
                .map(|t| T::StorageTrackType::from(t))
                .collect());
        }

        let chunk_nb = (final_limit as f32 / CHUNK_SIZE as f32).ceil() as u32;

        let mut all_tracks = Vec::new();

        // Process chunks sequentially
        for chunk_index in 0..chunk_nb {
            println!("Processing chunk {}/{}", chunk_index + 1, chunk_nb);
            let chunk_params = params.clone();

            // Calculate how many API calls we need for this chunk
            let chunk_api_calls = if chunk_index == chunk_nb - 1 {
                // Last chunk
                final_limit % CHUNK_SIZE / API_MAX_LIMIT + 1
            } else {
                CHUNK_SIZE / API_MAX_LIMIT
            };

            // Create futures for concurrent API calls within this chunk
            let api_call_futures: Vec<_> = (0..chunk_api_calls)
                .map(|call_index| {
                    let mut call_params = chunk_params.clone();
                    let call_limit =
                        (final_limit - chunk_index * CHUNK_SIZE - call_index * API_MAX_LIMIT)
                            .min(API_MAX_LIMIT);

                    let page = chunk_index * CHUNK_SIZE / API_MAX_LIMIT + call_index + 1;

                    call_params.insert("limit".to_string(), call_limit.to_string());
                    call_params.insert("page".to_string(), page.to_string());

                    async move {
                        let response: T = self.fetch(method, &call_params).await?;
                        Ok::<_, Error>(
                            response
                                .tracks()
                                .into_iter()
                                .take(call_limit as usize)
                                .map(|t| T::StorageTrackType::from(t))
                                .collect::<Vec<_>>(),
                        )
                    }
                })
                .collect();

            // Process all API calls in this chunk concurrently
            let chunk_results = join_all(api_call_futures).await;

            // Collect results from this chunk
            for result in chunk_results {
                all_tracks.extend(result?);
            }
        }

        Ok(all_tracks)
    }

    ///
    /// # fetch
    /// Fetch data from the LastFM API.
    ///
    /// ## Arguments
    /// * `method` - The method to call.
    /// * `params` - The parameters to pass to the API.
    ///
    /// ## Returns
    /// * `Result<T, Error>` - The fetched data.
    async fn fetch<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &QueryParams,
    ) -> Result<T, Error> {
        let mut final_params = self.base_options.clone();
        final_params.insert("method".to_string(), method.to_string());
        final_params.extend(params.clone());

        let base_url = self.url.clone().add_args(final_params).build();

        println!("Fetching: {}", base_url);

        let response = reqwest::get(&base_url).await?;
        let parsed_response = response.json::<T>().await?;

        Ok(parsed_response)
    }

    ///
    /// # get_and_save_recent_tracks
    /// Get and save recent tracks to a file.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    /// * `format` - The file format to save the tracks in.
    ///
    /// ## Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the saved file.
    #[allow(dead_code)]
    pub async fn get_and_save_recent_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        format: FileFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tracks = self.get_user_recent_tracks(limit).await?;
        let filename = FileHandler::save(&tracks, format, "recent_tracks")?;
        Ok(filename)
    }

    ///
    /// # get_and_save_loved_tracks
    /// Get and save loved tracks to a file.
    ///
    /// ## Arguments
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    /// * `format` - The file format to save the tracks in.
    ///
    /// ## Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the saved file.
    #[allow(dead_code)]
    pub async fn get_and_save_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        format: FileFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tracks = self.get_user_loved_tracks(limit).await?;
        let filename = FileHandler::save(&tracks, format, "loved_tracks")?;
        Ok(filename)
    }

    ///
    /// # `get_user_recent_tracks_since`
    /// Get recent tracks for a user since a given timestamp.
    ///
    /// ## Arguments
    /// * `timestamp` - The timestamp to fetch tracks since.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Vec<RecentTrack>` - The fetched tracks.
    pub async fn get_user_recent_tracks_since(
        &self,
        timestamp: u32,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<RecentTrack>, Error> {
        let mut params = QueryParams::new();
        params.insert("from".to_string(), timestamp.to_string());

        self.get_user_tracks::<UserRecentTracks>("user.getrecenttracks", limit.into(), Some(params))
            .await
    }

    ///
    /// # `get_user_loved_tracks_since`
    /// Get loved tracks for a user since a given timestamp.
    ///
    /// ## Arguments
    /// * `timestamp` - The timestamp to fetch tracks since.
    /// * `limit` - The number of tracks to fetch. If None, fetch all tracks.
    ///
    /// ## Returns
    /// * `Vec<LovedTrack>` - The fetched tracks.
    pub async fn get_user_loved_tracks_since(
        &self,
        timestamp: u32,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<LovedTrack>, Error> {
        let tracks = self.get_user_loved_tracks(limit).await?;

        Ok(tracks
            .into_iter()
            .filter(|track| track.date.uts > timestamp)
            .collect())
    }

    ///
    /// # `update_tracks_file`
    /// Update a tracks file with new tracks.
    ///
    /// ## Arguments
    /// * `file_path` - Path to the file to update.
    /// * `fetch_since` - Function to fetch tracks since a given timestamp.
    ///
    /// ## Returns
    /// * `Result<String, Box<dyn std::error::Error>>` - The filename of the updated file.
    pub async fn update_tracks_file<T: DeserializeOwned + Serialize + Timestamped>(
        &self,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Get the most recent timestamp from the file
        let last_timestamp =
            AnalysisHandler::get_most_recent_timestamp::<T>(file_path)?.unwrap_or(0);

        // Find the recent tracks in the file
        let recent_tracks = self
            .get_user_recent_tracks_since(last_timestamp, None)
            .await?;

        let file_path_str = file_path.to_str().unwrap();

        // // Append the new tracks to the file
        let updated_file = FileHandler::append(&recent_tracks, file_path_str)?;

        Ok(updated_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use mockito;
    use serde_json::json;

    fn setup() -> (mockito::Server, LastFMHandler) {
        dotenv().ok();
        std::env::set_var("LAST_FM_API_KEY", "test_api_key"); // Add this line

        let opts = mockito::ServerOpts {
            assert_on_drop: true,
            ..Default::default()
        };
        let server = mockito::Server::new_with_opts(opts);

        let url = Url::new(&server.url());
        let handler = LastFMHandler::new(url, "tom_planche");

        (server, handler)
    }

    #[tokio::test]
    async fn test_get_user_loved_tracks_single_page() {
        let (mut server, handler) = setup();

        let mock_response = json!({
          "lovedtracks": {
            "track": [
              {
                "artist": {
                  "url": "https://www.last.fm/music/Emmanuelle+Swiercz-Lamoure",
                  "name": "Emmanuelle Swiercz-Lamoure",
                  "mbid": ""
                },
                "date": {
                  "uts": "1732028251",
                  "#text": "19 Nov 2024, 14:57"
                },
                "mbid": "",
                "url": "https://www.last.fm/music/Emmanuelle+Swiercz-Lamoure/_/Valse+en+Fa+Di%C3%A8se+Mineur,+KKIb%2F7+%22Valse+m%C3%A9lancolique%22",
                "name": "Valse en Fa Dièse Mineur, KKIb/7 \"Valse mélancolique\"",
                "image": [
                  {
                    "size": "small",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/34s/2a96cbd8b46e442fc41c2b86b821562f.png"
                  },
                  {
                    "size": "medium",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/64s/2a96cbd8b46e442fc41c2b86b821562f.png"
                  },
                  {
                    "size": "large",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/174s/2a96cbd8b46e442fc41c2b86b821562f.png"
                  },
                  {
                    "size": "extralarge",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/300x300/2a96cbd8b46e442fc41c2b86b821562f.png"
                  }
                ],
                "streamable": {
                  "fulltrack": "0",
                  "#text": "0"
                }
              }
            ],
            "@attr": {
              "user": "Tom_planche",
              "totalPages": "74",
              "page": "1",
              "perPage": "1",
              "total": "74"
            }
          }
        });

        // Mock initial request for total count
        server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("limit".into(), "1".into()),
                mockito::Matcher::UrlEncoded("method".into(), "user.getlovedtracks".into()),
                mockito::Matcher::UrlEncoded("format".into(), "json".into()),
                mockito::Matcher::UrlEncoded("api_key".into(), "test_api_key".into()),
                mockito::Matcher::UrlEncoded("user".into(), "tom_planche".into()),
            ]))
            .with_status(200)
            .with_body(mock_response.to_string())
            .expect(2)
            .create();

        let result = handler.get_user_loved_tracks(Some(1)).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.first().unwrap().artist.name,
            "Emmanuelle Swiercz-Lamoure"
        );
        assert_eq!(
            result.first().unwrap().name,
            "Valse en Fa Dièse Mineur, KKIb/7 \"Valse mélancolique\""
        );
    }

    #[tokio::test]
    async fn test_get_user_recent_tracks_single_page() {
        let (mut server, handler) = setup();

        let mock_response = json!({
          "recenttracks": {
            "track": [
              {
                "artist": {
                  "mbid": "b90c4001-4c7d-4de2-a3e0-1afbc548af54",
                  "#text": "Samson François"
                },
                "streamable": "0",
                "image": [
                  {
                    "size": "small",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/34s/2a96cbd8b46e442fc41c2b86b821562f.png"
                  },
                  {
                    "size": "medium",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/64s/2a96cbd8b46e442fc41c2b86b821562f.png"
                  },
                  {
                    "size": "large",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/174s/2a96cbd8b46e442fc41c2b86b821562f.png"
                  },
                  {
                    "size": "extralarge",
                    "#text": "https://lastfm.freetls.fastly.net/i/u/300x300/2a96cbd8b46e442fc41c2b86b821562f.png"
                  }
                ],
                "mbid": "",
                "album": {
                  "mbid": "",
                  "#text": "Chopin: 14 Waltzes [2011 - Remaster] (2011 - Remaster)"
                },
                "name": "Valse n°10 en si mineur Op.69 n°2 (Remasterisé en 2011 - Multi channel)",
                "url": "https://www.last.fm/music/Samson+Fran%C3%A7ois/_/Valse+n%C2%B010+en+si+mineur+Op.69+n%C2%B02+(Remasteris%C3%A9+en+2011+-+Multi+channel)",
                "date": {
                  "uts": "1732815200",
                  "#text": "28 Nov 2024, 17:33"
                }
              }
            ],
            "@attr": {
              "user": "Tom_planche",
              "totalPages": "99718",
              "page": "1",
              "perPage": "1",
              "total": "99718"
            }
          }
        });

        // Mock initial request for total count
        server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("limit".into(), "1".into()),
                mockito::Matcher::UrlEncoded("method".into(), "user.getrecenttracks".into()),
                mockito::Matcher::UrlEncoded("format".into(), "json".into()),
                mockito::Matcher::UrlEncoded("api_key".into(), "test_api_key".into()),
                mockito::Matcher::UrlEncoded("user".into(), "tom_planche".into()),
            ]))
            .with_status(200)
            .with_body(mock_response.to_string())
            .expect(2)
            .create();

        let result = handler.get_user_recent_tracks(Some(1)).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().unwrap().artist.text, "Samson François");
        assert_eq!(
            result.first().unwrap().name,
            "Valse n°10 en si mineur Op.69 n°2 (Remasterisé en 2011 - Multi channel)"
        );
    }
}
