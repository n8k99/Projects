//! Library Catalog — three-layer identity, hierarchical subjects, FIFO holds, renewal.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyStatus {
    Available,
    CheckedOut,
    Lost,
    Withdrawn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoldStatus {
    Waiting,
    Fulfilled,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Title {
    pub id: String,
    pub name: String,
    pub authors: Vec<String>,
    /// Dot-delimited path, e.g. "500.512.5".
    pub subject_path: String,
    pub isbn: String,
}

#[derive(Debug, Clone)]
pub struct Copy {
    pub id: String,
    pub title_id: String,
    pub location: String,
    pub status: CopyStatus,
}

#[derive(Debug, Clone)]
pub struct Patron {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Loan {
    pub id: u64,
    pub copy_id: String,
    pub patron_id: String,
    /// Unix seconds.
    pub checked_out_at: u64,
    pub due_at: u64,
    pub returned_at: Option<u64>,
    pub renewal_count: u32,
}

impl Loan {
    pub fn is_active(&self) -> bool {
        self.returned_at.is_none()
    }

    pub fn is_overdue(&self, now: u64) -> bool {
        self.is_active() && now > self.due_at
    }
}

#[derive(Debug, Clone)]
pub struct Hold {
    pub id: u64,
    pub title_id: String,
    pub patron_id: String,
    pub placed_at: u64,
    pub status: HoldStatus,
    pub fulfilled_copy_id: Option<String>,
}

#[derive(Debug)]
pub enum CatalogError {
    UnknownTitle(String),
    UnknownCopy(String),
    UnknownPatron(String),
    UnknownLoan(u64),
    UnknownHold(u64),
    Duplicate(String),
    AlreadyOut(String),
    AlreadyReturned(u64),
    HoldNotWaiting(u64),
    RenewalMaxed(u64),
    RenewalBlockedByHold,
}

pub struct LibraryCatalog {
    pub titles: HashMap<String, Title>,
    pub copies: HashMap<String, Copy>,
    pub patrons: HashMap<String, Patron>,
    pub loans: HashMap<u64, Loan>,
    pub holds: HashMap<u64, Hold>,
    next_loan_id: u64,
    next_hold_id: u64,
}

impl LibraryCatalog {
    pub const SECONDS_PER_DAY: u64 = 86_400;
    pub const DEFAULT_LOAN_DAYS: u64 = 21;
    pub const MAX_RENEWALS: u32 = 2;

    pub fn new() -> Self {
        Self {
            titles: HashMap::new(),
            copies: HashMap::new(),
            patrons: HashMap::new(),
            loans: HashMap::new(),
            holds: HashMap::new(),
            next_loan_id: 1,
            next_hold_id: 1,
        }
    }

    pub fn add_title(&mut self, t: Title) -> Result<(), CatalogError> {
        if self.titles.contains_key(&t.id) {
            return Err(CatalogError::Duplicate(t.id));
        }
        self.titles.insert(t.id.clone(), t);
        Ok(())
    }

    pub fn add_copy(&mut self, c: Copy) -> Result<(), CatalogError> {
        if !self.titles.contains_key(&c.title_id) {
            return Err(CatalogError::UnknownTitle(c.title_id));
        }
        if self.copies.contains_key(&c.id) {
            return Err(CatalogError::Duplicate(c.id));
        }
        self.copies.insert(c.id.clone(), c);
        Ok(())
    }

    pub fn add_patron(&mut self, p: Patron) -> Result<(), CatalogError> {
        if self.patrons.contains_key(&p.id) {
            return Err(CatalogError::Duplicate(p.id));
        }
        self.patrons.insert(p.id.clone(), p);
        Ok(())
    }

    pub fn check_out(
        &mut self,
        copy_id: &str,
        patron_id: &str,
        duration_days: u64,
        now: u64,
    ) -> Result<u64, CatalogError> {
        if !self.patrons.contains_key(patron_id) {
            return Err(CatalogError::UnknownPatron(patron_id.to_string()));
        }
        let copy = self
            .copies
            .get_mut(copy_id)
            .ok_or_else(|| CatalogError::UnknownCopy(copy_id.to_string()))?;
        if copy.status != CopyStatus::Available {
            return Err(CatalogError::AlreadyOut(copy_id.to_string()));
        }
        copy.status = CopyStatus::CheckedOut;

        let id = self.next_loan_id;
        self.next_loan_id += 1;
        self.loans.insert(
            id,
            Loan {
                id,
                copy_id: copy_id.to_string(),
                patron_id: patron_id.to_string(),
                checked_out_at: now,
                due_at: now + duration_days * Self::SECONDS_PER_DAY,
                returned_at: None,
                renewal_count: 0,
            },
        );
        Ok(id)
    }

    /// Returns the loan id and, if any hold was fulfilled, its id.
    pub fn return_copy(&mut self, loan_id: u64, now: u64) -> Result<Option<u64>, CatalogError> {
        let (copy_id, title_id) = {
            let loan = self
                .loans
                .get_mut(&loan_id)
                .ok_or(CatalogError::UnknownLoan(loan_id))?;
            if loan.returned_at.is_some() {
                return Err(CatalogError::AlreadyReturned(loan_id));
            }
            loan.returned_at = Some(now);
            let copy_id = loan.copy_id.clone();
            let title_id = self.copies.get(&copy_id)
                .map(|c| c.title_id.clone())
                .unwrap_or_default();
            (copy_id, title_id)
        };
        if let Some(c) = self.copies.get_mut(&copy_id) {
            c.status = CopyStatus::Available;
        }
        Ok(self.fulfill_next_hold(&title_id, &copy_id))
    }

    fn fulfill_next_hold(&mut self, title_id: &str, copy_id: &str) -> Option<u64> {
        let mut candidates: Vec<&Hold> = self
            .holds
            .values()
            .filter(|h| h.title_id == title_id && h.status == HoldStatus::Waiting)
            .collect();
        candidates.sort_by_key(|h| h.placed_at);
        let id = candidates.first()?.id;
        let hold = self.holds.get_mut(&id)?;
        hold.status = HoldStatus::Fulfilled;
        hold.fulfilled_copy_id = Some(copy_id.to_string());
        Some(id)
    }

    pub fn renew(&mut self, loan_id: u64, additional_days: u64) -> Result<(), CatalogError> {
        let title_id = {
            let loan = self
                .loans
                .get(&loan_id)
                .ok_or(CatalogError::UnknownLoan(loan_id))?;
            if !loan.is_active() {
                return Err(CatalogError::AlreadyReturned(loan_id));
            }
            if loan.renewal_count >= Self::MAX_RENEWALS {
                return Err(CatalogError::RenewalMaxed(loan_id));
            }
            self.copies
                .get(&loan.copy_id)
                .map(|c| c.title_id.clone())
                .unwrap_or_default()
        };
        let has_waiting = self
            .holds
            .values()
            .any(|h| h.title_id == title_id && h.status == HoldStatus::Waiting);
        if has_waiting {
            return Err(CatalogError::RenewalBlockedByHold);
        }
        let loan = self.loans.get_mut(&loan_id).unwrap();
        loan.due_at += additional_days * Self::SECONDS_PER_DAY;
        loan.renewal_count += 1;
        Ok(())
    }

    pub fn place_hold(&mut self, title_id: &str, patron_id: &str, now: u64) -> Result<u64, CatalogError> {
        if !self.titles.contains_key(title_id) {
            return Err(CatalogError::UnknownTitle(title_id.to_string()));
        }
        if !self.patrons.contains_key(patron_id) {
            return Err(CatalogError::UnknownPatron(patron_id.to_string()));
        }
        let id = self.next_hold_id;
        self.next_hold_id += 1;
        self.holds.insert(
            id,
            Hold {
                id,
                title_id: title_id.to_string(),
                patron_id: patron_id.to_string(),
                placed_at: now,
                status: HoldStatus::Waiting,
                fulfilled_copy_id: None,
            },
        );
        Ok(id)
    }

    pub fn cancel_hold(&mut self, hold_id: u64) -> Result<(), CatalogError> {
        let hold = self
            .holds
            .get_mut(&hold_id)
            .ok_or(CatalogError::UnknownHold(hold_id))?;
        if hold.status != HoldStatus::Waiting {
            return Err(CatalogError::HoldNotWaiting(hold_id));
        }
        hold.status = HoldStatus::Cancelled;
        Ok(())
    }

    pub fn hold_queue(&self, title_id: &str) -> Vec<&Hold> {
        let mut out: Vec<&Hold> = self
            .holds
            .values()
            .filter(|h| h.title_id == title_id && h.status == HoldStatus::Waiting)
            .collect();
        out.sort_by_key(|h| h.placed_at);
        out
    }

    pub fn copies_of(&self, title_id: &str) -> Vec<&Copy> {
        self.copies.values().filter(|c| c.title_id == title_id).collect()
    }

    pub fn available_copies(&self, title_id: &str) -> Vec<&Copy> {
        self.copies
            .values()
            .filter(|c| c.title_id == title_id && c.status == CopyStatus::Available)
            .collect()
    }

    pub fn is_available(&self, title_id: &str) -> bool {
        !self.available_copies(title_id).is_empty()
    }

    /// Tree-prefix subject query.
    pub fn search_by_subject(&self, prefix: &str) -> Vec<&Title> {
        self.titles
            .values()
            .filter(|t| t.subject_path == prefix || t.subject_path.starts_with(&format!("{prefix}.")))
            .collect()
    }

    pub fn search_by_author(&self, author: &str) -> Vec<&Title> {
        self.titles
            .values()
            .filter(|t| t.authors.iter().any(|a| a == author))
            .collect()
    }

    pub fn overdue(&self, now: u64) -> Vec<&Loan> {
        self.loans.values().filter(|l| l.is_overdue(now)).collect()
    }
}

impl Default for LibraryCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> LibraryCatalog {
        let mut c = LibraryCatalog::new();
        c.add_title(Title {
            id: "T1".into(), name: "1984".into(), authors: vec!["Orwell".into()],
            subject_path: "800.813".into(), isbn: "".into(),
        }).unwrap();
        c.add_title(Title {
            id: "T2".into(), name: "Algebra".into(), authors: vec!["Pinter".into()],
            subject_path: "500.512.02".into(), isbn: "".into(),
        }).unwrap();
        c.add_copy(Copy { id: "C1".into(), title_id: "T1".into(),
            location: "Main".into(), status: CopyStatus::Available }).unwrap();
        c.add_copy(Copy { id: "C2".into(), title_id: "T1".into(),
            location: "Branch".into(), status: CopyStatus::Available }).unwrap();
        c.add_patron(Patron { id: "P1".into(), name: "N".into() }).unwrap();
        c.add_patron(Patron { id: "P2".into(), name: "K".into() }).unwrap();
        c
    }

    #[test]
    fn checkout_marks_copy_unavailable() {
        let mut c = setup();
        c.check_out("C1", "P1", 21, 1000).unwrap();
        assert_eq!(c.available_copies("T1").len(), 1);
    }

    #[test]
    fn subject_prefix_is_tree_query() {
        let c = setup();
        let math = c.search_by_subject("500");
        let names: Vec<&str> = math.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(names, vec!["T2"]);
        let pure = c.search_by_subject("500.512");
        assert_eq!(pure.len(), 1);
        let all = c.search_by_subject("500.999");
        assert!(all.is_empty());
    }

    #[test]
    fn hold_fulfilled_on_return_in_fifo_order() {
        let mut c = setup();
        let l1 = c.check_out("C1", "P1", 21, 1000).unwrap();
        c.check_out("C2", "P2", 21, 1000).unwrap();
        let h1 = c.place_hold("T1", "P1", 1100).unwrap();
        let _ = c.place_hold("T1", "P2", 1200).unwrap();
        let fulfilled = c.return_copy(l1, 2000).unwrap();
        assert_eq!(fulfilled, Some(h1));
        assert_eq!(c.holds[&h1].status, HoldStatus::Fulfilled);
    }

    #[test]
    fn renewal_blocked_when_holds_exist() {
        let mut c = setup();
        let l1 = c.check_out("C1", "P1", 21, 1000).unwrap();
        c.place_hold("T1", "P2", 1100).unwrap();
        assert!(matches!(c.renew(l1, 21), Err(CatalogError::RenewalBlockedByHold)));
    }

    #[test]
    fn renewal_succeeds_without_holds() {
        let mut c = setup();
        let l1 = c.check_out("C1", "P1", 21, 1000).unwrap();
        c.renew(l1, 21).unwrap();
        assert_eq!(c.loans[&l1].renewal_count, 1);
    }
}
