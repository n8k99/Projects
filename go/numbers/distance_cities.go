package numbers

// Distance Between Two Cities — great-circle distance via Haversine.

import (
	"fmt"
	"math"
)

// Earth's mean radius in kilometres.
const earthRadiusKm = 6371.0

type cityCoord struct {
	lat, lon float64
}

var cityTable = map[string]cityCoord{
	"New York":  {40.7128, -74.0060},
	"London":    {51.5074, -0.1278},
	"Tokyo":     {35.6762, 139.6503},
	"Sydney":    {-33.8688, 151.2093},
	"São Paulo": {-23.5505, -46.6333},
	"Cairo":     {30.0444, 31.2357},
}

func deg2rad(d float64) float64 {
	return d * math.Pi / 180.0
}

// Haversine returns the great-circle distance in km between two points
// given as latitude/longitude in degrees.
func Haversine(lat1, lon1, lat2, lon2 float64) float64 {
	lat1, lon1 = deg2rad(lat1), deg2rad(lon1)
	lat2, lon2 = deg2rad(lat2), deg2rad(lon2)
	dlat := lat2 - lat1
	dlon := lon2 - lon1
	a := math.Pow(math.Sin(dlat/2), 2) +
		math.Cos(lat1)*math.Cos(lat2)*math.Pow(math.Sin(dlon/2), 2)
	c := 2 * math.Atan2(math.Sqrt(a), math.Sqrt(1-a))
	return earthRadiusKm * c
}

// CityDistance returns the great-circle distance in km between two named cities.
func CityDistance(city1, city2 string) (float64, error) {
	c1, ok1 := cityTable[city1]
	if !ok1 {
		return 0, fmt.Errorf("unknown city: %s", city1)
	}
	c2, ok2 := cityTable[city2]
	if !ok2 {
		return 0, fmt.Errorf("unknown city: %s", city2)
	}
	return Haversine(c1.lat, c1.lon, c2.lat, c2.lon), nil
}
