//! CD Burning App — multi-session disc model + bin-packing + ISO filenames.
//!
//! Seventh Graphics project. Distinctive moves:
//!
//!   * **Multi-session overhead** — each session has a leadin + leadout
//!     cost; usable capacity = disc capacity - sum(overheads). Adding a
//!     second session removes ~35 MB of file space, not zero.
//!   * **First-fit-decreasing bin-packing** — sort files by size desc,
//!     place each in the first bin that fits. Deterministic, fast, and
//!     near-optimal for disc packing.
//!   * **ISO 9660 8.3 filename canonicalization** — uppercase, restrict
//!     charset, truncate to 8 + 3, and suffix `~N` on collision.
//!   * **Finalize FSM** — Open → SessionOpen → SessionClosed → Finalized.
//!     Finalized discs reject all new sessions. Non-final sessions can
//!     be closed and re-opened with another session.

use std::collections::BTreeSet;

pub const LEADIN_BYTES: u64 = 13_000_000;
pub const LEADOUT_BYTES: u64 = 22_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BurnFile {
    pub name: String,
    pub size_bytes: u64,
}

impl BurnFile {
    pub fn new(name: impl Into<String>, size_bytes: u64) -> Self {
        Self { name: name.into(), size_bytes }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState { Open, Closed }

#[derive(Debug, Clone)]
pub struct Session {
    pub tracks: Vec<BurnFile>,
    pub state: SessionState,
}

impl Session {
    pub fn new() -> Self {
        Self { tracks: vec![], state: SessionState::Open }
    }

    pub fn total_track_bytes(&self) -> u64 {
        self.tracks.iter().map(|f| f.size_bytes).sum()
    }

    pub fn overhead_bytes(&self) -> u64 {
        LEADIN_BYTES + LEADOUT_BYTES
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_track_bytes() + self.overhead_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscState {
    Blank,
    SessionOpen,
    SessionClosed,
    Finalized,
}

#[derive(Debug, Clone)]
pub struct Disc {
    pub capacity_bytes: u64,
    pub sessions: Vec<Session>,
    pub state: DiscState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BurnError {
    DiscFinalized,
    NoOpenSession,
    NotEnoughSpace { needed: u64, available: u64 },
    SessionAlreadyOpen,
}

impl Disc {
    pub fn new(capacity_bytes: u64) -> Self {
        Self { capacity_bytes, sessions: vec![], state: DiscState::Blank }
    }

    pub fn used_bytes(&self) -> u64 {
        self.sessions.iter().map(|s| s.total_bytes()).sum()
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.capacity_bytes.saturating_sub(self.used_bytes())
    }

    /// Usable capacity for one more file in the current open session,
    /// or for a hypothetical new session if there's no open session.
    pub fn usable_for_next_file(&self) -> u64 {
        match self.state {
            DiscState::Finalized => 0,
            DiscState::SessionOpen => self.remaining_bytes(),
            DiscState::Blank | DiscState::SessionClosed => {
                // a new session would cost leadin + leadout
                self.remaining_bytes().saturating_sub(LEADIN_BYTES + LEADOUT_BYTES)
            }
        }
    }

    pub fn open_session(&mut self) -> Result<(), BurnError> {
        match self.state {
            DiscState::Finalized => Err(BurnError::DiscFinalized),
            DiscState::SessionOpen => Err(BurnError::SessionAlreadyOpen),
            DiscState::Blank | DiscState::SessionClosed => {
                self.sessions.push(Session::new());
                self.state = DiscState::SessionOpen;
                Ok(())
            }
        }
    }

    pub fn add_file(&mut self, file: BurnFile) -> Result<(), BurnError> {
        if self.state == DiscState::Finalized { return Err(BurnError::DiscFinalized); }
        if self.state != DiscState::SessionOpen { return Err(BurnError::NoOpenSession); }
        let available = self.remaining_bytes();
        if file.size_bytes > available {
            return Err(BurnError::NotEnoughSpace {
                needed: file.size_bytes, available
            });
        }
        self.sessions.last_mut().unwrap().tracks.push(file);
        Ok(())
    }

    pub fn close_session(&mut self) -> Result<(), BurnError> {
        if self.state == DiscState::Finalized { return Err(BurnError::DiscFinalized); }
        if self.state != DiscState::SessionOpen { return Err(BurnError::NoOpenSession); }
        self.sessions.last_mut().unwrap().state = SessionState::Closed;
        self.state = DiscState::SessionClosed;
        Ok(())
    }

    pub fn finalize(&mut self) -> Result<(), BurnError> {
        if self.state == DiscState::Finalized { return Err(BurnError::DiscFinalized); }
        if self.state == DiscState::SessionOpen {
            self.close_session()?;
        }
        self.state = DiscState::Finalized;
        Ok(())
    }
}

/// First-fit-decreasing bin-packing: for each disc capacity, pack as many
/// files as will fit (largest first). Returns the packed file lists per
/// disc and any remainder that didn't fit.
#[derive(Debug, Clone)]
pub struct PackResult {
    pub discs: Vec<Vec<BurnFile>>,
    pub overflow: Vec<BurnFile>,
}

pub fn pack_files_ffd(files: &[BurnFile], disc_capacity: u64, num_discs: usize)
    -> PackResult
{
    let mut sorted: Vec<BurnFile> = files.to_vec();
    sorted.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    // per-disc we charge one session overhead upfront
    let usable = disc_capacity.saturating_sub(LEADIN_BYTES + LEADOUT_BYTES);

    let mut discs: Vec<Vec<BurnFile>> = vec![Vec::new(); num_discs];
    let mut remaining: Vec<u64> = vec![usable; num_discs];
    let mut overflow: Vec<BurnFile> = vec![];

    for file in sorted {
        let mut placed = false;
        for i in 0..num_discs {
            if file.size_bytes <= remaining[i] {
                remaining[i] -= file.size_bytes;
                discs[i].push(file.clone());
                placed = true;
                break;
            }
        }
        if !placed {
            overflow.push(file);
        }
    }

    PackResult { discs, overflow }
}

/// Canonicalize a filename into ISO 9660 Level 1 (8.3) form, disambiguating
/// against an existing set of names with `~N` suffixes.
pub fn canonicalize_iso_name(name: &str, existing: &BTreeSet<String>) -> String {
    let (stem, ext) = split_ext(name);
    let stem_clean = clean_chars(stem);
    let ext_clean = clean_chars(ext);
    let ext_trunc: String = ext_clean.chars().take(3).collect();
    let stem_base: String = stem_clean.chars().take(8).collect();
    let base = if ext_trunc.is_empty() {
        stem_base.clone()
    } else {
        format!("{}.{}", stem_base, ext_trunc)
    };
    if !existing.contains(&base) { return base; }

    // collision: truncate stem to make room for ~N suffix
    for n in 1u32..=999 {
        let suffix = format!("~{}", n);
        let room = 8usize.saturating_sub(suffix.len());
        let short: String = stem_clean.chars().take(room).collect();
        let candidate = if ext_trunc.is_empty() {
            format!("{}{}", short, suffix)
        } else {
            format!("{}{}.{}", short, suffix, ext_trunc)
        };
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
    // unreachable in practice
    base
}

fn split_ext(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i+1..]),
        _ => (name, ""),
    }
}

fn clean_chars(s: &str) -> String {
    s.chars()
        .map(|c| c.to_ascii_uppercase())
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    const CD_CAPACITY: u64 = 700_000_000;

    #[test]
    fn blank_disc_has_full_capacity() {
        let d = Disc::new(CD_CAPACITY);
        assert_eq!(d.used_bytes(), 0);
        assert_eq!(d.state, DiscState::Blank);
    }

    #[test]
    fn open_session_transitions_state() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        assert_eq!(d.state, DiscState::SessionOpen);
    }

    #[test]
    fn cannot_open_session_twice() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        assert_eq!(d.open_session(), Err(BurnError::SessionAlreadyOpen));
    }

    #[test]
    fn add_file_without_session_fails() {
        let mut d = Disc::new(CD_CAPACITY);
        let r = d.add_file(BurnFile::new("a.mp3", 1_000_000));
        assert_eq!(r, Err(BurnError::NoOpenSession));
    }

    #[test]
    fn add_file_succeeds_in_open_session() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        d.add_file(BurnFile::new("a.mp3", 1_000_000)).unwrap();
        assert_eq!(d.sessions[0].tracks.len(), 1);
    }

