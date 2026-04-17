// Site Checker with Time Scheduling — periodic health monitoring of web endpoints.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteStatus {
    Up, Down, Degraded, Unknown,
}

impl SiteStatus {
    pub fn name(&self) -> &'static str {
        match self { Self::Up => "up", Self::Down => "down",
                     Self::Degraded => "degraded", Self::Unknown => "unknown" }
    }
}

#[derive(Debug, Clone)]
pub struct CheckConfig {
    pub url: String,
    pub name: String,
    pub interval_secs: u64,
    pub timeout_ms: u64,
    pub expected_status: u16,
    pub max_response_ms: f64,
}

impl CheckConfig {
    pub fn new(url: &str, name: &str, interval_secs: u64) -> Self {
        CheckConfig {
            url: url.to_string(),
            name: if name.is_empty() { url.to_string() } else { name.to_string() },
            interval_secs, timeout_ms: 10000, expected_status: 200, max_response_ms: 5000.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckResultEntry {
    pub url: String,
    pub status: SiteStatus,
    pub http_status: u16,
    pub response_time_ms: f64,
    pub error: String,
    pub checked_at: u64,
}

#[derive(Debug)]
pub struct SiteHistory {
    pub config: CheckConfig,
    pub checks: Vec<CheckResultEntry>,
    pub consecutive_failures: u32,
}

impl SiteHistory {
    pub fn new(config: CheckConfig) -> Self {
        SiteHistory { config, checks: Vec::new(), consecutive_failures: 0 }
    }

    pub fn current_status(&self) -> SiteStatus {
        self.checks.last().map(|c| c.status).unwrap_or(SiteStatus::Unknown)
    }

    pub fn uptime_pct(&self) -> f64 {
        if self.checks.is_empty() { return 0.0; }
        let up = self.checks.iter().filter(|c| c.status == SiteStatus::Up).count();
        (up as f64 / self.checks.len() as f64) * 100.0
    }

    pub fn avg_response_ms(&self) -> f64 {
        let times: Vec<f64> = self.checks.iter()
            .filter(|c| c.status == SiteStatus::Up)
            .map(|c| c.response_time_ms).collect();
        if times.is_empty() { 0.0 } else { times.iter().sum::<f64>() / times.len() as f64 }
    }

    pub fn add_check(&mut self, result: CheckResultEntry) -> Option<String> {
        let prev = self.current_status();
        match result.status {
            SiteStatus::Down | SiteStatus::Degraded => self.consecutive_failures += 1,
            _ => self.consecutive_failures = 0,
        }
        self.checks.push(result.clone());
        if prev != result.status && prev != SiteStatus::Unknown {
            Some(format!("{}: {} -> {}", self.config.name, prev.name(), result.status.name()))
        } else {
            None
        }
    }

    pub fn summary(&self) -> String {
        format!("{}: {} | uptime {:.1}% | avg {:.0}ms | {} checks | {} consecutive failures",
                self.config.name, self.current_status().name(), self.uptime_pct(),
                self.avg_response_ms(), self.checks.len(), self.consecutive_failures)
    }
}

#[derive(Debug)]
struct ScheduleEntry {
    config: CheckConfig,
    next_run: u64,
    enabled: bool,
}

pub struct SiteChecker {
    schedules: HashMap<String, ScheduleEntry>,
    pub histories: HashMap<String, SiteHistory>,
}

impl SiteChecker {
    pub fn new() -> Self {
        SiteChecker { schedules: HashMap::new(), histories: HashMap::new() }
    }

    pub fn add_site(&mut self, config: CheckConfig) {
        let url = config.url.clone();
        self.schedules.insert(url.clone(), ScheduleEntry {
            config: config.clone(), next_run: now_secs(), enabled: true,
        });
        self.histories.insert(url, SiteHistory::new(config));
    }

    pub fn remove_site(&mut self, url: &str) {
        self.schedules.remove(url);
        self.histories.remove(url);
    }

    /// Simulate a check result (for testing — actual HTTP not implemented in std-only Rust)
    pub fn record_check(&mut self, result: CheckResultEntry) -> Option<String> {
        if let Some(history) = self.histories.get_mut(&result.url) {
            let alert = history.add_check(result);
            return alert;
        }
        None
    }

    pub fn sites_by_status(&self, status: SiteStatus) -> Vec<&str> {
        self.histories.iter()
            .filter(|(_, h)| h.current_status() == status)
            .map(|(url, _)| url.as_str())
            .collect()
    }

    pub fn dashboard(&self) -> String {
        let mut lines = vec!["Site Monitor Dashboard".to_string(), "=".repeat(60)];
        let mut urls: Vec<&String> = self.histories.keys().collect();
        urls.sort();
        for url in urls {
            lines.push(self.histories[url].summary());
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(url: &str, status: SiteStatus, http: u16, ms: f64) -> CheckResultEntry {
        CheckResultEntry {
            url: url.to_string(), status, http_status: http,
            response_time_ms: ms, error: String::new(), checked_at: now_secs(),
        }
    }

    #[test]
    fn test_site_history_uptime() {
        let config = CheckConfig::new("http://example.com", "Example", 60);
        let mut history = SiteHistory::new(config);
        history.add_check(make_result("http://example.com", SiteStatus::Up, 200, 100.0));
        history.add_check(make_result("http://example.com", SiteStatus::Up, 200, 200.0));
        history.add_check(make_result("http://example.com", SiteStatus::Down, 0, 0.0));
        assert!((history.uptime_pct() - 66.6).abs() < 1.0);
        assert_eq!(history.consecutive_failures, 1);
    }

    #[test]
    fn test_status_change_alert() {
        let config = CheckConfig::new("http://example.com", "Example", 60);
        let mut history = SiteHistory::new(config);
        let alert = history.add_check(make_result("http://example.com", SiteStatus::Up, 200, 100.0));
        assert!(alert.is_none()); // first check, no prior status
        let alert = history.add_check(make_result("http://example.com", SiteStatus::Down, 0, 0.0));
        assert!(alert.is_some());
        assert!(alert.unwrap().contains("up -> down"));
    }

    #[test]
    fn test_consecutive_failures() {
        let config = CheckConfig::new("http://example.com", "Example", 60);
        let mut history = SiteHistory::new(config);
        history.add_check(make_result("http://example.com", SiteStatus::Down, 0, 0.0));
        history.add_check(make_result("http://example.com", SiteStatus::Down, 0, 0.0));
        history.add_check(make_result("http://example.com", SiteStatus::Down, 0, 0.0));
        assert_eq!(history.consecutive_failures, 3);
        history.add_check(make_result("http://example.com", SiteStatus::Up, 200, 50.0));
        assert_eq!(history.consecutive_failures, 0);
    }

    #[test]
    fn test_avg_response() {
        let config = CheckConfig::new("http://example.com", "Example", 60);
        let mut history = SiteHistory::new(config);
        history.add_check(make_result("http://example.com", SiteStatus::Up, 200, 100.0));
        history.add_check(make_result("http://example.com", SiteStatus::Up, 200, 300.0));
        history.add_check(make_result("http://example.com", SiteStatus::Down, 0, 0.0)); // excluded
        assert!((history.avg_response_ms() - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_checker_sites_by_status() {
        let mut checker = SiteChecker::new();
        checker.add_site(CheckConfig::new("http://a.com", "A", 60));
        checker.add_site(CheckConfig::new("http://b.com", "B", 60));
        checker.record_check(make_result("http://a.com", SiteStatus::Up, 200, 50.0));
        checker.record_check(make_result("http://b.com", SiteStatus::Down, 0, 0.0));
        let up = checker.sites_by_status(SiteStatus::Up);
        assert_eq!(up.len(), 1);
        assert!(up.contains(&"http://a.com"));
        let down = checker.sites_by_status(SiteStatus::Down);
        assert_eq!(down.len(), 1);
    }

    #[test]
    fn test_dashboard() {
        let mut checker = SiteChecker::new();
        checker.add_site(CheckConfig::new("http://example.com", "Example", 60));
        checker.record_check(make_result("http://example.com", SiteStatus::Up, 200, 100.0));
        let dash = checker.dashboard();
        assert!(dash.contains("Example"));
        assert!(dash.contains("up"));
    }
}
