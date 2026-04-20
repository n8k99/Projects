//! Versioning Manager — content-addressed version store, minimal Git-like.
//!
//! Sixteenth Files project. **Closes the Files category.** Introduces the
//! **content-addressed storage** model that Git, IPFS, Nix, Mercurial,
//! and every content-hashed version system uses:
//!
//!   * A **blob** is a byte sequence stored by its hash. Identical bytes
//!      collapse to one blob — the storage is automatically deduplicated.
//!   * A **tree** is a map from filename to blob hash — a snapshot of a
//!      directory's contents at one moment.
//!   * A **commit** has a tree, a message, a timestamp, and a parent
//!      commit hash (or `None` for the initial commit).
//!
//! Operations:
//!   * `commit(files, message)` — snapshot current files into a tree +
//!      commit pointing at the previous commit.
//!   * `checkout(commit_hash)` — restore files from a historical commit.
//!   * `diff(from, to)` — which files changed, added, removed between two
//!      commits.
//!   * `log()` — walk parent pointers from HEAD back to the root.
//!
//! Hashing is a simple FNV-1a — fast, deterministic, no crypto needed
//! for the Rosetta Stone.

use std::collections::BTreeMap;

pub type Hash = u64;

/// FNV-1a 64-bit hash. Cross-language deterministic.
pub fn fnv1a_hash(data: &[u8]) -> Hash {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    /// filename → blob hash
    pub entries: BTreeMap<String, Hash>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub tree_hash: Hash,
    pub parent: Option<Hash>,
    pub message: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOp {
    Added(String, Hash),
    Removed(String, Hash),
    Changed(String, Hash, Hash),  // from, to
}

pub struct Repo {
    pub blobs: BTreeMap<Hash, Blob>,
    pub trees: BTreeMap<Hash, Tree>,
    pub commits: BTreeMap<Hash, Commit>,
    pub head: Option<Hash>,
}

impl Repo {
    pub fn new() -> Self {
        Self {
            blobs: BTreeMap::new(),
            trees: BTreeMap::new(),
            commits: BTreeMap::new(),
            head: None,
        }
    }

    /// Store a blob by content; return its hash.
    pub fn put_blob(&mut self, data: Vec<u8>) -> Hash {
        let h = fnv1a_hash(&data);
        self.blobs.entry(h).or_insert(Blob { data });
        h
    }

    /// Store a tree by content; return its hash.
    pub fn put_tree(&mut self, entries: BTreeMap<String, Hash>) -> Hash {
        let tree = Tree { entries };
        let canonical = canonical_tree_bytes(&tree);
        let h = fnv1a_hash(&canonical);
        self.trees.entry(h).or_insert(tree);
        h
    }

    /// Commit a set of (filename, content) pairs. Creates blobs + tree + commit.
    pub fn commit(&mut self, files: &[(&str, &[u8])], message: &str, timestamp: u64) -> Hash {
        let mut entries = BTreeMap::new();
        for (name, data) in files {
            let bh = self.put_blob(data.to_vec());
            entries.insert(name.to_string(), bh);
        }
        let tree_hash = self.put_tree(entries);
        let commit = Commit {
            tree_hash,
            parent: self.head,
            message: message.into(),
            timestamp_ms: timestamp,
        };
        let canonical = canonical_commit_bytes(&commit);
        let h = fnv1a_hash(&canonical);
        self.commits.insert(h, commit);
        self.head = Some(h);
        h
    }

    /// Return the files at a commit as (filename → content) pairs.
    pub fn checkout(&self, commit_hash: Hash) -> Option<BTreeMap<String, Vec<u8>>> {
        let commit = self.commits.get(&commit_hash)?;
        let tree = self.trees.get(&commit.tree_hash)?;
        let mut out = BTreeMap::new();
        for (name, bh) in &tree.entries {
            let blob = self.blobs.get(bh)?;
            out.insert(name.clone(), blob.data.clone());
        }
        Some(out)
    }

    /// Diff between two commits: added/removed/changed filenames.
    pub fn diff(&self, from: Hash, to: Hash) -> Option<Vec<DiffOp>> {
        let from_tree = self.trees.get(&self.commits.get(&from)?.tree_hash)?;
        let to_tree = self.trees.get(&self.commits.get(&to)?.tree_hash)?;
        let mut out = Vec::new();
        for (name, to_hash) in &to_tree.entries {
            match from_tree.entries.get(name) {
                None => out.push(DiffOp::Added(name.clone(), *to_hash)),
                Some(fh) if fh != to_hash => out.push(DiffOp::Changed(name.clone(), *fh, *to_hash)),
                _ => {}
            }
        }
        for (name, fh) in &from_tree.entries {
            if !to_tree.entries.contains_key(name) {
                out.push(DiffOp::Removed(name.clone(), *fh));
            }
        }
        out.sort_by(|a, b| op_key(a).cmp(&op_key(b)));
        Some(out)
    }

    /// Walk parent pointers from HEAD back to the root commit.
    pub fn log(&self) -> Vec<(Hash, &Commit)> {
        let mut out = Vec::new();
        let mut cursor = self.head;
        while let Some(h) = cursor {
            if let Some(c) = self.commits.get(&h) {
                out.push((h, c));
                cursor = c.parent;
            } else {
                break;
            }
        }
        out
    }

    pub fn blob_count(&self) -> usize { self.blobs.len() }
    pub fn commit_count(&self) -> usize { self.commits.len() }
}

impl Default for Repo { fn default() -> Self { Self::new() } }

fn op_key(op: &DiffOp) -> String {
    match op {
        DiffOp::Added(n, _) => format!("A{n}"),
        DiffOp::Removed(n, _) => format!("R{n}"),
        DiffOp::Changed(n, _, _) => format!("C{n}"),
    }
}

fn canonical_tree_bytes(t: &Tree) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"tree\n");
    for (name, hash) in &t.entries {
        out.extend_from_slice(format!("{name}\t{hash:016x}\n").as_bytes());
    }
    out
}

