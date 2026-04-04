use std::sync::Arc;

use crate::api::constants::{
    BASE_URL, METHOD_WEEKLY_ALBUM_CHART, METHOD_WEEKLY_ARTIST_CHART, METHOD_WEEKLY_CHART_LIST,
    METHOD_WEEKLY_TRACK_CHART,
};
use crate::api::user_params;
use crate::client::HttpClient;
use crate::config::Config;
use crate::error::Result;
use crate::types::{
    WeeklyAlbum, WeeklyAlbumChartResponse, WeeklyArtist, WeeklyArtistChartResponse,
    WeeklyChartListResponse, WeeklyChartRange, WeeklyTrack, WeeklyTrackChartResponse,
};
use crate::url_builder::Url;

// ── WeeklyChartListRequestBuilder ─────────────────────────────────────────────

/// Builder for `user.getWeeklyChartList` requests
#[derive(Debug)]
pub struct WeeklyChartListRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
}

impl WeeklyChartListRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
        }
    }

    /// Fetch the list of available weekly chart periods.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<WeeklyChartRange>> {
        let params = user_params(
            METHOD_WEEKLY_CHART_LIST,
            &self.username,
            self.config.api_key(),
        );
        let url = Url::new(BASE_URL).add_args(params).build();
        let value = self.http.get(&url).await?;
        let response: WeeklyChartListResponse = serde_json::from_value(value)?;

        Ok(Vec::from(response))
    }
}

// ── WeeklyTrackChartRequestBuilder ────────────────────────────────────────────

/// Builder for `user.getWeeklyTrackChart` requests
#[derive(Debug)]
pub struct WeeklyTrackChartRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    from: Option<u32>,
    to: Option<u32>,
}

impl WeeklyTrackChartRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            from: None,
            to: None,
        }
    }

    /// Set the start of the chart period (Unix timestamp).
    ///
    /// Should be used together with [`to`](Self::to).
    /// If omitted, returns the most recent week's chart.
    #[must_use]
    pub const fn from(mut self, timestamp: u32) -> Self {
        self.from = Some(timestamp);
        self
    }

    /// Set the end of the chart period (Unix timestamp).
    ///
    /// Should be used together with [`from`](Self::from).
    #[must_use]
    pub const fn to(mut self, timestamp: u32) -> Self {
        self.to = Some(timestamp);
        self
    }

    /// Set the chart period from a [`WeeklyChartRange`].
    #[must_use]
    pub const fn range(self, range: &WeeklyChartRange) -> Self {
        self.from(range.from).to(range.to)
    }

    /// Fetch the weekly track chart.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<WeeklyTrack>> {
        let mut params = user_params(
            METHOD_WEEKLY_TRACK_CHART,
            &self.username,
            self.config.api_key(),
        );

        if let Some(from) = self.from {
            params.insert("from".to_string(), from.to_string());
        }

        if let Some(to) = self.to {
            params.insert("to".to_string(), to.to_string());
        }

        let url = Url::new(BASE_URL).add_args(params).build();
        let value = self.http.get(&url).await?;
        let response: WeeklyTrackChartResponse = serde_json::from_value(value)?;

        Ok(Vec::from(response))
    }
}

// ── WeeklyArtistChartRequestBuilder ──────────────────────────────────────────

/// Builder for `user.getWeeklyArtistChart` requests
#[derive(Debug)]
pub struct WeeklyArtistChartRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    from: Option<u32>,
    to: Option<u32>,
}

impl WeeklyArtistChartRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            from: None,
            to: None,
        }
    }

    /// Set the start of the chart period (Unix timestamp).
    #[must_use]
    pub const fn from(mut self, timestamp: u32) -> Self {
        self.from = Some(timestamp);
        self
    }

    /// Set the end of the chart period (Unix timestamp).
    #[must_use]
    pub const fn to(mut self, timestamp: u32) -> Self {
        self.to = Some(timestamp);
        self
    }

    /// Set the chart period from a [`WeeklyChartRange`].
    #[must_use]
    pub const fn range(self, range: &WeeklyChartRange) -> Self {
        self.from(range.from).to(range.to)
    }

    /// Fetch the weekly artist chart.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<WeeklyArtist>> {
        let mut params = user_params(
            METHOD_WEEKLY_ARTIST_CHART,
            &self.username,
            self.config.api_key(),
        );

        if let Some(from) = self.from {
            params.insert("from".to_string(), from.to_string());
        }

        if let Some(to) = self.to {
            params.insert("to".to_string(), to.to_string());
        }

        let url = Url::new(BASE_URL).add_args(params).build();
        let value = self.http.get(&url).await?;
        let response: WeeklyArtistChartResponse = serde_json::from_value(value)?;

        Ok(Vec::from(response))
    }
}

// ── WeeklyAlbumChartRequestBuilder ───────────────────────────────────────────

/// Builder for `user.getWeeklyAlbumChart` requests
#[derive(Debug)]
pub struct WeeklyAlbumChartRequestBuilder {
    http: Arc<dyn HttpClient>,
    config: Arc<Config>,
    username: String,
    from: Option<u32>,
    to: Option<u32>,
}

impl WeeklyAlbumChartRequestBuilder {
    pub(crate) fn new(http: Arc<dyn HttpClient>, config: Arc<Config>, username: String) -> Self {
        Self {
            http,
            config,
            username,
            from: None,
            to: None,
        }
    }

