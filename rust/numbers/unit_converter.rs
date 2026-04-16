//! Unit Converter — convert between measurement units.

/// Return the category and base-unit factor for a ratio-based unit.
/// Base units: meters (distance), grams (weight), liters (volume).
fn unit_info(unit: &str) -> Option<(&'static str, f64)> {
    match unit {
        // distance -> meters
        "km" => Some(("distance", 1000.0)),
        "m"  => Some(("distance", 1.0)),
        "mi" => Some(("distance", 1609.344)),
        "ft" => Some(("distance", 0.3048)),
        // weight -> grams
        "kg" => Some(("weight", 1000.0)),
        "g"  => Some(("weight", 1.0)),
        "lb" => Some(("weight", 453.592)),
        "oz" => Some(("weight", 28.3495)),
        // volume -> liters
        "L"  => Some(("volume", 1.0)),
        "gal" => Some(("volume", 3.78541)),
        "mL" => Some(("volume", 0.001)),
        "cup" => Some(("volume", 0.236588)),
        _ => None,
    }
}

/// Convert to celsius from the given temperature unit.
fn to_celsius(value: f64, unit: &str) -> Result<f64, String> {
    match unit {
        "celsius"    => Ok(value),
        "fahrenheit" => Ok((value - 32.0) * 5.0 / 9.0),
        "kelvin"     => Ok(value - 273.15),
        _ => Err(format!("unknown temperature unit: {unit}")),
    }
}

/// Convert from celsius to the given temperature unit.
fn from_celsius(value: f64, unit: &str) -> Result<f64, String> {
    match unit {
        "celsius"    => Ok(value),
        "fahrenheit" => Ok(value * 9.0 / 5.0 + 32.0),
        "kelvin"     => Ok(value + 273.15),
        _ => Err(format!("unknown temperature unit: {unit}")),
    }
}

fn is_temperature(unit: &str) -> bool {
    matches!(unit, "celsius" | "fahrenheit" | "kelvin")
}

/// Convert a value from one unit to another.
///
/// Supports temperature (celsius, fahrenheit, kelvin),
/// distance (km, mi, m, ft), weight (kg, lb, oz, g),
/// and volume (L, gal, mL, cup).
pub fn convert(value: f64, from: &str, to: &str) -> Result<f64, String> {
    if from == to {
        return Ok(value);
    }

    // Temperature: formula-based.
    if is_temperature(from) || is_temperature(to) {
        if !is_temperature(from) || !is_temperature(to) {
            return Err(format!("cannot convert {from} to {to}"));
        }
        return from_celsius(to_celsius(value, from)?, to);
    }

    // Ratio-based: normalize to base unit, then to target.
    let (from_cat, from_factor) = unit_info(from)
        .ok_or_else(|| format!("unknown unit: {from}"))?;
    let (to_cat, to_factor) = unit_info(to)
        .ok_or_else(|| format!("unknown unit: {to}"))?;

    if from_cat != to_cat {
        return Err(format!("cannot convert {from} ({from_cat}) to {to} ({to_cat})"));
    }

    Ok(value * from_factor / to_factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celsius_to_fahrenheit() {
        let r = convert(100.0, "celsius", "fahrenheit").unwrap();
        assert!((r - 212.0).abs() < 1e-10);
    }

    #[test]
    fn fahrenheit_to_celsius() {
        let r = convert(32.0, "fahrenheit", "celsius").unwrap();
        assert!(r.abs() < 1e-10);
    }

    #[test]
    fn celsius_to_kelvin() {
        let r = convert(0.0, "celsius", "kelvin").unwrap();
        assert!((r - 273.15).abs() < 1e-10);
    }

    #[test]
    fn miles_to_km() {
        let r = convert(1.0, "mi", "km").unwrap();
        assert!((r - 1.609344).abs() < 1e-4);
    }

    #[test]
    fn kg_to_lb() {
        let r = convert(1.0, "kg", "lb").unwrap();
        assert!((r - 2.20462).abs() < 0.001);
    }

    #[test]
    fn gallon_to_liters() {
        let r = convert(1.0, "gal", "L").unwrap();
        assert!((r - 3.78541).abs() < 0.001);
    }

    #[test]
    fn same_unit() {
        assert_eq!(convert(42.0, "m", "m").unwrap(), 42.0);
    }

    #[test]
    fn cross_category_error() {
        assert!(convert(1.0, "km", "kg").is_err());
    }

    #[test]
    fn unknown_unit_error() {
        assert!(convert(1.0, "parsec", "km").is_err());
    }
}
