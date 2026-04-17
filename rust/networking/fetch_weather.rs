// Fetch Current Weather — HTTP client for live weather data from external APIs.
//
// Uses only std::net for HTTP (no async runtime dependency). The module is a
// callable library — not a binary that prints and exits.

use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, Read as _, Write as _};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct WindInfo {
    pub speed_mps: f64,
    pub direction_deg: f64,
    pub gust_mps: Option<f64>,
}

impl WindInfo {
    pub fn speed_mph(&self) -> f64 {
        self.speed_mps * 2.23694
    }

    pub fn speed_kmh(&self) -> f64 {
        self.speed_mps * 3.6
    }

    pub fn cardinal(&self) -> &'static str {
        const DIRS: [&str; 16] = [
            "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
            "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
        ];
        let idx = ((self.direction_deg / 22.5).round() as usize) % 16;
        DIRS[idx]
    }
}

#[derive(Debug, Clone)]
pub struct WeatherCondition {
    pub id: u32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone)]
pub struct WeatherReport {
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub temperature_k: f64,
    pub feels_like_k: f64,
    pub humidity: u32,
    pub pressure_hpa: u32,
    pub visibility_m: u32,
    pub conditions: Vec<WeatherCondition>,
    pub wind: WindInfo,
    pub clouds_pct: u32,
    pub timestamp: u64,
    pub sunrise: Option<u64>,
    pub sunset: Option<u64>,
}

impl WeatherReport {
    pub fn temperature_c(&self) -> f64 {
        self.temperature_k - 273.15
    }

    pub fn temperature_f(&self) -> f64 {
        self.temperature_c() * 9.0 / 5.0 + 32.0
    }

    pub fn feels_like_c(&self) -> f64 {
        self.feels_like_k - 273.15
    }

    pub fn feels_like_f(&self) -> f64 {
        self.feels_like_c() * 9.0 / 5.0 + 32.0
    }

    pub fn description(&self) -> &str {
        self.conditions
            .first()
            .map(|c| c.description.as_str())
            .unwrap_or("unknown")
    }
}

impl fmt::Display for WeatherReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}: {:.1}°C ({:.1}°F), {}, humidity {}%, wind {:.1} m/s {}",
            self.city,
            self.country,
            self.temperature_c(),
            self.temperature_f(),
            self.description(),
            self.humidity,
            self.wind.speed_mps,
            self.wind.cardinal(),
        )
    }
}

pub struct WeatherClient {
    api_key: String,
    host: String,
}

impl WeatherClient {
    pub fn new(api_key: &str) -> Self {
        WeatherClient {
            api_key: api_key.to_string(),
            host: "api.openweathermap.org".to_string(),
        }
    }

    pub fn with_host(api_key: &str, host: &str) -> Self {
        WeatherClient {
            api_key: api_key.to_string(),
            host: host.to_string(),
        }
    }

