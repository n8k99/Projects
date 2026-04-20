//! Address Book — contacts with normalised-identifier deduplication and merge.
//!
//! Eighth Databases project. An address book is a map from contact ID to
//! contact record, but the *interesting* part is **deduplication**:
//! finding contacts that represent the same real-world person (same
//! email, same phone, same "name + one matching field") and merging
//! them into one while preserving the union of their data.
//!
//! Distinctive moves:
//!   * **Canonical identifiers** — emails are lowercased (with
//!      gmail-style dot stripping); phones are digit-only (country code
//!      normalised). This is what every CRM does before comparing.
//!   * **Duplicate detection via identifier sharing** — if two contacts
//!      share any canonical email or phone, they're a duplicate group.
//!   * **Merge preserves union** — combined contact keeps all emails,
//!      phones, tags, and the earliest creation timestamp. Notes are
//!      concatenated. Name is the longest non-empty value (heuristic).
//!
//! The patterns here appear in every contacts app: iCloud, Google
//! Contacts, Salesforce, HubSpot, every CRM that handles imports.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    pub id: u64,
    pub name: String,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
    pub tags: Vec<String>,
    pub address: String,
    pub notes: String,
    pub created_ms: i64,
}

impl Contact {
    pub fn new(id: u64, name: &str, created_ms: i64) -> Self {
        Self {
            id, name: name.into(),
            emails: Vec::new(), phones: Vec::new(), tags: Vec::new(),
            address: String::new(), notes: String::new(),
            created_ms,
        }
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.emails.push(email.into()); self
    }

    pub fn with_phone(mut self, phone: &str) -> Self {
        self.phones.push(phone.into()); self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.into()); self
    }

    pub fn canonical_emails(&self) -> Vec<String> {
        self.emails.iter().map(|e| canonical_email(e)).collect()
    }

    pub fn canonical_phones(&self) -> Vec<String> {
        self.phones.iter().map(|p| canonical_phone(p)).collect()
    }
}

/// `Foo.Bar+baz@Gmail.com` → `foobar@gmail.com`
pub fn canonical_email(email: &str) -> String {
    let lower = email.trim().to_lowercase();
    let Some(at) = lower.find('@') else { return lower; };
    let local = &lower[..at];
    let domain = &lower[at + 1..];
    // Strip +tag.
    let local = match local.find('+') {
        Some(i) => &local[..i],
        None => local,
    };
    let canonical_local = if domain == "gmail.com" || domain == "googlemail.com" {
        local.replace('.', "")
    } else {
        local.to_string()
    };
    let canonical_domain = if domain == "googlemail.com" { "gmail.com" } else { domain };
    format!("{canonical_local}@{canonical_domain}")
}

/// Strip non-digits; drop a leading `1` for 11-digit US numbers.
pub fn canonical_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 && digits.starts_with('1') {
        digits[1..].to_string()
    } else {
        digits
    }
}

#[derive(Debug, Clone)]
pub struct AddressBook {
    pub contacts: BTreeMap<u64, Contact>,
    next_id: u64,
}

impl AddressBook {
    pub fn new() -> Self {
        Self { contacts: BTreeMap::new(), next_id: 1 }
    }

    pub fn add(&mut self, contact: Contact) -> u64 {
        let id = contact.id;
        if id >= self.next_id { self.next_id = id + 1; }
        self.contacts.insert(id, contact);
        id
    }

    pub fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn get(&self, id: u64) -> Option<&Contact> { self.contacts.get(&id) }

    /// Return groups of contact IDs that share at least one canonical
    /// email or phone. Each group has size >= 2.
    pub fn find_duplicates(&self) -> Vec<Vec<u64>> {
        // Build reverse index: identifier → set of contact IDs with it.
        let mut by_email: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
        let mut by_phone: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
        for contact in self.contacts.values() {
            for e in contact.canonical_emails() {
                if !e.is_empty() {
                    by_email.entry(e).or_default().insert(contact.id);
                }
            }
            for p in contact.canonical_phones() {
                if !p.is_empty() {
                    by_phone.entry(p).or_default().insert(contact.id);
                }
            }
        }

        // Union-find style: merge IDs that share any identifier.
        let mut parent: BTreeMap<u64, u64> = self.contacts.keys().map(|&id| (id, id)).collect();
        fn find(parent: &mut BTreeMap<u64, u64>, id: u64) -> u64 {
            let p = *parent.get(&id).unwrap();
            if p == id { return id; }
            let root = find(parent, p);
            parent.insert(id, root);
            root
        }
        fn union(parent: &mut BTreeMap<u64, u64>, a: u64, b: u64) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb { parent.insert(ra, rb); }
        }
        for ids in by_email.values().chain(by_phone.values()) {
            let v: Vec<u64> = ids.iter().copied().collect();
            for i in 1..v.len() { union(&mut parent, v[0], v[i]); }
        }

