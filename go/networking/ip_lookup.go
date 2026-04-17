// Country from IP Lookup — resolve IP addresses to geographic location data.
package networking

import (
	"fmt"
	"math"
	"sort"
	"strings"
)

// GeoIPResult holds geographic data for an IP.
type GeoIPResult struct {
	IP          string
	CountryCode string
	CountryName string
	Region      string
	City        string
	Latitude    float64
	Longitude   float64
	Timezone    string
	IsPrivate   bool
}

// Summary returns a one-line description.
func (r GeoIPResult) Summary() string {
	if r.IsPrivate {
		return fmt.Sprintf("%s -> private/reserved", r.IP)
	}
	var parts []string
	for _, s := range []string{r.City, r.Region, r.CountryName} {
		if s != "" {
			parts = append(parts, s)
		}
	}
	return fmt.Sprintf("%s -> %s (%s) [%.4f, %.4f]",
		r.IP, strings.Join(parts, ", "), r.CountryCode, r.Latitude, r.Longitude)
}

// GeoIPRange maps a range of IPs to a location.
type GeoIPRange struct {
	Start       uint32
	End         uint32
	CountryCode string
	CountryName string
	Region      string
	City        string
	Latitude    float64
	Longitude   float64
	Timezone    string
}

// IPToUint32 converts a dotted-quad IP to uint32.
func IPToUint32(ip string) uint32 {
	var a, b, c, d uint32
	fmt.Sscanf(ip, "%d.%d.%d.%d", &a, &b, &c, &d)
	return (a << 24) | (b << 16) | (c << 8) | d
}

// Uint32ToIP converts uint32 to dotted-quad.
func Uint32ToIP(n uint32) string {
	return fmt.Sprintf("%d.%d.%d.%d", (n>>24)&0xFF, (n>>16)&0xFF, (n>>8)&0xFF, n&0xFF)
}

// IsPrivateIP checks if an IP is private/reserved.
func IsPrivateIP(ip string) bool {
	n := IPToUint32(ip)
	return (n >= 0x0A000000 && n <= 0x0AFFFFFF) || // 10/8
		(n >= 0xAC100000 && n <= 0xAC1FFFFF) || // 172.16/12
		(n >= 0xC0A80000 && n <= 0xC0A8FFFF) || // 192.168/16
		(n >= 0x7F000000 && n <= 0x7FFFFFFF) || // 127/8
		n <= 0x00FFFFFF // 0/8
}

func builtinRanges() []GeoIPRange {
	r := func(s, e, cc, cn, rg, ct string, lat, lon float64, tz string) GeoIPRange {
		return GeoIPRange{IPToUint32(s), IPToUint32(e), cc, cn, rg, ct, lat, lon, tz}
	}
	return []GeoIPRange{
		r("1.0.0.0", "1.0.0.255", "AU", "Australia", "Queensland", "Brisbane", -27.4679, 153.0281, "Australia/Brisbane"),
		r("1.1.1.0", "1.1.1.255", "AU", "Australia", "New South Wales", "Sydney", -33.8688, 151.2093, "Australia/Sydney"),
		r("8.8.4.0", "8.8.8.255", "US", "United States", "California", "Mountain View", 37.3861, -122.0839, "America/Los_Angeles"),
		r("9.0.0.0", "9.255.255.255", "US", "United States", "New York", "Armonk", 41.1085, -73.7205, "America/New_York"),
		r("13.0.0.0", "13.255.255.255", "US", "United States", "Virginia", "Ashburn", 39.0438, -77.4874, "America/New_York"),
		r("17.0.0.0", "17.255.255.255", "US", "United States", "California", "Cupertino", 37.3230, -122.0322, "America/Los_Angeles"),
		r("41.0.0.0", "41.255.255.255", "ZA", "South Africa", "Gauteng", "Johannesburg", -26.2041, 28.0473, "Africa/Johannesburg"),
		r("77.0.0.0", "77.255.255.255", "DE", "Germany", "Hessen", "Frankfurt", 50.1109, 8.6821, "Europe/Berlin"),
		r("101.0.0.0", "101.255.255.255", "JP", "Japan", "Tokyo", "Tokyo", 35.6762, 139.6503, "Asia/Tokyo"),
		r("103.0.0.0", "103.255.255.255", "IN", "India", "Maharashtra", "Mumbai", 19.0760, 72.8777, "Asia/Kolkata"),
		r("144.126.0.0", "144.126.255.255", "US", "United States", "New York", "New York", 40.7128, -74.0060, "America/New_York"),
		r("185.0.0.0", "185.255.255.255", "NL", "Netherlands", "North Holland", "Amsterdam", 52.3676, 4.9041, "Europe/Amsterdam"),
		r("212.0.0.0", "212.255.255.255", "GB", "United Kingdom", "England", "London", 51.5074, -0.1278, "Europe/London"),
	}
}

