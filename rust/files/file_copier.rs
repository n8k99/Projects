//! File Copy Utility — recursive tree copy with policy + filter + progress.
//!
//! Fourteenth Files project. Building on G087's virtual filesystem,
//! G098 introduces **recursive subtree copy with explicit policy**. A
//! straightforward `cp -r` is three decisions stitched together:
//!   * **Overwrite policy** — skip, overwrite, or rename with suffix.
//!   * **Filter** — include/exclude by name pattern (simple glob prefix).
//!   * **Progress** — report (source, dest, bytes) per file copied.
//!
//! The copy logic is pure: the source tree is a `BTreeMap<String, Item>`
//! (path → item) and the destination is another. No real disk I/O.
//! Progress is captured into a `Vec<CopyEvent>` instead of printed, for
//! testability.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    File { size: u64 },
    Dir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    Skip,
    Overwrite,
    RenameWithSuffix,  // append `.N` where N=1,2,3...
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub include: Vec<String>,   // empty = everything
    pub exclude: Vec<String>,
}

impl Filter {
    pub fn allow_all() -> Self { Self { include: vec![], exclude: vec![] } }
    pub fn accepts(&self, path: &str) -> bool {
        if !self.include.is_empty() && !self.include.iter().any(|pat| path.contains(pat)) {
            return false;
        }
        if self.exclude.iter().any(|pat| path.contains(pat)) {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyEvent {
    pub kind: EventKind,
    pub source: String,
    pub dest: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Copied,
    Skipped,
    Renamed,
    CreatedDir,
}

#[derive(Debug, Clone)]
pub struct Fs {
    pub items: BTreeMap<String, Item>,
}

impl Fs {
    pub fn new() -> Self { Self { items: BTreeMap::new() } }

    pub fn from_items(entries: &[(&str, Item)]) -> Self {
        let mut fs = Self::new();
        for (path, item) in entries {
            fs.items.insert(path.to_string(), item.clone());
        }
        fs
    }

    /// Copy `source_root` subtree into `dest_root` in this fs.
    ///
    /// Returns the list of events describing what was done.
    pub fn copy_tree(
        &mut self,
        src_root: &str,
        dest_root: &str,
        policy: Overwrite,
        filter: &Filter,
    ) -> Result<Vec<CopyEvent>, String> {
        let src_items: Vec<(String, Item)> = self.items.iter()
            .filter(|(p, _)| p == &src_root || p.starts_with(&format!("{src_root}/")))
            .map(|(p, i)| (p.clone(), i.clone()))
            .collect();
        if src_items.is_empty() {
            return Err(format!("source not found: {src_root}"));
        }

        let mut events = Vec::new();
        for (src_path, item) in &src_items {
            if !filter.accepts(src_path) {
                continue;
            }
            let rel = if src_path == src_root {
                ""
            } else {
                &src_path[src_root.len() + 1..]
            };
            let dest_path = if rel.is_empty() {
                dest_root.to_string()
            } else {
                format!("{dest_root}/{rel}")
            };

            let final_dest = match (self.items.contains_key(&dest_path), policy) {
                (false, _) => dest_path.clone(),
                (true, Overwrite::Skip) => {
                    events.push(CopyEvent {
                        kind: EventKind::Skipped,
                        source: src_path.clone(),
                        dest: dest_path.clone(),
                        bytes: 0,
                    });
                    continue;
                }
                (true, Overwrite::Overwrite) => dest_path.clone(),
                (true, Overwrite::RenameWithSuffix) => {
                    let renamed = next_available_name(&self.items, &dest_path);
                    events.push(CopyEvent {
                        kind: EventKind::Renamed,
                        source: src_path.clone(),
                        dest: renamed.clone(),
                        bytes: 0,
                    });
                    renamed
                }
            };

            match item {
                Item::Dir => {
                    self.items.insert(final_dest.clone(), Item::Dir);
                    events.push(CopyEvent {
                        kind: EventKind::CreatedDir,
                        source: src_path.clone(),
                        dest: final_dest,
                        bytes: 0,
                    });
                }
                Item::File { size } => {
                    self.items.insert(final_dest.clone(), Item::File { size: *size });
                    events.push(CopyEvent {
                        kind: EventKind::Copied,
                        source: src_path.clone(),
                        dest: final_dest,
                        bytes: *size,
                    });
                }
            }
        }
        Ok(events)
    }

    pub fn total_size_copied(&self, events: &[CopyEvent]) -> u64 {
        events.iter().filter(|e| e.kind == EventKind::Copied).map(|e| e.bytes).sum()
    }

    pub fn paths(&self) -> Vec<String> {
        self.items.keys().cloned().collect()
    }
}

impl Default for Fs { fn default() -> Self { Self::new() } }

fn next_available_name(items: &BTreeMap<String, Item>, base: &str) -> String {
    let mut n = 1;
    loop {
        let candidate = format!("{base}.{n}");
        if !items.contains_key(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Fs {
        Fs::from_items(&[
            ("src", Item::Dir),
            ("src/a.txt", Item::File { size: 100 }),
            ("src/b.txt", Item::File { size: 200 }),
            ("src/sub", Item::Dir),
            ("src/sub/c.txt", Item::File { size: 50 }),
        ])
    }

    #[test]
    fn copy_tree_to_fresh_dest() {
        let mut fs = fixture();
        let events = fs.copy_tree("src", "dest", Overwrite::Skip, &Filter::allow_all()).unwrap();
        let copied = fs.total_size_copied(&events);
        assert_eq!(copied, 350);  // 100+200+50
        assert!(fs.items.contains_key("dest/a.txt"));
        assert!(fs.items.contains_key("dest/b.txt"));
        assert!(fs.items.contains_key("dest/sub/c.txt"));
    }

    #[test]
    fn skip_policy_preserves_existing_files() {
        let mut fs = fixture();
        fs.items.insert("dest".into(), Item::Dir);
        fs.items.insert("dest/a.txt".into(), Item::File { size: 999 });
        let events = fs.copy_tree("src", "dest", Overwrite::Skip, &Filter::allow_all()).unwrap();
        // a.txt should be skipped; its size should still be 999.
        assert_eq!(fs.items.get("dest/a.txt"), Some(&Item::File { size: 999 }));
        assert!(events.iter().any(|e| e.kind == EventKind::Skipped));
    }

    #[test]
    fn overwrite_policy_replaces_existing_files() {
        let mut fs = fixture();
        fs.items.insert("dest".into(), Item::Dir);
        fs.items.insert("dest/a.txt".into(), Item::File { size: 999 });
        fs.copy_tree("src", "dest", Overwrite::Overwrite, &Filter::allow_all()).unwrap();
        assert_eq!(fs.items.get("dest/a.txt"), Some(&Item::File { size: 100 }));
    }

    #[test]
    fn rename_policy_preserves_existing_and_adds_suffix() {
        let mut fs = fixture();
        fs.items.insert("dest".into(), Item::Dir);
        fs.items.insert("dest/a.txt".into(), Item::File { size: 999 });
        fs.copy_tree("src", "dest", Overwrite::RenameWithSuffix, &Filter::allow_all()).unwrap();
        assert_eq!(fs.items.get("dest/a.txt"), Some(&Item::File { size: 999 }));  // original
        assert_eq!(fs.items.get("dest/a.txt.1"), Some(&Item::File { size: 100 }));  // copy
    }

    #[test]
    fn filter_include_only_matching() {
        let mut fs = fixture();
        let filter = Filter { include: vec!["a.txt".into()], exclude: vec![] };
        fs.copy_tree("src", "dest", Overwrite::Overwrite, &filter).unwrap();
        assert!(fs.items.contains_key("dest/a.txt"));
        assert!(!fs.items.contains_key("dest/b.txt"));
    }

    #[test]
    fn filter_exclude_drops_matching() {
        let mut fs = fixture();
        let filter = Filter { include: vec![], exclude: vec!["b.txt".into()] };
        fs.copy_tree("src", "dest", Overwrite::Overwrite, &filter).unwrap();
        assert!(fs.items.contains_key("dest/a.txt"));
        assert!(!fs.items.contains_key("dest/b.txt"));
    }

    #[test]
    fn missing_source_returns_error() {
        let mut fs = fixture();
        let err = fs.copy_tree("nope", "dest", Overwrite::Overwrite, &Filter::allow_all());
        assert!(err.is_err());
    }

    #[test]
    fn events_track_every_item() {
        let mut fs = fixture();
        let events = fs.copy_tree("src", "dest", Overwrite::Overwrite, &Filter::allow_all()).unwrap();
        // src (dir) + a.txt + b.txt + sub (dir) + sub/c.txt = 5 events
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn rename_with_suffix_handles_multiple_collisions() {
        let mut fs = Fs::from_items(&[
            ("src", Item::Dir),
            ("src/a", Item::File { size: 10 }),
            ("dest", Item::Dir),
            ("dest/a", Item::File { size: 1 }),
            ("dest/a.1", Item::File { size: 2 }),
            ("dest/a.2", Item::File { size: 3 }),
        ]);
        fs.copy_tree("src", "dest", Overwrite::RenameWithSuffix, &Filter::allow_all()).unwrap();
        assert_eq!(fs.items.get("dest/a.3"), Some(&Item::File { size: 10 }));
    }

    #[test]
    fn copy_single_file_as_source() {
        let mut fs = Fs::from_items(&[
            ("hello.txt", Item::File { size: 42 }),
        ]);
        fs.copy_tree("hello.txt", "world.txt", Overwrite::Overwrite, &Filter::allow_all()).unwrap();
        assert_eq!(fs.items.get("world.txt"), Some(&Item::File { size: 42 }));
    }

    #[test]
    fn empty_filter_accepts_everything() {
        let f = Filter::allow_all();
        assert!(f.accepts("anything"));
        assert!(f.accepts(""));
    }

    #[test]
    fn filter_rejects_when_include_doesnt_match() {
        let f = Filter { include: vec![".jpg".into()], exclude: vec![] };
        assert!(f.accepts("photo.jpg"));
        assert!(!f.accepts("doc.pdf"));
    }

    #[test]
    fn copied_events_carry_bytes() {
        let mut fs = fixture();
        let events = fs.copy_tree("src", "dest", Overwrite::Overwrite, &Filter::allow_all()).unwrap();
        let copied: Vec<&CopyEvent> = events.iter().filter(|e| e.kind == EventKind::Copied).collect();
        let total: u64 = copied.iter().map(|e| e.bytes).sum();
        assert_eq!(total, 350);
    }
}
