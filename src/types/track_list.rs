use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::Serialize;

/// A list of Last.fm items (tracks, artists, albums) with display support.
///
/// `TrackList<T>` is a thin newtype around `Vec<T>` that adds a `Display`
/// implementation. When formatted, items are printed in descending order —
/// most recent first for time-stamped types, most-played first for playcount
/// types — relying on each element's `Ord` impl.
///
/// Because it derefs to `Vec<T>`, all slice and vector methods are available
/// transparently.
#[derive(Debug, Clone, Serialize)]
pub struct TrackList<T>(Vec<T>);

impl<T: Ord + fmt::Display> fmt::Display for TrackList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Sort by index to avoid cloning T; reverse order = highest first.
        let mut indices: Vec<usize> = (0..self.0.len()).collect();
        indices.sort_unstable_by(|&a, &b| self.0[b].cmp(&self.0[a]));
        for (i, idx) in indices.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", self.0[*idx])?;
        }
        Ok(())
    }
}

impl<T> Deref for TrackList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for TrackList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Vec<T>> for TrackList<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<T> From<TrackList<T>> for Vec<T> {
    fn from(list: TrackList<T>) -> Self {
        list.0
    }
}

impl<T> IntoIterator for TrackList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a TrackList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut TrackList<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T> FromIterator<T> for TrackList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
