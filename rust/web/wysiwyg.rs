//! WYSIWYG Editor — document model where edit and render are the same structure.
//!
//! The essence of WYSIWYG is that there is no distinction between "source code"
//! and "rendered output." The document IS the display. Operations on the
//! document produce the display directly; there is no parsing, no compilation,
//! no intermediate format.
//!
//! The document is a list of **runs**, each a contiguous span of characters
//! sharing the same set of formatting attributes. Insertions, deletions, and
//! style changes reshape the runs — splitting where attrs change, merging
//! where adjacent runs end up with identical attrs. Positions are user-facing
//! character offsets; runs are the implementation detail.
//!
//! This is the first Rosetta Stone project where the data structure is
//! *itself the thing the user sees*. Every prior project had a clean
//! separation between the model and its display.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Attr {
    Bold,
    Italic,
    Underline,
    H1,
    H2,
}

#[derive(Debug, Clone)]
pub struct Run {
    pub text: String,
    pub attrs: BTreeSet<Attr>,
}

impl Run {
    fn new(text: &str, attrs: BTreeSet<Attr>) -> Self {
        Self { text: text.into(), attrs }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Document {
    runs: Vec<Run>,
}

impl Document {
    pub fn new() -> Self { Self { runs: Vec::new() } }

    pub fn len(&self) -> usize {
        self.runs.iter().map(|r| r.text.chars().count()).sum()
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Insert `text` at character offset `pos` with the given attributes.
    pub fn insert(&mut self, pos: usize, text: &str, attrs: BTreeSet<Attr>) {
        assert!(pos <= self.len(), "insert position out of range");
        // Split at pos, then insert a new run between the halves.
        let (left, right) = self.split_at(pos);
        self.runs = left;
        self.runs.push(Run::new(text, attrs));
        self.runs.extend(right);
        self.normalize();
    }

    /// Delete characters in the half-open range `[start, end)`.
    pub fn delete(&mut self, start: usize, end: usize) {
        assert!(start <= end && end <= self.len(), "delete range out of bounds");
        if start == end { return; }
        let (left, rest) = self.split_at(start);
        let mut rest_doc = Document { runs: rest };
        let (_deleted, right) = rest_doc.split_at(end - start);
        self.runs = left;
        self.runs.extend(right);
        self.normalize();
    }

    /// Add `attr` to every character in `[start, end)`. Existing attrs preserved.
    pub fn apply_style(&mut self, start: usize, end: usize, attr: Attr) {
        self.modify_attrs(start, end, |attrs| { attrs.insert(attr); });
    }

    /// Remove `attr` from every character in `[start, end)`.
    pub fn remove_style(&mut self, start: usize, end: usize, attr: Attr) {
        self.modify_attrs(start, end, |attrs| { attrs.remove(&attr); });
    }

    fn modify_attrs<F: Fn(&mut BTreeSet<Attr>)>(&mut self, start: usize, end: usize, f: F) {
        assert!(start <= end && end <= self.len(), "style range out of bounds");
        if start == end { return; }
        let (left, rest) = self.split_at(start);
        let mut rest_doc = Document { runs: rest };
        let (middle, right) = rest_doc.split_at(end - start);
        let mut modified = middle;
        for r in modified.iter_mut() { f(&mut r.attrs); }
        self.runs = left;
        self.runs.extend(modified);
        self.runs.extend(right);
        self.normalize();
    }

    /// Plain text with formatting stripped.
    pub fn to_plain(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }

    /// Render to HTML-like markup. Block-level attrs (H1, H2) wrap the run;
    /// inline attrs (Bold, Italic, Underline) wrap inside. Tag order is
    /// deterministic so the rendering is stable.
    pub fn to_html(&self) -> String {
        let mut out = String::new();
        for r in &self.runs {
            let text = html_escape(&r.text);
            let mut open = String::new();
            let mut close = String::new();
            // Block-level first (outermost).
            for a in &r.attrs {
                match a {
                    Attr::H1 => { open.push_str("<h1>"); close.insert_str(0, "</h1>"); }
                    Attr::H2 => { open.push_str("<h2>"); close.insert_str(0, "</h2>"); }
                    _ => {}
                }
            }
            for a in &r.attrs {
                match a {
                    Attr::Bold => { open.push_str("<b>"); close.insert_str(0, "</b>"); }
                    Attr::Italic => { open.push_str("<i>"); close.insert_str(0, "</i>"); }
                    Attr::Underline => { open.push_str("<u>"); close.insert_str(0, "</u>"); }
                    _ => {}
                }
            }
            out.push_str(&open);
            out.push_str(&text);
            out.push_str(&close);
        }
        out
    }

    pub fn runs(&self) -> &[Run] { &self.runs }

    /// Split the document at character offset `pos`; return (left runs, right runs).
    /// Does not modify self.
    fn split_at(&self, pos: usize) -> (Vec<Run>, Vec<Run>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut seen = 0;
        for r in &self.runs {
            let len = r.text.chars().count();
            if pos >= seen + len {
                left.push(r.clone());
            } else if pos <= seen {
                right.push(r.clone());
            } else {
                let split_at = pos - seen;
                let (l, r2) = split_string_at(&r.text, split_at);
                left.push(Run::new(&l, r.attrs.clone()));
                right.push(Run::new(&r2, r.attrs.clone()));
            }
            seen += len;
        }
        (left, right)
    }

    /// Merge adjacent runs with identical attrs; drop empty runs.
    fn normalize(&mut self) {
        let mut out: Vec<Run> = Vec::with_capacity(self.runs.len());
        for r in self.runs.drain(..) {
            if r.text.is_empty() { continue; }
            match out.last_mut() {
                Some(last) if last.attrs == r.attrs => last.text.push_str(&r.text),
                _ => out.push(r),
            }
        }
        self.runs = out;
    }
}

fn split_string_at(s: &str, char_pos: usize) -> (String, String) {
    let mut byte_pos = 0;
    for (i, _) in s.char_indices().take(char_pos) {
        byte_pos = i;
    }
    // byte_pos is now the start of the (char_pos-1)th char; we want the start
    // of the char_pos-th char.
    if char_pos == 0 {
        ("".into(), s.into())
    } else if let Some((i, _)) = s.char_indices().nth(char_pos) {
        (s[..i].into(), s[i..].into())
    } else {
        (s.into(), "".into())
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(list: &[Attr]) -> BTreeSet<Attr> { list.iter().copied().collect() }

    #[test]
    fn insert_into_empty_document() {
        let mut d = Document::new();
        d.insert(0, "hello world", attrs(&[]));
        assert_eq!(d.to_plain(), "hello world");
        assert_eq!(d.len(), 11);
    }

    #[test]
    fn apply_style_splits_runs() {
        let mut d = Document::new();
        d.insert(0, "hello world", attrs(&[]));
        d.apply_style(0, 5, Attr::Bold);
        assert_eq!(d.runs().len(), 2, "expected split into bold+plain");
        assert_eq!(d.runs()[0].text, "hello");
        assert!(d.runs()[0].attrs.contains(&Attr::Bold));
        assert_eq!(d.runs()[1].text, " world");
        assert!(d.runs()[1].attrs.is_empty());
    }

    #[test]
    fn remove_style_from_middle_splits_into_three() {
        let mut d = Document::new();
        d.insert(0, "hello", attrs(&[Attr::Bold]));
        d.remove_style(1, 4, Attr::Bold);
        // Expect: [bold "h"] [plain "ell"] [bold "o"]
        assert_eq!(d.runs().len(), 3);
        assert_eq!(d.runs()[0].text, "h");
        assert_eq!(d.runs()[1].text, "ell");
        assert_eq!(d.runs()[2].text, "o");
    }

    #[test]
    fn adjacent_identical_runs_merge() {
        let mut d = Document::new();
        d.insert(0, "foo", attrs(&[Attr::Bold]));
        d.insert(3, "bar", attrs(&[Attr::Bold]));
        assert_eq!(d.runs().len(), 1, "adjacent bold runs must merge");
        assert_eq!(d.runs()[0].text, "foobar");
    }

    #[test]
    fn delete_removes_range_and_merges() {
        let mut d = Document::new();
        d.insert(0, "hello world", attrs(&[]));
        d.delete(5, 6);   // delete the space
        assert_eq!(d.to_plain(), "helloworld");
        assert_eq!(d.runs().len(), 1);
    }

    #[test]
    fn composability_bold_plus_italic() {
        let mut d = Document::new();
        d.insert(0, "x", attrs(&[Attr::Bold]));
        d.apply_style(0, 1, Attr::Italic);
        let r = &d.runs()[0];
        assert!(r.attrs.contains(&Attr::Bold));
        assert!(r.attrs.contains(&Attr::Italic));
    }

    #[test]
    fn html_renders_attributes() {
        let mut d = Document::new();
        d.insert(0, "hi", attrs(&[Attr::Bold, Attr::Italic]));
        // <b><i>hi</i></b> or <i><b>hi</b></i> — Bold iterates before Italic.
        let html = d.to_html();
        assert!(html.contains("<b>"));
        assert!(html.contains("<i>"));
        assert!(html.contains("hi"));
    }

    #[test]
    fn html_escapes_special_chars() {
        let mut d = Document::new();
        d.insert(0, "a<b&c", attrs(&[]));
        assert_eq!(d.to_html(), "a&lt;b&amp;c");
    }
}