    /// Set the start of the chart period (Unix timestamp).
    #[must_use]
    pub const fn from(mut self, timestamp: u32) -> Self {
        self.from = Some(timestamp);
        self
    }

    /// Set the end of the chart period (Unix timestamp).
    #[must_use]
    pub const fn to(mut self, timestamp: u32) -> Self {
        self.to = Some(timestamp);
        self
    }

    /// Set the chart period from a [`WeeklyChartRange`].
    #[must_use]
    pub const fn range(self, range: &WeeklyChartRange) -> Self {
        self.from(range.from).to(range.to)
    }

    /// Fetch the weekly album chart.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn fetch(self) -> Result<Vec<WeeklyAlbum>> {
        let mut params = user_params(
            METHOD_WEEKLY_ALBUM_CHART,
            &self.username,
            self.config.api_key(),
        );

        if let Some(from) = self.from {
            params.insert("from".to_string(), from.to_string());
        }

        if let Some(to) = self.to {
            params.insert("to".to_string(), to.to_string());
        }

        let url = Url::new(BASE_URL).add_args(params).build();
        let value = self.http.get(&url).await?;
        let response: WeeklyAlbumChartResponse = serde_json::from_value(value)?;

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

    fn http_config(
        method: &str,
        response: serde_json::Value,
    ) -> (Arc<dyn HttpClient>, Arc<Config>) {
        let config = Arc::new(ConfigBuilder::new().api_key("test_key").build().unwrap());
        let mock = Arc::new(MockClient::new().with_response(method, response));
        (mock, config)
    }

    #[tokio::test]
    async fn test_fetch_chart_list() {
        let (http, config) = http_config(
            "user.getweeklychartlist",
            json!({
                "weeklychartlist": {
                    "@attr": { "user": "testuser" },
                    "chart": [
                        { "from": "1108296000", "to": "1108900800" },
                        { "from": "1108900800", "to": "1109505600" }
                    ]
                }
            }),
        );

        let ranges = WeeklyChartListRequestBuilder::new(http, config, "testuser".to_string())
            .fetch()
            .await
            .unwrap();
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].from, 1_108_296_000);
        assert_eq!(ranges[0].to, 1_108_900_800);
    }

    #[tokio::test]
    async fn test_fetch_weekly_track_chart() {
        let (http, config) = http_config(
            "user.getweeklytrackchart",
            json!({
                "weeklytrackchart": {
                    "@attr": { "user": "testuser", "from": "1108296000", "to": "1108900800" },
                    "track": [
                        {
                            "name": "Test Track",
                            "mbid": "",
                            "url": "https://www.last.fm/music/Artist/_/Test+Track",
                            "playcount": "5",
                            "artist": { "#text": "Test Artist", "mbid": "" },
                            "@attr": { "rank": "1" }
                        }
                    ]
                }
            }),
        );

        let tracks = WeeklyTrackChartRequestBuilder::new(http, config, "testuser".to_string())
            .fetch()
            .await
            .unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Test Track");
        assert_eq!(tracks[0].playcount, 5);
        assert_eq!(tracks[0].artist_name, "Test Artist");
        assert_eq!(tracks[0].rank, 1);
    }

    #[tokio::test]
    async fn test_fetch_weekly_artist_chart() {
        let (http, config) = http_config(
            "user.getweeklyartistchart",
            json!({
                "weeklyartistchart": {
                    "@attr": { "user": "testuser", "from": "1108296000", "to": "1108900800" },
                    "artist": [
                        {
                            "name": "Test Artist",
                            "mbid": "",
                            "url": "https://www.last.fm/music/Test+Artist",
                            "playcount": "10",
                            "@attr": { "rank": "1" }
                        }
                    ]
                }
            }),
        );

        let artists = WeeklyArtistChartRequestBuilder::new(http, config, "testuser".to_string())
            .fetch()
            .await
            .unwrap();
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].name, "Test Artist");
        assert_eq!(artists[0].playcount, 10);
        assert_eq!(artists[0].rank, 1);
    }

    #[tokio::test]
    async fn test_fetch_weekly_album_chart() {
        let (http, config) = http_config(
            "user.getweeklyalbumchart",
            json!({
                "weeklyalbumchart": {
                    "@attr": { "user": "testuser", "from": "1108296000", "to": "1108900800" },
                    "album": [
                        {
                            "name": "Test Album",
                            "mbid": "",
                            "url": "https://www.last.fm/music/Artist/Test+Album",
                            "playcount": "8",
                            "artist": { "#text": "Test Artist", "mbid": "" },
                            "@attr": { "rank": "1" }
                        }
                    ]
                }
            }),
        );

        let albums = WeeklyAlbumChartRequestBuilder::new(http, config, "testuser".to_string())
            .fetch()
            .await
            .unwrap();
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].name, "Test Album");
        assert_eq!(albums[0].playcount, 8);
        assert_eq!(albums[0].artist_name, "Test Artist");
        assert_eq!(albums[0].rank, 1);
    }

    #[test]
    fn test_range_builder() {
        let range = WeeklyChartRange {
            from: 1_108_296_000,
            to: 1_108_900_800,
        };
        let config = Arc::new(ConfigBuilder::new().api_key("test_key").build().unwrap());
        let mock = Arc::new(MockClient::new());
        let builder =
            WeeklyTrackChartRequestBuilder::new(mock, config, "testuser".to_string()).range(&range);
        assert_eq!(builder.from, Some(1_108_296_000));
        assert_eq!(builder.to, Some(1_108_900_800));
    }
}