        // Group by root.
        let mut groups: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        let ids: Vec<u64> = self.contacts.keys().copied().collect();
        for id in ids {
            let root = find(&mut parent, id);
            groups.entry(root).or_default().push(id);
        }

        let mut out: Vec<Vec<u64>> = groups.into_values()
            .filter(|g| g.len() >= 2)
            .collect();
        for g in &mut out { g.sort(); }
        out.sort_by(|a, b| a[0].cmp(&b[0]));
        out
    }

    /// Merge a list of contact IDs into a single new contact.
    /// The sources are removed; the merged contact is inserted with a new ID.
    /// Returns the new contact's ID or an error string.
    pub fn merge(&mut self, ids: &[u64]) -> Result<u64, String> {
        if ids.len() < 2 { return Err("need at least 2 ids to merge".into()); }
        let sources: Vec<Contact> = ids.iter()
            .filter_map(|id| self.contacts.get(id).cloned())
            .collect();
        if sources.len() != ids.len() {
            return Err("one or more ids not found".into());
        }

        let merged_id = self.new_id();
        let earliest_created = sources.iter().map(|c| c.created_ms).min().unwrap();
        let longest_name = sources.iter()
            .map(|c| c.name.clone())
            .filter(|n| !n.is_empty())
            .max_by_key(|n| n.len())
            .unwrap_or_default();

        let mut emails = BTreeSet::new();
        let mut phones = BTreeSet::new();
        let mut tags = BTreeSet::new();
        let mut addresses = Vec::new();
        let mut notes = Vec::new();
        for c in &sources {
            for e in &c.emails { if !e.is_empty() { emails.insert(e.clone()); } }
            for p in &c.phones { if !p.is_empty() { phones.insert(p.clone()); } }
            for t in &c.tags { if !t.is_empty() { tags.insert(t.clone()); } }
            if !c.address.is_empty() { addresses.push(c.address.clone()); }
            if !c.notes.is_empty() { notes.push(c.notes.clone()); }
        }
        let merged = Contact {
            id: merged_id,
            name: longest_name,
            emails: emails.into_iter().collect(),
            phones: phones.into_iter().collect(),
            tags: tags.into_iter().collect(),
            address: addresses.join(" | "),
            notes: notes.join("\n---\n"),
            created_ms: earliest_created,
        };

        for id in ids { self.contacts.remove(id); }
        self.contacts.insert(merged_id, merged);
        Ok(merged_id)
    }

    /// Search by name substring (case-insensitive). Returns sorted IDs.
    pub fn search_by_name(&self, needle: &str) -> Vec<u64> {
        let lc = needle.to_lowercase();
        let mut out: Vec<u64> = self.contacts.values()
            .filter(|c| c.name.to_lowercase().contains(&lc))
            .map(|c| c.id)
            .collect();
        out.sort();
        out
    }

    /// Search by canonical email. Returns sorted IDs of contacts with that email.
    pub fn search_by_email(&self, email: &str) -> Vec<u64> {
        let canonical = canonical_email(email);
        let mut out: Vec<u64> = self.contacts.values()
            .filter(|c| c.canonical_emails().iter().any(|e| e == &canonical))
            .map(|c| c.id)
            .collect();
        out.sort();
        out
    }
}

impl Default for AddressBook { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_email_lowercases_and_trims() {
        assert_eq!(canonical_email("  Alice@Example.COM  "), "alice@example.com");
    }

    #[test]
    fn canonical_email_strips_gmail_dots() {
        assert_eq!(canonical_email("foo.bar@gmail.com"), "foobar@gmail.com");
        assert_eq!(canonical_email("f.o.o.bar@gmail.com"), "foobar@gmail.com");
    }

    #[test]
    fn canonical_email_preserves_non_gmail_dots() {
        assert_eq!(canonical_email("foo.bar@example.com"), "foo.bar@example.com");
    }

    #[test]
    fn canonical_email_strips_plus_tag() {
        assert_eq!(canonical_email("alice+newsletter@example.com"),
                    "alice@example.com");
    }

    #[test]
    fn canonical_email_treats_googlemail_as_gmail() {
        assert_eq!(canonical_email("alice@googlemail.com"), "alice@gmail.com");
    }

    #[test]
    fn canonical_phone_strips_non_digits() {
        assert_eq!(canonical_phone("(555) 123-4567"), "5551234567");
        assert_eq!(canonical_phone("+1.555.123.4567"), "5551234567");  // US +1 stripped
    }

