//! Image Gallery — external storage refs, content-addressed dedup, set-algebra tag queries.
//!
//! The image bytes live OUTSIDE the entity. `Image` is a metadata envelope.
//! Tags form a many-to-many relation queried with set algebra. Albums are
//! curated ordered collections, distinct from tags (which are intrinsic).

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct BlobRef {
    pub path: String,
    pub size_bytes: u64,
    pub exists: bool,
}

impl BlobRef {
    pub fn new(path: &str, size_bytes: u64) -> Self {
        Self { path: path.to_string(), size_bytes, exists: true }
    }
}

#[derive(Debug, Clone)]
pub struct GalleryImage {
    pub id: String,
    pub filename: String,
    pub source: BlobRef,
    pub thumbnail: Option<BlobRef>,
    pub preview: Option<BlobRef>,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub content_hash: String,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_ids: Vec<String>,
    pub curator: String,
}

#[derive(Debug)]
pub enum GalleryError {
    UnknownImage(String),
    UnknownAlbum(String),
    Duplicate(String),
}

pub struct Gallery {
    pub images: HashMap<String, GalleryImage>,
    pub albums: HashMap<String, Album>,
}

impl Gallery {
    pub fn new() -> Self {
        Self { images: HashMap::new(), albums: HashMap::new() }
    }

    pub fn add_image(&mut self, image: GalleryImage) -> Result<(), GalleryError> {
        if self.images.contains_key(&image.id) {
            return Err(GalleryError::Duplicate(image.id));
        }
        self.images.insert(image.id.clone(), image);
        Ok(())
    }

    pub fn remove_image(&mut self, image_id: &str) -> Result<(), GalleryError> {
        if !self.images.contains_key(image_id) {
            return Err(GalleryError::UnknownImage(image_id.to_string()));
        }
        self.images.remove(image_id);
        for album in self.albums.values_mut() {
            album.image_ids.retain(|i| i != image_id);
        }
        Ok(())
    }

    pub fn tag(&mut self, image_id: &str, tags: &[&str]) -> Result<(), GalleryError> {
        let image = self
            .images
            .get_mut(image_id)
            .ok_or_else(|| GalleryError::UnknownImage(image_id.to_string()))?;
        for t in tags {
            image.tags.insert(t.to_string());
        }
        Ok(())
    }

    pub fn untag(&mut self, image_id: &str, tags: &[&str]) -> Result<(), GalleryError> {
        let image = self
            .images
            .get_mut(image_id)
            .ok_or_else(|| GalleryError::UnknownImage(image_id.to_string()))?;
        for t in tags {
            image.tags.remove(*t);
        }
        Ok(())
    }

    pub fn all_tags(&self) -> HashSet<String> {
        let mut out = HashSet::new();
        for image in self.images.values() {
            for t in &image.tags {
                out.insert(t.clone());
            }
        }
        out
    }

    pub fn tag_counts(&self) -> HashMap<String, u32> {
        let mut out: HashMap<String, u32> = HashMap::new();
        for image in self.images.values() {
            for t in &image.tags {
                *out.entry(t.clone()).or_insert(0) += 1;
            }
        }
        out
    }

    /// Intersection: images whose tag set is a superset of `tags`.
    pub fn with_all_tags(&self, tags: &HashSet<String>) -> Vec<&GalleryImage> {
        if tags.is_empty() {
            return self.images.values().collect();
        }
        self.images
            .values()
            .filter(|i| tags.iter().all(|t| i.tags.contains(t)))
            .collect()
    }

    /// Union: images with at least one of the given tags.
    pub fn with_any_tag(&self, tags: &HashSet<String>) -> Vec<&GalleryImage> {
        if tags.is_empty() {
            return Vec::new();
        }
        self.images
            .values()
            .filter(|i| i.tags.intersection(tags).next().is_some())
            .collect()
    }

    /// Difference: match `include` (intersection) AND NOT match any of `exclude`.
    pub fn without_tags(
        &self,
        include: &HashSet<String>,
        exclude: &HashSet<String>,
    ) -> Vec<&GalleryImage> {
        self.images
            .values()
            .filter(|i| {
                let has_all = include.iter().all(|t| i.tags.contains(t));
                let has_any_excluded = i.tags.intersection(exclude).next().is_some();
                has_all && !has_any_excluded
            })
            .collect()
    }

    /// Group images by content_hash. Only hashes with more than one image appear.
    pub fn duplicates(&self) -> HashMap<String, Vec<String>> {
        let mut buckets: HashMap<String, Vec<String>> = HashMap::new();
        for image in self.images.values() {
            if image.content_hash.is_empty() {
                continue;
            }
            buckets.entry(image.content_hash.clone()).or_default().push(image.id.clone());
        }
        buckets.into_iter().filter(|(_, v)| v.len() > 1).collect()
    }

    pub fn find_by_hash(&self, content_hash: &str) -> Vec<&GalleryImage> {
        self.images.values().filter(|i| i.content_hash == content_hash).collect()
    }

