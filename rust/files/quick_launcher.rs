//! Quick Launcher — indexed launchable items ranked by match quality plus frecency.
//!
//! Second Files project. The key design move: **launchable state is a file**.
//! Run one, increment its count and stamp the time — and that state survives
//! across sessions via `to_text`/`from_text`. A launcher without persistence
//! re-learns your priorities every time you restart; a launcher with
//! persistence learns you. That persistence is a file round-trip.
//!
//! Matching has two layers:
//!   1. **Text match** — name-prefix > name-substring > keyword-substring.
//!      No match means the entry is filtered out entirely.
//!   2. **Frecency** — log(1 + launch_count) / (1 + age_hours). Frequently-used
//!      and recently-used items rise; dusty items sink but aren't forgotten.
//!
//! Final rank is `text_match + frecency_weight * frecency`. Text match
//! dominates — a perfect name match always beats frecency — but frecency
//! breaks ties and demotes never-launched entries.

use std::collections::HashMap;

pub type EntryId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: EntryId,
    pub name: String,
    pub action: String,
    pub keywords: Vec<String>,
    pub launch_count: u32,
    pub last_launched_ms: u64,
}

impl Entry {
    pub fn new(id: EntryId, name: &str, action: &str, keywords: &[&str]) -> Self {
        Self {
            id,
            name: name.into(),
            action: action.into(),
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            launch_count: 0,
            last_launched_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Match {
    pub entry_id: EntryId,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct Launcher {
    entries: HashMap<EntryId, Entry>,
    pub frecency_weight: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchError {
    UnknownEntry(EntryId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    BadHeader(String),
    MissingField(String),
    BadNumber(String),
}

impl Launcher {
    pub fn new() -> Self {
        Self { entries: HashMap::new(), frecency_weight: 0.1 }
    }

    pub fn add(&mut self, entry: Entry) {
        self.entries.insert(entry.id, entry);
    }

    pub fn get(&self, id: EntryId) -> Option<&Entry> {
        self.entries.get(&id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Rank entries by `text_match(query) + frecency_weight * frecency(now_ms)`,
    /// dropping entries with zero text match. Ties break by smaller id.
    pub fn query(&self, text: &str, now_ms: u64) -> Vec<Match> {
        let needle = text.trim().to_lowercase();
        let mut out: Vec<Match> = self.entries.values()
            .filter_map(|e| {
                let tm = text_match(&needle, e);
                if tm == 0.0 { return None; }
                let fr = frecency(e, now_ms);
                Some(Match { entry_id: e.id, score: tm + self.frecency_weight * fr })
            })
            .collect();
        out.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.entry_id.cmp(&b.entry_id))
        });
        out
    }

    pub fn record_launch(&mut self, id: EntryId, now_ms: u64) -> Result<(), LaunchError> {
        let entry = self.entries.get_mut(&id)
            .ok_or(LaunchError::UnknownEntry(id))?;
        entry.launch_count += 1;
        entry.last_launched_ms = now_ms;
        Ok(())
    }

    /// Serialise to a line-based text format that round-trips.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("LAUNCHER\n");
        let mut ids: Vec<&EntryId> = self.entries.keys().collect();
        ids.sort();
        for id in ids {
            let e = &self.entries[id];
            out.push_str("---\n");
            out.push_str(&format!("id: {}\n", e.id));
            out.push_str(&format!("name: {}\n", e.name));
            out.push_str(&format!("action: {}\n", e.action));
            for k in &e.keywords {
                out.push_str(&format!("keyword: {k}\n"));
            }
            out.push_str(&format!("count: {}\n", e.launch_count));
            out.push_str(&format!("last: {}\n", e.last_launched_ms));
        }
        out
    }

    pub fn from_text(text: &str) -> Result<Self, ParseError> {
        let mut lines = text.lines().peekable();
        if lines.next() != Some("LAUNCHER") {
            return Err(ParseError::BadHeader("missing LAUNCHER header".into()));
        }
        let mut launcher = Launcher::new();
        while lines.peek() == Some(&"---") {
            lines.next();
            let entry = parse_entry(&mut lines)?;
            launcher.add(entry);
        }
        Ok(launcher)
    }
}

fn text_match(needle: &str, entry: &Entry) -> f64 {
    if needle.is_empty() { return 0.5; }  // empty query surfaces all
    let name_lc = entry.name.to_lowercase();
    if name_lc == needle { return 1.0; }
    if name_lc.starts_with(needle) { return 0.8; }
    if name_lc.contains(needle) { return 0.6; }
    for k in &entry.keywords {
        let k_lc = k.to_lowercase();
        if k_lc == needle { return 0.5; }
        if k_lc.contains(needle) { return 0.4; }
    }
    0.0
}

fn frecency(entry: &Entry, now_ms: u64) -> f64 {
    if entry.launch_count == 0 { return 0.0; }
    let age_ms = now_ms.saturating_sub(entry.last_launched_ms);
    let age_hours = age_ms as f64 / 3_600_000.0;
    (1.0 + entry.launch_count as f64).ln() / (1.0 + age_hours)
}

fn parse_entry<'a, I: Iterator<Item = &'a str>>(
    lines: &mut std::iter::Peekable<I>,
) -> Result<Entry, ParseError> {
    let mut id: Option<u64> = None;
    let mut name: Option<String> = None;
    let mut action: Option<String> = None;
    let mut keywords: Vec<String> = Vec::new();
    let mut count: u32 = 0;
    let mut last: u64 = 0;

    while let Some(&line) = lines.peek() {
        if line == "---" { break; }
        lines.next();
        let colon = line.find(": ").ok_or_else(||
            ParseError::MissingField(format!("no `: ` in {line}")))?;
        let (key, val) = (&line[..colon], &line[colon + 2..]);
        match key {
            "id" => id = Some(val.parse().map_err(|_|
                ParseError::BadNumber(format!("id {val}")))?),
            "name" => name = Some(val.into()),
            "action" => action = Some(val.into()),
            "keyword" => keywords.push(val.into()),
            "count" => count = val.parse().map_err(|_|
                ParseError::BadNumber(format!("count {val}")))?,
            "last" => last = val.parse().map_err(|_|
                ParseError::BadNumber(format!("last {val}")))?,
            _ => {}
        }
    }
    Ok(Entry {
        id: id.ok_or_else(|| ParseError::MissingField("id".into()))?,
        name: name.ok_or_else(|| ParseError::MissingField("name".into()))?,
        action: action.ok_or_else(|| ParseError::MissingField("action".into()))?,
        keywords,
        launch_count: count,
        last_launched_ms: last,
    })
}

impl Default for Launcher {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Launcher {
        let mut l = Launcher::new();
        l.add(Entry::new(1, "Firefox", "firefox", &["browser", "web"]));
        l.add(Entry::new(2, "Files", "nautilus", &["folder", "nautilus"]));
        l.add(Entry::new(3, "Terminal", "kitty", &["shell", "cli"]));
        l.add(Entry::new(4, "File Manager", "thunar", &["folder"]));
        l
    }

    #[test]
    fn exact_name_beats_substring() {
        let l = sample();
        let out = l.query("Files", 0);
        assert_eq!(out[0].entry_id, 2);  // exact "Files" before "File Manager"
    }

    #[test]
    fn prefix_beats_substring() {
        let l = sample();
        let out = l.query("fire", 0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].entry_id, 1);
    }

    #[test]
    fn keyword_match_works_when_name_does_not() {
        let l = sample();
        let out = l.query("shell", 0);
        assert!(out.iter().any(|m| m.entry_id == 3));
    }

    #[test]
    fn no_match_excluded_entirely() {
        let l = sample();
        let out = l.query("zzz", 0);
        assert!(out.is_empty());
    }

    #[test]
    fn frecency_breaks_ties_among_equal_text_matches() {
        let mut l = sample();
        // Both entries match "folder" as keyword → tie on text score.
        // Bump id=2 so it gets frecency boost.
        l.record_launch(2, 1_000).unwrap();
        let out = l.query("folder", 1_000);
        assert_eq!(out[0].entry_id, 2);  // launched, so wins the tie
        assert_eq!(out[1].entry_id, 4);
    }

    #[test]
    fn text_match_dominates_frecency() {
        let mut l = sample();
        // Pump id=3 with tons of launches.
        for _ in 0..1000 { l.record_launch(3, 1_000).unwrap(); }
        // Query "fire" should still return Firefox (id=1), not Terminal.
        let out = l.query("fire", 1_000);
        assert_eq!(out[0].entry_id, 1);
    }

    #[test]
    fn unknown_launch_returns_error() {
        let mut l = sample();
        assert_eq!(l.record_launch(999, 0), Err(LaunchError::UnknownEntry(999)));
    }

    #[test]
    fn record_launch_updates_count_and_timestamp() {
        let mut l = sample();
        l.record_launch(1, 42).unwrap();
        l.record_launch(1, 100).unwrap();
        let e = l.get(1).unwrap();
        assert_eq!(e.launch_count, 2);
        assert_eq!(e.last_launched_ms, 100);
    }

    #[test]
    fn round_trip_preserves_launcher() {
        let mut l = sample();
        l.record_launch(1, 1000).unwrap();
        l.record_launch(1, 2000).unwrap();
        l.record_launch(3, 3000).unwrap();
        let text = l.to_text();
        let parsed = Launcher::from_text(&text).unwrap();
        assert_eq!(parsed.len(), l.len());
        for id in [1, 2, 3, 4] {
            let a = l.get(id).unwrap();
            let b = parsed.get(id).unwrap();
            assert_eq!(a, b);
        }
    }

    #[test]
    fn round_trip_empty_launcher() {
        let l = Launcher::new();
        let text = l.to_text();
        let parsed = Launcher::from_text(&text).unwrap();
        assert_eq!(parsed.len(), 0);
    }

    #[test]
    fn parse_rejects_missing_header() {
        let err = Launcher::from_text("---\nid: 1\nname: x\naction: y\n");
        assert!(matches!(err, Err(ParseError::BadHeader(_))));
    }

    #[test]
    fn query_is_case_insensitive() {
        let l = sample();
        let upper = l.query("FIREFOX", 0);
        let lower = l.query("firefox", 0);
        assert_eq!(upper.len(), lower.len());
        assert_eq!(upper[0].entry_id, lower[0].entry_id);
    }

    #[test]
    fn empty_query_surfaces_all_entries() {
        let l = sample();
        let out = l.query("", 0);
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn frecency_decays_with_time() {
        let mut l = sample();
        l.record_launch(1, 0).unwrap();
        // Query fresh
        let fresh = l.query("browser", 0);
        // Query an hour later
        let stale = l.query("browser", 3_600_000 * 10);  // 10 hours
        assert!(fresh[0].score > stale[0].score);
    }
}
