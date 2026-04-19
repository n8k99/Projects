//! Template Maker — meta-level tool for building and validating templates.
//!
//! Where G081 E-Card consumed pre-built templates, G083 is the builder for
//! templates themselves. A TemplateBuilder holds the work-in-progress
//! definition: slot declarations, HTML template draft, sample values for
//! preview. Validation finds mismatches — slots declared but never
//! referenced, `{{placeholder}}` strings in HTML with no matching slot,
//! required slots with no sample value for preview.
//!
//! This is the first Rosetta Stone project where **the code that produces
//! templates is itself a data model**. You can inspect, edit, validate, and
//! serialise the template-in-progress; you don't have to run code to know
//! what the template will do.

use std::collections::{BTreeMap, BTreeSet};

pub use crate::web::ecard::{Slot, SlotKind};

pub type BuilderId = u64;

#[derive(Debug, Clone)]
pub struct TemplateBuilder {
    pub id: BuilderId,
    pub name: String,
    pub category: String,
    pub slots: Vec<Slot>,
    pub html_draft: String,
    pub sample_data: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssue {
    /// Slot declared but never referenced in the HTML draft.
    UnusedSlot(String),
    /// `{{name}}` appears in HTML but no slot with that name is declared.
    UndeclaredPlaceholder(String),
    /// Required slot has no sample value — preview will show the placeholder.
    MissingSampleForRequired(String),
    /// Two slots have the same name.
    DuplicateSlotName(String),
    /// Slot name contains characters outside [A-Za-z0-9_] or is empty.
    InvalidSlotName(String),
}

#[derive(Debug)]
pub enum BuilderError {
    UnknownBuilder(BuilderId),
    SlotAlreadyExists(String),
    SlotNotFound(String),
}

pub struct TemplateMaker {
    builders: Vec<TemplateBuilder>,
    next_id: BuilderId,
}

impl TemplateMaker {
    pub fn new() -> Self { Self { builders: Vec::new(), next_id: 1 } }

    pub fn new_template(&mut self, name: &str, category: &str) -> BuilderId {
        let id = self.next_id;
        self.next_id += 1;
        self.builders.push(TemplateBuilder {
            id, name: name.into(), category: category.into(),
            slots: Vec::new(),
            html_draft: String::new(),
            sample_data: BTreeMap::new(),
        });
        id
    }

    pub fn builder(&self, id: BuilderId) -> Option<&TemplateBuilder> {
        self.builders.iter().find(|b| b.id == id)
    }

    fn require_mut(&mut self, id: BuilderId) -> Result<&mut TemplateBuilder, BuilderError> {
        self.builders.iter_mut().find(|b| b.id == id)
            .ok_or(BuilderError::UnknownBuilder(id))
    }

    pub fn add_slot(&mut self, id: BuilderId, slot: Slot) -> Result<(), BuilderError> {
        let builder = self.require_mut(id)?;
        if builder.slots.iter().any(|s| s.name == slot.name) {
            return Err(BuilderError::SlotAlreadyExists(slot.name));
        }
        builder.slots.push(slot);
        Ok(())
    }

    pub fn remove_slot(&mut self, id: BuilderId, slot_name: &str) -> Result<(), BuilderError> {
        let builder = self.require_mut(id)?;
        let pos = builder.slots.iter().position(|s| s.name == slot_name)
            .ok_or_else(|| BuilderError::SlotNotFound(slot_name.into()))?;
        builder.slots.remove(pos);
        builder.sample_data.remove(slot_name);
        Ok(())
    }

    pub fn update_html(&mut self, id: BuilderId, html: &str) -> Result<(), BuilderError> {
        let builder = self.require_mut(id)?;
        builder.html_draft = html.into();
        Ok(())
    }

    pub fn set_sample(&mut self, id: BuilderId, slot_name: &str, value: &str)
        -> Result<(), BuilderError> {
        let builder = self.require_mut(id)?;
        if !builder.slots.iter().any(|s| s.name == slot_name) {
            return Err(BuilderError::SlotNotFound(slot_name.into()));
        }
        builder.sample_data.insert(slot_name.into(), value.into());
        Ok(())
    }

    /// Return a list of validation issues. Empty list = template is clean.
    pub fn validate(&self, id: BuilderId) -> Vec<ValidationIssue> {
        let Some(builder) = self.builder(id) else { return Vec::new(); };
        let mut issues = Vec::new();

        // Duplicate/invalid slot names.
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for slot in &builder.slots {
            if slot.name.is_empty() || !slot.name.chars().all(valid_slot_char) {
                issues.push(ValidationIssue::InvalidSlotName(slot.name.clone()));
            }
            if !seen.insert(slot.name.clone()) {
                issues.push(ValidationIssue::DuplicateSlotName(slot.name.clone()));
            }
        }

        let declared: BTreeSet<&str> = builder.slots.iter()
            .map(|s| s.name.as_str()).collect();
        let referenced = extract_placeholders(&builder.html_draft);

        // UnusedSlot: declared but not referenced.
        for slot in &builder.slots {
            if !referenced.contains(slot.name.as_str()) {
                issues.push(ValidationIssue::UnusedSlot(slot.name.clone()));
            }
        }
        // UndeclaredPlaceholder: referenced but not declared. Ignore the
        // special `_recipient` and `_sender`.
        for placeholder in &referenced {
            if placeholder.starts_with('_') { continue; }
            if !declared.contains(placeholder.as_str()) {
                issues.push(ValidationIssue::UndeclaredPlaceholder(placeholder.clone()));
            }
        }
        // MissingSampleForRequired.
        for slot in &builder.slots {
            if slot.required && !builder.sample_data.contains_key(&slot.name) {
                issues.push(ValidationIssue::MissingSampleForRequired(slot.name.clone()));
            }
        }
        issues
    }

