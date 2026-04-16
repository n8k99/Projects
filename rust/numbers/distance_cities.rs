//! Distance Between Two Cities — great-circle distance via Haversine.

use std::f64::consts::PI;

/// Earth's mean radius in kilometres.
const R: f64 = 6371.0;

/// City entry: (name, latitude, longitude) in degrees.
const CITIES: &[(&str, f64, f64)] = &[
    ("New York",   40.7128,  -74.0060),
    ("London",     51.5074,   -0.1278),
    ("Tokyo",      35.6762,  139.6503),
    ("Sydney",    -33.8688,  151.2093),
    ("São Paulo", -23.5505,  -46.6333),
    ("Cairo",      30.0444,   31.2357),
];

fn deg2rad(d: f64) -> f64 {
    d * PI / 180.0
}

/// Return great-circle distance in km between two points given in degrees.
pub fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (lat1, lon1) = (deg2rad(lat1), deg2rad(lon1));
    let (lat2, lon2) = (deg2rad(lat2), deg2rad(lon2));
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}

fn find_city(name: &str) -> Option<(f64, f64)> {
    CITIES.iter().find(|c| c.0 == name).map(|c| (c.1, c.2))
}

/// Return great-circle distance in km between two named cities.
pub fn city_distance(city1: &str, city2: &str) -> Result<f64, String> {
    let (lat1, lon1) = find_city(city1)
        .ok_or_else(|| format!("unknown city: {city1}"))?;
    let (lat2, lon2) = find_city(city2)
        .ok_or_else(|| format!("unknown city: {city2}"))?;
    Ok(haversine(lat1, lon1, lat2, lon2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nyc_to_london() {
        let d = city_distance("New York", "London").unwrap();
        assert!((d - 5570.0).abs() < 10.0, "expected ~5570 km, got {d}");
    }

    #[test]
    fn tokyo_to_sydney() {
        let d = city_distance("Tokyo", "Sydney").unwrap();
        assert!((d - 7800.0).abs() < 50.0, "expected ~7800 km, got {d}");
    }

    #[test]
    fn same_city() {
        let d = city_distance("Cairo", "Cairo").unwrap();
        assert!(d.abs() < 1e-10);
    }

    #[test]
    fn unknown_city() {
        assert!(city_distance("Atlantis", "London").is_err());
    }
}
