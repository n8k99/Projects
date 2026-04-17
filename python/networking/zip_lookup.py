"""Zip / Postal Code Lookup — resolve postal codes to location data."""

from dataclasses import dataclass, field
from typing import Optional
import math


@dataclass
class PostalResult:
    """Location data for a postal code."""
    postal_code: str
    country_code: str = ""
    country_name: str = ""
    state: str = ""
    state_abbr: str = ""
    city: str = ""
    latitude: float = 0.0
    longitude: float = 0.0
    timezone: str = ""

    def summary(self) -> str:
        parts = [p for p in [self.city, self.state_abbr or self.state, self.country_code] if p]
        loc = ", ".join(parts)
        return f"{self.postal_code} -> {loc} [{self.latitude:.4f}, {self.longitude:.4f}]"


@dataclass
class PostalEntry:
    """Database entry for a postal code."""
    postal_code: str
    country_code: str
    country_name: str
    state: str
    state_abbr: str
    city: str
    latitude: float
    longitude: float
    timezone: str = ""


# Built-in database: a selection of US and international postal codes.
BUILTIN_ENTRIES: list[PostalEntry] = [
    # Florida
    PostalEntry("32099", "US", "United States", "Florida", "FL", "Jacksonville", 30.3322, -81.6557, "America/New_York"),
    PostalEntry("32202", "US", "United States", "Florida", "FL", "Jacksonville", 30.3280, -81.6550, "America/New_York"),
    PostalEntry("32244", "US", "United States", "Florida", "FL", "Jacksonville", 30.2266, -81.7360, "America/New_York"),
    PostalEntry("32084", "US", "United States", "Florida", "FL", "St. Augustine", 29.8947, -81.3145, "America/New_York"),
    PostalEntry("33101", "US", "United States", "Florida", "FL", "Miami", 25.7617, -80.1918, "America/New_York"),
    PostalEntry("32801", "US", "United States", "Florida", "FL", "Orlando", 28.5383, -81.3792, "America/New_York"),
    # Other US
    PostalEntry("10001", "US", "United States", "New York", "NY", "New York", 40.7484, -73.9967, "America/New_York"),
    PostalEntry("10013", "US", "United States", "New York", "NY", "New York", 40.7209, -74.0048, "America/New_York"),
    PostalEntry("90210", "US", "United States", "California", "CA", "Beverly Hills", 34.0901, -118.4065, "America/Los_Angeles"),
    PostalEntry("94105", "US", "United States", "California", "CA", "San Francisco", 37.7893, -122.3932, "America/Los_Angeles"),
    PostalEntry("02101", "US", "United States", "Massachusetts", "MA", "Boston", 42.3601, -71.0589, "America/New_York"),
    PostalEntry("60601", "US", "United States", "Illinois", "IL", "Chicago", 41.8819, -87.6278, "America/Chicago"),
    PostalEntry("78201", "US", "United States", "Texas", "TX", "San Antonio", 29.4685, -98.5254, "America/Chicago"),
    PostalEntry("98101", "US", "United States", "Washington", "WA", "Seattle", 47.6062, -122.3321, "America/Los_Angeles"),
    # International
    PostalEntry("SW1A 1AA", "GB", "United Kingdom", "England", "", "London", 51.5014, -0.1419, "Europe/London"),
    PostalEntry("EC1A 1BB", "GB", "United Kingdom", "England", "", "London", 51.5203, -0.0985, "Europe/London"),
    PostalEntry("75001", "FR", "France", "Île-de-France", "", "Paris", 48.8606, 2.3376, "Europe/Paris"),
    PostalEntry("10115", "DE", "Germany", "Berlin", "", "Berlin", 52.5326, 13.3884, "Europe/Berlin"),
    PostalEntry("100-0001", "JP", "Japan", "Tokyo", "", "Chiyoda", 35.6895, 139.6917, "Asia/Tokyo"),
    PostalEntry("2000", "AU", "Australia", "New South Wales", "NSW", "Sydney", -33.8688, 151.2093, "Australia/Sydney"),
]


