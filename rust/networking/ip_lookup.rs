// Country from IP Lookup — resolve IP addresses to geographic location data.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GeoResult {
    pub ip: String,
    pub country_code: String,
    pub country_name: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub is_private: bool,
}

impl GeoResult {
    pub fn summary(&self) -> String {
        if self.is_private {
            return format!("{} -> private/reserved", self.ip);
        }
        let parts: Vec<&str> = [self.city.as_str(), self.region.as_str(), self.country_name.as_str()]
            .iter().filter(|s| !s.is_empty()).copied().collect();
        format!("{} -> {} ({}) [{:.4}, {:.4}]",
                self.ip, parts.join(", "), self.country_code, self.latitude, self.longitude)
    }

    fn empty(ip: &str) -> Self {
        GeoResult {
            ip: ip.to_string(), country_code: String::new(), country_name: String::new(),
            region: String::new(), city: String::new(), latitude: 0.0, longitude: 0.0,
            timezone: String::new(), is_private: false,
        }
    }

    fn private(ip: &str) -> Self {
        let mut r = Self::empty(ip);
        r.is_private = true;
        r
    }
}

#[derive(Debug, Clone)]
pub struct IpRange {
    pub start: u32,
    pub end: u32,
    pub country_code: String,
    pub country_name: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

pub fn ip_to_u32(ip: &str) -> u32 {
    let parts: Vec<u32> = ip.split('.').filter_map(|s| s.parse().ok()).collect();
    if parts.len() != 4 { return 0; }
    (parts[0] << 24) | (parts[1] << 16) | (parts[2] << 8) | parts[3]
}

pub fn u32_to_ip(n: u32) -> String {
    format!("{}.{}.{}.{}", (n >> 24) & 0xFF, (n >> 16) & 0xFF, (n >> 8) & 0xFF, n & 0xFF)
}

pub fn is_private_ip(ip: &str) -> bool {
    let n = ip_to_u32(ip);
    (0x0A000000..=0x0AFFFFFF).contains(&n) ||  // 10.0.0.0/8
    (0xAC100000..=0xAC1FFFFF).contains(&n) ||  // 172.16.0.0/12
    (0xC0A80000..=0xC0A8FFFF).contains(&n) ||  // 192.168.0.0/16
    (0x7F000000..=0x7FFFFFFF).contains(&n) ||  // 127.0.0.0/8
    n <= 0x00FFFFFF                              // 0.0.0.0/8
}

fn builtin_ranges() -> Vec<IpRange> {
    let r = |s: &str, e: &str, cc: &str, cn: &str, rg: &str, ct: &str, lat: f64, lon: f64, tz: &str| {
        IpRange {
            start: ip_to_u32(s), end: ip_to_u32(e),
            country_code: cc.into(), country_name: cn.into(),
            region: rg.into(), city: ct.into(),
            latitude: lat, longitude: lon, timezone: tz.into(),
        }
    };
    vec![
        r("1.0.0.0", "1.0.0.255", "AU", "Australia", "Queensland", "Brisbane", -27.4679, 153.0281, "Australia/Brisbane"),
        r("1.1.1.0", "1.1.1.255", "AU", "Australia", "New South Wales", "Sydney", -33.8688, 151.2093, "Australia/Sydney"),
        r("8.8.4.0", "8.8.8.255", "US", "United States", "California", "Mountain View", 37.3861, -122.0839, "America/Los_Angeles"),
        r("9.0.0.0", "9.255.255.255", "US", "United States", "New York", "Armonk", 41.1085, -73.7205, "America/New_York"),
        r("13.0.0.0", "13.255.255.255", "US", "United States", "Virginia", "Ashburn", 39.0438, -77.4874, "America/New_York"),
        r("17.0.0.0", "17.255.255.255", "US", "United States", "California", "Cupertino", 37.3230, -122.0322, "America/Los_Angeles"),
        r("41.0.0.0", "41.255.255.255", "ZA", "South Africa", "Gauteng", "Johannesburg", -26.2041, 28.0473, "Africa/Johannesburg"),
        r("77.0.0.0", "77.255.255.255", "DE", "Germany", "Hessen", "Frankfurt", 50.1109, 8.6821, "Europe/Berlin"),
        r("101.0.0.0", "101.255.255.255", "JP", "Japan", "Tokyo", "Tokyo", 35.6762, 139.6503, "Asia/Tokyo"),
        r("103.0.0.0", "103.255.255.255", "IN", "India", "Maharashtra", "Mumbai", 19.0760, 72.8777, "Asia/Kolkata"),
        r("144.126.0.0", "144.126.255.255", "US", "United States", "New York", "New York", 40.7128, -74.0060, "America/New_York"),
        r("185.0.0.0", "185.255.255.255", "NL", "Netherlands", "North Holland", "Amsterdam", 52.3676, 4.9041, "Europe/Amsterdam"),
        r("212.0.0.0", "212.255.255.255", "GB", "United Kingdom", "England", "London", 51.5074, -0.1278, "Europe/London"),
    ]
}

pub struct GeoIpDatabase {
    ranges: Vec<IpRange>,
}

impl GeoIpDatabase {
    pub fn new() -> Self {
        let mut ranges = builtin_ranges();
        ranges.sort_by_key(|r| r.start);
        GeoIpDatabase { ranges }
    }

