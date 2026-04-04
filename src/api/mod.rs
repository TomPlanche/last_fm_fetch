/// Extension traits shared across all API request builders
pub mod builder_ext;
/// API constants (base URL, limits, chunk sizes)
pub mod constants;
mod fetch_utils;
#[cfg(feature = "progress")]
mod progress;
/// Last.fm `user.*` API namespace
pub mod user;

pub use builder_ext::{Analyze, FetchAndSave, FetchAndUpdate, LimitBuilder};
pub(crate) use fetch_utils::user_params;
pub use fetch_utils::{Period, ProgressCallback, ResourceContainer};
pub use user::{FriendsRequestBuilder, LovedTracksRequestBuilder, PersonalTagsRequestBuilder};
pub use user::{RecentTracksRequestBuilder, TopAlbumsRequestBuilder, TopArtistsRequestBuilder};
pub use user::{TopTagsRequestBuilder, TopTracksRequestBuilder, UserInfoRequestBuilder};
pub use user::{
    WeeklyAlbumChartRequestBuilder, WeeklyArtistChartRequestBuilder, WeeklyChartListRequestBuilder,
    WeeklyTrackChartRequestBuilder,
};
