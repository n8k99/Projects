//! Sort File Records Utility — parse delimited records, sort by multi-key spec.
//!
//! Fourth Files project. Core move: **records are tables**. A file containing
//! CSV-like lines becomes `Vec<Record>`, where each record is a typed row.
//! Sorting takes a **sort spec** — a list of `(field_index, order, kind)` —
//! so multi-key sort (last name asc, then first name asc, then age desc)
//! composes from primitives.
//!
//! Parsing is explicit: comma-delimited, single-line, no quoting. Typing is
//! inferred per column — strings by default, numeric if all values parse as
//! integers. This is the "cheap CSV" contract: useful for the common case,
//! breaks on anything with embedded commas. That's fine for the Rosetta
//! Stone; a real project uses a library.
//!
//! Round-trip holds: parse → sort → serialise equals parse → (skip sort)
//! → serialise with rows reordered. The exact bytes of each row survive.

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    Str(String),
    Int(i64),
}

impl Field {
    pub fn from_str(s: &str) -> Self {
        if let Ok(n) = s.parse::<i64>() { Field::Int(n) }
        else { Field::Str(s.to_string()) }
    }

    pub fn as_str(&self) -> String {
        match self {
            Field::Str(s) => s.clone(),
            Field::Int(n) => n.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub fields: Vec<Field>,
}

impl Record {
    pub fn parse(line: &str) -> Self {
        Self {
            fields: line.split(',').map(|s| Field::from_str(s.trim())).collect(),
        }
    }

    pub fn serialise(&self) -> String {
        self.fields.iter().map(Field::as_str).collect::<Vec<_>>().join(",")
    }

    pub fn get(&self, i: usize) -> Option<&Field> {
        self.fields.get(i)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order { Asc, Desc }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { Str, Int }

#[derive(Debug, Clone, Copy)]
pub struct SortKey {
    pub field: usize,
    pub order: Order,
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub header: Option<Vec<String>>,
    pub rows: Vec<Record>,
}

impl Table {
    pub fn parse(text: &str, has_header: bool) -> Self {
        let mut lines = text.lines();
        let header = if has_header {
            lines.next().map(|h| h.split(',').map(|s| s.trim().to_string()).collect())
        } else {
            None
        };
        let rows: Vec<Record> = lines
            .filter(|l| !l.is_empty())
            .map(Record::parse)
            .collect();
        Self { header, rows }
    }

    pub fn serialise(&self) -> String {
        let mut out = String::new();
        if let Some(h) = &self.header {
            out.push_str(&h.join(","));
            out.push('\n');
        }
        for r in &self.rows {
            out.push_str(&r.serialise());
            out.push('\n');
        }
        out
    }

    /// Stable sort by the spec — sorts the `rows` field in place.
    pub fn sort_by(&mut self, spec: &[SortKey]) {
        self.rows.sort_by(|a, b| compare_records(a, b, spec));
    }

    /// Resolve a header name to a field index (case-insensitive).
    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.header.as_ref()?.iter().position(|h| h.eq_ignore_ascii_case(name))
    }
}

fn compare_records(a: &Record, b: &Record, spec: &[SortKey]) -> Ordering {
    for key in spec {
        let fa = a.get(key.field);
        let fb = b.get(key.field);
        let ord = compare_fields(fa, fb, key.kind);
        let ord = if key.order == Order::Desc { ord.reverse() } else { ord };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

fn compare_fields(a: Option<&Field>, b: Option<&Field>, kind: Kind) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,   // missing sorts first
        (Some(_), None) => Ordering::Greater,
        (Some(fa), Some(fb)) => match kind {
            Kind::Int => {
                let ia = as_int(fa);
                let ib = as_int(fb);
                ia.cmp(&ib)
            }
            Kind::Str => fa.as_str().cmp(&fb.as_str()),
        },
    }
}

fn as_int(f: &Field) -> i64 {
    match f {
        Field::Int(n) => *n,
        Field::Str(s) => s.parse().unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "name,age,score\nAlice,30,85\nBob,25,92\nCarol,30,78\nDave,25,88\n";

    #[test]
    fn record_parse_round_trip() {
        let r = Record::parse("Alice,30,85");
        assert_eq!(r.serialise(), "Alice,30,85");
    }

    #[test]
    fn field_type_inference() {
        let r = Record::parse("Alice,30,85");
        assert_eq!(r.fields[0], Field::Str("Alice".into()));
        assert_eq!(r.fields[1], Field::Int(30));
        assert_eq!(r.fields[2], Field::Int(85));
    }

    #[test]
    fn table_parses_header() {
        let t = Table::parse(SAMPLE, true);
        assert_eq!(t.header.as_ref().unwrap(), &vec!["name", "age", "score"]);
        assert_eq!(t.rows.len(), 4);
    }

    #[test]
    fn sort_by_single_int_key_asc() {
        let mut t = Table::parse(SAMPLE, true);
        t.sort_by(&[SortKey { field: 1, order: Order::Asc, kind: Kind::Int }]);
        let names: Vec<String> = t.rows.iter().map(|r| r.fields[0].as_str()).collect();
        // Bob, Dave = 25; Alice, Carol = 30. Stable within ties.
        assert_eq!(names, vec!["Bob", "Dave", "Alice", "Carol"]);
    }

    #[test]
    fn sort_by_single_int_key_desc() {
        let mut t = Table::parse(SAMPLE, true);
        t.sort_by(&[SortKey { field: 2, order: Order::Desc, kind: Kind::Int }]);
        let names: Vec<String> = t.rows.iter().map(|r| r.fields[0].as_str()).collect();
        assert_eq!(names, vec!["Bob", "Dave", "Alice", "Carol"]);  // 92,88,85,78
    }

    #[test]
    fn sort_by_two_keys() {
        let mut t = Table::parse(SAMPLE, true);
        t.sort_by(&[
            SortKey { field: 1, order: Order::Asc, kind: Kind::Int },   // age asc
            SortKey { field: 2, order: Order::Desc, kind: Kind::Int },  // score desc
        ]);
        let names: Vec<String> = t.rows.iter().map(|r| r.fields[0].as_str()).collect();
        // age=25: Bob(92), Dave(88); age=30: Alice(85), Carol(78)
        assert_eq!(names, vec!["Bob", "Dave", "Alice", "Carol"]);
    }

    #[test]
    fn sort_is_stable_on_ties() {
        let mut t = Table::parse("a,b\nx,1\ny,1\nz,1\n", true);
        t.sort_by(&[SortKey { field: 1, order: Order::Asc, kind: Kind::Int }]);
        let names: Vec<String> = t.rows.iter().map(|r| r.fields[0].as_str()).collect();
        assert_eq!(names, vec!["x", "y", "z"]);  // original order preserved
    }

    #[test]
    fn field_index_case_insensitive() {
        let t = Table::parse(SAMPLE, true);
        assert_eq!(t.field_index("age"), Some(1));
        assert_eq!(t.field_index("AGE"), Some(1));
        assert_eq!(t.field_index("nope"), None);
    }

    #[test]
    fn round_trip_without_sorting() {
        let t = Table::parse(SAMPLE, true);
        assert_eq!(t.serialise(), SAMPLE);
    }

    #[test]
    fn sort_reorders_but_preserves_rows() {
        let t = Table::parse(SAMPLE, true);
        let original_rows: Vec<String> = t.rows.iter().map(Record::serialise).collect();
        let mut sorted = t.clone();
        sorted.sort_by(&[SortKey { field: 0, order: Order::Asc, kind: Kind::Str }]);
        let mut sorted_rows: Vec<String> = sorted.rows.iter().map(Record::serialise).collect();
        sorted_rows.sort();
        let mut original_sorted = original_rows.clone();
        original_sorted.sort();
        assert_eq!(sorted_rows, original_sorted);
    }

    #[test]
    fn string_sort_is_lexicographic() {
        let mut t = Table::parse("x\nbanana\napple\ncherry\n", true);
        t.sort_by(&[SortKey { field: 0, order: Order::Asc, kind: Kind::Str }]);
        let vals: Vec<String> = t.rows.iter().map(|r| r.fields[0].as_str()).collect();
        assert_eq!(vals, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn missing_field_sorts_first() {
        let mut t = Table::parse("a,b\nx\ny,2\nz,1\n", true);
        t.sort_by(&[SortKey { field: 1, order: Order::Asc, kind: Kind::Int }]);
        let vals: Vec<String> = t.rows.iter().map(|r| r.fields[0].as_str()).collect();
        assert_eq!(vals[0], "x");  // missing second field = sorts first
    }
}
