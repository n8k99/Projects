//! Bookmark Collector and Sorter — multi-axis organisation over a URL keyed collection.
//!
//! Two orthogonal ways to organise the same bookmarks: **folders** (hierarchy,
//! one folder per bookmark) and **tags** (flat set, many-to-many). Real users
//! use both, and pretending one is primary breaks the other. G076 treats them
//! as equal citizens — a bookmark has a folder AND a tag set; queries by
//! folder and queries by tag both walk the same underlying list.
//!
//! URL is the natural key — adding a bookmark with an existing URL updates
//! the existing entry rather than creating a duplicate. Sort order is never
//! stored; every sort is a **view** computed on query. The stored list has
//! one natural order (insertion), every other order is derived.

use std::collections::BTreeSet;

pub type BookmarkId = u64;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub url: String,
    pub title: String,
    pub tags: BTreeSet<String>,
    pub folder: String,        // e.g. "/personal/news"
    pub added_at_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum SortBy {
    Title,
    Url,
    AddedAt,
    Folder,
}

pub struct BookmarkCollection {
    bookmarks: Vec<Bookmark>,
    next_id: BookmarkId,
}

impl BookmarkCollection {
    pub fn new() -> Self {
        Self { bookmarks: Vec::new(), next_id: 1 }
    }

    /// Add a bookmark. If the URL already exists, update its title+folder+tags
    /// and keep the existing id (URL is the natural key).
    pub fn add(&mut self, url: &str, title: &str, folder: &str,
               tags: BTreeSet<String>, added_at_ms: u64) -> BookmarkId {
        if let Some(existing) = self.bookmarks.iter_mut().find(|b| b.url == url) {
            existing.title = title.into();
            existing.folder = folder.into();
            existing.tags = tags;
            return existing.id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.bookmarks.push(Bookmark {
            id, url: url.into(), title: title.into(),
            tags, folder: folder.into(), added_at_ms,
        });
        id
    }

    pub fn remove(&mut self, id: BookmarkId) -> bool {
        match self.bookmarks.iter().position(|b| b.id == id) {
            Some(pos) => { self.bookmarks.remove(pos); true }
            None => false,
        }
    }

    pub fn count(&self) -> usize { self.bookmarks.len() }

    pub fn get(&self, id: BookmarkId) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.id == id)
    }

    pub fn set_tags(&mut self, id: BookmarkId, tags: BTreeSet<String>) -> bool {
        if let Some(b) = self.bookmarks.iter_mut().find(|b| b.id == id) {
            b.tags = tags; true
        } else { false }
    }

    pub fn move_to_folder(&mut self, id: BookmarkId, folder: &str) -> bool {
        if let Some(b) = self.bookmarks.iter_mut().find(|b| b.id == id) {
            b.folder = folder.into(); true
        } else { false }
    }

    /// All bookmarks, in insertion order.
    pub fn all(&self) -> &[Bookmark] { &self.bookmarks }

    /// Case-insensitive substring search across title, url, and tags.
    pub fn search(&self, query: &str) -> Vec<&Bookmark> {
        let q = query.to_lowercase();
        self.bookmarks.iter().filter(|b| {
            b.title.to_lowercase().contains(&q)
                || b.url.to_lowercase().contains(&q)
                || b.tags.iter().any(|t| t.to_lowercase().contains(&q))
        }).collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&Bookmark> {
        self.bookmarks.iter().filter(|b| b.tags.contains(tag)).collect()
    }

    /// Bookmarks in `folder`. If `recursive`, also matches any subfolder
    /// (prefix match on "/" boundaries).
    pub fn by_folder(&self, folder: &str, recursive: bool) -> Vec<&Bookmark> {
        self.bookmarks.iter().filter(|b| {
            if recursive {
                b.folder == folder
                    || b.folder.starts_with(&format!("{folder}/"))
            } else {
                b.folder == folder
            }
        }).collect()
    }

    /// Return a view over the collection sorted by `criterion`. Stable sort —
    /// bookmarks with equal keys stay in insertion order.
    pub fn sort_by(&self, criterion: SortBy) -> Vec<&Bookmark> {
        let mut view: Vec<&Bookmark> = self.bookmarks.iter().collect();
        match criterion {
            SortBy::Title => view.sort_by(|a, b| a.title.cmp(&b.title)),
            SortBy::Url => view.sort_by(|a, b| a.url.cmp(&b.url)),
            SortBy::AddedAt => view.sort_by(|a, b| a.added_at_ms.cmp(&b.added_at_ms)),
            SortBy::Folder => view.sort_by(|a, b| a.folder.cmp(&b.folder)),
        }
        view
    }

    /// All tags across all bookmarks, sorted alphabetically.
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: BTreeSet<String> = BTreeSet::new();
        for b in &self.bookmarks {
            for t in &b.tags { tags.insert(t.clone()); }
        }
        tags.into_iter().collect()
    }

    /// All folder paths that have at least one bookmark.
    pub fn all_folders(&self) -> Vec<String> {
        let mut folders: BTreeSet<String> = BTreeSet::new();
        for b in &self.bookmarks { folders.insert(b.folder.clone()); }
        folders.into_iter().collect()
    }
}