    /// Render the template with sample_data filled in. Unfilled optional slots
    /// fall back to their declared default; unfilled required slots without
    /// samples leave their placeholder visible (validate() would have flagged them).
    pub fn preview(&self, id: BuilderId) -> Option<String> {
        let builder = self.builder(id)?;
        let mut out = builder.html_draft.clone();
        for slot in &builder.slots {
            let value = builder.sample_data.get(&slot.name)
                .cloned()
                .or_else(|| slot.default.clone());
            if let Some(v) = value {
                out = out.replace(&format!("{{{{{}}}}}", slot.name), &v);
            }
        }
        // Provide pleasant defaults for the special slots.
        out = out.replace("{{_recipient}}", "[preview recipient]");
        out = out.replace("{{_sender}}", "[preview sender]");
        Some(out)
    }

    pub fn builders(&self) -> &[TemplateBuilder] { &self.builders }

    pub fn remove_builder(&mut self, id: BuilderId) -> bool {
        match self.builders.iter().position(|b| b.id == id) {
            Some(pos) => { self.builders.remove(pos); true }
            None => false,
        }
    }
}

impl Default for TemplateMaker {
    fn default() -> Self { Self::new() }
}

fn valid_slot_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Extract every `{{name}}` placeholder from the given HTML.
fn extract_placeholders(html: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut j = start;
            while j + 1 < bytes.len() && !(bytes[j] == b'}' && bytes[j + 1] == b'}') {
                j += 1;
            }
            if j + 1 < bytes.len() {
                let name = &html[start..j];
                if !name.is_empty() { out.insert(name.to_string()); }
                i = j + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(name: &str, required: bool, default: Option<&str>) -> Slot {
        Slot {
            name: name.into(),
            kind: SlotKind::Text,
            required,
            default: default.map(String::from),
        }
    }

    #[test]
    fn new_template_starts_empty() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("Birthday", "greeting");
        let b = m.builder(id).unwrap();
        assert_eq!(b.slots.len(), 0);
        assert_eq!(b.html_draft, "");
    }

    #[test]
    fn add_slot_and_update_html() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("name", true, None)).unwrap();
        m.update_html(id, "Hello, {{name}}!").unwrap();
        assert!(m.validate(id).iter().any(|i|
            matches!(i, ValidationIssue::MissingSampleForRequired(n) if n == "name")));
    }

    #[test]
    fn duplicate_slot_rejected() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("name", true, None)).unwrap();
        let err = m.add_slot(id, slot("name", false, None));
        assert!(matches!(err, Err(BuilderError::SlotAlreadyExists(_))));
    }

    #[test]
    fn unused_slot_detected() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("name", false, Some("X"))).unwrap();
        m.add_slot(id, slot("unused", false, Some("Y"))).unwrap();
        m.update_html(id, "hi {{name}}").unwrap();
        let issues = m.validate(id);
        assert!(issues.iter().any(|i|
            matches!(i, ValidationIssue::UnusedSlot(n) if n == "unused")));
    }

    #[test]
    fn undeclared_placeholder_detected() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("name", false, Some("X"))).unwrap();
        m.update_html(id, "hi {{name}} from {{typo}}").unwrap();
        let issues = m.validate(id);
        assert!(issues.iter().any(|i|
            matches!(i, ValidationIssue::UndeclaredPlaceholder(n) if n == "typo")));
    }

    #[test]
    fn special_recipient_sender_not_flagged() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("name", false, Some("X"))).unwrap();
        m.update_html(id, "hi {{name}} from {{_sender}} to {{_recipient}}").unwrap();
        let issues = m.validate(id);
        assert!(!issues.iter().any(|i|
            matches!(i, ValidationIssue::UndeclaredPlaceholder(_))));
    }

    #[test]
    fn preview_substitutes_samples() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("name", true, None)).unwrap();
        m.update_html(id, "hi {{name}}!").unwrap();
        m.set_sample(id, "name", "Alice").unwrap();
        let preview = m.preview(id).unwrap();
        assert_eq!(preview, "hi Alice!");
    }

    #[test]
    fn preview_uses_default_when_sample_absent() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("color", false, Some("#ffcc00"))).unwrap();
        m.update_html(id, "color={{color}}").unwrap();
        let preview = m.preview(id).unwrap();
        assert_eq!(preview, "color=#ffcc00");
    }

    #[test]
    fn validate_clean_template_has_no_issues() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("name", true, None)).unwrap();
        m.add_slot(id, slot("color", false, Some("#fff"))).unwrap();
        m.update_html(id, "<h1>{{name}}</h1><div>{{color}}</div>").unwrap();
        m.set_sample(id, "name", "Alice").unwrap();
        assert_eq!(m.validate(id), Vec::new());
    }

    #[test]
    fn remove_slot_also_clears_sample() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("x", true, None)).unwrap();
        m.set_sample(id, "x", "sample").unwrap();
        m.remove_slot(id, "x").unwrap();
        assert!(m.builder(id).unwrap().sample_data.get("x").is_none());
    }

    #[test]
    fn invalid_slot_name_detected() {
        let mut m = TemplateMaker::new();
        let id = m.new_template("T", "c");
        m.add_slot(id, slot("bad name!", false, None)).unwrap();
        let issues = m.validate(id);
        assert!(issues.iter().any(|i| matches!(i, ValidationIssue::InvalidSlotName(_))));
    }

    #[test]
    fn extract_placeholders_finds_all() {
        let html = "{{a}} and {{b}} and text {{c}} end";
        let placeholders = extract_placeholders(html);
        assert!(placeholders.contains("a"));
        assert!(placeholders.contains("b"));
        assert!(placeholders.contains("c"));
        assert_eq!(placeholders.len(), 3);
    }
}
