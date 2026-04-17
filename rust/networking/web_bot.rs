// Web Bot — automated web interaction: crawling, scraping, and task automation.

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct BotPage {
    pub url: String,
    pub status_code: u16,
    pub body: String,
}

impl BotPage {
    pub fn title(&self) -> String {
        if let Some(start) = self.body.find("<title>") {
            if let Some(end) = self.body[start..].find("</title>") {
                return self.body[start + 7..start + end].trim().to_string();
            }
        }
        String::new()
    }

    pub fn extract_links(&self) -> Vec<String> {
        let mut links = Vec::new();
        let mut pos = 0;
        while let Some(href_pos) = self.body[pos..].find("href=\"") {
            let abs = pos + href_pos + 6;
            if let Some(end) = self.body[abs..].find('"') {
                links.push(self.body[abs..abs + end].to_string());
            }
            pos = abs + 1;
        }
        links
    }

    pub fn extract_text(&self) -> String {
        let mut text = String::new();
        let mut in_tag = false;
        for ch in self.body.chars() {
            if ch == '<' { in_tag = true; }
            else if ch == '>' { in_tag = false; text.push(' '); }
            else if !in_tag { text.push(ch); }
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        words.join(" ")
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.body.contains(needle)
    }
}

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub pages: HashMap<String, BotPage>,
    pub errors: HashMap<String, String>,
    pub link_graph: HashMap<String, Vec<String>>,
}

impl CrawlResult {
    pub fn new() -> Self {
        CrawlResult { pages: HashMap::new(), errors: HashMap::new(), link_graph: HashMap::new() }
    }

    pub fn page_count(&self) -> usize { self.pages.len() }
    pub fn error_count(&self) -> usize { self.errors.len() }

    pub fn all_links(&self) -> HashSet<String> {
        self.link_graph.values().flat_map(|v| v.iter().cloned()).collect()
    }

    pub fn summary(&self) -> String {
        format!("Crawl: {} pages, {} errors, {} unique links",
                self.page_count(), self.error_count(), self.all_links().len())
    }
}

#[derive(Debug, Clone)]
pub struct BotAction {
    pub action_type: String,
    pub target: String,
    pub value: String,
    pub result: String,
    pub success: bool,
    pub error: String,
}

impl BotAction {
    fn new(action_type: &str, target: &str, value: &str) -> Self {
        BotAction {
            action_type: action_type.to_string(), target: target.to_string(),
            value: value.to_string(), result: String::new(), success: true, error: String::new(),
        }
    }
}

pub struct BotScript {
    pub name: String,
    pub actions: Vec<BotAction>,
    pub variables: HashMap<String, String>,
}

impl BotScript {
    pub fn new(name: &str) -> Self {
        BotScript { name: name.to_string(), actions: Vec::new(), variables: HashMap::new() }
    }
    pub fn navigate(&mut self, url: &str) -> &mut Self {
        self.actions.push(BotAction::new("navigate", url, ""));
        self
    }
    pub fn assert_check(&mut self, condition: &str) -> &mut Self {
        self.actions.push(BotAction::new("assert", condition, ""));
        self
    }
    pub fn extract(&mut self, target: &str, var_name: &str) -> &mut Self {
        self.actions.push(BotAction::new("extract", target, var_name));
        self
    }
    pub fn fill(&mut self, field: &str, value: &str) -> &mut Self {
        self.actions.push(BotAction::new("fill", field, value));
        self
    }
}

pub struct WebBot {
    page_store: HashMap<String, BotPage>,
    pub visited: HashSet<String>,
    pub history: Vec<String>,
    pub current_page: Option<BotPage>,
    pub form_data: HashMap<String, String>,
}

impl WebBot {
    pub fn new() -> Self {
        WebBot {
            page_store: HashMap::new(), visited: HashSet::new(),
            history: Vec::new(), current_page: None, form_data: HashMap::new(),
        }
    }

    pub fn add_page(&mut self, url: &str, body: &str, status: u16) {
        self.page_store.insert(url.to_string(), BotPage {
            url: url.to_string(), status_code: status, body: body.to_string(),
        });
    }

    pub fn fetch(&mut self, url: &str) -> BotPage {
        self.visited.insert(url.to_string());
        self.history.push(url.to_string());
        self.page_store.get(url).cloned().unwrap_or(BotPage {
            url: url.to_string(), status_code: 404, body: "Not Found".to_string(),
        })
    }

    pub fn navigate(&mut self, url: &str) -> &BotPage {
        let page = self.fetch(url);
        self.current_page = Some(page);
        self.current_page.as_ref().unwrap()
    }

