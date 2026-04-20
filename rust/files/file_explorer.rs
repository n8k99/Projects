//! File Explorer — navigable virtual filesystem with path resolution and listing.
//!
//! Third Files project. The fundamental file-manager operation is **path
//! resolution**: given a current-directory and a user-typed path (absolute,
//! relative, with `.` and `..`), compute the canonical target path. Every
//! `cd`, `ls`, `cat` depends on it. Get it wrong and the user ends up
//! somewhere unexpected or escapes the root.
//!
//! The model here is a **virtual filesystem** — no real disk I/O, just an
//! in-memory tree. This keeps the Rosetta Stone module pure (same inputs,
//! same outputs across languages) while exercising the hard part: path
//! normalisation, traversal, and listing with metadata.
//!
//! Key invariants:
//!   * Paths canonicalise: `/a/b/../c` ⇒ `/a/c`; no `.` or `..` survive.
//!   * Root escape is impossible: `cd ../../..` from `/a/b` lands at `/`.
//!   * Listings return entries in a stable order (name ascending, dirs first).

use std::collections::BTreeMap;

pub type Size = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    File { size: Size, mtime_ms: u64 },
    Dir { children: BTreeMap<String, Node> },
}

impl Node {
    pub fn is_dir(&self) -> bool { matches!(self, Node::Dir { .. }) }
    pub fn is_file(&self) -> bool { matches!(self, Node::File { .. }) }
}

#[derive(Debug, Clone)]
pub struct Explorer {
    root: Node,
    cwd: Vec<String>,  // empty = root
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub name: String,
    pub kind: EntryKind,
    pub size: Size,   // 0 for dirs
    pub mtime_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind { File, Dir }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsError {
    NotFound(String),
    NotADir(String),
    NotAFile(String),
    AlreadyExists(String),
}

impl Explorer {
    pub fn new() -> Self {
        Self { root: Node::Dir { children: BTreeMap::new() }, cwd: Vec::new() }
    }

    pub fn cwd(&self) -> String { "/".to_string() + &self.cwd.join("/") }

    /// Resolve a path (absolute or relative) into a canonical segment list.
    /// Returns `None` for malformed input (empty path).
    pub fn resolve(&self, path: &str) -> Option<Vec<String>> {
        if path.is_empty() { return None; }
        let mut out: Vec<String> = if path.starts_with('/') {
            Vec::new()
        } else {
            self.cwd.clone()
        };
        for seg in path.split('/') {
            if seg.is_empty() || seg == "." { continue; }
            if seg == ".." { out.pop(); continue; }
            out.push(seg.to_string());
        }
        Some(out)
    }

