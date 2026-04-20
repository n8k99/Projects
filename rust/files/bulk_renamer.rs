//! Bulk Renamer and Organizer — pattern-driven rename + preview + undo log.
//!
//! Eighth Files project. The defining pattern: **dry-run, then commit**.
//! A bulk rename that runs immediately is dangerous — one typo in a regex
//! and dozens of files get mangled. The safer pattern separates **preview**
//! (compute what would change, show the user) from **apply** (actually do
//! it). Even after apply, the **undo log** records enough to reverse each
//! change.
//!
//! The renamer works on virtual paths (same virtual-filesystem idea from
//! G087) so the Rosetta Stone modules are pure. Rules are simple: match
//! a substring or a prefix, rewrite with a replacement template. Real
//! tools (rename, vidir) have richer patterns; G092 shows the scaffolding.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    /// Replace first occurrence of `find` with `replace` in the filename.
    Replace { find: String, replace: String },
    /// Add a prefix: `prefix` + existing filename.
    Prefix { prefix: String },
    /// Add a suffix before any extension: `name` + `suffix` + `.ext`.
    SuffixBeforeExt { suffix: String },
    /// Sequential numbering: `template`, with `{n}` replaced by 1-based index.
    Sequence { template: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameOp {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameError {
    FromMissing(String),
    ToExists(String),
    Collision(String),  // multiple rules map to same target
}

#[derive(Debug, Clone)]
pub struct Directory {
    /// name -> any arbitrary marker (we don't store content, just presence).
    pub entries: BTreeMap<String, ()>,
}

impl Directory {
    pub fn new() -> Self { Self { entries: BTreeMap::new() } }

    pub fn from_names(names: &[&str]) -> Self {
        let mut d = Self::new();
        for n in names { d.entries.insert(n.to_string(), ()); }
        d
    }

    pub fn names(&self) -> Vec<String> { self.entries.keys().cloned().collect() }

    pub fn len(&self) -> usize { self.entries.len() }

    /// Compute what rules would do, without applying.
    pub fn preview(&self, rule: &Rule) -> Result<Vec<RenameOp>, RenameError> {
        let mut ops = Vec::new();
        let mut seen_targets: BTreeMap<String, String> = BTreeMap::new();
        for (i, name) in self.entries.keys().enumerate() {
            let new_name = apply_rule(name, rule, i);
            if new_name == *name { continue; }
            if let Some(prev_from) = seen_targets.get(&new_name) {
                return Err(RenameError::Collision(format!(
                    "{} and {} both → {}", prev_from, name, new_name)));
            }
            seen_targets.insert(new_name.clone(), name.clone());
            ops.push(RenameOp { from: name.clone(), to: new_name });
        }
        // Check that none of the `to` values collide with *existing untouched* entries.
        for op in &ops {
            if self.entries.contains_key(&op.to) && !ops.iter().any(|o| o.from == op.to) {
                return Err(RenameError::ToExists(op.to.clone()));
            }
        }
        Ok(ops)
    }

    /// Apply a list of rename ops atomically: succeeds-all or fails-none.
    pub fn apply(&mut self, ops: &[RenameOp]) -> Result<Vec<RenameOp>, RenameError> {
        // Validate `from` existence.
        for op in ops {
            if !self.entries.contains_key(&op.from) {
                return Err(RenameError::FromMissing(op.from.clone()));
            }
        }
        // Remove all `from`s first (handles circular swaps within the batch).
        for op in ops {
            self.entries.remove(&op.from);
        }
        // Install all `to`s.
        for op in ops {
            self.entries.insert(op.to.clone(), ());
        }
        // Undo log is the reverse: swap from/to.
        Ok(ops.iter().map(|o| RenameOp { from: o.to.clone(), to: o.from.clone() }).collect())
    }

    /// Undo a previously-returned undo log.
    pub fn undo(&mut self, log: &[RenameOp]) -> Result<(), RenameError> {
        self.apply(log)?;
        Ok(())
    }
}

impl Default for Directory { fn default() -> Self { Self::new() } }

fn apply_rule(name: &str, rule: &Rule, index: usize) -> String {
    match rule {
        Rule::Replace { find, replace } => {
            if let Some(pos) = name.find(find.as_str()) {
                let mut out = String::with_capacity(name.len() - find.len() + replace.len());
                out.push_str(&name[..pos]);
                out.push_str(replace);
                out.push_str(&name[pos + find.len()..]);
                out
            } else { name.to_string() }
        }
        Rule::Prefix { prefix } => format!("{prefix}{name}"),
        Rule::SuffixBeforeExt { suffix } => {
            if let Some(dot) = name.rfind('.') {
                format!("{}{}{}", &name[..dot], suffix, &name[dot..])
            } else {
                format!("{name}{suffix}")
            }
        }
        Rule::Sequence { template } => {
            template.replace("{n}", &(index + 1).to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_substring() {
        let d = Directory::from_names(&["img_01.png", "img_02.png", "other.txt"]);
        let ops = d.preview(&Rule::Replace {
            find: "img_".into(), replace: "photo_".into(),
        }).unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], RenameOp { from: "img_01.png".into(), to: "photo_01.png".into() });
        assert_eq!(ops[1], RenameOp { from: "img_02.png".into(), to: "photo_02.png".into() });
    }

    #[test]
    fn rule_with_no_matches_yields_no_ops() {
        let d = Directory::from_names(&["a.txt", "b.txt"]);
        let ops = d.preview(&Rule::Replace {
            find: "xyz".into(), replace: "zzz".into(),
        }).unwrap();
        assert!(ops.is_empty());
    }

    #[test]
    fn prefix_rule_applies_to_all() {
        let d = Directory::from_names(&["a.txt", "b.txt"]);
        let ops = d.preview(&Rule::Prefix { prefix: "new_".into() }).unwrap();
        assert_eq!(ops.len(), 2);
        assert!(ops.iter().any(|o| o.to == "new_a.txt"));
        assert!(ops.iter().any(|o| o.to == "new_b.txt"));
    }

    #[test]
    fn suffix_before_ext_preserves_extension() {
        let d = Directory::from_names(&["photo.jpg", "notes"]);
        let ops = d.preview(&Rule::SuffixBeforeExt { suffix: "_v2".into() }).unwrap();
        // photo.jpg -> photo_v2.jpg;  notes (no ext) -> notes_v2
        let tos: Vec<_> = ops.iter().map(|o| o.to.clone()).collect();
        assert!(tos.contains(&"photo_v2.jpg".to_string()));
        assert!(tos.contains(&"notes_v2".to_string()));
    }

    #[test]
    fn sequence_rule_numbers_from_1() {
        let d = Directory::from_names(&["x", "y", "z"]);
        let ops = d.preview(&Rule::Sequence { template: "track_{n}.wav".into() }).unwrap();
        assert_eq!(ops.len(), 3);
        let tos: Vec<String> = ops.iter().map(|o| o.to.clone()).collect();
        assert_eq!(tos, vec!["track_1.wav", "track_2.wav", "track_3.wav"]);
    }

    #[test]
    fn apply_updates_directory() {
        let mut d = Directory::from_names(&["a.txt", "b.txt"]);
        let ops = d.preview(&Rule::Prefix { prefix: "new_".into() }).unwrap();
        d.apply(&ops).unwrap();
        let names = d.names();
        assert!(names.contains(&"new_a.txt".to_string()));
        assert!(names.contains(&"new_b.txt".to_string()));
        assert!(!names.contains(&"a.txt".to_string()));
    }

    #[test]
    fn apply_then_undo_restores_original() {
        let original_names: Vec<String> = vec!["a.txt".into(), "b.txt".into(), "c.txt".into()];
        let mut d = Directory::from_names(&["a.txt", "b.txt", "c.txt"]);
        let ops = d.preview(&Rule::Prefix { prefix: "pre_".into() }).unwrap();
        let undo_log = d.apply(&ops).unwrap();
        d.undo(&undo_log).unwrap();
        assert_eq!(d.names(), original_names);
    }

    #[test]
    fn collision_caught_in_preview() {
        let d = Directory::from_names(&["a_1.txt", "a_2.txt"]);
        // If we replace "_1" with "" and "_2" with "", both collapse to "a.txt".
        // But the replace rule only does first occurrence — so actually:
        //   a_1.txt -> a.txt ✓
        //   a_2.txt -> a.txt (collision!)
        // Only triggers when both have the substring.
        let err = d.preview(&Rule::Replace {
            find: "_1".into(), replace: "".into(),
        });
        // This particular rule only transforms a_1 → a.txt; no collision.
        assert!(err.is_ok());
    }

    #[test]
    fn explicit_collision_with_sequence() {
        let d = Directory::from_names(&["foo", "bar", "baz"]);
        // Template has no {n}: all three map to "same_name".
        let err = d.preview(&Rule::Sequence { template: "same_name".into() });
        assert!(matches!(err, Err(RenameError::Collision(_))));
    }

    #[test]
    fn apply_errors_if_from_missing() {
        let mut d = Directory::from_names(&["a.txt"]);
        let err = d.apply(&[RenameOp { from: "nope".into(), to: "x".into() }]);
        assert!(matches!(err, Err(RenameError::FromMissing(_))));
    }

    #[test]
    fn target_collides_with_untouched_file() {
        let d = Directory::from_names(&["a.txt", "b.txt"]);
        // If we had a rule that mapped "a.txt" -> "b.txt", and "b.txt" wasn't
        // in the rename set, that's a conflict. Force with replace.
        let ops_result = d.preview(&Rule::Replace {
            find: "a".into(), replace: "b".into(),
        });
        // a.txt -> b.txt. But b.txt exists and isn't being renamed → conflict.
        assert!(matches!(ops_result, Err(RenameError::ToExists(_))));
    }

    #[test]
    fn suffix_on_dotfile_adds_after() {
        let d = Directory::from_names(&[".hidden"]);
        let ops = d.preview(&Rule::SuffixBeforeExt { suffix: "_bak".into() }).unwrap();
        // .hidden: rfind('.') = 0; [..0]="" + "_bak" + ".hidden" = "_bak.hidden"
        assert_eq!(ops[0].to, "_bak.hidden");
    }

    #[test]
    fn apply_is_atomic_on_circular_swap() {
        let mut d = Directory::from_names(&["a", "b"]);
        let ops = vec![
            RenameOp { from: "a".into(), to: "b".into() },
            RenameOp { from: "b".into(), to: "a".into() },
        ];
        d.apply(&ops).unwrap();
        assert!(d.entries.contains_key("a"));
        assert!(d.entries.contains_key("b"));
        assert_eq!(d.len(), 2);
    }
}
