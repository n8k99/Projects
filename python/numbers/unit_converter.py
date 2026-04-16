"""Unit Converter — convert between measurement units."""


# Base units: meters (distance), grams (weight), liters (volume).
# Temperature is special-cased with formulas.

# Conversion factors TO base unit.
_TO_BASE = {
    # distance -> meters
    "km": 1000.0,
    "m": 1.0,
    "mi": 1609.344,
    "ft": 0.3048,
    # weight -> grams
    "kg": 1000.0,
    "g": 1.0,
    "lb": 453.592,
    "oz": 28.3495,
    # volume -> liters
    "L": 1.0,
    "gal": 3.78541,
    "mL": 0.001,
    "cup": 0.236588,
}

_CATEGORIES = {
    "distance": {"km", "m", "mi", "ft"},
    "weight": {"kg", "g", "lb", "oz"},
    "volume": {"L", "gal", "mL", "cup"},
    "temperature": {"celsius", "fahrenheit", "kelvin"},
}

_UNIT_TO_CATEGORY = {}
for cat, units in _CATEGORIES.items():
    for u in units:
        _UNIT_TO_CATEGORY[u] = cat


def _to_celsius(value: float, unit: str) -> float:
    if unit == "celsius":
        return value
    if unit == "fahrenheit":
        return (value - 32.0) * 5.0 / 9.0
    if unit == "kelvin":
        return value - 273.15
    raise ValueError(f"unknown temperature unit: {unit}")


def _from_celsius(value: float, unit: str) -> float:
    if unit == "celsius":
        return value
    if unit == "fahrenheit":
        return value * 9.0 / 5.0 + 32.0
    if unit == "kelvin":
        return value + 273.15
    raise ValueError(f"unknown temperature unit: {unit}")


def convert(value: float, from_unit: str, to_unit: str) -> float:
    """Convert a value from one unit to another.

    Supports temperature (celsius, fahrenheit, kelvin),
    distance (km, mi, m, ft), weight (kg, lb, oz, g),
    and volume (L, gal, mL, cup).
    """
    if from_unit == to_unit:
        return value

    from_cat = _UNIT_TO_CATEGORY.get(from_unit)
    to_cat = _UNIT_TO_CATEGORY.get(to_unit)

    if from_cat is None:
        raise ValueError(f"unknown unit: {from_unit}")
    if to_cat is None:
        raise ValueError(f"unknown unit: {to_unit}")
    if from_cat != to_cat:
        raise ValueError(f"cannot convert {from_unit} ({from_cat}) to {to_unit} ({to_cat})")

    if from_cat == "temperature":
        return _from_celsius(_to_celsius(value, from_unit), to_unit)

    # Ratio-based: convert to base, then from base to target.
    base_value = value * _TO_BASE[from_unit]
    return base_value / _TO_BASE[to_unit]


if __name__ == "__main__":
    demos = [
        (100, "celsius", "fahrenheit"),
        (1, "mi", "km"),
        (2.5, "kg", "lb"),
        (1, "gal", "L"),
    ]
    for val, fr, to in demos:
        result = convert(val, fr, to)
        print(f"{val} {fr} = {result:.4f} {to}")
