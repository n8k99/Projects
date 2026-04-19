// Chart Making — a visualization pipeline as composed functions.
package classes

import (
	"fmt"
	"math"
	"strings"
)

// DataPoint is one observation with a label and a numeric value.
type DataPoint struct {
	Label  string
	Value  float64
	Series string
}

// LinearScale maps a numeric domain to a numeric range.
type LinearScale struct {
	DomainMin, DomainMax float64
	RangeMin, RangeMax   float64
}

// At evaluates the scale at x.
func (s LinearScale) At(x float64) float64 {
	if s.DomainMax == s.DomainMin {
		return (s.RangeMin + s.RangeMax) / 2.0
	}
	t := (x - s.DomainMin) / (s.DomainMax - s.DomainMin)
	return s.RangeMin + t*(s.RangeMax-s.RangeMin)
}

// FitLinearScale builds a scale that comfortably spans the provided values.
func FitLinearScale(values []float64, rangeMin, rangeMax float64, includeZero bool) LinearScale {
	if len(values) == 0 {
		return LinearScale{DomainMin: 0, DomainMax: 1, RangeMin: rangeMin, RangeMax: rangeMax}
	}
	lo := math.Inf(1)
	hi := math.Inf(-1)
	for _, v := range values {
		if v < lo {
			lo = v
		}
		if v > hi {
			hi = v
		}
	}
	if includeZero {
		lo = math.Min(lo, 0)
		hi = math.Max(hi, 0)
	}
	if lo == hi {
		lo -= 1
		hi += 1
	}
	return LinearScale{DomainMin: lo, DomainMax: hi, RangeMin: rangeMin, RangeMax: rangeMax}
}

// OrdinalScale maps discrete categories to consecutive positions.
type OrdinalScale struct {
	Categories         []string
	RangeMin, RangeMax int
}

// At returns the position for a category, or -1 if unknown.
func (s OrdinalScale) At(category string) int {
	idx := -1
	for i, c := range s.Categories {
		if c == category {
			idx = i
			break
		}
	}
	if idx < 0 {
		return -1
	}
	n := len(s.Categories)
	if n <= 1 {
		return (s.RangeMin + s.RangeMax) / 2
	}
	step := float64(s.RangeMax-s.RangeMin) / float64(n-1)
	return int(math.Round(float64(s.RangeMin) + float64(idx)*step))
}

// BarChart is a horizontal ASCII bar chart spec.
type BarChart struct {
	Title  string
	Points []DataPoint
	Width  int
}

// Render produces the chart as a string.
func (c BarChart) Render() string {
	if len(c.Points) == 0 {
		return c.Title + "\n(no data)"
	}
	values := make([]float64, len(c.Points))
	for i, p := range c.Points {
		values[i] = p.Value
	}
	scale := FitLinearScale(values, 0, float64(c.Width), true)
	labelW := 0
	for _, p := range c.Points {
		if len(p.Label) > labelW {
			labelW = len(p.Label)
		}
	}
	var b strings.Builder
	b.WriteString(c.Title)
	b.WriteByte('\n')
	b.WriteString(strings.Repeat("=", labelW+4+c.Width+10))
	b.WriteByte('\n')
	for _, p := range c.Points {
		barLen := int(math.Round(scale.At(p.Value)))
		if barLen < 0 {
			barLen = 0
		}
		bar := strings.Repeat("█", barLen)
		fmt.Fprintf(&b, "  %-*s  %-*s  %.2f\n", labelW, p.Label, c.Width, bar, p.Value)
	}
	return strings.TrimRight(b.String(), "\n")
}

// LineChart is a vertical ASCII line-chart spec.
type LineChart struct {
	Title          string
	Points         []DataPoint
	Width, Height  int
}

// Render produces the chart as a string.
func (c LineChart) Render() string {
	if len(c.Points) == 0 {
		return c.Title + "\n(no data)"
	}
	n := len(c.Points)
	values := make([]float64, n)
	for i, p := range c.Points {
		values[i] = p.Value
	}
	xDomainMax := float64(n - 1)
	if xDomainMax < 1 {
		xDomainMax = 1
	}
	xScale := LinearScale{DomainMin: 0, DomainMax: xDomainMax,
		RangeMin: 0, RangeMax: float64(c.Width - 1)}
	yScale := FitLinearScale(values, 0, float64(c.Height-1), false)

	grid := make([][]rune, c.Height)
	for i := range grid {
		grid[i] = make([]rune, c.Width)
		for j := range grid[i] {
			grid[i][j] = ' '
		}
	}
	plot := func(col, row int, r rune) {
		if col >= 0 && col < c.Width && row >= 0 && row < c.Height {
			grid[row][col] = r
		}
	}

	var pc, pr int
	var havePrev bool
	for i, p := range c.Points {
		col := int(math.Round(xScale.At(float64(i))))
		row := c.Height - 1 - int(math.Round(yScale.At(p.Value)))
		plot(col, row, '●')
		if havePrev {
			dx := col - pc
			dy := row - pr
			steps := int(math.Max(math.Abs(float64(dx)), math.Abs(float64(dy))))
			if steps > 1 {
				for s := 1; s < steps; s++ {
					t := float64(s) / float64(steps)
					ic := int(math.Round(float64(pc) + float64(dx)*t))
					ir := int(math.Round(float64(pr) + float64(dy)*t))
					if ir >= 0 && ir < c.Height && ic >= 0 && ic < c.Width && grid[ir][ic] == ' ' {
						plot(ic, ir, '·')
					}
				}
			}
		}
		pc, pr = col, row
		havePrev = true
	}

	var b strings.Builder
	b.WriteString(c.Title)
	b.WriteByte('\n')
	hi := yScale.DomainMax
	lo := yScale.DomainMin
	for r := 0; r < c.Height; r++ {
		var left string
		switch r {
		case 0:
			left = fmt.Sprintf("%7.2f ┤", hi)
		case c.Height - 1:
			left = fmt.Sprintf("%7.2f ┤", lo)
		default:
			left = "        │"
		}
		b.WriteString(left)
		b.WriteString(string(grid[r]))
		b.WriteByte('\n')
	}
	b.WriteString("        └")
	b.WriteString(strings.Repeat("─", c.Width))
	return b.String()
}