impl Default for BookmarkCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn add_assigns_ids_and_counts() {
        let mut c = BookmarkCollection::new();
        c.add("https://a", "A", "/", tags(&[]), 1000);
        c.add("https://b", "B", "/", tags(&[]), 2000);
        assert_eq!(c.count(), 2);
    }

    #[test]
    fn add_same_url_updates_existing() {
        let mut c = BookmarkCollection::new();
        let id = c.add("https://x", "Old Title", "/old", tags(&["a"]), 1000);
        let id2 = c.add("https://x", "New Title", "/new", tags(&["b"]), 1500);
        assert_eq!(id, id2);
        assert_eq!(c.count(), 1);
        let bm = c.get(id).unwrap();
        assert_eq!(bm.title, "New Title");
        assert_eq!(bm.folder, "/new");
        assert!(bm.tags.contains("b"));
        assert!(!bm.tags.contains("a"));
    }

    #[test]
    fn search_matches_title_url_or_tags() {
        let mut c = BookmarkCollection::new();
        c.add("https://rust-lang.org", "Rust Language", "/dev",
              tags(&["language", "systems"]), 0);
        c.add("https://python.org", "Python",  "/dev", tags(&["language"]), 1);
        c.add("https://kernel.org", "Linux", "/dev", tags(&["systems"]), 2);
        assert_eq!(c.search("rust").len(), 1);      // title+url+maybe tags
        assert_eq!(c.search("language").len(), 2);  // matches two via tags/title
        assert_eq!(c.search("systems").len(), 2);
    }

    #[test]
    fn by_tag_filters() {
        let mut c = BookmarkCollection::new();
        c.add("https://a", "A", "/", tags(&["x", "y"]), 0);
        c.add("https://b", "B", "/", tags(&["y", "z"]), 0);
        c.add("https://c", "C", "/", tags(&["z"]), 0);
        assert_eq!(c.by_tag("y").len(), 2);
        assert_eq!(c.by_tag("z").len(), 2);
        assert_eq!(c.by_tag("x").len(), 1);
        assert_eq!(c.by_tag("nope").len(), 0);
    }

    #[test]
    fn by_folder_non_recursive() {
        let mut c = BookmarkCollection::new();
        c.add("https://a", "A", "/news", tags(&[]), 0);
        c.add("https://b", "B", "/news/tech", tags(&[]), 0);
        c.add("https://d", "D", "/other", tags(&[]), 0);
        assert_eq!(c.by_folder("/news", false).len(), 1);       // only exact match
        assert_eq!(c.by_folder("/news", true).len(), 2);        // includes /news/tech
        assert_eq!(c.by_folder("/news/tech", true).len(), 1);
    }

    #[test]
    fn sort_by_title_is_stable_and_alphabetical() {
        let mut c = BookmarkCollection::new();
        c.add("https://c", "Charlie", "/", tags(&[]), 0);
        c.add("https://a", "Alpha",   "/", tags(&[]), 0);
        c.add("https://b", "Bravo",   "/", tags(&[]), 0);
        let view = c.sort_by(SortBy::Title);
        let titles: Vec<&str> = view.iter().map(|b| b.title.as_str()).collect();
        assert_eq!(titles, vec!["Alpha", "Bravo", "Charlie"]);
    }

    #[test]
    fn sort_by_added_at_is_chronological() {
        let mut c = BookmarkCollection::new();
        c.add("https://late",  "Late",  "/", tags(&[]), 3000);
        c.add("https://early", "Early", "/", tags(&[]), 1000);
        c.add("https://mid",   "Mid",   "/", tags(&[]), 2000);
        let view = c.sort_by(SortBy::AddedAt);
        let titles: Vec<&str> = view.iter().map(|b| b.title.as_str()).collect();
        assert_eq!(titles, vec!["Early", "Mid", "Late"]);
    }

    #[test]
    fn all_tags_are_sorted_and_deduplicated() {
        let mut c = BookmarkCollection::new();
        c.add("https://a", "A", "/", tags(&["b", "a"]), 0);
        c.add("https://c", "C", "/", tags(&["a", "c"]), 0);
        assert_eq!(c.all_tags(), vec!["a", "b", "c"]);
    }

    #[test]
    fn set_tags_replaces_entire_set() {
        let mut c = BookmarkCollection::new();
        let id = c.add("https://x", "X", "/", tags(&["old1", "old2"]), 0);
        c.set_tags(id, tags(&["new"]));
        let bm = c.get(id).unwrap();
        assert_eq!(bm.tags.len(), 1);
        assert!(bm.tags.contains("new"));
    }

    #[test]
    fn move_to_folder_changes_folder() {
        let mut c = BookmarkCollection::new();
        let id = c.add("https://x", "X", "/a", tags(&[]), 0);
        c.move_to_folder(id, "/b");
        assert_eq!(c.get(id).unwrap().folder, "/b");
    }

    #[test]
    fn remove_removes() {
        let mut c = BookmarkCollection::new();
        let id = c.add("https://x", "X", "/", tags(&[]), 0);
        assert!(c.remove(id));
        assert_eq!(c.count(), 0);
        assert!(!c.remove(id));
    }
}