    #[test]
    fn overhead_counts_against_usable() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        let overhead = LEADIN_BYTES + LEADOUT_BYTES;
        assert_eq!(d.used_bytes(), overhead);
        assert_eq!(d.remaining_bytes(), CD_CAPACITY - overhead);
    }

    #[test]
    fn add_too_big_file_fails() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        let r = d.add_file(BurnFile::new("huge.iso", 800_000_000));
        match r {
            Err(BurnError::NotEnoughSpace { .. }) => {}
            other => panic!("expected NotEnoughSpace, got {:?}", other),
        }
    }

    #[test]
    fn close_session_transitions() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        d.close_session().unwrap();
        assert_eq!(d.state, DiscState::SessionClosed);
    }

    #[test]
    fn can_open_another_session_after_close() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        d.close_session().unwrap();
        d.open_session().unwrap();
        assert_eq!(d.sessions.len(), 2);
    }

    #[test]
    fn second_session_pays_another_overhead() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        d.close_session().unwrap();
        d.open_session().unwrap();
        let overhead = LEADIN_BYTES + LEADOUT_BYTES;
        assert_eq!(d.used_bytes(), overhead * 2);
    }

    #[test]
    fn finalize_locks_disc() {
        let mut d = Disc::new(CD_CAPACITY);
        d.open_session().unwrap();
        d.finalize().unwrap();
        assert_eq!(d.state, DiscState::Finalized);
        assert_eq!(d.open_session(), Err(BurnError::DiscFinalized));
        assert_eq!(d.add_file(BurnFile::new("a.mp3", 100)), Err(BurnError::DiscFinalized));
    }

    #[test]
    fn ffd_packs_largest_first() {
        let files = vec![
            BurnFile::new("small.txt", 1_000_000),
            BurnFile::new("big.iso", 500_000_000),
            BurnFile::new("mid.mp4", 200_000_000),
        ];
        let result = pack_files_ffd(&files, CD_CAPACITY, 2);
        // single disc has ~665MB usable; big (500) + mid (200) = 700 won't fit → big alone on disc 1
        assert_eq!(result.discs[0][0].name, "big.iso");
        assert_eq!(result.overflow.len(), 0);
        // total placed
        let total: usize = result.discs.iter().map(|d| d.len()).sum();
        assert_eq!(total, 3);
    }

    #[test]
    fn ffd_overflow_when_no_fit() {
        let files = vec![
            BurnFile::new("a", 400_000_000),
            BurnFile::new("b", 400_000_000),
            BurnFile::new("c", 400_000_000),
        ];
        let result = pack_files_ffd(&files, CD_CAPACITY, 2);
        // 2 discs × 1 file each = 2 placed, 1 overflow
        let total: usize = result.discs.iter().map(|d| d.len()).sum();
        assert_eq!(total, 2);
        assert_eq!(result.overflow.len(), 1);
    }

    #[test]
    fn iso_canonical_uppercases_and_truncates() {
        let empty = BTreeSet::new();
        assert_eq!(canonicalize_iso_name("Report.Document.Txt", &empty), "REPORTDO.TXT");
        assert_eq!(canonicalize_iso_name("verylongfilename.html", &empty), "VERYLONG.HTM");
    }

    #[test]
    fn iso_canonical_collision_suffix() {
        let mut existing = BTreeSet::new();
        existing.insert("REPORT.TXT".to_string());
        let second = canonicalize_iso_name("report.txt", &existing);
        assert_eq!(second, "REPORT~1.TXT");
        existing.insert(second);
        let third = canonicalize_iso_name("report.txt", &existing);
        assert_eq!(third, "REPORT~2.TXT");
    }

    #[test]
    fn iso_canonical_no_extension() {
        let empty = BTreeSet::new();
        assert_eq!(canonicalize_iso_name("README", &empty), "README");
    }

    #[test]
    fn iso_canonical_strips_invalid_chars() {
        let empty = BTreeSet::new();
        assert_eq!(canonicalize_iso_name("my file@2026!.txt", &empty), "MYFILE20.TXT");
    }
}
