use std::sync::Arc;

use crate::api::constants::{BASE_URL, METHOD_TOP_TAGS};
use crate::api::user_params;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{TopTagsResponse, UserTopTag};
use crate::url_builder::Url;

/// Maximum number of tags returned by `user.getTopTags` (Last.fm API cap)
const MAX_LIMIT: u32 = 50;

/// Builder for `user.getTopTags` requests
#[derive(Debug)]
pub struct TopTagsRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    /// Number of tags to return (1–50, default 10)
    limit: Option<u32>,
}

impl TopTagsRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
        }
    }

    /// Set the maximum number of tags to return (1–50, default 10).
    ///
    /// Values above 50 are clamped to 50 by the Last.fm API.
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(if limit < MAX_LIMIT { limit } else { MAX_LIMIT });
        self
    }

    /// Fetch the user's top tags.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<UserTopTag>> {
        let mut params = user_params(METHOD_TOP_TAGS, &self.username, self.config.api_key());

        if let Some(limit) = self.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let url = Url::new(BASE_URL).add_args(params).build();
        let value = self.http.get(&url).await?;

        let response: TopTagsResponse = serde_json::from_value(value)?;

        Ok(Vec::from(response))
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

    fn make_builder(response: serde_json::Value) -> TopTagsRequestBuilder {
        let config = Arc::new(ConfigBuilder::new().api_key("test_key").build().unwrap());
        let mock = Arc::new(MockClient::new().with_response("user.gettoptags", response));
        TopTagsRequestBuilder::new(mock, config, "testuser".to_string())
    }

    #[tokio::test]
    async fn test_fetch_top_tags() {
        let builder = make_builder(json!({
            "toptags": {
                "@attr": { "user": "testuser" },
                "tag": [
                    { "name": "rock", "count": "100", "url": "https://www.last.fm/tag/rock" },
                    { "name": "indie", "count": "50", "url": "https://www.last.fm/tag/indie" }
                ]
            }
        }));

        let tags = builder.fetch().await.unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "rock");
        assert_eq!(tags[0].count, 100);
        assert_eq!(tags[1].name, "indie");
    }

    #[tokio::test]
    async fn test_fetch_top_tags_empty() {
        let builder = make_builder(json!({
            "toptags": {
                "@attr": { "user": "testuser" },
                "tag": []
            }
        }));

        let tags = builder.fetch().await.unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_limit_clamped_to_50() {
        let config = Arc::new(ConfigBuilder::new().api_key("test_key").build().unwrap());
        let mock = Arc::new(MockClient::new());
        let builder = TopTagsRequestBuilder::new(mock, config, "testuser".to_string()).limit(100);
        assert_eq!(builder.limit, Some(50));
    }
}
