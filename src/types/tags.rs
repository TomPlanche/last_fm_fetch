use serde::Deserialize;

// ── Internal deserialization helpers ─────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub(crate) struct TopTagsResponse {
    toptags: TopTagsRaw,
}

#[derive(Deserialize, Debug)]
struct TopTagsRaw {
    tag: Vec<UserTopTagRaw>,
}

#[derive(Deserialize, Debug)]
struct UserTopTagRaw {
    name: String,
    #[serde(deserialize_with = "crate::types::utils::u32_from_str")]
    count: u32,
    url: String,
}

// ── Public types ──────────────────────────────────────────────────────────────

/// A single tag from a user's top tags list.
///
/// Returned by [`user.getTopTags`](https://www.last.fm/api/show/user.getTopTags).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct UserTopTag {
    /// Tag name (e.g. `"rock"`)
    pub name: String,
    /// Number of times this tag has been applied by the user
    pub count: u32,
    /// Last.fm URL for this tag
    pub url: String,
}

impl From<UserTopTagRaw> for UserTopTag {
    fn from(r: UserTopTagRaw) -> Self {
        Self {
            name: r.name,
            count: r.count,
            url: r.url,
        }
    }
}

impl From<TopTagsResponse> for Vec<UserTopTag> {
    fn from(r: TopTagsResponse) -> Self {
        r.toptags.tag.into_iter().map(UserTopTag::from).collect()
    }
}
