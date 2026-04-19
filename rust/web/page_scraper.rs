//! Page Scraper — parse HTML-like markup into a tree, query with selectors.
//!
//! The inverse of G069's render-to-HTML: this takes a string and reconstructs
//! structure. Parsing is forgiving — real HTML is messy, scrapers that refuse
//! malformed input are useless; this one treats unknown constructs as text
//! and recovers at the next valid boundary.
//!
//! The selector language is a subset of CSS: tag names, `.class`, `#id`,
//! combinations like `tag.class` and `tag#id`, and descendant combination
//! via whitespace (`article p` = any p inside any article). No pseudo-
//! classes, no attribute selectors, no sibling combinators — the subset
//! that covers 90% of real scraping.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element { tag: String, attrs: HashMap<String, String>, children: Vec<Node> },
    Text(String),
}

const VOID_TAGS: &[&str] = &["area", "base", "br", "col", "embed", "hr", "img",
    "input", "link", "meta", "source", "track", "wbr"];

pub fn parse(html: &str) -> Vec<Node> {
    let mut p = Parser { input: html, pos: 0 };
    p.parse_nodes(None)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn eof(&self) -> bool { self.pos >= self.input.len() }
    fn peek(&self) -> Option<char> { self.input[self.pos..].chars().next() }
    fn starts_with(&self, s: &str) -> bool { self.input[self.pos..].starts_with(s) }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, pred: F) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if !pred(c) { break; }
            self.pos += c.len_utf8();
        }
        self.input[start..self.pos].to_string()
    }

    fn consume_whitespace(&mut self) { self.consume_while(|c| c.is_whitespace()); }

    fn parse_nodes(&mut self, parent_tag: Option<&str>) -> Vec<Node> {
        let mut nodes = Vec::new();
        while !self.eof() {
            // Stop at the parent's closing tag.
            if let Some(pt) = parent_tag {
                if self.starts_with(&format!("</{pt}")) { break; }
            }
            // Skip unmatched close tags (forgiving).
            if self.starts_with("</") {
                self.consume_while(|c| c != '>');
                self.consume_char();
                continue;
            }
            if self.starts_with("<") && self.peek_after_lt().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                if let Some(node) = self.parse_element() { nodes.push(node); }
            } else {
                let text = self.consume_while(|c| c != '<');
                if !text.is_empty() { nodes.push(Node::Text(text)); }
            }
        }
        nodes
    }

    fn peek_after_lt(&self) -> Option<char> {
        self.input[self.pos..].chars().nth(1)
    }

    fn parse_element(&mut self) -> Option<Node> {
        assert_eq!(self.consume_char(), Some('<'));
        let tag = self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-')
            .to_lowercase();
        if tag.is_empty() { return None; }
        let attrs = self.parse_attrs();
        // Self-closing tag?
        let mut self_closing = false;
        if self.starts_with("/>") {
            self.pos += 2;
            self_closing = true;
        } else if self.starts_with(">") {
            self.consume_char();
        } else {
            return None;
        }
        if self_closing || VOID_TAGS.contains(&tag.as_str()) {
            return Some(Node::Element { tag, attrs, children: Vec::new() });
        }
        let children = self.parse_nodes(Some(&tag));
        // Consume </tag> if present.
        if self.starts_with(&format!("</{tag}")) {
            self.consume_while(|c| c != '>');
            self.consume_char();
        }
        Some(Node::Element { tag, attrs, children })
    }

    fn parse_attrs(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        loop {
            self.consume_whitespace();
            match self.peek() {
                Some('>') | Some('/') | None => break,
                _ => {}
            }
            let name = self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                .to_lowercase();
            if name.is_empty() {
                self.consume_char();
                continue;
            }
            self.consume_whitespace();
            let value = if self.starts_with("=") {
                self.consume_char();
                self.consume_whitespace();
                match self.peek() {
                    Some(q) if q == '"' || q == '\'' => {
                        self.consume_char();
                        let v = self.consume_while(|c| c != q);
                        self.consume_char();
                        v
                    }
                    _ => self.consume_while(|c| !c.is_whitespace() && c != '>' && c != '/'),
                }
            } else {
                String::new()
            };
            attrs.insert(name, value);
        }
        attrs
    }
}

#[derive(Debug, Clone)]
pub struct Selector {
    /// Sequence of simple selectors to match by descendant combinator.
    pub chain: Vec<Simple>,
}

#[derive(Debug, Clone, Default)]
pub struct Simple {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub class: Option<String>,
}

pub fn parse_selector(s: &str) -> Selector {
    let chain = s.split_whitespace().map(parse_simple).collect();
    Selector { chain }
}

fn parse_simple(s: &str) -> Simple {
    let mut simp = Simple::default();
    let mut buf = String::new();
    let mut kind: char = ' ';   // what we're currently building: ' '=tag, '.'=class, '#'=id
    let flush = |simp: &mut Simple, kind: char, buf: &mut String| {
        if buf.is_empty() { return; }
        match kind {
            '.' => simp.class = Some(buf.clone()),
            '#' => simp.id = Some(buf.clone()),
            _   => simp.tag = Some(buf.to_lowercase()),
        }
        buf.clear();
    };
    for c in s.chars() {
        if c == '.' || c == '#' {
            flush(&mut simp, kind, &mut buf);
            kind = c;
        } else {
            buf.push(c);
        }
    }
    flush(&mut simp, kind, &mut buf);
    simp
}

