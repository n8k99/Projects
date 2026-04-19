"""Matrix Class — a 2D value class whose shape constrains its operations.

Shape (rows × cols) is part of the matrix's identity. Addition requires both
operands share a shape. Multiplication requires the left's columns to equal
the right's rows; the result takes the outer dimensions. Determinant is
defined only on square matrices.

This is the first project in the Rosetta Stone where:
  * The operand SHAPE (not just its value) gates which operations are legal.
  * An operation (multiplication) is NON-COMMUTATIVE. Order matters.
  * Operations are *partially defined* — `det(A)` exists only when A is square.
  * There is a two-sided identity (`I_n × A = A = A × I_m`), and the identity
    itself depends on the shape — a taste of dependent types.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Iterable


class MatrixError(Exception):
    """Base for matrix errors."""


class ShapeMismatch(MatrixError):
    """Operands have incompatible shapes."""


class NotSquare(MatrixError):
    """Operation requires a square matrix."""


@dataclass
class Matrix:
    """An m×n matrix of floats. Rows are stored as a list of row-lists."""
    rows: list[list[float]]

    def __post_init__(self) -> None:
        if not self.rows:
            raise MatrixError("matrix must have at least one row")
        cols = len(self.rows[0])
        if cols == 0:
            raise MatrixError("matrix must have at least one column")
        for r in self.rows:
            if len(r) != cols:
                raise ShapeMismatch(
                    f"row length mismatch: {len(r)} vs {cols}")

    # --- shape accessors ---
    @property
    def shape(self) -> tuple[int, int]:
        return (len(self.rows), len(self.rows[0]))

    @property
    def nrows(self) -> int: return len(self.rows)

    @property
    def ncols(self) -> int: return len(self.rows[0])

    @property
    def is_square(self) -> bool: return self.nrows == self.ncols

    # --- constructors ---
    @staticmethod
    def zeros(rows: int, cols: int) -> "Matrix":
        if rows <= 0 or cols <= 0:
            raise MatrixError(f"non-positive dimensions: {rows}x{cols}")
        return Matrix([[0.0] * cols for _ in range(rows)])

    @staticmethod
    def identity(n: int) -> "Matrix":
        if n <= 0:
            raise MatrixError(f"non-positive size: {n}")
        m = Matrix.zeros(n, n)
        for i in range(n):
            m.rows[i][i] = 1.0
        return m

    # --- element access ---
    def get(self, i: int, j: int) -> float: return self.rows[i][j]
    def set(self, i: int, j: int, v: float) -> None: self.rows[i][j] = v

    # --- equality ---
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Matrix):
            return NotImplemented
        return self.shape == other.shape and all(
            abs(self.rows[i][j] - other.rows[i][j]) < 1e-12
            for i in range(self.nrows) for j in range(self.ncols))

    # --- addition / subtraction (shape-gated) ---
    def add(self, other: "Matrix") -> "Matrix":
        if self.shape != other.shape:
            raise ShapeMismatch(
                f"cannot add: {self.shape} + {other.shape}")
        return Matrix([
            [self.rows[i][j] + other.rows[i][j] for j in range(self.ncols)]
            for i in range(self.nrows)
        ])

    def sub(self, other: "Matrix") -> "Matrix":
        if self.shape != other.shape:
            raise ShapeMismatch(
                f"cannot subtract: {self.shape} - {other.shape}")
        return Matrix([
            [self.rows[i][j] - other.rows[i][j] for j in range(self.ncols)]
            for i in range(self.nrows)
        ])

    def scalar_mul(self, k: float) -> "Matrix":
        return Matrix([[k * v for v in row] for row in self.rows])

    # --- matrix multiplication (inner dimensions must match) ---
    def matmul(self, other: "Matrix") -> "Matrix":
        if self.ncols != other.nrows:
            raise ShapeMismatch(
                f"cannot multiply: inner dimensions differ "
                f"({self.shape}) x ({other.shape})")
        m, n, p = self.nrows, self.ncols, other.ncols
        out = Matrix.zeros(m, p)
        for i in range(m):
            for j in range(p):
                s = 0.0
                for k in range(n):
                    s += self.rows[i][k] * other.rows[k][j]
                out.rows[i][j] = s
        return out

    # --- operator overloading ---
    def __add__(self, other: "Matrix") -> "Matrix": return self.add(other)
    def __sub__(self, other: "Matrix") -> "Matrix": return self.sub(other)
    def __matmul__(self, other: "Matrix") -> "Matrix": return self.matmul(other)

    # --- transpose ---
    def transpose(self) -> "Matrix":
        return Matrix([[self.rows[i][j] for i in range(self.nrows)]
                       for j in range(self.ncols)])

    # --- determinant via cofactor expansion (square matrices only) ---
    def determinant(self) -> float:
        if not self.is_square:
            raise NotSquare(f"determinant needs square, got {self.shape}")
        return _det(self.rows)

    # --- formatting ---
    def __str__(self) -> str:
        w = max(len(f"{v:g}") for row in self.rows for v in row)
        lines = [
            "│ " + " ".join(f"{v:>{w}g}" for v in row) + " │"
            for row in self.rows
        ]
        return "\n".join(lines)


def _det(rows: list[list[float]]) -> float:
    """Cofactor-expansion determinant. Direct formulas for 1×1 and 2×2."""
    n = len(rows)
    if n == 1:
        return rows[0][0]
    if n == 2:
        return rows[0][0] * rows[1][1] - rows[0][1] * rows[1][0]
    total = 0.0
    for c in range(n):
        minor = [
            [rows[i][k] for k in range(n) if k != c]
            for i in range(1, n)
        ]
        sign = -1.0 if (c % 2) else 1.0
        total += sign * rows[0][c] * _det(minor)
    return total


# --- demo ---
if __name__ == "__main__":
    A = Matrix([[1, 2, 3], [4, 5, 6]])        # 2×3
    B = Matrix([[7, 8], [9, 10], [11, 12]])   # 3×2
    print("A (2×3):")
    print(A)
    print("\nB (3×2):")
    print(B)

    C = A @ B                                  # 2×2
    print("\nA × B (2×2):")
    print(C)

    print("\nB × A (3×3):")
    print(B @ A)

    print("\nA × B ≠ B × A — matrix multiplication is non-commutative.")
    print(f"A × B shape: {(A @ B).shape}    B × A shape: {(B @ A).shape}")

    # Transpose
    print("\nA transposed (3×2):")
    print(A.transpose())

    # Identity as neutral element
    I2 = Matrix.identity(2)
    I3 = Matrix.identity(3)
    print(f"\nI2 × A == A? {I2 @ A == A}")
    print(f"A × I3 == A? {A @ I3 == A}")

    # Determinants (square only)
    M = Matrix([[4, 3, 0], [2, 5, 6], [1, 0, 2]])
    print(f"\ndet({M.shape}) = {M.determinant():g}")

    # Shape-gated errors
    try:
        A.add(B)       # 2×3 + 3×2
    except ShapeMismatch as e:
        print(f"\ncaught: {e}")
    try:
        A.determinant()  # non-square
    except NotSquare as e:
        print(f"caught: {e}")
