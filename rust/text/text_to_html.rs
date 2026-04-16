//! Text to HTML Generator — convert formatted plain text to HTML.

/// Escape HTML entities in content.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Apply inline formatting: code, bold, italic, links.
fn apply_inline(text: &str) -> String {
    let mut result = text.to_string();

    // Code spans: `code` -> <code>code</code>
    let re_code = regex::Regex::new(r"`([^`]+)`").unwrap();
    result = re_code.replace_all(&result, "<code>$1</code>").to_string();

    // Bold: **text** -> <strong>text</strong>
    let re_bold = regex::Regex::new(r"\*\*(.+?)\*\*").unwrap();
    result = re_bold
        .replace_all(&result, "<strong>$1</strong>")
        .to_string();

    // Italic: *text* -> <em>text</em>
    let re_italic = regex::Regex::new(r"\*(.+?)\*").unwrap();
    result = re_italic.replace_all(&result, "<em>$1</em>").to_string();

    // Links: [text](url) -> <a href="url">text</a>
    let re_link = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    result = re_link
        .replace_all(&result, r#"<a href="$2">$1</a>"#)
        .to_string();

    result
}

/// Convert formatted plain text to HTML.
///
/// Supports paragraphs, bold, italic, headers (H1-H3), links,
/// unordered lists, inline code, line breaks, and HTML entity escaping.
pub fn text_to_html(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut blocks: Vec<String> = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut list_items: Vec<String> = Vec::new();

    let flush_paragraph = |para: &mut Vec<String>, blocks: &mut Vec<String>| {
        if para.is_empty() {
            return;
        }
        let joined = para.join("<br>\n");
        blocks.push(format!("<p>{}</p>", apply_inline(&joined)));
        para.clear();
    };

    let flush_list = |items: &mut Vec<String>, blocks: &mut Vec<String>| {
        if items.is_empty() {
            return;
        }
        let inner: Vec<String> = items
            .iter()
            .map(|item| format!("<li>{}</li>", apply_inline(item)))
            .collect();
        blocks.push(format!("<ul>\n{}\n</ul>", inner.join("\n")));
        items.clear();
    };

    for line in &lines {
        let trimmed = line.trim();

        // Blank line
        if trimmed.is_empty() {
            flush_list(&mut list_items, &mut blocks);
            flush_paragraph(&mut paragraph, &mut blocks);
            continue;
        }

        // Header
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= 3 && trimmed.len() > level && trimmed.as_bytes()[level] == b' ' {
                flush_list(&mut list_items, &mut blocks);
                flush_paragraph(&mut paragraph, &mut blocks);
                let content = escape_html(&trimmed[level + 1..]);
                blocks.push(format!("<h{level}>{}</h{level}>", apply_inline(&content)));
                continue;
            }
        }

        // List item
        if trimmed.starts_with("- ") {
            flush_paragraph(&mut paragraph, &mut blocks);
            list_items.push(escape_html(&trimmed[2..]));
            continue;
        }

        // Flush list if transitioning out
        if !list_items.is_empty() {
            flush_list(&mut list_items, &mut blocks);
        }

        // Regular paragraph text
        paragraph.push(escape_html(trimmed));
    }

    flush_list(&mut list_items, &mut blocks);
    flush_paragraph(&mut paragraph, &mut blocks);

    blocks.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraphs() {
        let input = "First paragraph.\n\nSecond paragraph.";
        let result = text_to_html(input);
        assert!(result.contains("<p>First paragraph.</p>"));
        assert!(result.contains("<p>Second paragraph.</p>"));
    }

    #[test]
    fn test_bold_and_italic() {
        assert_eq!(
            text_to_html("**bold**"),
            "<p><strong>bold</strong></p>"
        );
        assert_eq!(text_to_html("*italic*"), "<p><em>italic</em></p>");
    }

    #[test]
    fn test_headers() {
        assert_eq!(text_to_html("# Title"), "<h1>Title</h1>");
        assert_eq!(text_to_html("## Subtitle"), "<h2>Subtitle</h2>");
        assert_eq!(text_to_html("### Section"), "<h3>Section</h3>");
    }

    #[test]
    fn test_links() {
        let result = text_to_html("[click](https://example.com)");
        assert_eq!(
            result,
            r#"<p><a href="https://example.com">click</a></p>"#
        );
    }

    #[test]
    fn test_unordered_list() {
        let input = "- alpha\n- beta\n- gamma";
        let result = text_to_html(input);
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>alpha</li>"));
        assert!(result.contains("<li>beta</li>"));
        assert!(result.contains("<li>gamma</li>"));
        assert!(result.contains("</ul>"));
    }

    #[test]
    fn test_code_spans() {
        assert_eq!(
            text_to_html("use `cargo build`"),
            "<p>use <code>cargo build</code></p>"
        );
    }

    #[test]
    fn test_line_breaks() {
        let result = text_to_html("line one\nline two");
        assert!(result.contains("line one<br>\nline two"));
    }

    #[test]
    fn test_html_escaping() {
        let result = text_to_html("a < b & c > d");
        assert!(result.contains("a &lt; b &amp; c &gt; d"));
    }

    #[test]
    fn test_mixed_content() {
        let input = "# Hello\n\nA **bold** word and *italic* too.\n\n- item one\n- item two";
        let result = text_to_html(input);
        assert!(result.contains("<h1>Hello</h1>"));
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
        assert!(result.contains("<li>item one</li>"));
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(text_to_html(""), "");
    }
}
