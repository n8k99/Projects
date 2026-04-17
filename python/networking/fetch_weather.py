"""Fetch Current Weather — HTTP client for live weather data from external APIs."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional
import json
import urllib.request
import urllib.error
import urllib.parse


@dataclass
class WindInfo:
    """Wind conditions."""
    speed_mps: float       # meters per second
    direction_deg: float   # compass degrees
    gust_mps: Optional[float] = None

    @property
    def speed_mph(self) -> float:
        return self.speed_mps * 2.23694

    @property
    def speed_kmh(self) -> float:
        return self.speed_mps * 3.6

    @property
    def cardinal(self) -> str:
        directions = ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
                       "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"]
        idx = round(self.direction_deg / 22.5) % 16
        return directions[idx]


@dataclass
class WeatherCondition:
    """A weather condition descriptor."""
    id: int
    main: str          # e.g. "Rain", "Clear", "Clouds"
    description: str   # e.g. "light rain", "clear sky"
    icon: str          # icon code


@dataclass
class WeatherReport:
    """Structured weather report for a location."""
    city: str
    country: str
    latitude: float
    longitude: float
    temperature_k: float    # Kelvin (canonical unit)
    feels_like_k: float
    humidity: int           # percent
    pressure_hpa: int       # hectopascals
    visibility_m: int       # meters
    conditions: list[WeatherCondition]
    wind: WindInfo
    clouds_pct: int         # cloud cover percentage
    timestamp: datetime
    sunrise: Optional[datetime] = None
    sunset: Optional[datetime] = None

    @property
    def temperature_c(self) -> float:
        return self.temperature_k - 273.15

    @property
    def temperature_f(self) -> float:
        return self.temperature_c * 9 / 5 + 32

    @property
    def feels_like_c(self) -> float:
        return self.feels_like_k - 273.15

    @property
    def feels_like_f(self) -> float:
        return self.feels_like_c * 9 / 5 + 32

    @property
    def description(self) -> str:
        if self.conditions:
            return self.conditions[0].description
        return "unknown"

    def summary(self) -> str:
        return (
            f"{self.city}, {self.country}: {self.temperature_c:.1f}°C "
            f"({self.temperature_f:.1f}°F), {self.description}, "
            f"humidity {self.humidity}%, wind {self.wind.speed_mps:.1f} m/s {self.wind.cardinal}"
        )


class WeatherClient:
    """HTTP client for fetching weather data from OpenWeatherMap API.

    Works with any OpenWeatherMap-compatible endpoint. The API key and
    base URL are configurable for testing or alternative providers.
    """

    def __init__(self, api_key: str, base_url: str = "https://api.openweathermap.org") -> None:
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")

    def _get(self, path: str, params: dict) -> dict:
        """Make an HTTP GET request and return parsed JSON."""
        params["appid"] = self.api_key
        query = urllib.parse.urlencode(params)
        url = f"{self.base_url}{path}?{query}"
        req = urllib.request.Request(url, headers={"User-Agent": "rosetta-weather/1.0"})
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def _parse_report(self, data: dict) -> WeatherReport:
        """Parse OpenWeatherMap JSON into a WeatherReport."""
        sys_data = data.get("sys", {})
        wind_data = data.get("wind", {})
        main_data = data.get("main", {})

        conditions = [
            WeatherCondition(
                id=w["id"],
                main=w["main"],
                description=w["description"],
                icon=w["icon"],
            )
            for w in data.get("weather", [])
        ]

        sunrise = None
        sunset = None
        if "sunrise" in sys_data:
            sunrise = datetime.fromtimestamp(sys_data["sunrise"], tz=timezone.utc)
        if "sunset" in sys_data:
            sunset = datetime.fromtimestamp(sys_data["sunset"], tz=timezone.utc)

        return WeatherReport(
            city=data.get("name", "Unknown"),
            country=sys_data.get("country", "??"),
            latitude=data["coord"]["lat"],
            longitude=data["coord"]["lon"],
            temperature_k=main_data["temp"],
            feels_like_k=main_data["feels_like"],
            humidity=main_data["humidity"],
            pressure_hpa=main_data["pressure"],
            visibility_m=data.get("visibility", 0),
            conditions=conditions,
            wind=WindInfo(
                speed_mps=wind_data.get("speed", 0.0),
                direction_deg=wind_data.get("deg", 0.0),
                gust_mps=wind_data.get("gust"),
            ),
            clouds_pct=data.get("clouds", {}).get("all", 0),
            timestamp=datetime.fromtimestamp(data["dt"], tz=timezone.utc),
            sunrise=sunrise,
            sunset=sunset,
        )

    def by_city(self, city: str, country_code: Optional[str] = None) -> WeatherReport:
        """Fetch current weather by city name."""
        q = f"{city},{country_code}" if country_code else city
        data = self._get("/data/2.5/weather", {"q": q})
        return self._parse_report(data)

    def by_coords(self, lat: float, lon: float) -> WeatherReport:
        """Fetch current weather by latitude and longitude."""
        data = self._get("/data/2.5/weather", {"lat": lat, "lon": lon})
        return self._parse_report(data)

    def by_zip(self, zip_code: str, country_code: str = "US") -> WeatherReport:
        """Fetch current weather by zip/postal code."""
        data = self._get("/data/2.5/weather", {"zip": f"{zip_code},{country_code}"})
        return self._parse_report(data)


def compare_weather(reports: list[WeatherReport]) -> dict:
    """Compare weather across multiple locations.

    Returns a dict with warmest, coldest, most humid, and windiest locations.
    """
    if not reports:
        return {}
    return {
        "warmest": max(reports, key=lambda r: r.temperature_k).city,
        "coldest": min(reports, key=lambda r: r.temperature_k).city,
        "most_humid": max(reports, key=lambda r: r.humidity).city,
        "windiest": max(reports, key=lambda r: r.wind.speed_mps).city,
    }


# --- demo ---
if __name__ == "__main__":
    import os

    key = os.environ.get("OWM_API_KEY", "")
    if not key:
        print("Set OWM_API_KEY environment variable to run the demo.")
        print("Example: OWM_API_KEY=your_key python fetch_weather.py")
    else:
        client = WeatherClient(key)
        report = client.by_city("Jacksonville", "US")
        print(report.summary())
