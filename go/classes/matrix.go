// Matrix Class — a 2D value class whose shape constrains its operations.
package classes

import (
	"errors"
	"fmt"
)

// Matrix is a rectangular array of float64 organised row-major.
type Matrix struct {
	rows [][]float64
}

// Matrix errors.
var (
	ErrEmpty              = errors.New("empty matrix")
	ErrNotRectangular     = errors.New("rows have inconsistent lengths")
	ErrShapeMismatch      = errors.New("shape mismatch")
	ErrMatrixNotSquare    = errors.New("not square")
	ErrNonPositiveDim     = errors.New("non-positive dimension")
)

// NewMatrix builds a Matrix from rows, validating rectangularity.
func NewMatrix(rows [][]float64) (*Matrix, error) {
	if len(rows) == 0 {
		return nil, ErrEmpty
	}
	cols := len(rows[0])
	if cols == 0 {
		return nil, ErrEmpty
	}
	for _, r := range rows {
		if len(r) != cols {
			return nil, ErrNotRectangular
		}
	}
	// Defensive copy so caller mutations don't leak in.
	copied := make([][]float64, len(rows))
	for i, r := range rows {
		copied[i] = append([]float64(nil), r...)
	}
	return &Matrix{rows: copied}, nil
}

// Zeros builds a rows×cols matrix of zeros.
func Zeros(rows, cols int) (*Matrix, error) {
	if rows <= 0 || cols <= 0 {
		return nil, ErrNonPositiveDim
	}
	data := make([][]float64, rows)
	for i := range data {
		data[i] = make([]float64, cols)
	}
	return &Matrix{rows: data}, nil
}

// Identity builds an n×n identity matrix.
func Identity(n int) (*Matrix, error) {
	m, err := Zeros(n, n)
	if err != nil {
		return nil, err
	}
	for i := 0; i < n; i++ {
		m.rows[i][i] = 1.0
	}
	return m, nil
}

// NRows returns the row count.
func (m *Matrix) NRows() int { return len(m.rows) }

// NCols returns the column count.
func (m *Matrix) NCols() int { return len(m.rows[0]) }

// Shape returns (rows, cols).
func (m *Matrix) Shape() (int, int) { return m.NRows(), m.NCols() }

// IsSquare reports whether rows == cols.
func (m *Matrix) IsSquare() bool { return m.NRows() == m.NCols() }

// Get returns element at (i, j).
func (m *Matrix) Get(i, j int) float64 { return m.rows[i][j] }

// Set writes element at (i, j).
func (m *Matrix) Set(i, j int, v float64) { m.rows[i][j] = v }

// Add returns m + other; shapes must match.
func (m *Matrix) Add(other *Matrix) (*Matrix, error) {
	mr, mc := m.Shape()
	or, oc := other.Shape()
	if mr != or || mc != oc {
		return nil, fmt.Errorf("%w: %dx%d + %dx%d", ErrShapeMismatch, mr, mc, or, oc)
	}
	out, _ := Zeros(mr, mc)
	for i := 0; i < mr; i++ {
		for j := 0; j < mc; j++ {
			out.rows[i][j] = m.rows[i][j] + other.rows[i][j]
		}
	}
	return out, nil
}

// Sub returns m - other; shapes must match.
func (m *Matrix) Sub(other *Matrix) (*Matrix, error) {
	mr, mc := m.Shape()
	or, oc := other.Shape()
	if mr != or || mc != oc {
		return nil, fmt.Errorf("%w: %dx%d - %dx%d", ErrShapeMismatch, mr, mc, or, oc)
	}
	out, _ := Zeros(mr, mc)
	for i := 0; i < mr; i++ {
		for j := 0; j < mc; j++ {
			out.rows[i][j] = m.rows[i][j] - other.rows[i][j]
		}
	}
	return out, nil
}

// ScalarMul returns k * m.
func (m *Matrix) ScalarMul(k float64) *Matrix {
	out, _ := Zeros(m.Shape())
	for i := 0; i < m.NRows(); i++ {
		for j := 0; j < m.NCols(); j++ {
			out.rows[i][j] = k * m.rows[i][j]
		}
	}
	return out
}

// MatMul returns m × other; inner dimensions must match.
func (m *Matrix) MatMul(other *Matrix) (*Matrix, error) {
	if m.NCols() != other.NRows() {
		return nil, fmt.Errorf("%w: cannot multiply %dx%d * %dx%d",
			ErrShapeMismatch, m.NRows(), m.NCols(), other.NRows(), other.NCols())
	}
	mR, inner, oC := m.NRows(), m.NCols(), other.NCols()
	out, _ := Zeros(mR, oC)
	for i := 0; i < mR; i++ {
		for j := 0; j < oC; j++ {
			s := 0.0
			for k := 0; k < inner; k++ {
				s += m.rows[i][k] * other.rows[k][j]
			}
			out.rows[i][j] = s
		}
	}
	return out, nil
}

// Transpose returns mᵀ.
func (m *Matrix) Transpose() *Matrix {
	r, c := m.Shape()
	out, _ := Zeros(c, r)
	for i := 0; i < r; i++ {
		for j := 0; j < c; j++ {
			out.rows[j][i] = m.rows[i][j]
		}
	}
	return out
}

// Determinant returns |m|; m must be square.
func (m *Matrix) Determinant() (float64, error) {
	if !m.IsSquare() {
		return 0, fmt.Errorf("%w: %dx%d", ErrMatrixNotSquare, m.NRows(), m.NCols())
	}
	return detRecursive(m.rows), nil
}

// ApproxEqual reports whether two matrices match within eps.
func (m *Matrix) ApproxEqual(other *Matrix, eps float64) bool {
	if m.NRows() != other.NRows() || m.NCols() != other.NCols() {
		return false
	}
	for i := 0; i < m.NRows(); i++ {
		for j := 0; j < m.NCols(); j++ {
			d := m.rows[i][j] - other.rows[i][j]
			if d < 0 {
				d = -d
			}
			if d > eps {
				return false
			}
		}
	}
	return true
}

// detRecursive computes determinant via cofactor expansion on the first row.
func detRecursive(rows [][]float64) float64 {
	n := len(rows)
	if n == 1 {
		return rows[0][0]
	}
	if n == 2 {
		return rows[0][0]*rows[1][1] - rows[0][1]*rows[1][0]
	}
	total := 0.0
	for c := 0; c < n; c++ {
		minor := make([][]float64, n-1)
		for i := 1; i < n; i++ {
			row := make([]float64, 0, n-1)
			for k := 0; k < n; k++ {
				if k != c {
					row = append(row, rows[i][k])
				}
			}
			minor[i-1] = row
		}
		sign := 1.0
		if c%2 == 1 {
			sign = -1.0
		}
		total += sign * rows[0][c] * detRecursive(minor)
	}
	return total
}
