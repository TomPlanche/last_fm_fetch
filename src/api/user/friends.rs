use std::sync::Arc;

use crate::api::constants::{BASE_URL, METHOD_FRIENDS};
use crate::api::user_params;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{FriendProfile, FriendsPage, FriendsResponse};
use crate::url_builder::Url;

/// Builder for `user.getFriends` requests
#[derive(Debug)]
pub struct FriendsRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    limit: Option<u32>,
    page: Option<u32>,
    recent_tracks: bool,
}

impl FriendsRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            limit: None,
            page: None,
            recent_tracks: false,
        }
    }

    /// Set the number of results per page (default 50, max 50).
    #[must_use]
    pub const fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(if limit < 50 { limit } else { 50 });

        self
    }

    /// Set the page number (1-indexed).
    #[must_use]
    pub const fn page(mut self, page: u32) -> Self {
        self.page = Some(page);

        self
    }

    /// Include the user's most recent tracks in the response.
    #[must_use]
    pub const fn with_recent_tracks(mut self) -> Self {
        self.recent_tracks = true;

        self
    }

    /// Fetch one page of friends.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch_page(self) -> Result<FriendsPage> {
        let mut params = user_params(METHOD_FRIENDS, &self.username, self.config.api_key());

        if let Some(limit) = self.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        if let Some(page) = self.page {
            params.insert("page".to_string(), page.to_string());
        }

        if self.recent_tracks {
            params.insert("recenttracks".to_string(), "1".to_string());
        }

        let url = Url::new(BASE_URL).add_args(params).build();
        let value = self.http.get(&url).await?;
        let response: FriendsResponse = serde_json::from_value(value)?;

        Ok(FriendsPage::from(response))
    }

    /// Fetch all friends across all pages.
    ///
    /// # Errors
    /// Returns an error if any HTTP request fails or any response cannot be parsed.
    pub async fn fetch_all(self) -> Result<Vec<FriendProfile>> {
        let mut all_friends = Vec::new();
        let mut page = 1u32;

        loop {
            let mut params = user_params(METHOD_FRIENDS, &self.username, self.config.api_key());
            params.insert("page".to_string(), page.to_string());

            if let Some(l) = self.limit {
                params.insert("limit".to_string(), l.to_string());
            }

            if self.recent_tracks {
                params.insert("recenttracks".to_string(), "1".to_string());
            }

            let url = Url::new(BASE_URL).add_args(params).build();
            let value = self.http.get(&url).await?;
            let response: FriendsResponse = serde_json::from_value(value)?;
            let friends_page = FriendsPage::from(response);

            let total_pages = friends_page.total_pages;
            all_friends.extend(friends_page.friends);

            if page >= total_pages {
                break;
            }

            page += 1;
        }

        Ok(all_friends)
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

    fn make_builder(response: serde_json::Value) -> FriendsRequestBuilder {
        let config = Arc::new(ConfigBuilder::new().api_key("test_key").build().unwrap());
        let mock = Arc::new(MockClient::new().with_response("user.getfriends", response));

        FriendsRequestBuilder::new(mock, config, "testuser".to_string())
    }

    fn friend_json(name: &str) -> serde_json::Value {
        json!({
            "name": name,
            "realname": "",
            "url": format!("https://www.last.fm/user/{name}"),
            "country": "UK",
            "subscriber": "0",
            "image": [],
            "registered": { "unixtime": "1108296000", "#text": "2005-02-13 00:00" }
        })
    }

    #[tokio::test]
    async fn test_fetch_page() {
        let builder = make_builder(json!({
            "friends": {
                "@attr": { "user": "testuser", "total": "2", "page": "1", "totalPages": "1", "perPage": "50" },
                "user": [friend_json("alice"), friend_json("bob")]
            }
        }));

        let page = builder.fetch_page().await.unwrap();
        assert_eq!(page.friends.len(), 2);
        assert_eq!(page.total, 2);
        assert_eq!(page.friends[0].name, "alice");
        assert_eq!(page.friends[1].name, "bob");
    }

    #[tokio::test]
    async fn test_fetch_all_single_page() {
        let builder = make_builder(json!({
            "friends": {
                "@attr": { "user": "testuser", "total": "1", "page": "1", "totalPages": "1", "perPage": "50" },
                "user": [friend_json("alice")]
            }
        }));

        let friends = builder.fetch_all().await.unwrap();
        assert_eq!(friends.len(), 1);
        assert_eq!(friends[0].name, "alice");
    }

    #[test]
    fn test_limit_clamped_to_50() {
        let config = Arc::new(ConfigBuilder::new().api_key("test_key").build().unwrap());
        let mock = Arc::new(MockClient::new());
        let builder = FriendsRequestBuilder::new(mock, config, "testuser".to_string()).limit(100);
        assert_eq!(builder.limit, Some(50));
    }
}
