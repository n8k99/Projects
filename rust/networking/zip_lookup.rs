// Zip / Postal Code Lookup — resolve postal codes to location data.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PostalResult {
    pub postal_code: String,
    pub country_code: String,
    pub country_name: String,
    pub state: String,
    pub state_abbr: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

impl PostalResult {
    pub fn summary(&self) -> String {
        let st = if self.state_abbr.is_empty() { &self.state } else { &self.state_abbr };
        let parts: Vec<&str> = [self.city.as_str(), st, self.country_code.as_str()]
            .iter().filter(|s| !s.is_empty()).copied().collect();
        format!("{} -> {} [{:.4}, {:.4}]", self.postal_code, parts.join(", "),
                self.latitude, self.longitude)
    }
}

#[derive(Debug, Clone)]
struct PostalEntry {
    postal_code: String,
    country_code: String,
    country_name: String,
    state: String,
    state_abbr: String,
    city: String,
    latitude: f64,
    longitude: f64,
    timezone: String,
}

impl PostalEntry {
    fn to_result(&self) -> PostalResult {
        PostalResult {
            postal_code: self.postal_code.clone(), country_code: self.country_code.clone(),
            country_name: self.country_name.clone(), state: self.state.clone(),
            state_abbr: self.state_abbr.clone(), city: self.city.clone(),
            latitude: self.latitude, longitude: self.longitude, timezone: self.timezone.clone(),
        }
    }
}

fn builtin_entries() -> Vec<PostalEntry> {
    let e = |code: &str, cc: &str, cn: &str, st: &str, sa: &str, city: &str, lat: f64, lon: f64, tz: &str| {
        PostalEntry {
            postal_code: code.into(), country_code: cc.into(), country_name: cn.into(),
            state: st.into(), state_abbr: sa.into(), city: city.into(),
            latitude: lat, longitude: lon, timezone: tz.into(),
        }
    };
    vec![
        e("32099", "US", "United States", "Florida", "FL", "Jacksonville", 30.3322, -81.6557, "America/New_York"),
        e("32202", "US", "United States", "Florida", "FL", "Jacksonville", 30.3280, -81.6550, "America/New_York"),
        e("32244", "US", "United States", "Florida", "FL", "Jacksonville", 30.2266, -81.7360, "America/New_York"),
        e("32084", "US", "United States", "Florida", "FL", "St. Augustine", 29.8947, -81.3145, "America/New_York"),
        e("33101", "US", "United States", "Florida", "FL", "Miami", 25.7617, -80.1918, "America/New_York"),
        e("10001", "US", "United States", "New York", "NY", "New York", 40.7484, -73.9967, "America/New_York"),
        e("90210", "US", "United States", "California", "CA", "Beverly Hills", 34.0901, -118.4065, "America/Los_Angeles"),
        e("94105", "US", "United States", "California", "CA", "San Francisco", 37.7893, -122.3932, "America/Los_Angeles"),
        e("02101", "US", "United States", "Massachusetts", "MA", "Boston", 42.3601, -71.0589, "America/New_York"),
        e("60601", "US", "United States", "Illinois", "IL", "Chicago", 41.8819, -87.6278, "America/Chicago"),
        e("SW1A1AA", "GB", "United Kingdom", "England", "", "London", 51.5014, -0.1419, "Europe/London"),
        e("75001", "FR", "France", "Île-de-France", "", "Paris", 48.8606, 2.3376, "Europe/Paris"),
        e("10115", "DE", "Germany", "Berlin", "", "Berlin", 52.5326, 13.3884, "Europe/Berlin"),
        e("2000", "AU", "Australia", "New South Wales", "NSW", "Sydney", -33.8688, 151.2093, "Australia/Sydney"),
    ]
}

fn normalize_code(code: &str) -> String {
    code.to_uppercase().replace(' ', "")
}

pub struct PostalDatabase {
    by_code: HashMap<String, PostalEntry>,
    entries: Vec<PostalEntry>,
}

