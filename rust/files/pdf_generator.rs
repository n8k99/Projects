//! PDF Generator — document pagination via block flow layout.
//!
//! Seventh Files project. The defining problem of PDF generation is
//! **paginating a stream of content into fixed-size pages**. Every real
//! PDF library (iText, ReportLab, wkhtmltopdf, LaTeX) solves this: given
//! a linear stream of text/images/tables and a page size, assign each
//! block to a page without overflow.
//!
//! Here we skip binary PDF output entirely — a Rosetta Stone module
//! doesn't need to emit Adobe's object graph. Instead we produce a
//! **rendered document** model: pages, each page has a list of (block, y)
//! placements. A downstream renderer can turn that into real PDF, HTML,
//! PostScript, or terminal output.
//!
//! Layout is simple flow: blocks stack top-to-bottom. If the next block
//! doesn't fit on the current page, start a new page. Text blocks wrap
//! at word boundaries to fit the page width; long unbreakable words
//! overflow rather than truncate (like LaTeX's overfull hbox).

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading { text: String, level: u8 },
    Paragraph { text: String },
    Spacer { height: u32 },
    PageBreak,
}

impl Block {
    pub fn text(&self) -> Option<&str> {
        match self {
            Block::Heading { text, .. } => Some(text),
            Block::Paragraph { text } => Some(text),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageSize {
    pub width_chars: u32,
    pub height_lines: u32,
}

impl PageSize {
    pub const A4_LIKE: Self = PageSize { width_chars: 80, height_lines: 66 };
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderedLine {
    pub text: String,
    pub y: u32,
    pub is_heading: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub lines: Vec<RenderedLine>,
    pub number: u32,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub page_size: PageSize,
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new(page_size: PageSize) -> Self {
        Self { page_size, blocks: Vec::new() }
    }

    pub fn heading(&mut self, text: &str, level: u8) -> &mut Self {
        self.blocks.push(Block::Heading { text: text.into(), level });
        self
    }

    pub fn paragraph(&mut self, text: &str) -> &mut Self {
        self.blocks.push(Block::Paragraph { text: text.into() });
        self
    }

    pub fn spacer(&mut self, height: u32) -> &mut Self {
        self.blocks.push(Block::Spacer { height });
        self
    }

    pub fn page_break(&mut self) -> &mut Self {
        self.blocks.push(Block::PageBreak);
        self
    }

    /// Render the document into pages.
    pub fn render(&self) -> Vec<Page> {
        let mut pages: Vec<Page> = Vec::new();
        let mut current = Page { lines: Vec::new(), number: 1 };
        let mut y: u32 = 0;

        let push_page = |pages: &mut Vec<Page>, current: &mut Page, y: &mut u32| {
            let next_num = current.number + 1;
            pages.push(std::mem::replace(current, Page {
                lines: Vec::new(),
                number: next_num,
            }));
            *y = 0;
        };

        for block in &self.blocks {
            match block {
                Block::PageBreak => {
                    if !current.lines.is_empty() {
                        push_page(&mut pages, &mut current, &mut y);
                    }
                }
                Block::Spacer { height } => {
                    let h = *height;
                    if y + h > self.page_size.height_lines {
                        push_page(&mut pages, &mut current, &mut y);
                    } else {
                        y += h;
                    }
                }
                Block::Heading { text, level } => {
                    let h_height = 2;  // heading + blank line below
                    if y + h_height > self.page_size.height_lines {
                        push_page(&mut pages, &mut current, &mut y);
                    }
                    let rendered = match level {
                        1 => format!("# {}", text),
                        2 => format!("## {}", text),
                        _ => format!("### {}", text),
                    };
                    current.lines.push(RenderedLine { text: rendered, y, is_heading: true });
                    y += h_height;
                }
                Block::Paragraph { text } => {
                    let wrapped = wrap_text(text, self.page_size.width_chars);
                    for line in wrapped {
                        if y >= self.page_size.height_lines {
                            push_page(&mut pages, &mut current, &mut y);
                        }
                        current.lines.push(RenderedLine {
                            text: line, y, is_heading: false,
                        });
                        y += 1;
                    }
                    // Blank line after paragraph.
                    if y < self.page_size.height_lines {
                        y += 1;
                    }
                }
            }
        }

        if !current.lines.is_empty() {
            pages.push(current);
        }
        pages
    }

    pub fn page_count(&self) -> usize {
        self.render().len()
    }
}

/// Wrap text at word boundaries to fit width. Words longer than width overflow.
pub fn wrap_text(text: &str, width: u32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if (current.len() + 1 + word.len()) as u32 <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current.clear();
            current.push_str(word);
        }
    }
    if !current.is_empty() { lines.push(current); }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_document_renders_no_pages() {
        let doc = Document::new(PageSize::A4_LIKE);
        assert!(doc.render().is_empty());
    }

    #[test]
    fn single_short_paragraph_fits_one_page() {
        let mut doc = Document::new(PageSize::A4_LIKE);
        doc.paragraph("hello world");
        let pages = doc.render();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
        assert_eq!(pages[0].lines[0].text, "hello world");
    }

    #[test]
    fn long_paragraph_wraps_at_word_boundary() {
        let words: Vec<String> = (0..20).map(|i| format!("word{i}")).collect();
        let mut doc = Document::new(PageSize { width_chars: 30, height_lines: 66 });
        doc.paragraph(&words.join(" "));
        let pages = doc.render();
        assert_eq!(pages.len(), 1);
        // Every line is <= 30 chars.
        for line in &pages[0].lines {
            assert!(line.text.len() <= 30, "line too long: {:?}", line.text);
        }
    }

    #[test]
    fn content_overflows_to_new_page() {
        let mut doc = Document::new(PageSize { width_chars: 80, height_lines: 5 });
        for _ in 0..20 { doc.paragraph("hello"); }
        let pages = doc.render();
        assert!(pages.len() >= 2);
        // Each page has at most 5 lines.
        for page in &pages {
            assert!(page.lines.len() <= 5);
        }
    }

    #[test]
    fn page_break_starts_new_page() {
        let mut doc = Document::new(PageSize::A4_LIKE);
        doc.paragraph("top").page_break().paragraph("bottom");
        let pages = doc.render();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].lines[0].text, "top");
        assert_eq!(pages[1].lines[0].text, "bottom");
    }