    fn http_get(&self, path: &str) -> Result<String, String> {
        let addr = format!("{}:80", self.host);
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| format!("connect failed: {e}"))?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .ok();

        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {}\r\nUser-Agent: rosetta-weather/1.0\r\nConnection: close\r\n\r\n",
            self.host
        );
        stream.write_all(request.as_bytes())
            .map_err(|e| format!("write failed: {e}"))?;

        let mut reader = BufReader::new(&stream);

        // Read status line
        let mut status_line = String::new();
        reader.read_line(&mut status_line)
            .map_err(|e| format!("read status failed: {e}"))?;
        if !status_line.contains("200") {
            return Err(format!("HTTP error: {}", status_line.trim()));
        }

        // Skip headers until empty line
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)
                .map_err(|e| format!("read header failed: {e}"))?;
            if line.trim().is_empty() {
                break;
            }
        }

        // Read body
        let mut body = String::new();
        reader.read_to_string(&mut body)
            .map_err(|e| format!("read body failed: {e}"))?;

        Ok(body)
    }

    fn fetch(&self, params: &[(&str, &str)]) -> Result<String, String> {
        let mut query_parts: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlenc(v)))
            .collect();
        query_parts.push(format!("appid={}", urlenc(&self.api_key)));
        let path = format!("/data/2.5/weather?{}", query_parts.join("&"));
        self.http_get(&path)
    }

    fn parse_report(json_str: &str) -> Result<WeatherReport, String> {
        // Minimal JSON parsing using only std. Extracts fields positionally.
        let json = json_str.trim();
        let get = |key: &str| -> Option<&str> {
            json_find(json, key)
        };

        let coord_lat = get("\"lat\"").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let coord_lon = get("\"lon\"").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);

        let temp = get("\"temp\"").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let feels_like = get("\"feels_like\"").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let humidity = get("\"humidity\"").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        let pressure = get("\"pressure\"").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        let visibility = get("\"visibility\"").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);

        let wind_speed = get("\"speed\"").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let wind_deg = get("\"deg\"").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let wind_gust = get("\"gust\"").and_then(|v| v.parse::<f64>().ok());

        let clouds = get("\"all\"").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        let dt = get("\"dt\"").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);

        let name = get("\"name\"").map(|v| v.trim_matches('"').to_string()).unwrap_or_default();
        let country = get("\"country\"").map(|v| v.trim_matches('"').to_string()).unwrap_or_default();

        let sunrise = get("\"sunrise\"").and_then(|v| v.parse::<u64>().ok());
        let sunset = get("\"sunset\"").and_then(|v| v.parse::<u64>().ok());

        // Parse weather conditions array
        let conditions = parse_conditions(json);

        Ok(WeatherReport {
            city: name,
            country,
            latitude: coord_lat,
            longitude: coord_lon,
            temperature_k: temp,
            feels_like_k: feels_like,
            humidity,
            pressure_hpa: pressure,
            visibility_m: visibility,
            conditions,
            wind: WindInfo {
                speed_mps: wind_speed,
                direction_deg: wind_deg,
                gust_mps: wind_gust,
            },
            clouds_pct: clouds,
            timestamp: dt,
            sunrise,
            sunset,
        })
    }

    pub fn by_city(&self, city: &str, country_code: Option<&str>) -> Result<WeatherReport, String> {
        let q = match country_code {
            Some(cc) => format!("{},{}", city, cc),
            None => city.to_string(),
        };
        let body = self.fetch(&[("q", &q)])?;
        Self::parse_report(&body)
    }

    pub fn by_coords(&self, lat: f64, lon: f64) -> Result<WeatherReport, String> {
        let lat_s = lat.to_string();
        let lon_s = lon.to_string();
        let body = self.fetch(&[("lat", &lat_s), ("lon", &lon_s)])?;
        Self::parse_report(&body)
    }

    pub fn by_zip(&self, zip: &str, country_code: &str) -> Result<WeatherReport, String> {
        let z = format!("{},{}", zip, country_code);
        let body = self.fetch(&[("zip", &z)])?;
        Self::parse_report(&body)
    }
}

pub fn compare_weather(reports: &[WeatherReport]) -> HashMap<&str, &str> {
    let mut result = HashMap::new();
    if reports.is_empty() {
        return result;
    }
    if let Some(r) = reports.iter().max_by(|a, b| a.temperature_k.partial_cmp(&b.temperature_k).unwrap()) {
        result.insert("warmest", r.city.as_str());
    }
    if let Some(r) = reports.iter().min_by(|a, b| a.temperature_k.partial_cmp(&b.temperature_k).unwrap()) {
        result.insert("coldest", r.city.as_str());
    }
    if let Some(r) = reports.iter().max_by_key(|r| r.humidity) {
        result.insert("most_humid", r.city.as_str());
    }
    if let Some(r) = reports.iter().max_by(|a, b| a.wind.speed_mps.partial_cmp(&b.wind.speed_mps).unwrap()) {
        result.insert("windiest", r.city.as_str());
    }
    result
}

// --- helpers ---

fn urlenc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

fn json_find<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pos = json.find(key)?;
    let after = &json[pos + key.len()..];
    let after = after.trim_start().strip_prefix(':')?;
    let after = after.trim_start();
    // Find end of value
    if after.starts_with('"') {
        let end = after[1..].find('"')? + 1;
        Some(&after[..=end])
    } else {
        let end = after.find(|c: char| c == ',' || c == '}' || c == ']').unwrap_or(after.len());
        Some(after[..end].trim())
    }
}

