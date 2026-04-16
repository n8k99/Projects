"""Distance Between Two Cities — great-circle distance via Haversine."""

import math


# City coordinates: (latitude, longitude) in degrees.
CITIES = {
    "New York":  ( 40.7128,  -74.0060),
    "London":    ( 51.5074,   -0.1278),
    "Tokyo":     ( 35.6762,  139.6503),
    "Sydney":    (-33.8688,  151.2093),
    "São Paulo": (-23.5505,  -46.6333),
    "Cairo":     ( 30.0444,   31.2357),
}

# Earth's mean radius in kilometres.
_R = 6371.0


def haversine(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """Return great-circle distance in km between two points given in degrees."""
    lat1, lon1, lat2, lon2 = (math.radians(v) for v in (lat1, lon1, lat2, lon2))
    dlat = lat2 - lat1
    dlon = lon2 - lon1
    a = math.sin(dlat / 2) ** 2 + math.cos(lat1) * math.cos(lat2) * math.sin(dlon / 2) ** 2
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    return _R * c


def city_distance(city1: str, city2: str) -> float:
    """Return great-circle distance in km between two named cities."""
    if city1 not in CITIES:
        raise ValueError(f"unknown city: {city1}")
    if city2 not in CITIES:
        raise ValueError(f"unknown city: {city2}")
    lat1, lon1 = CITIES[city1]
    lat2, lon2 = CITIES[city2]
    return haversine(lat1, lon1, lat2, lon2)


if __name__ == "__main__":
    demos = [
        ("New York", "London"),
        ("Tokyo", "Sydney"),
        ("São Paulo", "Cairo"),
    ]
    for a, b in demos:
        km = city_distance(a, b)
        print(f"{a} → {b}: {km:.2f} km")
