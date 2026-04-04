/// Base URL for the Last.fm API
pub const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

/// Last.fm API method for fetching a user's recent tracks
pub const METHOD_RECENT_TRACKS: &str = "user.getrecenttracks";

/// Last.fm API method for fetching a user's loved tracks
pub const METHOD_LOVED_TRACKS: &str = "user.getlovedtracks";

/// Last.fm API method for fetching a user's top tracks
pub const METHOD_TOP_TRACKS: &str = "user.gettoptracks";

/// Last.fm API method for fetching a user's top artists
pub const METHOD_TOP_ARTISTS: &str = "user.gettopartists";

/// Last.fm API method for fetching a user's top albums
pub const METHOD_TOP_ALBUMS: &str = "user.gettopalbums";

/// Last.fm API method for fetching user info
pub const METHOD_USER_INFO: &str = "user.getinfo";

/// Last.fm API method for fetching a user's top tags
pub const METHOD_TOP_TAGS: &str = "user.gettoptags";

/// Last.fm API method for fetching a user's weekly chart list
pub const METHOD_WEEKLY_CHART_LIST: &str = "user.getweeklychartlist";

/// Last.fm API method for fetching a user's weekly track chart
pub const METHOD_WEEKLY_TRACK_CHART: &str = "user.getweeklytrackchart";

/// Last.fm API method for fetching a user's weekly artist chart
pub const METHOD_WEEKLY_ARTIST_CHART: &str = "user.getweeklyartistchart";

/// Last.fm API method for fetching a user's weekly album chart
pub const METHOD_WEEKLY_ALBUM_CHART: &str = "user.getweeklyalbumchart";

/// Last.fm API method for fetching a user's friends
pub const METHOD_FRIENDS: &str = "user.getfriends";

/// Last.fm API method for fetching a user's personal tags
pub const METHOD_PERSONAL_TAGS: &str = "user.getpersonaltags";

/// Maximum number of tracks that can be fetched in a single API request
pub const API_MAX_LIMIT: u32 = 1000;

/// Multiplier for chunking large requests
pub const CHUNK_MULTIPLIER: u32 = 5;

/// Size of each chunk for pagination
pub const CHUNK_SIZE: u32 = API_MAX_LIMIT * CHUNK_MULTIPLIER;
