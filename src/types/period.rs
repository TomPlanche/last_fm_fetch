/// Period options for Last.fm time range filters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Overall,
    Week,
    Month,
    ThreeMonth,
    SixMonth,
    TwelveMonth,
}

impl Period {}

/// Track limit options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackLimit {
    Limited(u32),
    Unlimited,
}

impl From<Option<u32>> for TrackLimit {
    fn from(opt: Option<u32>) -> Self {
        match opt {
            Some(limit) => TrackLimit::Limited(limit),
            None => TrackLimit::Unlimited,
        }
    }
}

impl From<u32> for TrackLimit {
    fn from(limit: u32) -> Self {
        TrackLimit::Limited(limit)
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
