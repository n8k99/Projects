//! Slide Show — presentation navigation with dual-track content.
//!
//! First Graphics project. A presentation is an ordered list of slides;
//! navigation (advance/back/goto) maintains a current index bounded
//! by [0, slides.len()). Each slide has two content tracks: the
//! **visible** content (what the audience sees) and **speaker notes**
//! (what only the presenter sees).
//!
//! Distinctive moves:
//!   * **Bounded navigation state** — current_index can't overflow.
//!   * **Dual-track content** — visible vs notes, rendered separately.
//!   * **Slide element enum** — Heading, Bullet, Image, Spacer; each
//!      rendered per target.
//!   * **Presentation modes** — speaker (shows notes), audience (hides
//!      notes), handout (all content, no transitions).

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideElement {
    Heading { text: String, level: u8 },
    Bullet { text: String, indent: u8 },
    Image { src: String, alt: String },
    Spacer { lines: u8 },
    Code { text: String, language: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition { None, Fade, Slide, Dissolve }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slide {
    pub title: String,
    pub content: Vec<SlideElement>,
    pub notes: String,
    pub transition: Transition,
}

impl Slide {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            notes: String::new(),
            transition: Transition::None,
        }
    }

    pub fn heading(mut self, text: &str, level: u8) -> Self {
        self.content.push(SlideElement::Heading { text: text.into(), level });
        self
    }

    pub fn bullet(mut self, text: &str, indent: u8) -> Self {
        self.content.push(SlideElement::Bullet { text: text.into(), indent });
        self
    }

    pub fn image(mut self, src: &str, alt: &str) -> Self {
        self.content.push(SlideElement::Image { src: src.into(), alt: alt.into() });
        self
    }

    pub fn code(mut self, text: &str, language: &str) -> Self {
        self.content.push(SlideElement::Code { text: text.into(), language: language.into() });
        self
    }

    pub fn with_notes(mut self, notes: &str) -> Self {
        self.notes = notes.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationMode { Speaker, Audience, Handout }

#[derive(Debug, Clone)]
pub struct Presentation {
    pub title: String,
    pub slides: Vec<Slide>,
    pub current_index: usize,
}

impl Presentation {
    pub fn new(title: &str) -> Self {
        Self { title: title.into(), slides: Vec::new(), current_index: 0 }
    }

    pub fn add_slide(&mut self, slide: Slide) {
        self.slides.push(slide);
    }

    pub fn slide_count(&self) -> usize { self.slides.len() }

    /// Current slide if any exist.
    pub fn current(&self) -> Option<&Slide> { self.slides.get(self.current_index) }

    /// Move to next slide. No-op at end.
    pub fn advance(&mut self) -> bool {
        if self.current_index + 1 < self.slides.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// Move to previous slide. No-op at start.
    pub fn back(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    /// Go to specific 0-indexed slide. Returns true if the index is valid.
    pub fn goto(&mut self, index: usize) -> bool {
        if index < self.slides.len() {
            self.current_index = index;
            true
        } else {
            false
        }
    }

    /// Progress as (current_1_indexed, total).
    pub fn progress(&self) -> (usize, usize) {
        if self.slides.is_empty() { (0, 0) }
        else { (self.current_index + 1, self.slides.len()) }
    }

    /// Render the current slide for a given mode.
    pub fn render_current(&self, mode: PresentationMode) -> String {
        match self.current() {
            Some(slide) => render_slide(slide, mode),
            None => String::new(),
        }
    }

    /// Render every slide as a single document (handout style).
    pub fn render_handout(&self) -> String {
        let mut out = format!("# {}\n\n", self.title);
        for (i, slide) in self.slides.iter().enumerate() {
            out.push_str(&format!("--- Slide {} ---\n", i + 1));
            out.push_str(&render_slide(slide, PresentationMode::Handout));
            out.push('\n');
        }
        out
    }

    /// Search slides whose title or visible content contains the needle.
    pub fn search(&self, needle: &str) -> Vec<usize> {
        let lc = needle.to_lowercase();
        self.slides.iter().enumerate()
            .filter(|(_, s)| {
                s.title.to_lowercase().contains(&lc)
                || s.content.iter().any(|e| element_text(e).to_lowercase().contains(&lc))
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Count occurrences of each transition type across the deck.
    pub fn transition_stats(&self) -> BTreeMap<String, u32> {
        let mut out: BTreeMap<String, u32> = BTreeMap::new();
        for s in &self.slides {
            let key = match s.transition {
                Transition::None => "none",
                Transition::Fade => "fade",
                Transition::Slide => "slide",
                Transition::Dissolve => "dissolve",
            }.to_string();
            *out.entry(key).or_insert(0) += 1;
        }
        out
    }
}

fn element_text(e: &SlideElement) -> String {
    match e {
        SlideElement::Heading { text, .. } => text.clone(),
        SlideElement::Bullet { text, .. } => text.clone(),
        SlideElement::Image { alt, .. } => alt.clone(),
        SlideElement::Code { text, .. } => text.clone(),
        SlideElement::Spacer { .. } => String::new(),
    }
}

fn render_element(e: &SlideElement) -> String {
    match e {
        SlideElement::Heading { text, level } => {
            let hashes = "#".repeat((*level).max(1).min(6) as usize);
            format!("{hashes} {text}")
        }
        SlideElement::Bullet { text, indent } => {
            let pad = "  ".repeat(*indent as usize);
            format!("{pad}- {text}")
        }
        SlideElement::Image { src, alt } => format!("![{alt}]({src})"),
        SlideElement::Code { text, language } => format!("```{language}\n{text}\n```"),
        SlideElement::Spacer { lines } => "\n".repeat(*lines as usize),
    }
}

fn render_slide(slide: &Slide, mode: PresentationMode) -> String {
    let mut out = format!("{}\n", slide.title);
    out.push_str(&"=".repeat(slide.title.len().min(80)));
    out.push('\n');
    for e in &slide.content {
        out.push_str(&render_element(e));
        out.push('\n');
    }
    match mode {
        PresentationMode::Speaker | PresentationMode::Handout => {
            if !slide.notes.is_empty() {
                out.push_str("\n[Speaker notes]\n");
                out.push_str(&slide.notes);
                out.push('\n');
            }
        }
        PresentationMode::Audience => {
            // notes hidden
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_presentation() -> Presentation {
        let mut p = Presentation::new("My Talk");
        p.add_slide(
            Slide::new("Introduction")
                .heading("Welcome", 1)
                .bullet("Agenda", 0)
                .bullet("Items", 1)
                .with_notes("Remember to pause for laughs.")
        );
        p.add_slide(
            Slide::new("The Problem")
                .heading("Pain points", 1)
                .bullet("Thing one", 0)
                .bullet("Thing two", 0)
                .with_notes("Be careful not to get political.")
        );
        p.add_slide(
            Slide::new("Q&A")
                .heading("Questions?", 1)
                .with_notes("Wrap by 5pm.")
        );
        p
    }

    #[test]
    fn starts_at_slide_zero() {
        let p = sample_presentation();
        assert_eq!(p.current_index, 0);
        assert_eq!(p.current().unwrap().title, "Introduction");
    }

    #[test]
    fn advance_increments_index() {
        let mut p = sample_presentation();
        assert!(p.advance());
        assert_eq!(p.current().unwrap().title, "The Problem");
    }

    #[test]
    fn advance_returns_false_at_end() {
        let mut p = sample_presentation();
        p.current_index = 2;
        assert!(!p.advance());
        assert_eq!(p.current_index, 2);  // index unchanged
    }

    #[test]
    fn back_decrements_index() {
        let mut p = sample_presentation();
        p.current_index = 1;
        assert!(p.back());
        assert_eq!(p.current_index, 0);
    }

    #[test]
    fn back_returns_false_at_start() {
        let mut p = sample_presentation();
        assert!(!p.back());
        assert_eq!(p.current_index, 0);
    }

    #[test]
    fn goto_valid_index() {
        let mut p = sample_presentation();
        assert!(p.goto(2));
        assert_eq!(p.current().unwrap().title, "Q&A");
    }

    #[test]
    fn goto_invalid_index_rejects() {
        let mut p = sample_presentation();
        assert!(!p.goto(10));
        assert_eq!(p.current_index, 0);
    }

    #[test]
    fn progress_is_1_indexed() {
        let mut p = sample_presentation();
        assert_eq!(p.progress(), (1, 3));
        p.advance();
        assert_eq!(p.progress(), (2, 3));
    }

    #[test]
    fn empty_presentation_has_zero_progress() {
        let p = Presentation::new("empty");
        assert_eq!(p.progress(), (0, 0));
        assert!(p.current().is_none());
    }

    #[test]
    fn audience_mode_hides_notes() {
        let p = sample_presentation();
        let rendered = p.render_current(PresentationMode::Audience);
        assert!(!rendered.contains("laughs"));
    }

    #[test]
    fn speaker_mode_shows_notes() {
        let p = sample_presentation();
        let rendered = p.render_current(PresentationMode::Speaker);
        assert!(rendered.contains("laughs"));
    }

    #[test]
    fn render_slide_includes_title() {
        let p = sample_presentation();
        let rendered = p.render_current(PresentationMode::Audience);
        assert!(rendered.contains("Introduction"));
    }

    #[test]
    fn render_bullet_with_indent() {
        let slide = Slide::new("x")
            .bullet("top", 0)
            .bullet("nested", 1);
        let s = render_slide(&slide, PresentationMode::Audience);
        assert!(s.contains("- top"));
        assert!(s.contains("  - nested"));
    }

    #[test]
    fn render_heading_level_controls_hashes() {
        let slide = Slide::new("x")
            .heading("H1", 1)
            .heading("H2", 2)
            .heading("H6", 6);
        let s = render_slide(&slide, PresentationMode::Audience);
        assert!(s.contains("# H1"));
        assert!(s.contains("## H2"));
        assert!(s.contains("###### H6"));
    }

    #[test]
    fn handout_includes_all_slides() {
        let p = sample_presentation();
        let h = p.render_handout();
        assert!(h.contains("Introduction"));
        assert!(h.contains("The Problem"));
        assert!(h.contains("Q&A"));
    }

    #[test]
    fn search_matches_title_and_content() {
        let p = sample_presentation();
        let by_title = p.search("introduction");
        assert_eq!(by_title, vec![0]);
        let by_bullet = p.search("agenda");
        assert_eq!(by_bullet, vec![0]);
    }

    #[test]
    fn search_returns_multiple_matches() {
        let mut p = sample_presentation();
        p.add_slide(Slide::new("Introduction Recap"));
        let all = p.search("introduction");
        assert_eq!(all, vec![0, 3]);
    }

    #[test]
    fn transition_stats_counts_per_kind() {
        let mut p = Presentation::new("x");
        let mut s1 = Slide::new("a"); s1.transition = Transition::Fade;
        let mut s2 = Slide::new("b"); s2.transition = Transition::Fade;
        let mut s3 = Slide::new("c"); s3.transition = Transition::Slide;
        p.add_slide(s1);
        p.add_slide(s2);
        p.add_slide(s3);
        let stats = p.transition_stats();
        assert_eq!(stats["fade"], 2);
        assert_eq!(stats["slide"], 1);
    }

    #[test]
    fn code_element_rendered_with_fence() {
        let slide = Slide::new("x").code("print('hi')", "python");
        let s = render_slide(&slide, PresentationMode::Audience);
        assert!(s.contains("```python"));
        assert!(s.contains("print('hi')"));
        assert!(s.contains("```"));
    }
}
