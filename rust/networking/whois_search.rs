// Whois Search Tool — query and parse domain registration data.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct WhoisContact {
    pub name: String,
    pub organization: String,
    pub email: String,
    pub phone: String,
    pub city: String,
    pub state: String,
    pub country: String,
}

impl WhoisContact {
    pub fn summary(&self) -> String {
        let parts: Vec<&str> = [self.name.as_str(), self.organization.as_str(),
                                 self.city.as_str(), self.country.as_str()]
            .iter().filter(|s| !s.is_empty()).copied().collect();
        if parts.is_empty() { "(redacted)".to_string() } else { parts.join(", ") }
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.organization.is_empty() && self.email.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct WhoisRecord {
    pub domain: String,
    pub registrar: String,
    pub registrant: Option<WhoisContact>,
    pub name_servers: Vec<String>,
    pub status: Vec<String>,
    pub created_date: String,
    pub updated_date: String,
    pub expiry_date: String,
    pub dnssec: String,
    pub raw_text: String,
}

impl WhoisRecord {
    pub fn tld(&self) -> &str {
        self.domain.rsplit('.').next().unwrap_or("")
    }

    pub fn summary(&self) -> String {
        let mut lines = vec![format!("Domain: {}", self.domain)];
        if !self.registrar.is_empty() {
            lines.push(format!("  Registrar: {}", self.registrar));
        }
        if let Some(ref r) = self.registrant {
            lines.push(format!("  Registrant: {}", r.summary()));
        }
        if !self.name_servers.is_empty() {
            lines.push(format!("  Name Servers: {}", self.name_servers.join(", ")));
        }
        if !self.created_date.is_empty() {
            lines.push(format!("  Created: {}", self.created_date));
        }
        if !self.expiry_date.is_empty() {
            lines.push(format!("  Expires: {}", self.expiry_date));
        }
        if !self.status.is_empty() {
            lines.push(format!("  Status: {}", self.status.join(", ")));
        }
        lines.join("\n")
    }
}

fn find_field<'a>(raw: &'a str, key: &str) -> &'a str {
    for line in raw.lines() {
        let lower = line.to_lowercase();
        let key_lower = key.to_lowercase();
        if lower.starts_with(&key_lower) {
            if let Some(pos) = line.find(':') {
                return line[pos + 1..].trim();
            }
        }
    }
    ""
}

fn find_all_fields<'a>(raw: &'a str, key: &str) -> Vec<&'a str> {
    let mut results = Vec::new();
    let key_lower = key.to_lowercase();
    for line in raw.lines() {
        if line.to_lowercase().starts_with(&key_lower) {
            if let Some(pos) = line.find(':') {
                let val = line[pos + 1..].trim();
                if !val.is_empty() {
                    results.push(val);
                }
            }
        }
    }
    results
}

pub fn parse_whois_text(domain: &str, raw: &str) -> WhoisRecord {
    let mut record = WhoisRecord {
        domain: domain.to_lowercase(),
        raw_text: raw.to_string(),
        ..Default::default()
    };

    record.registrar = find_field(raw, "Registrar").to_string();
    record.dnssec = find_field(raw, "DNSSEC").to_string();
    record.created_date = find_field(raw, "Creation Date").to_string();
    if record.created_date.is_empty() {
        record.created_date = find_field(raw, "Created Date").to_string();
    }
    record.updated_date = find_field(raw, "Updated Date").to_string();
    record.expiry_date = find_field(raw, "Registry Expiry Date").to_string();
    if record.expiry_date.is_empty() {
        record.expiry_date = find_field(raw, "Expiration Date").to_string();
    }

    record.name_servers = find_all_fields(raw, "Name Server")
        .into_iter().map(|s| s.to_lowercase()).collect();

    record.status = find_all_fields(raw, "Status")
        .into_iter().map(|s| s.to_string()).collect();
    // Also check "Domain Status"
    for s in find_all_fields(raw, "Domain Status") {
        let s = s.to_string();
        if !record.status.contains(&s) {
            record.status.push(s);
        }
    }

    let mut registrant = WhoisContact::default();
    registrant.name = find_field(raw, "Registrant Name").to_string();
    registrant.organization = find_field(raw, "Registrant Organization").to_string();
    registrant.email = find_field(raw, "Registrant Email").to_string();
    registrant.country = find_field(raw, "Registrant Country").to_string();
    registrant.city = find_field(raw, "Registrant City").to_string();
    registrant.state = find_field(raw, "Registrant State").to_string();
    if !registrant.is_empty() {
        record.registrant = Some(registrant);
    }

    record
}

pub struct WhoisClient {
    cache: HashMap<String, WhoisRecord>,
    simulated: HashMap<String, String>,
}

impl WhoisClient {
    pub fn new() -> Self {
        WhoisClient { cache: HashMap::new(), simulated: HashMap::new() }
    }

    pub fn add_simulated(&mut self, domain: &str, raw: &str) {
        self.simulated.insert(domain.to_lowercase(), raw.to_string());
    }

