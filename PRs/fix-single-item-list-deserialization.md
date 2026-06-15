# Fix: handle single-object list responses from Last.fm

## Summary

Fixes an intermittent deserialization failure that surfaced as:

```
Failed to update scrobbles db: Parse(Error("invalid type: map, expected a sequence", line: 0, column: 0))
```

This affected `fetch_extended_and_update_sqlite` (reported against a freshly created database), but the root cause impacts every recent, loved, and top resource fetch.

## Root cause

`fetch()` issues an initial probe request with `limit=1` to discover the total item count before paginating (`src/api/fetch_utils.rs`). Last.fm has a long-standing JSON quirk: when a result list contains exactly one item, the API returns that item as a bare object (a map) instead of a one-element array.

With `limit=1` and nothing currently playing, the `track` field of the response comes back as an object, which cannot deserialize into `Vec<RecentTrackExtended>`, producing `invalid type: map, expected a sequence`.

When a track is currently playing, the same `limit=1` probe returns two items (the now-playing track plus one historical track), so the field is an array and parsing succeeds. This is why the failure was intermittent.

The fresh-database path made it more visible: with no prior timestamp to use as a `since` filter, the update always exercises the probe path.

## Changes

- Add a `vec_or_single` custom deserializer in `src/types/utils.rs`. It accepts either a JSON array or a single object and always yields a `Vec`.
- Apply `#[serde(deserialize_with = "vec_or_single")]` to the list fields of all six response wrappers, since they share the same Last.fm quirk:
  - `RecentTracks.track`, `RecentTracksExtended.track`, `LovedTracks.track`, `TopTracks.track` (`src/types/tracks.rs`)
  - `TopArtists.artist` (`src/types/artists.rs`)
  - `TopAlbums.album` (`src/types/albums.rs`)
- Add regression tests for the deserializer covering the array, single-object, and empty-array cases.

## Testing

- `cargo test`: 124 passed, 12 ignored.
- New `vec_or_single` tests pass.
- No new clippy findings introduced (pre-existing lints in untouched files remain).

## Notes

The SQLite update path logs `Failed to update scrobbles db: {err}` and continues rather than aborting, which is why the error appeared repeatedly instead of crashing. The underlying parse error is now resolved.
