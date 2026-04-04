use std::sync::Arc;

use crate::api::constants::{BASE_URL, METHOD_USER_INFO};
use crate::api::user_params;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{UserInfo, UserInfoResponse};
use crate::url_builder::Url;

/// Builder for `user.getInfo` requests
#[derive(Debug)]
pub struct UserInfoRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
}

impl UserInfoRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
        }
    }

    /// Fetch the user profile.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the user does not exist, or the
    /// response cannot be parsed.
    pub async fn fetch(self) -> Result<UserInfo> {
        let params = user_params(METHOD_USER_INFO, &self.username, self.config.api_key());
        let url = Url::new(BASE_URL).add_args(params).build();
        let value = self.http.get(&url).await?;
        let response: UserInfoResponse = serde_json::from_value(value)?;

        Ok(UserInfo::from(response))
    }
}
