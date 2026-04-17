// Fetch Current Weather — HTTP client for live weather data from external APIs.
package networking

import (
	"encoding/json"
	"fmt"
	"io"
	"math"
	"net/http"
	"net/url"
	"time"
)

// WindInfo holds wind conditions.
type WindInfo struct {
	SpeedMps     float64  `json:"speed"`
	DirectionDeg float64  `json:"deg"`
	GustMps      *float64 `json:"gust,omitempty"`
}

// SpeedMph returns wind speed in miles per hour.
func (w WindInfo) SpeedMph() float64 { return w.SpeedMps * 2.23694 }

// SpeedKmh returns wind speed in km/h.
func (w WindInfo) SpeedKmh() float64 { return w.SpeedMps * 3.6 }

// Cardinal returns the compass direction as a string.
func (w WindInfo) Cardinal() string {
	dirs := [16]string{
		"N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
		"S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
	}
	idx := int(math.Round(w.DirectionDeg/22.5)) % 16
	return dirs[idx]
}

// WeatherCondition describes a weather state.
type WeatherCondition struct {
	ID          int    `json:"id"`
	Main        string `json:"main"`
	Description string `json:"description"`
	Icon        string `json:"icon"`
}

// WeatherReport holds structured weather data for a location.
type WeatherReport struct {
	City         string
	Country      string
	Latitude     float64
	Longitude    float64
	TemperatureK float64
	FeelsLikeK   float64
	Humidity     int
	PressureHpa  int
	VisibilityM  int
	Conditions   []WeatherCondition
	Wind         WindInfo
	CloudsPct    int
	Timestamp    time.Time
	Sunrise      *time.Time
	Sunset       *time.Time
}

// TemperatureC returns temperature in Celsius.
func (r WeatherReport) TemperatureC() float64 { return r.TemperatureK - 273.15 }

// TemperatureF returns temperature in Fahrenheit.
func (r WeatherReport) TemperatureF() float64 { return r.TemperatureC()*9.0/5.0 + 32.0 }

// FeelsLikeC returns feels-like in Celsius.
func (r WeatherReport) FeelsLikeC() float64 { return r.FeelsLikeK - 273.15 }

// FeelsLikeF returns feels-like in Fahrenheit.
func (r WeatherReport) FeelsLikeF() float64 { return r.FeelsLikeC()*9.0/5.0 + 32.0 }

// Description returns the primary weather description.
func (r WeatherReport) Description() string {
	if len(r.Conditions) > 0 {
		return r.Conditions[0].Description
	}
	return "unknown"
}

// Summary returns a one-line summary.
func (r WeatherReport) Summary() string {
	return fmt.Sprintf(
		"%s, %s: %.1f°C (%.1f°F), %s, humidity %d%%, wind %.1f m/s %s",
		r.City, r.Country, r.TemperatureC(), r.TemperatureF(),
		r.Description(), r.Humidity, r.Wind.SpeedMps, r.Wind.Cardinal(),
	)
}

// owmResponse mirrors the OpenWeatherMap JSON response.
type owmResponse struct {
	Coord struct {
		Lat float64 `json:"lat"`
		Lon float64 `json:"lon"`
	} `json:"coord"`
	Weather []WeatherCondition `json:"weather"`
	Main    struct {
		Temp      float64 `json:"temp"`
		FeelsLike float64 `json:"feels_like"`
		Pressure  int     `json:"pressure"`
		Humidity  int     `json:"humidity"`
	} `json:"main"`
	Visibility int      `json:"visibility"`
	Wind       WindInfo `json:"wind"`
	Clouds     struct {
		All int `json:"all"`
	} `json:"clouds"`
	Dt  int64 `json:"dt"`
	Sys struct {
		Country string `json:"country"`
		Sunrise int64  `json:"sunrise"`
		Sunset  int64  `json:"sunset"`
	} `json:"sys"`
	Name string `json:"name"`
}

// WeatherClient fetches weather data from an OpenWeatherMap-compatible API.
type WeatherClient struct {
	APIKey  string
	BaseURL string
	client  *http.Client
}