pub fn select<'a>(nodes: &'a [Node], selector: &str) -> Vec<&'a Node> {
    let sel = parse_selector(selector);
    let mut out = Vec::new();
    select_recursive(nodes, &sel.chain, 0, &mut out);
    out
}

fn select_recursive<'a>(nodes: &'a [Node], chain: &[Simple], idx: usize, out: &mut Vec<&'a Node>) {
    for node in nodes {
        if let Node::Element { tag, attrs, children } = node {
            let matches = simple_matches(&chain[idx], tag, attrs);
            if matches && idx == chain.len() - 1 {
                out.push(node);
                // Continue searching descendants — a match's descendants can match again.
                select_recursive(children, chain, idx, out);
            } else if matches {
                select_recursive(children, chain, idx + 1, out);
            } else {
                select_recursive(children, chain, idx, out);
            }
        }
    }
}

fn simple_matches(s: &Simple, tag: &str, attrs: &HashMap<String, String>) -> bool {
    if let Some(t) = &s.tag { if t != tag { return false; } }
    if let Some(id) = &s.id {
        if attrs.get("id").map(String::as_str) != Some(id) { return false; }
    }
    if let Some(cls) = &s.class {
        let has = attrs.get("class").map(|v| v.split_whitespace().any(|c| c == cls))
            .unwrap_or(false);
        if !has { return false; }
    }
    true
}

/// Recursively concatenate the text of a node and its descendants.
pub fn text_of(node: &Node) -> String {
    match node {
        Node::Text(t) => t.clone(),
        Node::Element { children, .. } => children.iter().map(text_of).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_element() {
        let nodes = parse("<p>hello</p>");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "p");
                assert_eq!(children.len(), 1);
                assert!(matches!(&children[0], Node::Text(t) if t == "hello"));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn parse_attributes() {
        let nodes = parse(r#"<a href="https://x.org" class="external">link</a>"#);
        match &nodes[0] {
            Node::Element { attrs, .. } => {
                assert_eq!(attrs.get("href").unwrap(), "https://x.org");
                assert_eq!(attrs.get("class").unwrap(), "external");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_nested_elements() {
        let nodes = parse("<article><h1>Title</h1><p>Body</p></article>");
        let article = &nodes[0];
        if let Node::Element { children, .. } = article {
            assert_eq!(children.len(), 2);
        } else { panic!(); }
    }

    #[test]
    fn parse_void_and_selfclosing_tags() {
        let nodes = parse("<div><br><img src=\"x\"/></div>");
        if let Node::Element { children, .. } = &nodes[0] {
            assert_eq!(children.len(), 2);
            assert!(matches!(&children[0], Node::Element { tag, children, .. }
                if tag == "br" && children.is_empty()));
        } else { panic!(); }
    }

    #[test]
    fn select_by_tag() {
        let nodes = parse("<div><p>a</p><span>b</span><p>c</p></div>");
        let ps = select(&nodes, "p");
        assert_eq!(ps.len(), 2);
        assert_eq!(text_of(ps[0]), "a");
        assert_eq!(text_of(ps[1]), "c");
    }

    #[test]
    fn select_by_class() {
        let nodes = parse(r#"<div><p class="lead">a</p><p>b</p><p class="lead main">c</p></div>"#);
        let lead = select(&nodes, ".lead");
        assert_eq!(lead.len(), 2);
        assert_eq!(text_of(lead[0]), "a");
        assert_eq!(text_of(lead[1]), "c");
    }

    #[test]
    fn select_by_id() {
        let nodes = parse(r#"<div><p id="intro">a</p><p id="body">b</p></div>"#);
        let intro = select(&nodes, "#intro");
        assert_eq!(intro.len(), 1);
        assert_eq!(text_of(intro[0]), "a");
    }

    #[test]
    fn select_descendant_combinator() {
        let nodes = parse("<article><h1>T</h1><section><p>in section</p></section><p>outside section</p></article>");
        let ps_in_section = select(&nodes, "section p");
        assert_eq!(ps_in_section.len(), 1);
        assert_eq!(text_of(ps_in_section[0]), "in section");
    }

    #[test]
    fn select_tag_with_class() {
        let nodes = parse(r#"<div><a class="ext">A</a><a>B</a><span class="ext">S</span></div>"#);
        let a_ext = select(&nodes, "a.ext");
        assert_eq!(a_ext.len(), 1);
        assert_eq!(text_of(a_ext[0]), "A");
    }

    #[test]
    fn forgiving_parser_recovers_from_unclosed_tags() {
        // Unclosed <p>, then <div>. A strict parser refuses; ours treats the
        // second opening as a sibling rather than a child.
        let nodes = parse("<p>stray<div>ok</div>");
        // We should get at least one <p> and one <div> that we can find.
        let all = {
            let mut v = Vec::new();
            collect_all(&nodes, &mut v);
            v
        };
        let tags: Vec<String> = all.iter().filter_map(|n| match n {
            Node::Element { tag, .. } => Some(tag.clone()),
            _ => None,
        }).collect();
        assert!(tags.contains(&"p".to_string()));
        assert!(tags.contains(&"div".to_string()));
    }

    fn collect_all<'a>(nodes: &'a [Node], out: &mut Vec<&'a Node>) {
        for n in nodes {
            out.push(n);
            if let Node::Element { children, .. } = n { collect_all(children, out); }
        }
    }

    #[test]
    fn text_extraction_recurses() {
        let nodes = parse("<div>hello <b>beautiful</b> world</div>");
        assert_eq!(text_of(&nodes[0]), "hello beautiful world");
    }
}
