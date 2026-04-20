//! Mp3 Tagger — file metadata as a queryable, indexed tag store.
//!
//! Ninth Files project. G087 gave us files with paths; G092 renamed them
//! in bulk. G093 introduces **metadata separate from data**: every file
//! carries a `Tag` (title, artist, album, year, genre, track). The tag
//! store indexes them for fast queries — "all 2026 tracks", "artist =
//! Björk", "tracks with no album set".
//!
//! Two pieces the Rosetta Stone formalises:
//!   1. **Missing vs empty** — a genre not set is `None`; a genre set to
//!      empty string is `Some("")`. The query layer distinguishes them.
//!   2. **Index vs scan** — simple fields (artist, album, year) have
//!      explicit indexes so equality queries are O(1). Free-form fields
//!      (title) only support scan.
//!
//! The tag store is a pure data structure — no real MP3 parsing. A real
//! implementation would read ID3v2 frames and write them back; the
//! Rosetta Stone's job is the tag *model*, not the binary format.

use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tag {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u16>,
    pub genre: Option<String>,
    pub track: Option<u16>,
}

impl Tag {
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.artist.is_none() && self.album.is_none()
            && self.year.is_none() && self.genre.is_none() && self.track.is_none()
    }

    pub fn missing_fields(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.title.is_none()  { out.push("title");  }
        if self.artist.is_none() { out.push("artist"); }
        if self.album.is_none()  { out.push("album");  }
        if self.year.is_none()   { out.push("year");   }
        if self.genre.is_none()  { out.push("genre");  }
        if self.track.is_none()  { out.push("track");  }
        out
    }
}

#[derive(Debug, Clone)]
pub struct TagStore {
    tags: HashMap<String, Tag>,
    artist_idx: HashMap<String, Vec<String>>,  // artist -> paths
    album_idx:  HashMap<String, Vec<String>>,
    year_idx:   BTreeMap<u16, Vec<String>>,    // BTree for range queries
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    ByArtist(String),
    ByAlbum(String),
    ByYear(u16),
    YearRange { from: u16, to: u16 },  // inclusive
    TitleContains(String),             // scan; substring (case-insensitive)
    MissingField(&'static str),
}

impl TagStore {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            artist_idx: HashMap::new(),
            album_idx: HashMap::new(),
            year_idx: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, path: &str, tag: Tag) {
        // Remove from indexes if path already present.
        if self.tags.contains_key(path) {
            self.remove_from_indexes(path);
        }
        // Update indexes.
        if let Some(a) = &tag.artist {
            self.artist_idx.entry(a.clone()).or_default().push(path.to_string());
        }
        if let Some(a) = &tag.album {
            self.album_idx.entry(a.clone()).or_default().push(path.to_string());
        }
        if let Some(y) = tag.year {
            self.year_idx.entry(y).or_default().push(path.to_string());
        }
        self.tags.insert(path.to_string(), tag);
    }

    pub fn get(&self, path: &str) -> Option<&Tag> { self.tags.get(path) }

    pub fn len(&self) -> usize { self.tags.len() }

    pub fn remove(&mut self, path: &str) -> Option<Tag> {
        self.remove_from_indexes(path);
        self.tags.remove(path)
    }

    fn remove_from_indexes(&mut self, path: &str) {
        if let Some(tag) = self.tags.get(path) {
            if let Some(a) = &tag.artist {
                if let Some(v) = self.artist_idx.get_mut(a) {
                    v.retain(|p| p != path);
                }
            }
            if let Some(a) = &tag.album {
                if let Some(v) = self.album_idx.get_mut(a) {
                    v.retain(|p| p != path);
                }
            }
            if let Some(y) = tag.year {
                if let Some(v) = self.year_idx.get_mut(&y) {
                    v.retain(|p| p != path);
                }
            }
        }
    }

    pub fn query(&self, q: &Query) -> Vec<String> {
        let mut paths: Vec<String> = match q {
            Query::ByArtist(a) => self.artist_idx.get(a).cloned().unwrap_or_default(),
            Query::ByAlbum(a) => self.album_idx.get(a).cloned().unwrap_or_default(),
            Query::ByYear(y) => self.year_idx.get(y).cloned().unwrap_or_default(),
            Query::YearRange { from, to } => {
                let mut out = Vec::new();
                for (_, v) in self.year_idx.range(from..=to) {
                    out.extend(v.iter().cloned());
                }
                out
            }
            Query::TitleContains(needle) => {
                let lc = needle.to_lowercase();
                self.tags.iter()
                    .filter(|(_, t)| t.title.as_ref()
                        .map_or(false, |ti| ti.to_lowercase().contains(&lc)))
                    .map(|(k, _)| k.clone())
                    .collect()
            }
            Query::MissingField(field) => {
                self.tags.iter()
                    .filter(|(_, t)| field_is_missing(t, field))
                    .map(|(k, _)| k.clone())
                    .collect()
            }
        };
        paths.sort();
        paths
    }
}

impl Default for TagStore { fn default() -> Self { Self::new() } }