impl PostalDatabase {
    pub fn new() -> Self {
        let entries = builtin_entries();
        let mut by_code = HashMap::new();
        for e in &entries {
            by_code.insert(normalize_code(&e.postal_code), e.clone());
        }
        PostalDatabase { by_code, entries }
    }

    pub fn lookup(&self, code: &str) -> Option<PostalResult> {
        self.by_code.get(&normalize_code(code)).map(|e| e.to_result())
    }

    pub fn search_by_city(&self, city: &str) -> Vec<PostalResult> {
        let city_lower = city.to_lowercase();
        self.entries.iter()
            .filter(|e| e.city.to_lowercase() == city_lower)
            .map(|e| e.to_result())
            .collect()
    }

    pub fn search_by_state(&self, state: &str) -> Vec<PostalResult> {
        let s = state.to_lowercase();
        self.entries.iter()
            .filter(|e| e.state.to_lowercase() == s || e.state_abbr.to_lowercase() == s)
            .map(|e| e.to_result())
            .collect()
    }

    pub fn distance_between(&self, a: &str, b: &str) -> Option<f64> {
        let ra = self.lookup(a)?;
        let rb = self.lookup(b)?;
        Some(postal_haversine(ra.latitude, ra.longitude, rb.latitude, rb.longitude))
    }

    pub fn codes_within_radius(&self, center: &str, radius_km: f64) -> Vec<(PostalResult, f64)> {
        let c = match self.lookup(center) { Some(r) => r, None => return vec![] };
        let center_norm = normalize_code(center);
        let mut results: Vec<(PostalResult, f64)> = self.entries.iter()
            .filter(|e| normalize_code(&e.postal_code) != center_norm)
            .filter_map(|e| {
                let d = postal_haversine(c.latitude, c.longitude, e.latitude, e.longitude);
                if d <= radius_km { Some((e.to_result(), d)) } else { None }
            })
            .collect();
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results
    }

    pub fn state_breakdown(&self, codes: &[&str]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for code in codes {
            let key = self.lookup(code)
                .map(|r| if r.state_abbr.is_empty() { r.state } else { r.state_abbr })
                .unwrap_or_else(|| "unknown".to_string());
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

fn postal_haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known() {
        let db = PostalDatabase::new();
        let r = db.lookup("32202").unwrap();
        assert_eq!(r.city, "Jacksonville");
        assert_eq!(r.state_abbr, "FL");
        assert_eq!(r.country_code, "US");
    }

    #[test]
    fn test_lookup_international() {
        let db = PostalDatabase::new();
        let r = db.lookup("SW1A 1AA").unwrap();
        assert_eq!(r.city, "London");
        assert_eq!(r.country_code, "GB");
    }

    #[test]
    fn test_lookup_unknown() {
        let db = PostalDatabase::new();
        assert!(db.lookup("99999").is_none());
    }

    #[test]
    fn test_search_by_city() {
        let db = PostalDatabase::new();
        let results = db.search_by_city("Jacksonville");
        assert!(results.len() >= 3);
        assert!(results.iter().all(|r| r.city == "Jacksonville"));
    }

    #[test]
    fn test_search_by_state() {
        let db = PostalDatabase::new();
        let results = db.search_by_state("FL");
        assert!(results.len() >= 4);
    }

    #[test]
    fn test_distance() {
        let db = PostalDatabase::new();
        let d = db.distance_between("32202", "32084").unwrap();
        // Jacksonville to St. Augustine ~ 50 km
        assert!(d > 40.0 && d < 70.0);
    }

    #[test]
    fn test_radius_search() {
        let db = PostalDatabase::new();
        let nearby = db.codes_within_radius("32202", 100.0);
        // Should find other Jacksonville zips and St. Augustine
        assert!(!nearby.is_empty());
        assert!(nearby.iter().any(|(r, _)| r.city == "St. Augustine"));
    }

    #[test]
    fn test_state_breakdown() {
        let db = PostalDatabase::new();
        let bd = db.state_breakdown(&["32202", "32084", "10001", "90210"]);
        assert_eq!(bd["FL"], 2);
        assert_eq!(bd["NY"], 1);
        assert_eq!(bd["CA"], 1);
    }
}