// GeoIPDatabase maps IP addresses to geographic locations.
type GeoIPDatabase struct {
	ranges []GeoIPRange
}

// NewGeoIPDatabase creates a database with built-in ranges.
func NewGeoIPDatabase() *GeoIPDatabase {
	ranges := builtinRanges()
	sort.Slice(ranges, func(i, j int) bool { return ranges[i].Start < ranges[j].Start })
	return &GeoIPDatabase{ranges: ranges}
}

// Lookup resolves an IP to geographic data.
func (db *GeoIPDatabase) Lookup(ip string) GeoIPResult {
	if IsPrivateIP(ip) {
		return GeoIPResult{IP: ip, IsPrivate: true}
	}
	n := IPToUint32(ip)
	lo, hi := 0, len(db.ranges)-1
	for lo <= hi {
		mid := (lo + hi) / 2
		r := &db.ranges[mid]
		if n < r.Start {
			hi = mid - 1
		} else if n > r.End {
			lo = mid + 1
		} else {
			return GeoIPResult{
				IP: ip, CountryCode: r.CountryCode, CountryName: r.CountryName,
				Region: r.Region, City: r.City, Latitude: r.Latitude,
				Longitude: r.Longitude, Timezone: r.Timezone,
			}
		}
	}
	return GeoIPResult{IP: ip}
}

// BulkLookup resolves multiple IPs.
func (db *GeoIPDatabase) BulkLookup(ips []string) []GeoIPResult {
	results := make([]GeoIPResult, len(ips))
	for i, ip := range ips {
		results[i] = db.Lookup(ip)
	}
	return results
}

// CountryBreakdown counts IPs per country.
func (db *GeoIPDatabase) CountryBreakdown(ips []string) map[string]int {
	counts := make(map[string]int)
	for _, ip := range ips {
		r := db.Lookup(ip)
		key := r.CountryCode
		if r.IsPrivate {
			key = "private"
		} else if key == "" {
			key = "unknown"
		}
		counts[key]++
	}
	return counts
}

// HaversineKm returns the distance in km between two lat/lon points.
func HaversineKm(lat1, lon1, lat2, lon2 float64) float64 {
	const R = 6371.0
	dLat := (lat2 - lat1) * math.Pi / 180
	dLon := (lon2 - lon1) * math.Pi / 180
	a := math.Sin(dLat/2)*math.Sin(dLat/2) +
		math.Cos(lat1*math.Pi/180)*math.Cos(lat2*math.Pi/180)*
			math.Sin(dLon/2)*math.Sin(dLon/2)
	return R * 2 * math.Atan2(math.Sqrt(a), math.Sqrt(1-a))
}

// NearestServer finds the geographically closest server to a client IP.
func NearestServer(clientIP string, serverIPs []string, db *GeoIPDatabase) string {
	client := db.Lookup(clientIP)
	if client.IsPrivate || client.CountryCode == "" {
		return ""
	}
	bestIP := ""
	bestDist := math.MaxFloat64
	for _, sip := range serverIPs {
		srv := db.Lookup(sip)
		if srv.IsPrivate || srv.CountryCode == "" {
			continue
		}
		d := HaversineKm(client.Latitude, client.Longitude, srv.Latitude, srv.Longitude)
		if d < bestDist {
			bestDist = d
			bestIP = sip
		}
	}
	return bestIP
}