    #[test]
    fn page_break_on_empty_page_is_noop() {
        let mut doc = Document::new(PageSize::A4_LIKE);
        doc.page_break().paragraph("first");
        let pages = doc.render();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
    }

    #[test]
    fn heading_rendered_with_markers() {
        let mut doc = Document::new(PageSize::A4_LIKE);
        doc.heading("Title", 1).paragraph("body");
        let pages = doc.render();
        assert_eq!(pages[0].lines[0].text, "# Title");
        assert!(pages[0].lines[0].is_heading);
        assert_eq!(pages[0].lines[1].text, "body");
    }

    #[test]
    fn heading_levels_produce_different_markers() {
        let mut doc = Document::new(PageSize::A4_LIKE);
        doc.heading("H1", 1).heading("H2", 2).heading("H3", 3);
        let pages = doc.render();
        assert_eq!(pages[0].lines[0].text, "# H1");
        assert_eq!(pages[0].lines[1].text, "## H2");
        assert_eq!(pages[0].lines[2].text, "### H3");
    }

    #[test]
    fn pages_are_numbered_starting_at_1() {
        let mut doc = Document::new(PageSize { width_chars: 80, height_lines: 3 });
        for i in 0..10 { doc.paragraph(&format!("p{i}")); }
        let pages = doc.render();
        for (i, page) in pages.iter().enumerate() {
            assert_eq!(page.number, (i + 1) as u32);
        }
    }

    #[test]
    fn wrap_text_on_empty_produces_single_empty_line() {
        let lines = wrap_text("", 80);
        assert_eq!(lines, vec![String::new()]);
    }

    #[test]
    fn long_unbreakable_word_overflows_rather_than_truncates() {
        let lines = wrap_text("supercalifragilistic", 10);
        assert_eq!(lines, vec!["supercalifragilistic".to_string()]);
    }

    #[test]
    fn spacer_consumes_vertical_space() {
        let mut doc = Document::new(PageSize { width_chars: 80, height_lines: 5 });
        doc.paragraph("top").spacer(3).paragraph("bottom");
        let pages = doc.render();
        // top takes 2 lines (paragraph + blank), spacer takes 3 = 5 used
        // bottom doesn't fit on page 1 → page 2
        assert_eq!(pages.len(), 2);
    }

    #[test]
    fn page_count_matches_render_length() {
        let mut doc = Document::new(PageSize { width_chars: 80, height_lines: 5 });
        for i in 0..20 { doc.paragraph(&format!("line {i}")); }
        assert_eq!(doc.page_count(), doc.render().len());
    }

    #[test]
    fn heading_forces_page_break_if_too_close_to_bottom() {
        let mut doc = Document::new(PageSize { width_chars: 80, height_lines: 4 });
        doc.paragraph("a").paragraph("b").heading("Forces break", 1);
        let pages = doc.render();
        // With 4 lines: "a"(y=0) blank(y=1) "b"(y=2) blank(y=3) → heading (h=2) overflows
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[1].lines[0].text, "# Forces break");
    }
}