    pub fn query(&mut self, domain: &str) -> WhoisRecord {
        let domain = domain.to_lowercase();
        if let Some(r) = self.cache.get(&domain) {
            return r.clone();
        }
        let raw = self.simulated.get(&domain).cloned().unwrap_or_default();
        let record = parse_whois_text(&domain, &raw);
        self.cache.insert(domain, record.clone());
        record
    }

    pub fn bulk_query(&mut self, domains: &[&str]) -> Vec<WhoisRecord> {
        domains.iter().map(|d| self.query(d)).collect()
    }

    pub fn registrar_breakdown(&mut self, domains: &[&str]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for d in domains {
            let r = self.query(d);
            let key = if r.registrar.is_empty() { "unknown".to_string() } else { r.registrar };
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

pub fn compare_records(before: &WhoisRecord, after: &WhoisRecord) -> Vec<(String, String, String)> {
    let mut changes = Vec::new();
    if before.registrar != after.registrar {
        changes.push(("registrar".into(), before.registrar.clone(), after.registrar.clone()));
    }
    if before.name_servers != after.name_servers {
        changes.push(("name_servers".into(),
                       before.name_servers.join(","), after.name_servers.join(",")));
    }
    if before.expiry_date != after.expiry_date {
        changes.push(("expiry_date".into(), before.expiry_date.clone(), after.expiry_date.clone()));
    }
    if before.status != after.status {
        changes.push(("status".into(), before.status.join(","), after.status.join(",")));
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_WHOIS: &str = r#"
Domain Name: example.com
Registrar: Test Registrar, Inc.
Creation Date: 2020-01-15T00:00:00Z
Updated Date: 2024-01-15T00:00:00Z
Registry Expiry Date: 2026-01-15T00:00:00Z
Registrant Name: John Doe
Registrant Organization: Example Corp
Registrant Country: US
Registrant City: New York
Name Server: ns1.example.com
Name Server: ns2.example.com
Domain Status: clientTransferProhibited
DNSSEC: unsigned
"#;

    #[test]
    fn test_parse_basic_fields() {
        let r = parse_whois_text("example.com", SAMPLE_WHOIS);
        assert_eq!(r.domain, "example.com");
        assert_eq!(r.registrar, "Test Registrar, Inc.");
        assert_eq!(r.created_date, "2020-01-15T00:00:00Z");
        assert_eq!(r.expiry_date, "2026-01-15T00:00:00Z");
        assert_eq!(r.dnssec, "unsigned");
    }

    #[test]
    fn test_parse_name_servers() {
        let r = parse_whois_text("example.com", SAMPLE_WHOIS);
        assert_eq!(r.name_servers.len(), 2);
        assert!(r.name_servers.contains(&"ns1.example.com".to_string()));
        assert!(r.name_servers.contains(&"ns2.example.com".to_string()));
    }

    #[test]
    fn test_parse_registrant() {
        let r = parse_whois_text("example.com", SAMPLE_WHOIS);
        let reg = r.registrant.unwrap();
        assert_eq!(reg.name, "John Doe");
        assert_eq!(reg.organization, "Example Corp");
        assert_eq!(reg.country, "US");
        assert_eq!(reg.city, "New York");
    }

    #[test]
    fn test_parse_status() {
        let r = parse_whois_text("example.com", SAMPLE_WHOIS);
        assert!(r.status.iter().any(|s| s.contains("clientTransferProhibited")));
    }

    #[test]
    fn test_tld() {
        let r = parse_whois_text("example.com", "");
        assert_eq!(r.tld(), "com");
        let r = parse_whois_text("example.co.uk", "");
        assert_eq!(r.tld(), "uk");
    }

    #[test]
    fn test_client_cache() {
        let mut client = WhoisClient::new();
        client.add_simulated("test.com", SAMPLE_WHOIS);
        let r1 = client.query("test.com");
        let r2 = client.query("test.com"); // cached
        assert_eq!(r1.registrar, r2.registrar);
    }

    #[test]
    fn test_compare_records() {
        let before = parse_whois_text("example.com", SAMPLE_WHOIS);
        let mut after = before.clone();
        after.registrar = "New Registrar".to_string();
        after.name_servers = vec!["ns1.new.com".to_string()];
        let changes = compare_records(&before, &after);
        assert_eq!(changes.len(), 2);
        assert!(changes.iter().any(|(f, _, _)| f == "registrar"));
        assert!(changes.iter().any(|(f, _, _)| f == "name_servers"));
    }

    #[test]
    fn test_registrar_breakdown() {
        let mut client = WhoisClient::new();
        client.add_simulated("a.com", "Registrar: RegA\n");
        client.add_simulated("b.com", "Registrar: RegA\n");
        client.add_simulated("c.com", "Registrar: RegB\n");
        let bd = client.registrar_breakdown(&["a.com", "b.com", "c.com"]);
        assert_eq!(bd["RegA"], 2);
        assert_eq!(bd["RegB"], 1);
    }
}