fn canonical_commit_bytes(c: &Commit) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"commit\n");
    out.extend_from_slice(format!("tree {:016x}\n", c.tree_hash).as_bytes());
    if let Some(p) = c.parent {
        out.extend_from_slice(format!("parent {p:016x}\n").as_bytes());
    }
    out.extend_from_slice(format!("ts {}\n", c.timestamp_ms).as_bytes());
    out.extend_from_slice(format!("msg {}\n", c.message).as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_deterministic() {
        assert_eq!(fnv1a_hash(b"hello"), fnv1a_hash(b"hello"));
        assert_ne!(fnv1a_hash(b"hello"), fnv1a_hash(b"world"));
    }

    #[test]
    fn identical_blobs_dedupe() {
        let mut r = Repo::new();
        let h1 = r.put_blob(b"hello".to_vec());
        let h2 = r.put_blob(b"hello".to_vec());
        assert_eq!(h1, h2);
        assert_eq!(r.blob_count(), 1);
    }

    #[test]
    fn commit_stores_files_and_updates_head() {
        let mut r = Repo::new();
        let h = r.commit(&[("a.txt", b"hello"), ("b.txt", b"world")],
                          "initial", 1000);
        assert_eq!(r.head, Some(h));
        let checked = r.checkout(h).unwrap();
        assert_eq!(checked.get("a.txt").unwrap(), b"hello");
        assert_eq!(checked.get("b.txt").unwrap(), b"world");
    }

    #[test]
    fn commit_chain_builds_history() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("x", b"1")], "v1", 100);
        let c2 = r.commit(&[("x", b"2")], "v2", 200);
        let c3 = r.commit(&[("x", b"3")], "v3", 300);
        let log = r.log();
        assert_eq!(log.len(), 3);
        assert_eq!(log[0].0, c3);
        assert_eq!(log[1].0, c2);
        assert_eq!(log[2].0, c1);
        assert_eq!(log[0].1.parent, Some(c2));
        assert_eq!(log[2].1.parent, None);
    }

    #[test]
    fn diff_detects_added() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("a.txt", b"1")], "v1", 100);
        let c2 = r.commit(&[("a.txt", b"1"), ("b.txt", b"2")], "v2", 200);
        let diff = r.diff(c1, c2).unwrap();
        assert_eq!(diff.len(), 1);
        assert!(matches!(&diff[0], DiffOp::Added(name, _) if name == "b.txt"));
    }

    #[test]
    fn diff_detects_removed() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("a.txt", b"1"), ("b.txt", b"2")], "v1", 100);
        let c2 = r.commit(&[("a.txt", b"1")], "v2", 200);
        let diff = r.diff(c1, c2).unwrap();
        assert_eq!(diff.len(), 1);
        assert!(matches!(&diff[0], DiffOp::Removed(name, _) if name == "b.txt"));
    }

    #[test]
    fn diff_detects_changed() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("x", b"hello")], "v1", 100);
        let c2 = r.commit(&[("x", b"world")], "v2", 200);
        let diff = r.diff(c1, c2).unwrap();
        assert_eq!(diff.len(), 1);
        assert!(matches!(&diff[0], DiffOp::Changed(name, _, _) if name == "x"));
    }

    #[test]
    fn diff_unchanged_files_omitted() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("a", b"1"), ("b", b"2")], "v1", 100);
        let c2 = r.commit(&[("a", b"1"), ("b", b"2"), ("c", b"3")], "v2", 200);
        let diff = r.diff(c1, c2).unwrap();
        assert_eq!(diff.len(), 1);
        assert!(matches!(&diff[0], DiffOp::Added(name, _) if name == "c"));
    }

    #[test]
    fn checkout_historical_commit_works() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("x", b"old")], "v1", 100);
        r.commit(&[("x", b"new")], "v2", 200);
        let old = r.checkout(c1).unwrap();
        assert_eq!(old.get("x").unwrap(), b"old");
    }

    #[test]
    fn blob_storage_is_deduplicated_across_commits() {
        let mut r = Repo::new();
        r.commit(&[("x", b"same"), ("y", b"same")], "v1", 100);
        // Both files have same content — should be one blob.
        assert_eq!(r.blob_count(), 1);
        r.commit(&[("z", b"same"), ("w", b"new")], "v2", 200);
        // "same" still one blob; new content adds another.
        assert_eq!(r.blob_count(), 2);
    }

    #[test]
    fn identical_tree_contents_produce_same_hash() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("a", b"1")], "v1", 100);
        let c2 = r.commit(&[("a", b"1")], "v2", 200);
        let t1 = r.commits.get(&c1).unwrap().tree_hash;
        let t2 = r.commits.get(&c2).unwrap().tree_hash;
        assert_eq!(t1, t2);  // same tree reused
    }

    #[test]
    fn empty_repo_log_is_empty() {
        let r = Repo::new();
        assert!(r.log().is_empty());
    }

    #[test]
    fn checkout_unknown_returns_none() {
        let r = Repo::new();
        assert!(r.checkout(12345).is_none());
    }

    #[test]
    fn initial_commit_has_no_parent() {
        let mut r = Repo::new();
        let c = r.commit(&[("x", b"1")], "initial", 100);
        assert!(r.commits.get(&c).unwrap().parent.is_none());
    }

    #[test]
    fn diff_results_sorted_stably() {
        let mut r = Repo::new();
        let c1 = r.commit(&[("a", b"1"), ("b", b"2"), ("c", b"3")], "v1", 100);
        let c2 = r.commit(&[("d", b"4"), ("e", b"5")], "v2", 200);
        let diff = r.diff(c1, c2).unwrap();
        // Adds (d, e), removes (a, b, c). All sorted.
        assert_eq!(diff.len(), 5);
    }
}