// NewWeatherClient creates a client with the given API key.
func NewWeatherClient(apiKey string) *WeatherClient {
	return &WeatherClient{
		APIKey:  apiKey,
		BaseURL: "https://api.openweathermap.org",
		client:  &http.Client{Timeout: 10 * time.Second},
	}
}

func (c *WeatherClient) fetch(params url.Values) (*WeatherReport, error) {
	params.Set("appid", c.APIKey)
	u := fmt.Sprintf("%s/data/2.5/weather?%s", c.BaseURL, params.Encode())

	req, err := http.NewRequest("GET", u, nil)
	if err != nil {
		return nil, fmt.Errorf("build request: %w", err)
	}
	req.Header.Set("User-Agent", "rosetta-weather/1.0")

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("fetch: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read body: %w", err)
	}
	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("API error %d: %s", resp.StatusCode, string(body))
	}

	var raw owmResponse
	if err := json.Unmarshal(body, &raw); err != nil {
		return nil, fmt.Errorf("parse JSON: %w", err)
	}
	return parseReport(&raw), nil
}

func parseReport(raw *owmResponse) *WeatherReport {
	ts := time.Unix(raw.Dt, 0).UTC()
	var sunrise, sunset *time.Time
	if raw.Sys.Sunrise > 0 {
		t := time.Unix(raw.Sys.Sunrise, 0).UTC()
		sunrise = &t
	}
	if raw.Sys.Sunset > 0 {
		t := time.Unix(raw.Sys.Sunset, 0).UTC()
		sunset = &t
	}
	return &WeatherReport{
		City:         raw.Name,
		Country:      raw.Sys.Country,
		Latitude:     raw.Coord.Lat,
		Longitude:    raw.Coord.Lon,
		TemperatureK: raw.Main.Temp,
		FeelsLikeK:   raw.Main.FeelsLike,
		Humidity:     raw.Main.Humidity,
		PressureHpa:  raw.Main.Pressure,
		VisibilityM:  raw.Visibility,
		Conditions:   raw.Weather,
		Wind:         raw.Wind,
		CloudsPct:    raw.Clouds.All,
		Timestamp:    ts,
		Sunrise:      sunrise,
		Sunset:       sunset,
	}
}

// ByCity fetches weather for a city name, with optional country code.
func (c *WeatherClient) ByCity(city string, countryCode string) (*WeatherReport, error) {
	q := city
	if countryCode != "" {
		q = city + "," + countryCode
	}
	params := url.Values{"q": {q}}
	return c.fetch(params)
}

// ByCoords fetches weather for a latitude/longitude pair.
func (c *WeatherClient) ByCoords(lat, lon float64) (*WeatherReport, error) {
	params := url.Values{
		"lat": {fmt.Sprintf("%f", lat)},
		"lon": {fmt.Sprintf("%f", lon)},
	}
	return c.fetch(params)
}

// ByZip fetches weather for a zip/postal code.
func (c *WeatherClient) ByZip(zip, countryCode string) (*WeatherReport, error) {
	params := url.Values{"zip": {zip + "," + countryCode}}
	return c.fetch(params)
}

// CompareWeather returns the warmest, coldest, most humid, and windiest
// locations from a set of reports.
func CompareWeather(reports []WeatherReport) map[string]string {
	result := make(map[string]string)
	if len(reports) == 0 {
		return result
	}
	warmest, coldest, humid, windy := &reports[0], &reports[0], &reports[0], &reports[0]
	for i := range reports {
		r := &reports[i]
		if r.TemperatureK > warmest.TemperatureK {
			warmest = r
		}
		if r.TemperatureK < coldest.TemperatureK {
			coldest = r
		}
		if r.Humidity > humid.Humidity {
			humid = r
		}
		if r.Wind.SpeedMps > windy.Wind.SpeedMps {
			windy = r
		}
	}
	result["warmest"] = warmest.City
	result["coldest"] = coldest.City
	result["most_humid"] = humid.City
	result["windiest"] = windy.City
	return result
}
