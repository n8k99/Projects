//! Code Snippet Manager — templated code fragments with tab-stop expansion.
//!
//! Fifteenth Files project. A snippet is code with **placeholder markers**
//! that an editor can insert and then let the user fill in. The canonical
//! format (TextMate, VS Code, Sublime, yasnippet all use variants):
//!
//! ```text
//! for ${1:item} in ${2:collection}:
//!     ${3:pass}
//! $0
//! ```
//!
//! Expanding produces the literal text (with default values) plus a list
//! of **tab stops** (positions where the cursor lands as the user tabs
//! through). `$0` is the final position after the last stop.
//!
//! G099 introduces:
//!   * **Placeholder parsing** — `${N:default}` and `$0` forms.
//!   * **Tab-stop tracking** — each stop has its (char offset, length, number).
//!   * **Stop ordering** — 1, 2, 3, ..., then 0 (final). Repeats allowed
//!      (typing in `$1` mirrors to every `$1` occurrence).
//!   * **Snippet library** — indexed by trigger + language + tags.

use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabStop {
    pub number: u32,       // stop order (0 = final cursor)
    pub offset: usize,     // char offset in expanded text
    pub length: usize,     // length of the default text (0 if no default)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    pub text: String,
    pub stops: Vec<TabStop>,  // sorted by number ascending, 0 last
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub trigger: String,
    pub language: String,
    pub tags: Vec<String>,
    pub body: String,
    pub description: String,
}

impl Snippet {
    pub fn expand(&self) -> Expansion {
        parse_and_expand(&self.body)
    }
}

#[derive(Debug, Clone)]
pub struct SnippetLibrary {
    pub snippets: Vec<Snippet>,
    by_trigger: HashMap<String, Vec<usize>>,
    by_language: HashMap<String, Vec<usize>>,
}

impl SnippetLibrary {
    pub fn new() -> Self {
        Self {
            snippets: Vec::new(),
            by_trigger: HashMap::new(),
            by_language: HashMap::new(),
        }
    }

    pub fn add(&mut self, snippet: Snippet) {
        let i = self.snippets.len();
        self.by_trigger.entry(snippet.trigger.clone()).or_default().push(i);
        self.by_language.entry(snippet.language.clone()).or_default().push(i);
        self.snippets.push(snippet);
    }

    pub fn find_by_trigger(&self, trigger: &str, language: &str) -> Option<&Snippet> {
        let ids = self.by_trigger.get(trigger)?;
        ids.iter()
            .map(|&i| &self.snippets[i])
            .find(|s| s.language == language)
    }

