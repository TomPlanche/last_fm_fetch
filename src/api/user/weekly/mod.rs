//! Weekly chart resource clients.

mod charts;

pub use charts::{
    WeeklyAlbumChartRequestBuilder, WeeklyArtistChartRequestBuilder, WeeklyChartListRequestBuilder,
    WeeklyTrackChartRequestBuilder,
};