class PostalDatabase:
    """Postal code database with lookup, search, and distance operations."""

    def __init__(self, entries: Optional[list[PostalEntry]] = None) -> None:
        self._by_code: dict[str, PostalEntry] = {}
        self._by_city: dict[str, list[PostalEntry]] = {}
        self._by_state: dict[str, list[PostalEntry]] = {}
        for e in (entries or BUILTIN_ENTRIES):
            self.add_entry(e)

    def add_entry(self, e: PostalEntry) -> None:
        self._by_code[e.postal_code.upper().replace(" ", "")] = e
        city_key = e.city.lower()
        self._by_city.setdefault(city_key, []).append(e)
        state_key = (e.state_abbr or e.state).lower()
        self._by_state.setdefault(state_key, []).append(e)

    def lookup(self, postal_code: str) -> Optional[PostalResult]:
        """Look up a postal code."""
        key = postal_code.upper().replace(" ", "")
        e = self._by_code.get(key)
        if e is None:
            return None
        return PostalResult(
            postal_code=e.postal_code, country_code=e.country_code,
            country_name=e.country_name, state=e.state, state_abbr=e.state_abbr,
            city=e.city, latitude=e.latitude, longitude=e.longitude, timezone=e.timezone,
        )

    def search_by_city(self, city: str) -> list[PostalResult]:
        """Find all postal codes for a city."""
        entries = self._by_city.get(city.lower(), [])
        return [PostalResult(
            postal_code=e.postal_code, country_code=e.country_code,
            country_name=e.country_name, state=e.state, state_abbr=e.state_abbr,
            city=e.city, latitude=e.latitude, longitude=e.longitude, timezone=e.timezone,
        ) for e in entries]

    def search_by_state(self, state: str) -> list[PostalResult]:
        """Find all postal codes in a state."""
        entries = self._by_state.get(state.lower(), [])
        return [PostalResult(
            postal_code=e.postal_code, country_code=e.country_code,
            country_name=e.country_name, state=e.state, state_abbr=e.state_abbr,
            city=e.city, latitude=e.latitude, longitude=e.longitude, timezone=e.timezone,
        ) for e in entries]

    def bulk_lookup(self, codes: list[str]) -> list[Optional[PostalResult]]:
        return [self.lookup(c) for c in codes]

    def distance_between(self, code_a: str, code_b: str) -> Optional[float]:
        """Return distance in km between two postal codes."""
        a = self.lookup(code_a)
        b = self.lookup(code_b)
        if a is None or b is None:
            return None
        return haversine_km(a.latitude, a.longitude, b.latitude, b.longitude)

    def codes_within_radius(self, center_code: str, radius_km: float) -> list[tuple[PostalResult, float]]:
        """Find all postal codes within radius_km of center_code."""
        center = self.lookup(center_code)
        if center is None:
            return []
        results = []
        for e in self._by_code.values():
            d = haversine_km(center.latitude, center.longitude, e.latitude, e.longitude)
            if d <= radius_km and e.postal_code != center.postal_code:
                results.append((PostalResult(
                    postal_code=e.postal_code, country_code=e.country_code,
                    country_name=e.country_name, state=e.state, state_abbr=e.state_abbr,
                    city=e.city, latitude=e.latitude, longitude=e.longitude, timezone=e.timezone,
                ), d))
        results.sort(key=lambda x: x[1])
        return results

    def state_breakdown(self, codes: list[str]) -> dict[str, int]:
        """Count postal codes per state."""
        counts: dict[str, int] = {}
        for c in codes:
            r = self.lookup(c)
            key = (r.state_abbr or r.state) if r else "unknown"
            counts[key] = counts.get(key, 0) + 1
        return counts


def haversine_km(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    R = 6371.0
    dlat = math.radians(lat2 - lat1)
    dlon = math.radians(lon2 - lon1)
    a = (math.sin(dlat / 2) ** 2 +
         math.cos(math.radians(lat1)) * math.cos(math.radians(lat2)) *
         math.sin(dlon / 2) ** 2)
    return R * 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))


# --- demo ---
if __name__ == "__main__":
    db = PostalDatabase()
    for code in ["32202", "10001", "90210", "SW1A 1AA", "75001"]:
        r = db.lookup(code)
        print(r.summary() if r else f"{code} -> not found")
    print()
    d = db.distance_between("32202", "32084")
    print(f"Jacksonville to St. Augustine: {d:.1f} km")
    print()
    nearby = db.codes_within_radius("32202", 500)
    print(f"Codes within 500km of Jacksonville: {len(nearby)}")
    for r, dist in nearby:
        print(f"  {r.postal_code} {r.city}: {dist:.0f} km")