    pub fn find_by_language(&self, language: &str) -> Vec<&Snippet> {
        let mut out: Vec<&Snippet> = self.by_language.get(language)
            .map(|ids| ids.iter().map(|&i| &self.snippets[i]).collect())
            .unwrap_or_default();
        out.sort_by(|a, b| a.trigger.cmp(&b.trigger));
        out
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&Snippet> {
        let mut out: Vec<&Snippet> = self.snippets.iter()
            .filter(|s| s.tags.iter().any(|t| t == tag))
            .collect();
        out.sort_by(|a, b| a.trigger.cmp(&b.trigger));
        out
    }

    pub fn len(&self) -> usize { self.snippets.len() }
}

impl Default for SnippetLibrary { fn default() -> Self { Self::new() } }

/// Parse a snippet body, emitting the expanded text + tab stops.
fn parse_and_expand(body: &str) -> Expansion {
    let mut out = String::new();
    let mut stops = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = body.chars().collect();
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                // ${N:default} or ${N}
                if let Some(close) = find_matching(&chars, i + 1) {
                    let inside: String = chars[i + 2..close].iter().collect();
                    let (number, default) = if let Some(colon) = inside.find(':') {
                        (inside[..colon].parse::<u32>().unwrap_or(0),
                         inside[colon + 1..].to_string())
                    } else {
                        (inside.parse::<u32>().unwrap_or(0), String::new())
                    };
                    stops.push(TabStop {
                        number,
                        offset: out.chars().count(),
                        length: default.chars().count(),
                    });
                    out.push_str(&default);
                    i = close + 1;
                    continue;
                }
            } else if chars[i + 1].is_ascii_digit() {
                // $N (no default)
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() { j += 1; }
                let number: u32 = chars[i + 1..j].iter().collect::<String>().parse().unwrap_or(0);
                stops.push(TabStop {
                    number,
                    offset: out.chars().count(),
                    length: 0,
                });
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    // Sort: non-zero stops ascending; 0 stop(s) last.
    stops.sort_by(|a, b| {
        match (a.number, b.number) {
            (0, 0) => std::cmp::Ordering::Equal,
            (0, _) => std::cmp::Ordering::Greater,
            (_, 0) => std::cmp::Ordering::Less,
            (x, y) => x.cmp(&y),
        }
    });
    Expansion { text: out, stops }
}

fn find_matching(chars: &[char], open: usize) -> Option<usize> {
    debug_assert_eq!(chars[open], '{');
    let mut depth = 1;
    let mut i = open + 1;
    while i < chars.len() {
        match chars[i] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 { return Some(i); }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Group stops by number — useful for mirroring edits across duplicate $N.
pub fn group_stops_by_number(stops: &[TabStop]) -> BTreeMap<u32, Vec<&TabStop>> {
    let mut m: BTreeMap<u32, Vec<&TabStop>> = BTreeMap::new();
    for s in stops {
        m.entry(s.number).or_default().push(s);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_simple_snippet() {
        let s = Snippet {
            trigger: "for".into(), language: "python".into(),
            tags: vec!["loop".into()],
            body: "for ${1:item} in ${2:collection}:\n    ${3:pass}\n$0".into(),
            description: "for loop".into(),
        };
        let e = s.expand();
        assert_eq!(e.text, "for item in collection:\n    pass\n");
        assert_eq!(e.stops.len(), 4);
        assert_eq!(e.stops[0].number, 1);
        assert_eq!(e.stops[1].number, 2);
        assert_eq!(e.stops[2].number, 3);
        assert_eq!(e.stops[3].number, 0);  // final
    }

    #[test]
    fn expand_stop_offsets_correct() {
        let s = Snippet {
            trigger: "x".into(), language: "y".into(), tags: vec![],
            body: "hello ${1:world}".into(), description: "".into(),
        };
        let e = s.expand();
        assert_eq!(e.text, "hello world");
        assert_eq!(e.stops[0].offset, 6);
        assert_eq!(e.stops[0].length, 5);  // "world"
    }

    #[test]
    fn expand_without_defaults() {
        let s = Snippet {
            trigger: "x".into(), language: "y".into(), tags: vec![],
            body: "a$1b$2c$0".into(), description: "".into(),
        };
        let e = s.expand();
        assert_eq!(e.text, "abc");
        assert_eq!(e.stops.len(), 3);
        assert_eq!(e.stops[0].number, 1);
        assert_eq!(e.stops[0].offset, 1);
        assert_eq!(e.stops[0].length, 0);
        assert_eq!(e.stops[1].number, 2);
        assert_eq!(e.stops[1].offset, 2);
    }

    #[test]
    fn expand_dollar_zero_at_end() {
        let s = Snippet {
            trigger: "x".into(), language: "y".into(), tags: vec![],
            body: "done$0".into(), description: "".into(),
        };
        let e = s.expand();
        assert_eq!(e.text, "done");
        assert_eq!(e.stops[0].number, 0);
        assert_eq!(e.stops[0].offset, 4);
    }

    #[test]
    fn snippet_no_placeholders_empty_stops() {
        let s = Snippet {
            trigger: "x".into(), language: "y".into(), tags: vec![],
            body: "hello world".into(), description: "".into(),
        };
        let e = s.expand();
        assert_eq!(e.text, "hello world");
        assert!(e.stops.is_empty());
    }

    #[test]
    fn duplicate_stop_numbers_preserved() {
        let s = Snippet {
            trigger: "x".into(), language: "y".into(), tags: vec![],
            body: "${1:x} and ${1:y}".into(), description: "".into(),
        };
        let e = s.expand();
        assert_eq!(e.text, "x and y");
        assert_eq!(e.stops.len(), 2);
        assert_eq!(e.stops[0].number, 1);
        assert_eq!(e.stops[1].number, 1);
    }

    #[test]
    fn library_find_by_trigger_filters_by_language() {
        let mut lib = SnippetLibrary::new();
        lib.add(Snippet {
            trigger: "for".into(), language: "python".into(),
            tags: vec![], body: "for $1 in $2: pass".into(),
            description: "".into(),
        });
        lib.add(Snippet {
            trigger: "for".into(), language: "rust".into(),
            tags: vec![], body: "for $1 in $2 { }".into(),
            description: "".into(),
        });
        let py = lib.find_by_trigger("for", "python").unwrap();
        assert_eq!(py.body, "for $1 in $2: pass");
        let rs = lib.find_by_trigger("for", "rust").unwrap();
        assert_eq!(rs.body, "for $1 in $2 { }");
    }

    #[test]
    fn library_find_by_language_sorted() {
        let mut lib = SnippetLibrary::new();
        lib.add(Snippet { trigger: "while".into(), language: "py".into(),
                          tags: vec![], body: "".into(), description: "".into() });
        lib.add(Snippet { trigger: "for".into(), language: "py".into(),
                          tags: vec![], body: "".into(), description: "".into() });
        let all = lib.find_by_language("py");
        assert_eq!(all[0].trigger, "for");
        assert_eq!(all[1].trigger, "while");
    }

    #[test]
    fn library_find_by_tag() {
        let mut lib = SnippetLibrary::new();
        lib.add(Snippet { trigger: "x".into(), language: "y".into(),
                          tags: vec!["loop".into()], body: "".into(),
                          description: "".into() });
        lib.add(Snippet { trigger: "z".into(), language: "y".into(),
                          tags: vec!["branch".into()], body: "".into(),
                          description: "".into() });
        let loops = lib.find_by_tag("loop");
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].trigger, "x");
    }

    #[test]
    fn group_stops_by_number() {
        let stops = vec![
            TabStop { number: 1, offset: 0, length: 3 },
            TabStop { number: 2, offset: 5, length: 0 },
            TabStop { number: 1, offset: 10, length: 3 },
            TabStop { number: 0, offset: 15, length: 0 },
        ];
        let grouped = super::group_stops_by_number(&stops);
        assert_eq!(grouped.get(&1).unwrap().len(), 2);
        assert_eq!(grouped.get(&2).unwrap().len(), 1);
        assert_eq!(grouped.get(&0).unwrap().len(), 1);
    }

    #[test]
    fn empty_snippet_expands_to_empty() {
        let s = Snippet {
            trigger: "x".into(), language: "y".into(), tags: vec![],
            body: "".into(), description: "".into(),
        };
        let e = s.expand();
        assert_eq!(e.text, "");
        assert!(e.stops.is_empty());
    }

    #[test]
    fn stop_ordering_puts_zero_last() {
        let s = Snippet {
            trigger: "x".into(), language: "y".into(), tags: vec![],
            body: "$0${1:a}$2".into(), description: "".into(),
        };
        let e = s.expand();
        // Text-order: $0, then "a" (1), then empty ($2)
        // After sort: 1, 2, 0
        assert_eq!(e.stops[0].number, 1);
        assert_eq!(e.stops[1].number, 2);
        assert_eq!(e.stops[2].number, 0);
    }
}
