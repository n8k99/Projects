"""Country from IP Lookup — resolve IP addresses to geographic location data."""

from dataclasses import dataclass, field
from typing import Optional
import struct
import socket


@dataclass
class GeoResult:
    """Geographic information for an IP address."""
    ip: str
    country_code: str = ""
    country_name: str = ""
    region: str = ""
    city: str = ""
    latitude: float = 0.0
    longitude: float = 0.0
    timezone: str = ""
    isp: str = ""
    is_private: bool = False

    def summary(self) -> str:
        if self.is_private:
            return f"{self.ip} -> private/reserved"
        parts = [self.city, self.region, self.country_name]
        location = ", ".join(p for p in parts if p)
        return f"{self.ip} -> {location} ({self.country_code}) [{self.latitude:.4f}, {self.longitude:.4f}]"


@dataclass
class IpRange:
    """An IP range with associated country data."""
    start: int
    end: int
    country_code: str
    country_name: str = ""
    region: str = ""
    city: str = ""
    latitude: float = 0.0
    longitude: float = 0.0
    timezone: str = ""


def ip_to_int(ip: str) -> int:
    """Convert dotted-quad IP to integer."""
    try:
        packed = socket.inet_aton(ip)
        return struct.unpack("!I", packed)[0]
    except (socket.error, struct.error):
        return 0


def int_to_ip(n: int) -> str:
    """Convert integer to dotted-quad IP."""
    return socket.inet_ntoa(struct.pack("!I", n))


def is_private_ip(ip: str) -> bool:
    """Check if an IP is in a private/reserved range."""
    n = ip_to_int(ip)
    # 10.0.0.0/8
    if 0x0A000000 <= n <= 0x0AFFFFFF:
        return True
    # 172.16.0.0/12
    if 0xAC100000 <= n <= 0xAC1FFFFF:
        return True
    # 192.168.0.0/16
    if 0xC0A80000 <= n <= 0xC0A8FFFF:
        return True
    # 127.0.0.0/8
    if 0x7F000000 <= n <= 0x7FFFFFFF:
        return True
    # 0.0.0.0/8
    if n <= 0x00FFFFFF:
        return True
    return False


# Built-in database: major IP ranges mapped to countries.
# A real implementation would load a MaxMind or IP2Location database.
# This provides enough structure to be a callable library.
BUILTIN_RANGES: list[IpRange] = [
    IpRange(ip_to_int("1.0.0.0"), ip_to_int("1.0.0.255"), "AU", "Australia",
            "Queensland", "Brisbane", -27.4679, 153.0281, "Australia/Brisbane"),
    IpRange(ip_to_int("1.1.1.0"), ip_to_int("1.1.1.255"), "AU", "Australia",
            "New South Wales", "Sydney", -33.8688, 151.2093, "Australia/Sydney"),
    IpRange(ip_to_int("8.8.4.0"), ip_to_int("8.8.8.255"), "US", "United States",
            "California", "Mountain View", 37.3861, -122.0839, "America/Los_Angeles"),
    IpRange(ip_to_int("9.0.0.0"), ip_to_int("9.255.255.255"), "US", "United States",
            "New York", "Armonk", 41.1085, -73.7205, "America/New_York"),
    IpRange(ip_to_int("13.0.0.0"), ip_to_int("13.255.255.255"), "US", "United States",
            "Virginia", "Ashburn", 39.0438, -77.4874, "America/New_York"),
    IpRange(ip_to_int("17.0.0.0"), ip_to_int("17.255.255.255"), "US", "United States",
            "California", "Cupertino", 37.3230, -122.0322, "America/Los_Angeles"),
    IpRange(ip_to_int("41.0.0.0"), ip_to_int("41.255.255.255"), "ZA", "South Africa",
            "Gauteng", "Johannesburg", -26.2041, 28.0473, "Africa/Johannesburg"),
    IpRange(ip_to_int("77.0.0.0"), ip_to_int("77.255.255.255"), "DE", "Germany",
            "Hessen", "Frankfurt", 50.1109, 8.6821, "Europe/Berlin"),
    IpRange(ip_to_int("101.0.0.0"), ip_to_int("101.255.255.255"), "JP", "Japan",
            "Tokyo", "Tokyo", 35.6762, 139.6503, "Asia/Tokyo"),
    IpRange(ip_to_int("103.0.0.0"), ip_to_int("103.255.255.255"), "IN", "India",
            "Maharashtra", "Mumbai", 19.0760, 72.8777, "Asia/Kolkata"),
    IpRange(ip_to_int("144.126.0.0"), ip_to_int("144.126.255.255"), "US", "United States",
            "New York", "New York", 40.7128, -74.0060, "America/New_York"),
    IpRange(ip_to_int("185.0.0.0"), ip_to_int("185.255.255.255"), "NL", "Netherlands",
            "North Holland", "Amsterdam", 52.3676, 4.9041, "Europe/Amsterdam"),
    IpRange(ip_to_int("212.0.0.0"), ip_to_int("212.255.255.255"), "GB", "United Kingdom",
            "England", "London", 51.5074, -0.1278, "Europe/London"),
]