    pub fn with_ranges(ranges: Vec<IpRange>) -> Self {
        let mut ranges = ranges;
        ranges.sort_by_key(|r| r.start);
        GeoIpDatabase { ranges }
    }

    pub fn add_range(&mut self, r: IpRange) {
        self.ranges.push(r);
        self.ranges.sort_by_key(|r| r.start);
    }

    fn binary_search(&self, ip_int: u32) -> Option<&IpRange> {
        let mut lo = 0usize;
        let mut hi = self.ranges.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            let r = &self.ranges[mid];
            if ip_int < r.start {
                hi = mid;
            } else if ip_int > r.end {
                lo = mid + 1;
            } else {
                return Some(r);
            }
        }
        None
    }

    pub fn lookup(&self, ip: &str) -> GeoResult {
        if is_private_ip(ip) {
            return GeoResult::private(ip);
        }
        let n = ip_to_u32(ip);
        match self.binary_search(n) {
            Some(r) => GeoResult {
                ip: ip.to_string(),
                country_code: r.country_code.clone(),
                country_name: r.country_name.clone(),
                region: r.region.clone(),
                city: r.city.clone(),
                latitude: r.latitude,
                longitude: r.longitude,
                timezone: r.timezone.clone(),
                is_private: false,
            },
            None => GeoResult::empty(ip),
        }
    }

    pub fn bulk_lookup(&self, ips: &[&str]) -> Vec<GeoResult> {
        ips.iter().map(|ip| self.lookup(ip)).collect()
    }

    pub fn country_breakdown(&self, ips: &[&str]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for ip in ips {
            let r = self.lookup(ip);
            let key = if r.is_private { "private".to_string() }
                      else if r.country_code.is_empty() { "unknown".to_string() }
                      else { r.country_code };
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

pub fn nearest_server(client_ip: &str, server_ips: &[&str], db: &GeoIpDatabase) -> Option<String> {
    let client = db.lookup(client_ip);
    if client.is_private || client.country_code.is_empty() { return None; }
    let mut best: Option<(String, f64)> = None;
    for sip in server_ips {
        let srv = db.lookup(sip);
        if srv.is_private || srv.country_code.is_empty() { continue; }
        let d = haversine_km(client.latitude, client.longitude, srv.latitude, srv.longitude);
        if best.is_none() || d < best.as_ref().unwrap().1 {
            best = Some((sip.to_string(), d));
        }
    }
    best.map(|(ip, _)| ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_conversion() {
        assert_eq!(ip_to_u32("8.8.8.8"), 0x08080808);
        assert_eq!(u32_to_ip(0x08080808), "8.8.8.8");
        assert_eq!(ip_to_u32("192.168.1.1"), 0xC0A80101);
    }

    #[test]
    fn test_private_ip() {
        assert!(is_private_ip("192.168.1.1"));
        assert!(is_private_ip("10.0.0.1"));
        assert!(is_private_ip("172.16.0.1"));
        assert!(is_private_ip("127.0.0.1"));
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("144.126.251.126"));
    }

    #[test]
    fn test_lookup_known() {
        let db = GeoIpDatabase::new();
        let r = db.lookup("8.8.8.8");
        assert_eq!(r.country_code, "US");
        assert_eq!(r.city, "Mountain View");
        assert!(!r.is_private);
    }

    #[test]
    fn test_lookup_private() {
        let db = GeoIpDatabase::new();
        let r = db.lookup("192.168.1.1");
        assert!(r.is_private);
    }

    #[test]
    fn test_lookup_droplet() {
        let db = GeoIpDatabase::new();
        let r = db.lookup("144.126.251.126");
        assert_eq!(r.country_code, "US");
        assert_eq!(r.city, "New York");
    }

    #[test]
    fn test_country_breakdown() {
        let db = GeoIpDatabase::new();
        let ips = vec!["8.8.8.8", "1.1.1.1", "192.168.1.1"];
        let bd = db.country_breakdown(&ips);
        assert_eq!(bd["US"], 1);
        assert_eq!(bd["AU"], 1);
        assert_eq!(bd["private"], 1);
    }

    #[test]
    fn test_haversine() {
        // NYC to London ~ 5570 km
        let d = haversine_km(40.7128, -74.0060, 51.5074, -0.1278);
        assert!(d > 5500.0 && d < 5700.0);
    }

    #[test]
    fn test_nearest_server() {
        let db = GeoIpDatabase::new();
        // Client in London area (212.x), servers in US and AU
        let nearest = nearest_server("212.58.244.70", &["8.8.8.8", "1.1.1.1"], &db);
        assert_eq!(nearest, Some("8.8.8.8".to_string())); // US closer to London than AU
    }
}
