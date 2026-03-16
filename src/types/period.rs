/// Period options for Last.fm time range filters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Period {
    /// All-time (no time limit)
    Overall,
    /// Last 7 days
    Week,
    /// Last 30 days
    Month,
    /// Last 3 months
    ThreeMonth,
    /// Last 6 months
    SixMonth,
    /// Last 12 months
    TwelveMonth,
}

impl Period {}

/// Track limit options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrackLimit {
    /// Fetch up to the specified number of items
    Limited(u32),
    /// Fetch all available items
    Unlimited,
}

impl From<Option<u32>> for TrackLimit {
    fn from(opt: Option<u32>) -> Self {
        opt.map_or(Self::Unlimited, Self::Limited)
    }
}

impl From<u32> for TrackLimit {
    fn from(limit: u32) -> Self {
        Self::Limited(limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_limit_from_option() {
        let limited: TrackLimit = Some(100).into();
        assert_eq!(limited, TrackLimit::Limited(100));

        let unlimited: TrackLimit = None.into();
        assert_eq!(unlimited, TrackLimit::Unlimited);
    }

    #[test]
    fn test_track_limit_from_u32() {
        let limited: TrackLimit = 50.into();
        assert_eq!(limited, TrackLimit::Limited(50));
    }
}