class GeoIpDatabase:
    """A geographic IP database that maps IPs to locations.

    Supports loading custom ranges and binary search for fast lookup.
    The built-in database covers major ranges; a production deployment
    would load MaxMind GeoLite2 or similar.
    """

    def __init__(self, ranges: Optional[list[IpRange]] = None) -> None:
        self.ranges = sorted(ranges or BUILTIN_RANGES, key=lambda r: r.start)

    def add_range(self, r: IpRange) -> None:
        """Add a range and maintain sort order."""
        self.ranges.append(r)
        self.ranges.sort(key=lambda r: r.start)

    def _binary_search(self, ip_int: int) -> Optional[IpRange]:
        """Binary search for the range containing ip_int."""
        lo, hi = 0, len(self.ranges) - 1
        while lo <= hi:
            mid = (lo + hi) // 2
            r = self.ranges[mid]
            if ip_int < r.start:
                hi = mid - 1
            elif ip_int > r.end:
                lo = mid + 1
            else:
                return r
        return None

    def lookup(self, ip: str) -> GeoResult:
        """Look up geographic information for an IP address."""
        if is_private_ip(ip):
            return GeoResult(ip=ip, is_private=True)

        ip_int = ip_to_int(ip)
        r = self._binary_search(ip_int)
        if r is None:
            return GeoResult(ip=ip)

        return GeoResult(
            ip=ip,
            country_code=r.country_code,
            country_name=r.country_name,
            region=r.region,
            city=r.city,
            latitude=r.latitude,
            longitude=r.longitude,
            timezone=r.timezone,
        )

    def bulk_lookup(self, ips: list[str]) -> list[GeoResult]:
        """Look up multiple IPs."""
        return [self.lookup(ip) for ip in ips]

    def country_breakdown(self, ips: list[str]) -> dict[str, int]:
        """Count IPs per country."""
        counts: dict[str, int] = {}
        for ip in ips:
            result = self.lookup(ip)
            key = result.country_code or ("private" if result.is_private else "unknown")
            counts[key] = counts.get(key, 0) + 1
        return counts


def distance_km(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """Haversine distance between two lat/lon points in kilometers."""
    import math
    R = 6371.0
    dlat = math.radians(lat2 - lat1)
    dlon = math.radians(lon2 - lon1)
    a = (math.sin(dlat / 2) ** 2 +
         math.cos(math.radians(lat1)) * math.cos(math.radians(lat2)) *
         math.sin(dlon / 2) ** 2)
    return R * 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))


def nearest_server(client_ip: str, server_ips: list[str],
                   db: Optional[GeoIpDatabase] = None) -> Optional[str]:
    """Find the geographically nearest server to a client IP."""
    db = db or GeoIpDatabase()
    client = db.lookup(client_ip)
    if client.is_private or not client.country_code:
        return None
    best_ip = None
    best_dist = float("inf")
    for sip in server_ips:
        srv = db.lookup(sip)
        if srv.is_private or not srv.country_code:
            continue
        d = distance_km(client.latitude, client.longitude, srv.latitude, srv.longitude)
        if d < best_dist:
            best_dist = d
            best_ip = sip
    return best_ip


# --- demo ---
if __name__ == "__main__":
    db = GeoIpDatabase()
    test_ips = ["8.8.8.8", "1.1.1.1", "192.168.1.1", "144.126.251.126",
                "77.100.50.25", "212.58.244.70"]
    for ip in test_ips:
        print(db.lookup(ip).summary())
    print()
    print("Country breakdown:", db.country_breakdown(test_ips))