    #[test]
    fn canonical_phone_preserves_international() {
        assert_eq!(canonical_phone("+44 20 1234 5678"), "442012345678");
    }

    #[test]
    fn find_duplicates_by_email() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "Alice", 100).with_email("alice@gmail.com"));
        ab.add(Contact::new(2, "Alice J.", 200).with_email("a.l.i.c.e@gmail.com"));
        ab.add(Contact::new(3, "Bob", 300).with_email("bob@gmail.com"));
        let dupes = ab.find_duplicates();
        assert_eq!(dupes.len(), 1);
        assert_eq!(dupes[0], vec![1, 2]);
    }

    #[test]
    fn find_duplicates_by_phone() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "Alice Work", 100).with_phone("555-123-4567"));
        ab.add(Contact::new(2, "Alice Personal", 200).with_phone("(555) 123-4567"));
        let dupes = ab.find_duplicates();
        assert_eq!(dupes.len(), 1);
        assert_eq!(dupes[0], vec![1, 2]);
    }

    #[test]
    fn find_duplicates_via_transitive_link() {
        // A shares phone with B, B shares email with C → all three group.
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "A", 100).with_phone("555-1234"));
        ab.add(Contact::new(2, "B", 200).with_phone("555-1234").with_email("x@example.com"));
        ab.add(Contact::new(3, "C", 300).with_email("x@example.com"));
        let dupes = ab.find_duplicates();
        assert_eq!(dupes.len(), 1);
        assert_eq!(dupes[0], vec![1, 2, 3]);
    }

    #[test]
    fn find_duplicates_returns_empty_for_no_dupes() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "A", 100).with_email("a@gmail.com"));
        ab.add(Contact::new(2, "B", 200).with_email("b@gmail.com"));
        assert!(ab.find_duplicates().is_empty());
    }

    #[test]
    fn merge_unions_all_fields() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "Alice", 100)
            .with_email("alice@gmail.com").with_phone("555-1111").with_tag("work"));
        ab.add(Contact::new(2, "Alice J. Smith", 200)
            .with_email("alice.smith@example.com").with_phone("555-2222").with_tag("family"));
        let merged_id = ab.merge(&[1, 2]).unwrap();
        let merged = ab.get(merged_id).unwrap();
        assert_eq!(merged.name, "Alice J. Smith");  // longest
        assert_eq!(merged.emails.len(), 2);
        assert_eq!(merged.phones.len(), 2);
        assert_eq!(merged.tags.len(), 2);
        assert_eq!(merged.created_ms, 100);  // earliest
        assert!(!ab.contacts.contains_key(&1));
        assert!(!ab.contacts.contains_key(&2));
    }

    #[test]
    fn merge_concatenates_notes() {
        let mut ab = AddressBook::new();
        let mut c1 = Contact::new(1, "A", 100);
        c1.notes = "first note".into();
        ab.add(c1);
        let mut c2 = Contact::new(2, "A", 200);
        c2.notes = "second note".into();
        ab.add(c2);
        let merged_id = ab.merge(&[1, 2]).unwrap();
        assert!(ab.get(merged_id).unwrap().notes.contains("first"));
        assert!(ab.get(merged_id).unwrap().notes.contains("second"));
    }

    #[test]
    fn merge_deduplicates_within_fields() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "A", 100).with_email("shared@example.com").with_tag("work"));
        ab.add(Contact::new(2, "A", 200).with_email("shared@example.com").with_tag("work"));
        let merged_id = ab.merge(&[1, 2]).unwrap();
        let merged = ab.get(merged_id).unwrap();
        assert_eq!(merged.emails.len(), 1);
        assert_eq!(merged.tags.len(), 1);
    }

    #[test]
    fn merge_errors_on_single_id() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "A", 0));
        assert!(ab.merge(&[1]).is_err());
    }

    #[test]
    fn merge_errors_on_unknown_id() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "A", 0));
        assert!(ab.merge(&[1, 999]).is_err());
    }

    #[test]
    fn search_by_name_case_insensitive() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "Alice Smith", 0));
        ab.add(Contact::new(2, "Bob Jones", 0));
        assert_eq!(ab.search_by_name("alice"), vec![1]);
        assert_eq!(ab.search_by_name("SMITH"), vec![1]);
    }

    #[test]
    fn search_by_email_uses_canonical_form() {
        let mut ab = AddressBook::new();
        ab.add(Contact::new(1, "Alice", 0).with_email("Alice.Foo@Gmail.com"));
        assert_eq!(ab.search_by_email("alicefoo@gmail.com"), vec![1]);
        assert_eq!(ab.search_by_email("a.l.i.c.e.foo@googlemail.com"), vec![1]);
    }
}
