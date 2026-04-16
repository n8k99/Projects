//! News Ticker and Game Scores — real-time prioritized message stream.

use std::fmt;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Category {
    News,
    Sports,
    Finance,
    Alert,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::News => write!(f, "news"),
            Category::Sports => write!(f, "sports"),
            Category::Finance => write!(f, "finance"),
            Category::Alert => write!(f, "alert"),
        }
    }
}

impl Category {
    pub fn from_str(s: &str) -> Option<Category> {
        match s {
            "news" => Some(Category::News),
            "sports" => Some(Category::Sports),
            "finance" => Some(Category::Finance),
            "alert" => Some(Category::Alert),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TickerItem {
    pub text: String,
    pub category: Category,
    pub priority: u8,
    pub timestamp: SystemTime,
    created_at: Instant,
    ttl: Option<Duration>,
}

impl TickerItem {
    pub fn is_expired(&self) -> bool {
        match self.ttl {
            Some(ttl) => self.created_at.elapsed() >= ttl,
            None => false,
        }
    }
}

pub struct NewsTicker {
    items: Vec<TickerItem>,
}

impl NewsTicker {
    pub fn new() -> Self {
        NewsTicker { items: Vec::new() }
    }

    pub fn post(
        &mut self,
        text: &str,
        category: Category,
        priority: u8,
        ttl_seconds: Option<u64>,
    ) -> &TickerItem {
        assert!((1..=5).contains(&priority), "Priority must be 1-5");

        let item = TickerItem {
            text: text.to_string(),
            category,
            priority,
            timestamp: SystemTime::now(),
            created_at: Instant::now(),
            ttl: ttl_seconds.map(Duration::from_secs),
        };
        self.items.push(item);
        self.items.last().unwrap()
    }

    pub fn breaking(&mut self, text: &str) -> &TickerItem {
        self.post(text, Category::Alert, 5, None)
    }

    pub fn get_current(&self) -> Vec<&TickerItem> {
        let mut active: Vec<&TickerItem> = self.items.iter().filter(|i| !i.is_expired()).collect();
        active.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });
        active
    }

    pub fn get_by_category(&self, category: &Category) -> Vec<&TickerItem> {
        self.get_current()
            .into_iter()
            .filter(|i| &i.category == category)
            .collect()
    }

    pub fn expire_old(&mut self) -> usize {
        let before = self.items.len();
        self.items.retain(|i| !i.is_expired());
        before - self.items.len()
    }

    pub fn display(&self) -> String {
        let current = self.get_current();
        if current.is_empty() {
            return ">>> No items >>>".to_string();
        }
        let texts: Vec<&str> = current.iter().map(|i| i.text.as_str()).collect();
        format!(">>> {} >>>", texts.join(" | "))
    }
}

impl Default for NewsTicker {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut ticker = NewsTicker::new();

    ticker.post("Markets rally on strong earnings", Category::Finance, 3, None);
    ticker.post("Lakers defeat Celtics 112-108", Category::Sports, 2, None);
    ticker.post("New climate accord signed by 40 nations", Category::News, 4, None);
    ticker.post("Flash sale: tech stocks surge 5%", Category::Finance, 2, Some(1));

    println!("=== Full Ticker ===");
    println!("{}", ticker.display());

    println!("\n=== Sports Only ===");
    for item in ticker.get_by_category(&Category::Sports) {
        println!("  [{}] {}", item.priority, item.text);
    }

    std::thread::sleep(Duration::from_millis(1100));

    let removed = ticker.expire_old();
    println!("\n=== After expiry ({} removed) ===", removed);
    println!("{}", ticker.display());

    ticker.breaking("EARTHQUAKE: 7.2 magnitude off Pacific coast");
    println!("\n=== BREAKING NEWS ===");
    println!("{}", ticker.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_post_and_get_current() {
        let mut ticker = NewsTicker::new();
        ticker.post("headline one", Category::News, 3, None);
        ticker.post("headline two", Category::Finance, 5, None);

        let current = ticker.get_current();
        assert_eq!(current.len(), 2);
        assert_eq!(current[0].priority, 5);
        assert_eq!(current[1].priority, 3);
    }

    #[test]
    fn test_breaking_is_max_priority() {
        let mut ticker = NewsTicker::new();
        ticker.breaking("urgent news");
        let current = ticker.get_current();
        assert_eq!(current[0].priority, 5);
        assert_eq!(current[0].category, Category::Alert);
    }

    #[test]
    fn test_get_by_category() {
        let mut ticker = NewsTicker::new();
        ticker.post("sports item", Category::Sports, 2, None);
        ticker.post("news item", Category::News, 3, None);
        ticker.post("another sports", Category::Sports, 4, None);

        let sports = ticker.get_by_category(&Category::Sports);
        assert_eq!(sports.len(), 2);
        assert!(sports.iter().all(|i| i.category == Category::Sports));
    }

    #[test]
    fn test_expiry() {
        let mut ticker = NewsTicker::new();
        ticker.post("expires fast", Category::News, 3, Some(0));
        thread::sleep(Duration::from_millis(50));

        let removed = ticker.expire_old();
        assert_eq!(removed, 1);
        assert!(ticker.get_current().is_empty());
    }

    #[test]
    fn test_display_format() {
        let mut ticker = NewsTicker::new();
        ticker.post("alpha", Category::News, 3, None);
        ticker.post("beta", Category::Finance, 3, None);

        let output = ticker.display();
        assert!(output.starts_with(">>>"));
        assert!(output.ends_with(">>>"));
        assert!(output.contains(" | "));
    }

    #[test]
    fn test_empty_display() {
        let ticker = NewsTicker::new();
        assert_eq!(ticker.display(), ">>> No items >>>");
    }

    #[test]
    #[should_panic(expected = "Priority must be 1-5")]
    fn test_invalid_priority() {
        let mut ticker = NewsTicker::new();
        ticker.post("bad", Category::News, 0, None);
    }
}