    pub fn crawl(&mut self, start_url: &str, max_pages: usize) -> CrawlResult {
        let mut result = CrawlResult::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(start_url.to_string());
        let domain = start_url.split('/').nth(2).unwrap_or("").to_string();

        while let Some(url) = queue.pop_front() {
            if result.pages.len() >= max_pages { break; }
            if result.pages.contains_key(&url) || result.errors.contains_key(&url) { continue; }
            let page = self.fetch(&url);
            if page.status_code != 200 {
                result.errors.insert(url, format!("HTTP {}", page.status_code));
                continue;
            }
            let links = page.extract_links();
            result.link_graph.insert(url.clone(), links.clone());
            result.pages.insert(url, page);
            for link in links {
                if result.pages.contains_key(&link) { continue; }
                if !domain.is_empty() {
                    let link_domain = link.split('/').nth(2).unwrap_or("");
                    if link_domain != domain { continue; }
                }
                queue.push_back(link);
            }
        }
        result
    }

    pub fn execute_script(&mut self, script: &mut BotScript) -> Vec<BotAction> {
        let mut executed = Vec::new();
        for action in &script.actions {
            let mut result = BotAction::new(&action.action_type, &action.target, &action.value);
            match action.action_type.as_str() {
                "navigate" => {
                    let page = self.fetch(&action.target);
                    result.success = page.status_code == 200;
                    result.result = format!("HTTP {}", page.status_code);
                    self.current_page = Some(page);
                    if !result.success { result.error = "Navigation failed".into(); }
                }
                "assert" => {
                    if let Some(ref page) = self.current_page {
                        if action.target.starts_with("status:") {
                            let expected: u16 = action.target[7..].parse().unwrap_or(0);
                            result.success = page.status_code == expected;
                        } else if action.target.starts_with("contains:") {
                            result.success = page.body.contains(&action.target[9..]);
                        } else if action.target.starts_with("title:") {
                            result.success = page.title() == action.target[6..];
                        }
                        if !result.success { result.error = format!("Assertion failed: {}", action.target); }
                    } else {
                        result.success = false;
                        result.error = "No page loaded".into();
                    }
                }
                "extract" => {
                    if let Some(ref page) = self.current_page {
                        result.result = format!("extracted from {}", page.url);
                    }
                }
                "fill" => {
                    self.form_data.insert(action.target.clone(), action.value.clone());
                    result.result = format!("Set {}={}", action.target, action.value);
                }
                _ => {}
            }
            let failed = !result.success;
            executed.push(result);
            if failed { break; }
        }
        executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_title() {
        let p = BotPage { url: "u".into(), status_code: 200,
            body: "<html><title>Hello</title></html>".into() };
        assert_eq!(p.title(), "Hello");
    }

    #[test]
    fn test_extract_links() {
        let p = BotPage { url: "u".into(), status_code: 200,
            body: r#"<a href="https://a.com">A</a><a href="https://b.com">B</a>"#.into() };
        let links = p.extract_links();
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://a.com".to_string()));
    }

    #[test]
    fn test_extract_text() {
        let p = BotPage { url: "u".into(), status_code: 200,
            body: "<p>Hello <b>world</b></p>".into() };
        assert_eq!(p.extract_text(), "Hello world");
    }

    #[test]
    fn test_crawl() {
        let mut bot = WebBot::new();
        bot.add_page("https://em.com/", r#"<a href="https://em.com/about">About</a>"#, 200);
        bot.add_page("https://em.com/about", "<p>About EM</p>", 200);
        let result = bot.crawl("https://em.com/", 10);
        assert_eq!(result.page_count(), 2);
        assert!(result.pages.contains_key("https://em.com/"));
        assert!(result.pages.contains_key("https://em.com/about"));
    }

    #[test]
    fn test_crawl_404() {
        let mut bot = WebBot::new();
        bot.add_page("https://em.com/", r#"<a href="https://em.com/missing">X</a>"#, 200);
        let result = bot.crawl("https://em.com/", 10);
        assert_eq!(result.page_count(), 1);
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_script_navigate_assert() {
        let mut bot = WebBot::new();
        bot.add_page("https://em.com/", "<html><title>EM</title></html>", 200);
        let mut script = BotScript::new("test");
        script.navigate("https://em.com/").assert_check("status:200").assert_check("title:EM");
        let actions = bot.execute_script(&mut script);
        assert_eq!(actions.len(), 3);
        assert!(actions.iter().all(|a| a.success));
    }

    #[test]
    fn test_script_failed_assert() {
        let mut bot = WebBot::new();
        bot.add_page("https://em.com/", "<html><title>EM</title></html>", 200);
        let mut script = BotScript::new("test");
        script.navigate("https://em.com/").assert_check("title:Wrong");
        let actions = bot.execute_script(&mut script);
        assert_eq!(actions.len(), 2);
        assert!(actions[0].success);
        assert!(!actions[1].success);
    }

    #[test]
    fn test_form_fill() {
        let mut bot = WebBot::new();
        bot.add_page("https://em.com/login", "<form></form>", 200);
        let mut script = BotScript::new("login");
        script.navigate("https://em.com/login")
              .fill("username", "nathan")
              .fill("password", "rosetta");
        bot.execute_script(&mut script);
        assert_eq!(bot.form_data.get("username").map(|s| s.as_str()), Some("nathan"));
        assert_eq!(bot.form_data.get("password").map(|s| s.as_str()), Some("rosetta"));
    }
}
