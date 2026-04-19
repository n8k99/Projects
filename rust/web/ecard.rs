//! E-Card Generator — template + data rendering with slot validation.
//!
//! First Rosetta Stone project where **content and presentation are separated
//! by a template**. A `Template` declares the slots it expects (name, kind,
//! required/optional) and owns the rendering surface (HTML with
//! `{{slot_name}}` placeholders). A `Card` is a filled-in instance:
//! (template_id, filled slot values). Rendering substitutes values into
//! the template's placeholders.
//!
//! Validation happens at card creation, not at render time — if the user
//! omits a required slot or fills an unknown one, `create_card` rejects
//! it with a structured error. This means **every stored card is valid**;
//! rendering can never fail on a stored card.

use std::collections::{HashMap, HashSet};

pub type TemplateId = u64;
pub type CardId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Text,
    LongText,
    ImageUrl,
    Color,
    Date,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub kind: SlotKind,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub id: TemplateId,
    pub name: String,
    pub category: String,
    pub slots: Vec<Slot>,
    /// HTML template with `{{slot_name}}` placeholders. Unknown placeholders
    /// left untouched — the caller sees the raw placeholder in the output.
    pub html_template: String,
}

impl Template {
    pub fn slot(&self, name: &str) -> Option<&Slot> {
        self.slots.iter().find(|s| s.name == name)
    }
    pub fn slot_names(&self) -> HashSet<&str> {
        self.slots.iter().map(|s| s.name.as_str()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub id: CardId,
    pub template_id: TemplateId,
    pub filled: HashMap<String, String>,   // includes defaults for unfilled optional slots
    pub recipient: String,
    pub sender: String,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnknownTemplate(TemplateId),
    MissingRequired(String),
    UnknownSlot(String),
}

pub struct Gallery {
    templates: Vec<Template>,
    cards: Vec<Card>,
    next_template_id: TemplateId,
    next_card_id: CardId,
}

impl Gallery {
    pub fn new() -> Self {
        Self { templates: Vec::new(), cards: Vec::new(),
               next_template_id: 1, next_card_id: 1 }
    }

    pub fn add_template(&mut self, name: &str, category: &str,
                        slots: Vec<Slot>, html_template: &str) -> TemplateId {
        let id = self.next_template_id;
        self.next_template_id += 1;
        self.templates.push(Template {
            id, name: name.into(), category: category.into(),
            slots, html_template: html_template.into(),
        });
        id
    }

    pub fn template(&self, id: TemplateId) -> Option<&Template> {
        self.templates.iter().find(|t| t.id == id)
    }

    pub fn templates(&self) -> &[Template] { &self.templates }

    pub fn templates_in_category(&self, category: &str) -> Vec<&Template> {
        self.templates.iter().filter(|t| t.category == category).collect()
    }

    /// Create a card from a template + filled slots. Validates the slot map.
    /// Defaults are applied for optional slots the caller didn't provide.
    pub fn create_card(
        &mut self, template_id: TemplateId,
        filled: HashMap<String, String>,
        recipient: &str, sender: &str, now_ms: u64,
    ) -> Result<CardId, ValidationError> {
        let template = self.templates.iter().find(|t| t.id == template_id)
            .ok_or(ValidationError::UnknownTemplate(template_id))?;

        // Unknown slot?
        let known: HashSet<&str> = template.slot_names();
        for key in filled.keys() {
            if !known.contains(key.as_str()) {
                return Err(ValidationError::UnknownSlot(key.clone()));
            }
        }

        // Required slot missing?
        for slot in &template.slots {
            if slot.required && !filled.contains_key(&slot.name) {
                return Err(ValidationError::MissingRequired(slot.name.clone()));
            }
        }

        // Apply defaults for absent optional slots.
        let mut final_filled = filled;
        for slot in &template.slots {
            if !final_filled.contains_key(&slot.name) {
                if let Some(default) = &slot.default {
                    final_filled.insert(slot.name.clone(), default.clone());
                }
            }
        }

        let id = self.next_card_id;
        self.next_card_id += 1;
        self.cards.push(Card {
            id, template_id, filled: final_filled,
            recipient: recipient.into(), sender: sender.into(),
            created_at_ms: now_ms,
        });
        Ok(id)
    }

    pub fn card(&self, id: CardId) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
    }

    pub fn cards(&self) -> &[Card] { &self.cards }

    pub fn cards_by_template(&self, template_id: TemplateId) -> Vec<&Card> {
        self.cards.iter().filter(|c| c.template_id == template_id).collect()
    }

    /// Render a card to HTML by substituting slot values into the template.
    /// Also substitutes `{{_recipient}}` and `{{_sender}}` special slots.
    /// Returns None if the card or its template doesn't exist.
    pub fn render(&self, card_id: CardId) -> Option<String> {
        let card = self.card(card_id)?;
        let template = self.template(card.template_id)?;
        let mut out = template.html_template.clone();
        for (k, v) in &card.filled {
            out = out.replace(&format!("{{{{{k}}}}}"), v);
        }
        out = out.replace("{{_recipient}}", &card.recipient);
        out = out.replace("{{_sender}}", &card.sender);
        Some(out)
    }

    /// Text-only rendering: same substitution, then strip tags.
    pub fn render_plain(&self, card_id: CardId) -> Option<String> {
        let html = self.render(card_id)?;
        Some(strip_tags(&html))
    }
}

impl Default for Gallery {
    fn default() -> Self { Self::new() }
}

/// Naive tag-stripper: removes `<...>` substrings. Not a full HTML parser.
fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn birthday_template() -> (Vec<Slot>, &'static str) {
        let slots = vec![
            Slot { name: "name".into(), kind: SlotKind::Text,
                   required: true, default: None },
            Slot { name: "age".into(), kind: SlotKind::Text,
                   required: false, default: Some("another year older".into()) },
            Slot { name: "color".into(), kind: SlotKind::Color,
                   required: false, default: Some("#ffcc00".into()) },
        ];
        let html = "<div style=\"background:{{color}}\">\
                    <h1>Happy Birthday, {{name}}!</h1>\
                    <p>From {{_sender}} — {{age}}.</p>\
                    </div>";
        (slots, html)
    }