fn field_is_missing(t: &Tag, field: &str) -> bool {
    match field {
        "title" => t.title.is_none(),
        "artist" => t.artist.is_none(),
        "album" => t.album.is_none(),
        "year" => t.year.is_none(),
        "genre" => t.genre.is_none(),
        "track" => t.track.is_none(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TagStore {
        let mut s = TagStore::new();
        s.insert("a.mp3", Tag {
            title: Some("Aeon".into()),
            artist: Some("Björk".into()),
            album: Some("Vespertine".into()),
            year: Some(2001),
            genre: Some("Electronic".into()),
            track: Some(1),
        });
        s.insert("b.mp3", Tag {
            title: Some("Pagan Poetry".into()),
            artist: Some("Björk".into()),
            album: Some("Vespertine".into()),
            year: Some(2001),
            genre: Some("Electronic".into()),
            track: Some(3),
        });
        s.insert("c.mp3", Tag {
            title: Some("Unplayed".into()),
            artist: None,
            album: None,
            year: None,
            genre: None,
            track: None,
        });
        s.insert("d.mp3", Tag {
            title: Some("Hyperballad".into()),
            artist: Some("Björk".into()),
            album: Some("Post".into()),
            year: Some(1995),
            genre: None,
            track: Some(2),
        });
        s
    }

    #[test]
    fn empty_tag_is_empty() {
        assert!(Tag::default().is_empty());
    }

    #[test]
    fn missing_fields_enumerated() {
        let t = Tag { title: Some("x".into()), ..Default::default() };
        let missing = t.missing_fields();
        assert!(!missing.contains(&"title"));
        assert!(missing.contains(&"artist"));
        assert_eq!(missing.len(), 5);
    }

    #[test]
    fn insert_then_get() {
        let s = sample();
        assert!(s.get("a.mp3").is_some());
        assert_eq!(s.get("a.mp3").unwrap().artist, Some("Björk".into()));
    }

    #[test]
    fn query_by_artist_returns_matching_paths() {
        let s = sample();
        let paths = s.query(&Query::ByArtist("Björk".into()));
        assert_eq!(paths, vec!["a.mp3", "b.mp3", "d.mp3"]);
    }

    #[test]
    fn query_by_artist_missing_returns_empty() {
        let s = sample();
        let paths = s.query(&Query::ByArtist("Not Björk".into()));
        assert!(paths.is_empty());
    }

    #[test]
    fn query_by_year_exact() {
        let s = sample();
        let paths = s.query(&Query::ByYear(2001));
        assert_eq!(paths, vec!["a.mp3", "b.mp3"]);
    }

    #[test]
    fn query_by_year_range() {
        let s = sample();
        let paths = s.query(&Query::YearRange { from: 2000, to: 2010 });
        assert_eq!(paths, vec!["a.mp3", "b.mp3"]);
    }

    #[test]
    fn query_title_contains_is_case_insensitive() {
        let s = sample();
        let paths = s.query(&Query::TitleContains("aeon".into()));
        assert_eq!(paths, vec!["a.mp3"]);
        let upper = s.query(&Query::TitleContains("AEON".into()));
        assert_eq!(upper, vec!["a.mp3"]);
    }

    #[test]
    fn query_missing_field() {
        let s = sample();
        let missing_artist = s.query(&Query::MissingField("artist"));
        assert_eq!(missing_artist, vec!["c.mp3"]);
        let missing_genre = s.query(&Query::MissingField("genre"));
        // c.mp3 has no genre, d.mp3 also has no genre.
        assert_eq!(missing_genre, vec!["c.mp3", "d.mp3"]);
    }

    #[test]
    fn update_refreshes_indexes() {
        let mut s = sample();
        s.insert("a.mp3", Tag {
            title: Some("Aeon Remix".into()),
            artist: Some("Björk & Thom Yorke".into()),
            album: Some("Vespertine".into()),
            year: Some(2001),
            genre: None,
            track: Some(1),
        });
        // Old artist no longer matches a.mp3 by itself.
        assert!(!s.query(&Query::ByArtist("Björk".into())).contains(&"a.mp3".to_string()));
        assert!(s.query(&Query::ByArtist("Björk & Thom Yorke".into())).contains(&"a.mp3".to_string()));
    }

    #[test]
    fn remove_cleans_indexes() {
        let mut s = sample();
        s.remove("a.mp3");
        let björk = s.query(&Query::ByArtist("Björk".into()));
        assert!(!björk.contains(&"a.mp3".to_string()));
        assert!(björk.contains(&"b.mp3".to_string()));
    }

    #[test]
    fn query_results_sorted() {
        let s = sample();
        let paths = s.query(&Query::ByArtist("Björk".into()));
        for window in paths.windows(2) {
            assert!(window[0] < window[1]);
        }
    }

    #[test]
    fn empty_query_on_empty_store() {
        let s = TagStore::new();
        assert!(s.query(&Query::ByArtist("x".into())).is_empty());
        assert!(s.query(&Query::MissingField("title")).is_empty());
    }
}
