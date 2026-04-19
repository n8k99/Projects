//! Web Browser with Tabs — state model for tabbed navigation.
//!
//! Each tab is an independent history timeline: its own back stack, forward
//! stack, and current URL. The browser holds the tabs, an active-tab pointer,
//! a closed-tab stack (for reopen-closed-tab / Ctrl+Shift+T), and browser-
//! level state (bookmarks) that transcends individual tabs.
//!
//! This is the first Rosetta Stone project with **two orthogonal undo-like
//! stacks**: per-tab navigation history (back/forward) and browser-level
//! closed-tab recovery. They answer different questions and have different
//! scopes.

use std::collections::VecDeque;

pub type TabId = u64;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub back: Vec<String>,     // URLs user navigated away from
    pub forward: Vec<String>,  // URLs cleared by navigate, accessible via forward()
}

#[derive(Debug, Clone)]
pub struct ClosedTab {
    pub url: String,
    pub title: String,
    pub back: Vec<String>,
    pub forward: Vec<String>,
}

pub struct Browser {
    tabs: Vec<Tab>,
    active: Option<TabId>,
    closed: VecDeque<ClosedTab>,   // newest first
    bookmarks: Vec<(String, String)>,  // (title, url)
    next_id: TabId,
}

impl Browser {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(), active: None,
            closed: VecDeque::new(),
            bookmarks: Vec::new(), next_id: 1,
        }
    }

    /// Open a new tab with the given URL. New tab becomes the active tab.
    pub fn open_tab(&mut self, url: &str) -> TabId {
        let id = self.next_id;
        self.next_id += 1;
        self.tabs.push(Tab {
            id, title: url.into(), url: url.into(),
            back: Vec::new(), forward: Vec::new(),
        });
        self.active = Some(id);
        id
    }

    /// Close a tab. If it was active, activation moves to the next tab
    /// (or previous if it was the last). Closed tabs are pushed onto the
    /// reopen stack.
    pub fn close_tab(&mut self, id: TabId) {
        let Some(pos) = self.tabs.iter().position(|t| t.id == id) else { return; };
        let t = self.tabs.remove(pos);
        self.closed.push_front(ClosedTab {
            url: t.url, title: t.title, back: t.back, forward: t.forward,
        });
        if self.active == Some(id) {
            // Activate the tab at the same position if any, else the previous.
            self.active = self.tabs.get(pos).map(|t| t.id)
                .or_else(|| self.tabs.get(pos.saturating_sub(1)).map(|t| t.id));
        }
    }

    pub fn switch_to(&mut self, id: TabId) -> bool {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active = Some(id);
            true
        } else {
            false
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.active.and_then(|id| self.tabs.iter().find(|t| t.id == id))
    }

    fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active.and_then(|id| self.tabs.iter_mut().find(|t| t.id == id))
    }

    /// Navigate the active tab to a new URL. Pushes the current URL onto
    /// the back stack and CLEARS the forward stack — user is branching
    /// off the history timeline, any future history is now unreachable.
    pub fn navigate(&mut self, url: &str) -> bool {
        let Some(tab) = self.active_tab_mut() else { return false; };
        let current = std::mem::replace(&mut tab.url, url.to_string());
        tab.back.push(current);
        tab.forward.clear();
        tab.title = url.to_string();
        true
    }

    /// Go back in the active tab. Returns the new URL, or None if no history.
    pub fn back(&mut self) -> Option<String> {
        let tab = self.active_tab_mut()?;
        let prev = tab.back.pop()?;
        let current = std::mem::replace(&mut tab.url, prev.clone());
        tab.forward.push(current);
        Some(prev)
    }

    /// Go forward in the active tab. Returns the new URL, or None.
    pub fn forward(&mut self) -> Option<String> {
        let tab = self.active_tab_mut()?;
        let next = tab.forward.pop()?;
        let current = std::mem::replace(&mut tab.url, next.clone());
        tab.back.push(current);
        Some(next)
    }

    /// Reopen the most recently closed tab (Ctrl+Shift+T).
    pub fn reopen_closed(&mut self) -> Option<TabId> {
        let ct = self.closed.pop_front()?;
        let id = self.next_id;
        self.next_id += 1;
        self.tabs.push(Tab {
            id, title: ct.title, url: ct.url,
            back: ct.back, forward: ct.forward,
        });
        self.active = Some(id);
        Some(id)
    }

    pub fn add_bookmark(&mut self) -> bool {
        let Some(tab) = self.active_tab() else { return false; };
        let entry = (tab.title.clone(), tab.url.clone());
        if !self.bookmarks.contains(&entry) { self.bookmarks.push(entry); }
        true
    }

    pub fn bookmarks(&self) -> &[(String, String)] { &self.bookmarks }
    pub fn tab_count(&self) -> usize { self.tabs.len() }
    pub fn closed_count(&self) -> usize { self.closed.len() }
    pub fn tabs(&self) -> &[Tab] { &self.tabs }
}