    #[test]
    fn add_and_retrieve_template() {
        let mut g = Gallery::new();
        let (slots, html) = birthday_template();
        let id = g.add_template("Classic Birthday", "birthday", slots, html);
        let t = g.template(id).unwrap();
        assert_eq!(t.name, "Classic Birthday");
        assert_eq!(t.slots.len(), 3);
    }

    #[test]
    fn create_card_substitutes_slots_in_render() {
        let mut g = Gallery::new();
        let (slots, html) = birthday_template();
        let tid = g.add_template("Classic Birthday", "birthday", slots, html);
        let filled: HashMap<String, String> = [
            ("name".to_string(), "Alice".to_string()),
        ].into_iter().collect();
        let cid = g.create_card(tid, filled, "Alice", "Bob", 1000).unwrap();
        let rendered = g.render(cid).unwrap();
        assert!(rendered.contains("Happy Birthday, Alice!"));
        assert!(rendered.contains("From Bob"));
        assert!(rendered.contains("another year older"));   // default applied
        assert!(rendered.contains("#ffcc00"));              // color default
    }

    #[test]
    fn missing_required_slot_rejected() {
        let mut g = Gallery::new();
        let (slots, html) = birthday_template();
        let tid = g.add_template("Classic Birthday", "birthday", slots, html);
        let filled: HashMap<String, String> = HashMap::new();  // no "name"
        let err = g.create_card(tid, filled, "Alice", "Bob", 1000);
        assert_eq!(err, Err(ValidationError::MissingRequired("name".into())));
    }

    #[test]
    fn unknown_slot_rejected() {
        let mut g = Gallery::new();
        let (slots, html) = birthday_template();
        let tid = g.add_template("Classic Birthday", "birthday", slots, html);
        let filled: HashMap<String, String> = [
            ("name".to_string(), "Alice".to_string()),
            ("typo_slot".to_string(), "value".to_string()),
        ].into_iter().collect();
        let err = g.create_card(tid, filled, "Alice", "Bob", 1000);
        assert_eq!(err, Err(ValidationError::UnknownSlot("typo_slot".into())));
    }

    #[test]
    fn unknown_template_rejected() {
        let mut g = Gallery::new();
        let err = g.create_card(9999, HashMap::new(), "A", "B", 1000);
        assert_eq!(err, Err(ValidationError::UnknownTemplate(9999)));
    }

    #[test]
    fn defaults_applied_for_optional_slots() {
        let mut g = Gallery::new();
        let (slots, html) = birthday_template();
        let tid = g.add_template("Classic Birthday", "birthday", slots, html);
        let filled: HashMap<String, String> = [
            ("name".to_string(), "Alice".to_string()),
        ].into_iter().collect();
        let cid = g.create_card(tid, filled, "A", "B", 0).unwrap();
        let card = g.card(cid).unwrap();
        assert_eq!(card.filled.get("color").unwrap(), "#ffcc00");
        assert_eq!(card.filled.get("age").unwrap(), "another year older");
    }

    #[test]
    fn render_plain_strips_html_tags() {
        let mut g = Gallery::new();
        let (slots, html) = birthday_template();
        let tid = g.add_template("Classic Birthday", "birthday", slots, html);
        let filled: HashMap<String, String> = [
            ("name".to_string(), "Alice".to_string()),
        ].into_iter().collect();
        let cid = g.create_card(tid, filled, "Alice", "Bob", 1000).unwrap();
        let plain = g.render_plain(cid).unwrap();
        assert!(!plain.contains("<h1>"));
        assert!(!plain.contains("<div"));
        assert!(plain.contains("Happy Birthday, Alice!"));
    }

    #[test]
    fn templates_in_category_filters() {
        let mut g = Gallery::new();
        g.add_template("Birthday A", "birthday", vec![], "");
        g.add_template("Birthday B", "birthday", vec![], "");
        g.add_template("Anniversary", "anniversary", vec![], "");
        assert_eq!(g.templates_in_category("birthday").len(), 2);
        assert_eq!(g.templates_in_category("anniversary").len(), 1);
    }

    #[test]
    fn cards_by_template_filters() {
        let mut g = Gallery::new();
        let tid1 = g.add_template("A", "x", vec![], "hello {{_recipient}}");
        let tid2 = g.add_template("B", "y", vec![], "hi {{_recipient}}");
        g.create_card(tid1, HashMap::new(), "u1", "v", 0).unwrap();
        g.create_card(tid1, HashMap::new(), "u2", "v", 0).unwrap();
        g.create_card(tid2, HashMap::new(), "u3", "v", 0).unwrap();
        assert_eq!(g.cards_by_template(tid1).len(), 2);
        assert_eq!(g.cards_by_template(tid2).len(), 1);
    }

    #[test]
    fn unreferenced_placeholder_left_as_is() {
        let mut g = Gallery::new();
        let tid = g.add_template("bad", "misc", vec![],
                                 "hi {{_recipient}} from {{nonexistent}}");
        let cid = g.create_card(tid, HashMap::new(), "Alice", "Bob", 0).unwrap();
        let rendered = g.render(cid).unwrap();
        assert!(rendered.contains("hi Alice"));
        // Template author's bug is visible in output, not hidden.
        assert!(rendered.contains("{{nonexistent}}"));
    }
}
