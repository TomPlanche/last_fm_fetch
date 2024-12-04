use crate::file_handler::{FileFormat, FileHandler};
use crate::types::*;
use crate::url_builder::{QueryParams, Url};

use reqwest::Error;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::env;

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
    type TrackType;

    fn total_tracks(&self) -> u32;
    fn tracks(self) -> Vec<Self::TrackType>;
}

impl TrackContainer for UserLovedTracks {
    type TrackType = LovedTrack;

    fn total_tracks(&self) -> u32 {
        self.lovedtracks.attr.total
    }
    fn tracks(self) -> Vec<Self::TrackType> {
        self.lovedtracks.track
    }
}

impl TrackContainer for UserRecentTracks {
    type TrackType = RecentTrack;

    fn total_tracks(&self) -> u32 {
        self.recenttracks.attr.total
    }
    fn tracks(self) -> Vec<Self::TrackType> {
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
    pub async fn get_user_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<LovedTrack>, Error> {
        self.get_user_tracks::<UserLovedTracks>("user.getlovedtracks", limit.into())
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
    pub async fn get_user_recent_tracks(
        &self,
        limit: impl Into<TrackLimit>,
    ) -> Result<Vec<RecentTrack>, Error> {
        self.get_user_tracks::<UserRecentTracks>("user.getrecenttracks", limit.into())
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
    ) -> Result<Vec<T::TrackType>, Error> {
        let mut all_tracks: Vec<T::TrackType> = Vec::new();

        // Make an initial request to get the total number of tracks
        let mut base_params: QueryParams = HashMap::new();
        base_params.insert("limit".to_string(), "1".to_string());

        let initial_response: T = self.fetch(method, &base_params).await?;
        let total_tracks = initial_response.total_tracks();

        let final_limit = match limit {
            TrackLimit::Limited(l) => l.min(total_tracks),
            TrackLimit::Unlimited => total_tracks,
        };

        if final_limit < API_MAX_LIMIT {
            // Directly fetch the data with the specified limit
            let mut base_params: QueryParams = HashMap::new();
            let final_limit_str = final_limit.to_string();
            base_params.insert("limit".to_string(), final_limit_str);

            let response: T = self.fetch(method, &base_params).await?;
            let total_tracks = response.total_tracks();

            // If the total tracks are less than the requested limit, adjust the final limit
            let actual_limit = final_limit.min(total_tracks);
            all_tracks.extend(response.tracks().into_iter().take(actual_limit as usize));

            return Ok(all_tracks);
        }

        // Make an initial request to get the total number of tracks
        let mut base_params: QueryParams = HashMap::new();
        base_params.insert("limit".to_string(), "1".to_string()); // Request only 1 track to get the total count

        let initial_response: T = self.fetch(method, &base_params).await?;
        let total_tracks = initial_response.total_tracks();

        // Determine the actual limit to use
        let actual_limit = final_limit.min(total_tracks);

        if actual_limit > API_MAX_LIMIT {
            let needed_chunks = ((actual_limit / CHUNK_SIZE) as f32).floor() as u32;

            println!("Needed chunks: {}", needed_chunks);

            for i in 0..needed_chunks {
                let mut all_fetches = Vec::new();

                println!("looping through chunks {}", i);

                for j in 0..CHUNK_MULTIPLIER {
                    println!("looping through chunk multiplier {}", j);

                    let chunk_offset = i * CHUNK_MULTIPLIER + (j + 1);
                    let final_limit_str = API_MAX_LIMIT.to_string();
                    let final_offset_str = chunk_offset.to_string();

                    // Create params inside this iteration to ensure it lives long enough
                    let mut params = self.base_options.clone();
                    params.insert("limit".to_string(), final_limit_str);
                    params.insert("page".to_string(), final_offset_str);

                    // Use async block to extend the lifetime of params
                    let fetch = async move { self.fetch::<T>(method, &params).await };
                    all_fetches.push(fetch);
                }

                // Await all fetches and collect results
                let chunk_results = futures::future::join_all(all_fetches).await;

                // Process and extend all_tracks with the results
                for result in chunk_results {
                    // Handle potential errors and add tracks
                    match result {
                        Ok(tracks) => all_tracks.extend(tracks.tracks()),
                        Err(e) => return Err(e), // Or handle errors as appropriate
                    }
                }
            }

            // Handle remainder
            let remainder = actual_limit % CHUNK_SIZE;
            println!("Remainder: {}", remainder);
            let needed_calls = (remainder as f32 / API_MAX_LIMIT as f32).ceil() as u32;

            let mut all_fetches = Vec::new();

            for i in 0..needed_calls {
                let final_limit_str = API_MAX_LIMIT.to_string();
                let final_offset_str = (CHUNK_MULTIPLIER * needed_chunks + i + 1).to_string();

                let mut params = self.base_options.clone();
                params.insert("limit".to_string(), final_limit_str);
                params.insert("page".to_string(), final_offset_str);

                let fetch = async move { self.fetch::<T>(method, &params).await };
                all_fetches.push(fetch);
            }

            let chunk_results = futures::future::join_all(all_fetches).await;

            for result in chunk_results {
                match result {
                    Ok(tracks) => all_tracks.extend(tracks.tracks()),
                    Err(e) => return Err(e),
                }
            }
        } else {
            let mut base_params: QueryParams = HashMap::new();
            let final_limit_str = actual_limit.to_string();

            base_params.insert("limit".to_string(), final_limit_str);

            let response: T = self.fetch(method, &base_params).await?;

            all_tracks.extend(response.tracks());
        }

        // trunc the vector to the final limit
        let final_tracks = all_tracks.into_iter().take(actual_limit as usize).collect();

        Ok(final_tracks)
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
    pub async fn get_and_save_loved_tracks(
        &self,
        limit: impl Into<TrackLimit>,
        format: FileFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tracks = self.get_user_loved_tracks(limit).await?;
        let filename = FileHandler::save(&tracks, format, "loved_tracks")?;
        Ok(filename)
    }

    #[allow(dead_code)]
    async fn test_fetch(
        &self,
        method: &str,
        params: &QueryParams,
    ) -> Result<UserRecentTracks, Error> {
        let mut final_params = self.base_options.clone();
        final_params.insert("method".to_string(), method.to_string());
        final_params.extend(params.clone());

        let base_url = self.url.clone().add_args(final_params).build();

        println!("[TEST] Fetching: {}", base_url);

        let a: UserRecentTracks = UserRecentTracks {
            recenttracks: RecentTracks {
                track: vec![],
                attr: BaseResponse {
                    user: "tom".to_string(),
                    total: 0,
                    total_pages: 0,
                    page: 0,
                    per_page: 0,
                },
            },
        };

        Ok(a)
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
        let count_mock = server
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
