package numbers

import "fmt"

// Unit Converter — convert between measurement units.

// unitConvEntry holds the category and factor-to-base for a ratio unit.
// Base units: meters (distance), grams (weight), liters (volume).
type unitConvEntry struct {
	category string
	toBase   float64
}

var unitConvTable = map[string]unitConvEntry{
	// distance -> meters
	"km": {"distance", 1000.0},
	"m":  {"distance", 1.0},
	"mi": {"distance", 1609.344},
	"ft": {"distance", 0.3048},
	// weight -> grams
	"kg": {"weight", 1000.0},
	"g":  {"weight", 1.0},
	"lb": {"weight", 453.592},
	"oz": {"weight", 28.3495},
	// volume -> liters
	"L":   {"volume", 1.0},
	"gal": {"volume", 3.78541},
	"mL":  {"volume", 0.001},
	"cup": {"volume", 0.236588},
}

var unitConvTemps = map[string]bool{
	"celsius": true, "fahrenheit": true, "kelvin": true,
}

func unitConvToCelsius(value float64, unit string) float64 {
	switch unit {
	case "celsius":
		return value
	case "fahrenheit":
		return (value - 32.0) * 5.0 / 9.0
	case "kelvin":
		return value - 273.15
	}
	return 0
}

func unitConvFromCelsius(value float64, unit string) float64 {
	switch unit {
	case "celsius":
		return value
	case "fahrenheit":
		return value*9.0/5.0 + 32.0
	case "kelvin":
		return value + 273.15
	}
	return 0
}

// ConvertUnit converts a value from one measurement unit to another.
// Supports temperature (celsius, fahrenheit, kelvin),
// distance (km, mi, m, ft), weight (kg, lb, oz, g),
// and volume (L, gal, mL, cup).
func ConvertUnit(value float64, from, to string) (float64, error) {
	if from == to {
		return value, nil
	}

	fromIsTemp := unitConvTemps[from]
	toIsTemp := unitConvTemps[to]

	if fromIsTemp || toIsTemp {
		if !fromIsTemp || !toIsTemp {
			return 0, fmt.Errorf("cannot convert %s to %s", from, to)
		}
		return unitConvFromCelsius(unitConvToCelsius(value, from), to), nil
	}

	fromInfo, fromOk := unitConvTable[from]
	toInfo, toOk := unitConvTable[to]

	if !fromOk {
		return 0, fmt.Errorf("unknown unit: %s", from)
	}
	if !toOk {
		return 0, fmt.Errorf("unknown unit: %s", to)
	}
	if fromInfo.category != toInfo.category {
		return 0, fmt.Errorf("cannot convert %s (%s) to %s (%s)",
			from, fromInfo.category, to, toInfo.category)
	}

	return value * fromInfo.toBase / toInfo.toBase, nil
}

// Demo usage (in a main package):
//
//	func main() {
//		demos := [][3]string{
//			{"100", "celsius", "fahrenheit"},
//			{"1", "mi", "km"},
//			{"2.5", "kg", "lb"},
//			{"1", "gal", "L"},
//		}
//		for _, d := range demos {
//			val, _ := strconv.ParseFloat(d[0], 64)
//			result, err := numbers.ConvertUnit(val, d[1], d[2])
//			if err != nil {
//				fmt.Printf("ERROR: %v\n", err)
//			} else {
//				fmt.Printf("%s %s = %.4f %s\n", d[0], d[1], result, d[2])
//			}
//		}
//	}