fn parse_conditions(json: &str) -> Vec<WeatherCondition> {
    let mut conditions = Vec::new();
    // Find the "weather" array
    let Some(start) = json.find("\"weather\"") else { return conditions };
    let rest = &json[start..];
    let Some(arr_start) = rest.find('[') else { return conditions };
    let Some(arr_end) = rest.find(']') else { return conditions };
    let arr = &rest[arr_start + 1..arr_end];

    // Split by },{
    for chunk in arr.split("},") {
        let chunk = chunk.trim().trim_start_matches('{').trim_end_matches('}');
        let id = json_find(chunk, "\"id\"").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        let main = json_find(chunk, "\"main\"").map(|v| v.trim_matches('"').to_string()).unwrap_or_default();
        let desc = json_find(chunk, "\"description\"").map(|v| v.trim_matches('"').to_string()).unwrap_or_default();
        let icon = json_find(chunk, "\"icon\"").map(|v| v.trim_matches('"').to_string()).unwrap_or_default();
        conditions.push(WeatherCondition { id, main, description: desc, icon });
    }
    conditions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_cardinal() {
        let w = WindInfo { speed_mps: 5.0, direction_deg: 0.0, gust_mps: None };
        assert_eq!(w.cardinal(), "N");
        let w = WindInfo { speed_mps: 5.0, direction_deg: 90.0, gust_mps: None };
        assert_eq!(w.cardinal(), "E");
        let w = WindInfo { speed_mps: 5.0, direction_deg: 225.0, gust_mps: None };
        assert_eq!(w.cardinal(), "SW");
    }

    #[test]
    fn test_temperature_conversion() {
        let report = WeatherReport {
            city: "Test".into(), country: "XX".into(),
            latitude: 0.0, longitude: 0.0,
            temperature_k: 300.0, feels_like_k: 298.0,
            humidity: 50, pressure_hpa: 1013, visibility_m: 10000,
            conditions: vec![], wind: WindInfo { speed_mps: 0.0, direction_deg: 0.0, gust_mps: None },
            clouds_pct: 0, timestamp: 0, sunrise: None, sunset: None,
        };
        assert!((report.temperature_c() - 26.85).abs() < 0.01);
        assert!((report.temperature_f() - 80.33).abs() < 0.01);
    }

    #[test]
    fn test_urlenc() {
        assert_eq!(urlenc("hello world"), "hello%20world");
        assert_eq!(urlenc("key=val"), "key%3Dval");
    }

    #[test]
    fn test_json_find() {
        let json = r#"{"temp": 300.5, "name": "London"}"#;
        assert_eq!(json_find(json, "\"temp\""), Some("300.5"));
        assert_eq!(json_find(json, "\"name\""), Some("\"London\""));
    }

    #[test]
    fn test_parse_report() {
        let json = r#"{"coord":{"lon":-81.66,"lat":30.33},"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"main":{"temp":300.0,"feels_like":298.0,"pressure":1013,"humidity":50},"visibility":10000,"wind":{"speed":3.5,"deg":180},"clouds":{"all":0},"dt":1700000000,"sys":{"country":"US","sunrise":1699990000,"sunset":1700030000},"name":"Jacksonville"}"#;
        let report = WeatherClient::parse_report(json).unwrap();
        assert_eq!(report.city, "Jacksonville");
        assert_eq!(report.country, "US");
        assert!((report.temperature_k - 300.0).abs() < 0.01);
        assert_eq!(report.humidity, 50);
        assert_eq!(report.conditions.len(), 1);
        assert_eq!(report.conditions[0].main, "Clear");
    }

    #[test]
    fn test_compare_weather() {
        let make = |city: &str, temp: f64, hum: u32, wind: f64| WeatherReport {
            city: city.into(), country: "XX".into(),
            latitude: 0.0, longitude: 0.0,
            temperature_k: temp, feels_like_k: temp,
            humidity: hum, pressure_hpa: 1013, visibility_m: 10000,
            conditions: vec![], wind: WindInfo { speed_mps: wind, direction_deg: 0.0, gust_mps: None },
            clouds_pct: 0, timestamp: 0, sunrise: None, sunset: None,
        };
        let reports = vec![
            make("Hot", 310.0, 30, 2.0),
            make("Cold", 260.0, 80, 5.0),
            make("Windy", 290.0, 50, 15.0),
        ];
        let cmp = compare_weather(&reports);
        assert_eq!(cmp["warmest"], "Hot");
        assert_eq!(cmp["coldest"], "Cold");
        assert_eq!(cmp["most_humid"], "Cold");
        assert_eq!(cmp["windiest"], "Windy");
    }
}
