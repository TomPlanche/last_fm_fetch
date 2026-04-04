use std::sync::Arc;

use crate::api::constants::{BASE_URL, METHOD_PERSONAL_TAGS};
use crate::api::user_params;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{
    PersonalTaggedAlbumsPage, PersonalTaggedAlbumsResponse, PersonalTaggedArtistsPage,
    PersonalTaggedArtistsResponse, PersonalTaggedTracksPage, PersonalTaggedTracksResponse,
};
use crate::url_builder::Url;

/// Builder for `user.getPersonalTags` requests
#[derive(Debug)]
pub struct PersonalTagsRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    tag: String,
    limit: Option<u32>,
    page: Option<u32>,
}

impl PersonalTagsRequestBuilder {
    pub(crate) fn new(
        http: Arc<dyn HttpClient>,
        config: Arc<Config>,
        username: String,
        tag: String,
    ) -> Self {
        Self {
            http,
            config,
            username,
            tag,
            limit: None,
            page: None,
        }
    }

    /// Set the number of results per page (default 50, max 1000).
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(if limit < 1000 { limit } else { 1000 });
        self
    }

    /// Set the page number (1-indexed).
    #[must_use]
    pub const fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    fn build_params(&self, tagging_type: &str) -> crate::url_builder::QueryParams {
        let mut params = user_params(METHOD_PERSONAL_TAGS, &self.username, self.config.api_key());
        params.insert("tag".to_string(), self.tag.clone());
        params.insert("taggingtype".to_string(), tagging_type.to_string());

        if let Some(limit) = self.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        if let Some(page) = self.page {
            params.insert("page".to_string(), page.to_string());
        }

        params
    }

    /// Fetch tracks tagged with this personal tag.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch_tracks(self) -> Result<PersonalTaggedTracksPage> {
        let url = Url::new(BASE_URL)
            .add_args(self.build_params("track"))
            .build();
        let value = self.http.get(&url).await?;
        let response: PersonalTaggedTracksResponse = serde_json::from_value(value)?;

        Ok(PersonalTaggedTracksPage::from(response))
    }

    /// Fetch artists tagged with this personal tag.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch_artists(self) -> Result<PersonalTaggedArtistsPage> {
        let url = Url::new(BASE_URL)
            .add_args(self.build_params("artist"))
            .build();
        let value = self.http.get(&url).await?;
        let response: PersonalTaggedArtistsResponse = serde_json::from_value(value)?;

        Ok(PersonalTaggedArtistsPage::from(response))
    }

    /// Fetch albums tagged with this personal tag.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch_albums(self) -> Result<PersonalTaggedAlbumsPage> {
        let url = Url::new(BASE_URL)
            .add_args(self.build_params("album"))
            .build();
        let value = self.http.get(&url).await?;
        let response: PersonalTaggedAlbumsResponse = serde_json::from_value(value)?;

        Ok(PersonalTaggedAlbumsPage::from(response))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::client::MockClient;
    use crate::config::ConfigBuilder;
    use serde_json::json;
    use std::sync::Arc;

    fn make_builder(method_response: serde_json::Value) -> PersonalTagsRequestBuilder {
        let config = Arc::new(ConfigBuilder::new().api_key("test_key").build().unwrap());
        let mock =
            Arc::new(MockClient::new().with_response("user.getpersonaltags", method_response));
        PersonalTagsRequestBuilder::new(mock, config, "testuser".to_string(), "indie".to_string())
    }

    #[tokio::test]
    async fn test_fetch_tagged_tracks() {
        let builder = make_builder(json!({
            "taggings": {
                "@attr": { "total": "1", "page": "1", "totalPages": "1", "perPage": "50" },
                "tracks": {
                    "track": [{
                        "name": "Test Track",
                        "mbid": "",
                        "url": "https://www.last.fm/music/Artist/_/Test+Track",
                        "artist": { "name": "Test Artist", "mbid": "", "url": "https://www.last.fm/music/Test+Artist" },
                        "image": []
                    }]
                }
            }
        }));

        let page = builder.fetch_tracks().await.unwrap();
        assert_eq!(page.tracks.len(), 1);
        assert_eq!(page.tracks[0].name, "Test Track");
        assert_eq!(page.tracks[0].artist_name, "Test Artist");
        assert_eq!(page.total, 1);
    }

    #[tokio::test]
    async fn test_fetch_tagged_artists() {
        let builder = make_builder(json!({
            "taggings": {
                "@attr": { "total": "1", "page": "1", "totalPages": "1", "perPage": "50" },
                "artists": {
                    "artist": [{
                        "name": "Test Artist",
                        "mbid": "",
                        "url": "https://www.last.fm/music/Test+Artist",
                        "image": []
                    }]
                }
            }
        }));

        let page = builder.fetch_artists().await.unwrap();
        assert_eq!(page.artists.len(), 1);
        assert_eq!(page.artists[0].name, "Test Artist");
    }

    #[tokio::test]
    async fn test_fetch_tagged_albums() {
        let builder = make_builder(json!({
            "taggings": {
                "@attr": { "total": "1", "page": "1", "totalPages": "1", "perPage": "50" },
                "albums": {
                    "album": [{
                        "name": "Test Album",
                        "mbid": "",
                        "url": "https://www.last.fm/music/Artist/Test+Album",
                        "artist": { "name": "Test Artist", "mbid": "", "url": "https://www.last.fm/music/Test+Artist" },
                        "image": []
                    }]
                }
            }
        }));

        let page = builder.fetch_albums().await.unwrap();
        assert_eq!(page.albums.len(), 1);
        assert_eq!(page.albums[0].name, "Test Album");
        assert_eq!(page.albums[0].artist_name, "Test Artist");
    }
}
