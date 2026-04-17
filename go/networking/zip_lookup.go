// Zip / Postal Code Lookup — resolve postal codes to location data.
package networking

import (
	"fmt"
	"math"
	"sort"
	"strings"
)

// PostalResult holds location data for a postal code.
type PostalResult struct {
	PostalCode  string
	CountryCode string
	CountryName string
	State       string
	StateAbbr   string
	City        string
	Latitude    float64
	Longitude   float64
	Timezone    string
}

// Summary returns a one-line description.
func (r PostalResult) PostalSummary() string {
	st := r.StateAbbr
	if st == "" {
		st = r.State
	}
	var parts []string
	for _, s := range []string{r.City, st, r.CountryCode} {
		if s != "" {
			parts = append(parts, s)
		}
	}
	return fmt.Sprintf("%s -> %s [%.4f, %.4f]", r.PostalCode, strings.Join(parts, ", "), r.Latitude, r.Longitude)
}

type postalEntry struct {
	PostalCode  string
	CountryCode string
	CountryName string
	State       string
	StateAbbr   string
	City        string
	Latitude    float64
	Longitude   float64
	Timezone    string
}

func (e *postalEntry) toResult() PostalResult {
	return PostalResult{e.PostalCode, e.CountryCode, e.CountryName, e.State, e.StateAbbr, e.City, e.Latitude, e.Longitude, e.Timezone}
}

func postalBuiltins() []postalEntry {
	e := func(code, cc, cn, st, sa, city string, lat, lon float64, tz string) postalEntry {
		return postalEntry{code, cc, cn, st, sa, city, lat, lon, tz}
	}
	return []postalEntry{
		e("32099", "US", "United States", "Florida", "FL", "Jacksonville", 30.3322, -81.6557, "America/New_York"),
		e("32202", "US", "United States", "Florida", "FL", "Jacksonville", 30.3280, -81.6550, "America/New_York"),
		e("32244", "US", "United States", "Florida", "FL", "Jacksonville", 30.2266, -81.7360, "America/New_York"),
		e("32084", "US", "United States", "Florida", "FL", "St. Augustine", 29.8947, -81.3145, "America/New_York"),
		e("33101", "US", "United States", "Florida", "FL", "Miami", 25.7617, -80.1918, "America/New_York"),
		e("10001", "US", "United States", "New York", "NY", "New York", 40.7484, -73.9967, "America/New_York"),
		e("90210", "US", "United States", "California", "CA", "Beverly Hills", 34.0901, -118.4065, "America/Los_Angeles"),
		e("94105", "US", "United States", "California", "CA", "San Francisco", 37.7893, -122.3932, "America/Los_Angeles"),
		e("SW1A1AA", "GB", "United Kingdom", "England", "", "London", 51.5014, -0.1419, "Europe/London"),
		e("75001", "FR", "France", "Île-de-France", "", "Paris", 48.8606, 2.3376, "Europe/Paris"),
		e("2000", "AU", "Australia", "New South Wales", "NSW", "Sydney", -33.8688, 151.2093, "Australia/Sydney"),
	}
}

func normalizePostalCode(code string) string {
	return strings.ToUpper(strings.ReplaceAll(code, " ", ""))
}

// PostalDatabase maps postal codes to locations.
type PostalDatabase struct {
	byCode  map[string]*postalEntry
	entries []postalEntry
}

// NewPostalDatabase creates a database with built-in entries.
func NewPostalDatabase() *PostalDatabase {
	entries := postalBuiltins()
	byCode := make(map[string]*postalEntry)
	for i := range entries {
		byCode[normalizePostalCode(entries[i].PostalCode)] = &entries[i]
	}
	return &PostalDatabase{byCode: byCode, entries: entries}
}

// PostalLookup resolves a postal code.
func (db *PostalDatabase) PostalLookup(code string) *PostalResult {
	e, ok := db.byCode[normalizePostalCode(code)]
	if !ok {
		return nil
	}
	r := e.toResult()
	return &r
}

// SearchByCity finds all codes for a city.
func (db *PostalDatabase) SearchByCity(city string) []PostalResult {
	cityLower := strings.ToLower(city)
	var results []PostalResult
	for _, e := range db.entries {
		if strings.ToLower(e.City) == cityLower {
			results = append(results, e.toResult())
		}
	}
	return results
}

// PostalDistanceBetween returns km between two postal codes.
func (db *PostalDatabase) PostalDistanceBetween(a, b string) (float64, bool) {
	ra := db.PostalLookup(a)
	rb := db.PostalLookup(b)
	if ra == nil || rb == nil {
		return 0, false
	}
	return postalHaversine(ra.Latitude, ra.Longitude, rb.Latitude, rb.Longitude), true
}

// CodesWithinRadius finds codes within radius_km of center.
func (db *PostalDatabase) CodesWithinRadius(center string, radiusKm float64) []struct {
	Result   PostalResult
	Distance float64
} {
	c := db.PostalLookup(center)
	if c == nil {
		return nil
	}
	centerNorm := normalizePostalCode(center)
	type rd struct {
		Result   PostalResult
		Distance float64
	}
	var results []rd
	for _, e := range db.entries {
		if normalizePostalCode(e.PostalCode) == centerNorm {
			continue
		}
		d := postalHaversine(c.Latitude, c.Longitude, e.Latitude, e.Longitude)
		if d <= radiusKm {
			results = append(results, rd{e.toResult(), d})
		}
	}
	sort.Slice(results, func(i, j int) bool { return results[i].Distance < results[j].Distance })
	out := make([]struct {
		Result   PostalResult
		Distance float64
	}, len(results))
	for i, r := range results {
		out[i] = struct {
			Result   PostalResult
			Distance float64
		}{r.Result, r.Distance}
	}
	return out
}

func postalHaversine(lat1, lon1, lat2, lon2 float64) float64 {
	const R = 6371.0
	dLat := (lat2 - lat1) * math.Pi / 180
	dLon := (lon2 - lon1) * math.Pi / 180
	a := math.Sin(dLat/2)*math.Sin(dLat/2) +
		math.Cos(lat1*math.Pi/180)*math.Cos(lat2*math.Pi/180)*
			math.Sin(dLon/2)*math.Sin(dLon/2)
	return R * 2 * math.Atan2(math.Sqrt(a), math.Sqrt(1-a))
}
