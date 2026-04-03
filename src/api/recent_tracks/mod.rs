//! Recent tracks API: client, request builder, and response types.

mod builder;
mod client;
mod extended;

pub use builder::RecentTracksRequestBuilder;
pub use client::RecentTracksClient;

use crate::api::fetch_utils::ResourceContainer;
use crate::types::{RecentTrack, RecentTrackExtended, UserRecentTracks, UserRecentTracksExtended};

impl ResourceContainer for UserRecentTracks {
    type ItemType = RecentTrack;

    fn total(&self) -> u32 {
        self.recenttracks.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.recenttracks.track
    }
}

impl ResourceContainer for UserRecentTracksExtended {
    type ItemType = RecentTrackExtended;

    fn total(&self) -> u32 {
        self.recenttracks.attr.total
    }

    fn items(self) -> Vec<Self::ItemType> {
        self.recenttracks.track
    }
}
