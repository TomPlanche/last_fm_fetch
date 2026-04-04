//! Last.fm `user.*` API namespace.

mod friends;
mod info;
mod loved_tracks;
mod personal_tags;
mod recent_tracks;
mod top;
mod weekly;

pub use friends::FriendsRequestBuilder;
pub use info::UserInfoRequestBuilder;
pub use loved_tracks::LovedTracksRequestBuilder;
pub use personal_tags::PersonalTagsRequestBuilder;
pub use recent_tracks::RecentTracksRequestBuilder;
pub use top::{
    TopAlbumsRequestBuilder, TopArtistsRequestBuilder, TopTagsRequestBuilder,
    TopTracksRequestBuilder,
};
pub use weekly::{
    WeeklyAlbumChartRequestBuilder, WeeklyArtistChartRequestBuilder, WeeklyChartListRequestBuilder,
    WeeklyTrackChartRequestBuilder,
};