    fn node_at<'a>(&'a self, segments: &[String]) -> Option<&'a Node> {
        let mut cur = &self.root;
        for seg in segments {
            match cur {
                Node::Dir { children } => {
                    cur = children.get(seg)?;
                }
                _ => return None,
            }
        }
        Some(cur)
    }

    fn node_at_mut<'a>(&'a mut self, segments: &[String]) -> Option<&'a mut Node> {
        let mut cur = &mut self.root;
        for seg in segments {
            match cur {
                Node::Dir { children } => {
                    cur = children.get_mut(seg)?;
                }
                _ => return None,
            }
        }
        Some(cur)
    }

    pub fn cd(&mut self, path: &str) -> Result<(), FsError> {
        let segs = self.resolve(path).ok_or_else(||
            FsError::NotFound(path.into()))?;
        match self.node_at(&segs) {
            None => Err(FsError::NotFound(path.into())),
            Some(Node::Dir { .. }) => { self.cwd = segs; Ok(()) }
            Some(_) => Err(FsError::NotADir(path.into())),
        }
    }

    pub fn ls(&self, path: &str) -> Result<Vec<Listing>, FsError> {
        let segs = if path.is_empty() { self.cwd.clone() }
            else { self.resolve(path).ok_or_else(|| FsError::NotFound(path.into()))? };
        let node = self.node_at(&segs).ok_or_else(||
            FsError::NotFound(path.into()))?;
        let children = match node {
            Node::Dir { children } => children,
            _ => return Err(FsError::NotADir(path.into())),
        };
        let mut out: Vec<Listing> = children.iter().map(|(name, child)| {
            match child {
                Node::Dir { .. } => Listing {
                    name: name.clone(), kind: EntryKind::Dir,
                    size: 0, mtime_ms: 0,
                },
                Node::File { size, mtime_ms } => Listing {
                    name: name.clone(), kind: EntryKind::File,
                    size: *size, mtime_ms: *mtime_ms,
                },
            }
        }).collect();
        // Dirs first, then alphabetical — stable presentation.
        out.sort_by(|a, b| {
            match (a.kind, b.kind) {
                (EntryKind::Dir, EntryKind::File) => std::cmp::Ordering::Less,
                (EntryKind::File, EntryKind::Dir) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        Ok(out)
    }

    pub fn mkdir(&mut self, path: &str) -> Result<(), FsError> {
        let segs = self.resolve(path).ok_or_else(||
            FsError::NotFound(path.into()))?;
        if segs.is_empty() {
            return Err(FsError::AlreadyExists("/".into()));
        }
        let (name, parent_segs) = segs.split_last().unwrap();
        let parent = self.node_at_mut(parent_segs).ok_or_else(||
            FsError::NotFound(path.into()))?;
        match parent {
            Node::Dir { children } => {
                if children.contains_key(name) {
                    return Err(FsError::AlreadyExists(path.into()));
                }
                children.insert(name.clone(), Node::Dir { children: BTreeMap::new() });
                Ok(())
            }
            _ => Err(FsError::NotADir(path.into())),
        }
    }

    pub fn touch(&mut self, path: &str, size: Size, mtime_ms: u64) -> Result<(), FsError> {
        let segs = self.resolve(path).ok_or_else(||
            FsError::NotFound(path.into()))?;
        if segs.is_empty() {
            return Err(FsError::AlreadyExists("/".into()));
        }
        let (name, parent_segs) = segs.split_last().unwrap();
        let parent = self.node_at_mut(parent_segs).ok_or_else(||
            FsError::NotFound(path.into()))?;
        match parent {
            Node::Dir { children } => {
                children.insert(name.clone(), Node::File { size, mtime_ms });
                Ok(())
            }
            _ => Err(FsError::NotADir(path.into())),
        }
    }

    pub fn stat(&self, path: &str) -> Result<Listing, FsError> {
        let segs = self.resolve(path).ok_or_else(||
            FsError::NotFound(path.into()))?;
        let name = segs.last().cloned().unwrap_or_else(|| "/".into());
        let node = self.node_at(&segs).ok_or_else(||
            FsError::NotFound(path.into()))?;
        match node {
            Node::Dir { .. } => Ok(Listing { name, kind: EntryKind::Dir, size: 0, mtime_ms: 0 }),
            Node::File { size, mtime_ms } => Ok(Listing {
                name, kind: EntryKind::File, size: *size, mtime_ms: *mtime_ms,
            }),
        }
    }

    /// Recursive total size of a path (dir walks all descendants).
    pub fn total_size(&self, path: &str) -> Result<Size, FsError> {
        let segs = self.resolve(path).ok_or_else(||
            FsError::NotFound(path.into()))?;
        let node = self.node_at(&segs).ok_or_else(||
            FsError::NotFound(path.into()))?;
        Ok(size_of(node))
    }
}

fn size_of(node: &Node) -> Size {
    match node {
        Node::File { size, .. } => *size,
        Node::Dir { children } => children.values().map(size_of).sum(),
    }
}

impl Default for Explorer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Explorer {
        let mut e = Explorer::new();
        e.mkdir("/home").unwrap();
        e.mkdir("/home/nathan").unwrap();
        e.mkdir("/home/nathan/docs").unwrap();
        e.touch("/home/nathan/docs/a.txt", 100, 1000).unwrap();
        e.touch("/home/nathan/docs/b.txt", 200, 2000).unwrap();
        e.touch("/home/nathan/notes.md", 50, 500).unwrap();
        e.mkdir("/tmp").unwrap();
        e
    }

    #[test]
    fn cd_absolute_and_relative() {
        let mut e = sample();
        e.cd("/home/nathan").unwrap();
        assert_eq!(e.cwd(), "/home/nathan");
        e.cd("docs").unwrap();
        assert_eq!(e.cwd(), "/home/nathan/docs");
    }

    #[test]
    fn dotdot_collapses() {
        let mut e = sample();
        e.cd("/home/nathan/docs").unwrap();
        e.cd("../..").unwrap();
        assert_eq!(e.cwd(), "/home");
    }

    #[test]
    fn dotdot_at_root_stays_at_root() {
        let mut e = sample();
        e.cd("/../../..").unwrap();
        assert_eq!(e.cwd(), "/");
    }

    #[test]
    fn cd_to_file_is_not_a_dir() {
        let mut e = sample();
        assert_eq!(e.cd("/home/nathan/notes.md"),
                   Err(FsError::NotADir("/home/nathan/notes.md".into())));
    }

    #[test]
    fn cd_to_missing_is_not_found() {
        let mut e = sample();
        assert_eq!(e.cd("/nowhere"),
                   Err(FsError::NotFound("/nowhere".into())));
    }

    #[test]
    fn ls_dirs_before_files_alphabetical() {
        let e = sample();
        let entries = e.ls("/home/nathan").unwrap();
        assert_eq!(entries[0].name, "docs");       // dir first
        assert_eq!(entries[0].kind, EntryKind::Dir);
        assert_eq!(entries[1].name, "notes.md");   // file after
        assert_eq!(entries[1].kind, EntryKind::File);
    }

    #[test]
    fn ls_empty_dir() {
        let e = sample();
        let entries = e.ls("/tmp").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn mkdir_already_exists() {
        let mut e = sample();
        assert_eq!(e.mkdir("/home"), Err(FsError::AlreadyExists("/home".into())));
    }

    #[test]
    fn total_size_recursive() {
        let e = sample();
        assert_eq!(e.total_size("/home/nathan/docs").unwrap(), 300);
        assert_eq!(e.total_size("/home/nathan").unwrap(), 350);
        assert_eq!(e.total_size("/home").unwrap(), 350);
    }

    #[test]
    fn total_size_of_file_is_its_size() {
        let e = sample();
        assert_eq!(e.total_size("/home/nathan/notes.md").unwrap(), 50);
    }

    #[test]
    fn stat_returns_metadata() {
        let e = sample();
        let s = e.stat("/home/nathan/docs/b.txt").unwrap();
        assert_eq!(s.kind, EntryKind::File);
        assert_eq!(s.size, 200);
        assert_eq!(s.mtime_ms, 2000);
    }

    #[test]
    fn resolve_is_canonical() {
        let e = sample();
        assert_eq!(e.resolve("/a/./b/../c"), Some(vec!["a".into(), "c".into()]));
        assert_eq!(e.resolve("/"), Some(vec![]));
    }

    #[test]
    fn relative_resolve_from_cwd() {
        let mut e = sample();
        e.cd("/home/nathan").unwrap();
        assert_eq!(e.resolve("docs/a.txt"),
                   Some(vec!["home".into(), "nathan".into(), "docs".into(), "a.txt".into()]));
    }
}