impl Default for Browser {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_tab_activates_it() {
        let mut b = Browser::new();
        let id = b.open_tab("https://example.org");
        assert_eq!(b.tab_count(), 1);
        assert_eq!(b.active_tab().unwrap().id, id);
    }

    #[test]
    fn navigate_pushes_back_clears_forward() {
        let mut b = Browser::new();
        b.open_tab("a");
        b.navigate("b");
        b.navigate("c");
        let t = b.active_tab().unwrap();
        assert_eq!(t.url, "c");
        assert_eq!(t.back, vec!["a", "b"]);
        assert!(t.forward.is_empty());
    }

    #[test]
    fn back_forward_preserves_chain() {
        let mut b = Browser::new();
        b.open_tab("a");
        b.navigate("b");
        b.navigate("c");
        assert_eq!(b.back().unwrap(), "b");
        assert_eq!(b.back().unwrap(), "a");
        assert!(b.back().is_none());
        assert_eq!(b.forward().unwrap(), "b");
        assert_eq!(b.forward().unwrap(), "c");
        assert!(b.forward().is_none());
    }

    #[test]
    fn navigate_after_back_truncates_forward() {
        // User goes back then branches — the rest of the forward history is gone.
        let mut b = Browser::new();
        b.open_tab("a");
        b.navigate("b");
        b.navigate("c");
        b.back();             // url now "b", forward=[c]
        b.navigate("x");      // url now "x", forward cleared
        let t = b.active_tab().unwrap();
        assert_eq!(t.url, "x");
        assert!(t.forward.is_empty());
        assert_eq!(t.back, vec!["a", "b"]);
    }

    #[test]
    fn per_tab_history_is_independent() {
        let mut b = Browser::new();
        let t1 = b.open_tab("tab1_a");
        b.navigate("tab1_b");
        let t2 = b.open_tab("tab2_a");  // activates t2
        b.navigate("tab2_b");
        // Switch back to t1 and verify its history is untouched.
        b.switch_to(t1);
        let tab1 = b.active_tab().unwrap();
        assert_eq!(tab1.url, "tab1_b");
        assert_eq!(tab1.back, vec!["tab1_a"]);
        // Go back in t1; t2 should be unchanged when we switch.
        b.back();
        b.switch_to(t2);
        let tab2 = b.active_tab().unwrap();
        assert_eq!(tab2.url, "tab2_b");
        assert_eq!(tab2.back, vec!["tab2_a"]);
    }

    #[test]
    fn close_and_reopen_tab_restores_state() {
        let mut b = Browser::new();
        let id = b.open_tab("a");
        b.navigate("b");
        b.navigate("c");
        b.back();
        b.close_tab(id);
        assert_eq!(b.tab_count(), 0);
        assert_eq!(b.closed_count(), 1);
        let reopened = b.reopen_closed().unwrap();
        let t = b.active_tab().unwrap();
        assert_eq!(t.id, reopened);
        assert_eq!(t.url, "b");
        assert_eq!(t.back, vec!["a"]);
        assert_eq!(t.forward, vec!["c"]);
    }

    #[test]
    fn closed_tab_stack_is_lifo() {
        let mut b = Browser::new();
        let a = b.open_tab("alpha");
        let beta = b.open_tab("beta");
        b.close_tab(a);
        b.close_tab(beta);
        // Reopen pops beta first (most recently closed).
        let first = b.reopen_closed().unwrap();
        assert_eq!(b.active_tab().unwrap().url, "beta");
        assert_eq!(first, b.active_tab().unwrap().id);
        b.reopen_closed();
        assert_eq!(b.active_tab().unwrap().url, "alpha");
    }

    #[test]
    fn bookmarks_are_browser_scoped_not_tab_scoped() {
        let mut b = Browser::new();
        b.open_tab("a");
        b.add_bookmark();
        let t2 = b.open_tab("b");
        b.add_bookmark();
        b.close_tab(t2);
        // Bookmarks survive the tab closing.
        assert_eq!(b.bookmarks().len(), 2);
    }
}