    pub fn create_album(&mut self, album: Album) -> Result<(), GalleryError> {
        if self.albums.contains_key(&album.id) {
            return Err(GalleryError::Duplicate(album.id));
        }
        for iid in &album.image_ids {
            if !self.images.contains_key(iid) {
                return Err(GalleryError::UnknownImage(iid.clone()));
            }
        }
        self.albums.insert(album.id.clone(), album);
        Ok(())
    }

    pub fn add_to_album(&mut self, album_id: &str, image_id: &str) -> Result<(), GalleryError> {
        if !self.images.contains_key(image_id) {
            return Err(GalleryError::UnknownImage(image_id.to_string()));
        }
        let album = self
            .albums
            .get_mut(album_id)
            .ok_or_else(|| GalleryError::UnknownAlbum(album_id.to_string()))?;
        if !album.image_ids.iter().any(|i| i == image_id) {
            album.image_ids.push(image_id.to_string());
        }
        Ok(())
    }

    pub fn remove_from_album(&mut self, album_id: &str, image_id: &str) -> Result<(), GalleryError> {
        let album = self
            .albums
            .get_mut(album_id)
            .ok_or_else(|| GalleryError::UnknownAlbum(album_id.to_string()))?;
        album.image_ids.retain(|i| i != image_id);
        Ok(())
    }

    pub fn album_contents(&self, album_id: &str) -> Result<Vec<&GalleryImage>, GalleryError> {
        let album = self
            .albums
            .get(album_id)
            .ok_or_else(|| GalleryError::UnknownAlbum(album_id.to_string()))?;
        Ok(album.image_ids.iter().filter_map(|i| self.images.get(i)).collect())
    }

    pub fn albums_containing(&self, image_id: &str) -> Vec<&Album> {
        self.albums
            .values()
            .filter(|a| a.image_ids.iter().any(|i| i == image_id))
            .collect()
    }

    pub fn total_bytes(&self) -> u64 {
        let mut total = 0u64;
        for image in self.images.values() {
            total += image.source.size_bytes;
            if let Some(ref t) = image.thumbnail { total += t.size_bytes; }
            if let Some(ref p) = image.preview   { total += p.size_bytes; }
        }
        total
    }
}

impl Default for Gallery {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(id: &str, hash: &str, tags: &[&str]) -> GalleryImage {
        GalleryImage {
            id: id.into(),
            filename: format!("{id}.jpg"),
            source: BlobRef::new(&format!("/p/{id}.jpg"), 1000),
            thumbnail: None, preview: None,
            width: 100, height: 100,
            mime_type: "image/jpeg".into(),
            content_hash: hash.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn set(xs: &[&str]) -> HashSet<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    fn setup() -> Gallery {
        let mut g = Gallery::new();
        g.add_image(img("A", "h1", &["sunset", "beach", "travel"])).unwrap();
        g.add_image(img("B", "h1", &["beach"])).unwrap();             // dup of A
        g.add_image(img("C", "h2", &["forest", "travel"])).unwrap();
        g.add_image(img("D", "h3", &["travel", "private"])).unwrap();
        g
    }

    #[test]
    fn with_all_tags_is_intersection() {
        let g = setup();
        let mut ids: Vec<&str> = g.with_all_tags(&set(&["travel", "beach"]))
            .iter().map(|i| i.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["A"]);
    }

    #[test]
    fn with_any_tag_is_union() {
        let g = setup();
        let mut ids: Vec<&str> = g.with_any_tag(&set(&["forest", "private"]))
            .iter().map(|i| i.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["C", "D"]);
    }

    #[test]
    fn without_tags_excludes_correctly() {
        let g = setup();
        let mut ids: Vec<&str> = g.without_tags(&set(&["travel"]), &set(&["private"]))
            .iter().map(|i| i.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["A", "C"]);
    }

    #[test]
    fn duplicates_group_by_content_hash() {
        let g = setup();
        let dups = g.duplicates();
        assert_eq!(dups.len(), 1);
        let mut ids = dups.get("h1").unwrap().clone();
        ids.sort();
        assert_eq!(ids, vec!["A", "B"]);
    }

    #[test]
    fn album_preserves_insertion_order() {
        let mut g = setup();
        g.create_album(Album {
            id: "alb".into(), name: "trip".into(), description: "".into(),
            image_ids: vec!["C".into(), "A".into()], curator: "me".into(),
        }).unwrap();
        let contents: Vec<&str> = g.album_contents("alb").unwrap()
            .iter().map(|i| i.id.as_str()).collect();
        assert_eq!(contents, vec!["C", "A"]); // curator order, not alphabetical
    }

    #[test]
    fn remove_image_orphans_album_references() {
        let mut g = setup();
        g.create_album(Album {
            id: "alb".into(), name: "x".into(), description: "".into(),
            image_ids: vec!["A".into(), "C".into()], curator: "".into(),
        }).unwrap();
        g.remove_image("A").unwrap();
        let contents: Vec<&str> = g.album_contents("alb").unwrap()
            .iter().map(|i| i.id.as_str()).collect();
        assert_eq!(contents, vec!["C"]);
    }
}
