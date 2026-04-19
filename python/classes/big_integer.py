"""Handle Large Numbers — an arbitrary-precision signed integer as a class.

The class IS the number. Not a container for values; not an envelope around
external bytes. One instance represents one mathematical integer of arbitrary
size, with operations that are ALGEBRAIC LAWS — addition is commutative and
associative, multiplication distributes over addition, comparison is a total
order.

Representation:
  * digits stored base-10, little-endian (digits[0] is the ones place).
  * no trailing zeros (canonical form).
  * zero is exactly {negative: False, digits: []}  — never -0.

Schoolbook algorithms throughout. This file is the arithmetic you learned in
elementary school, written down precisely.
"""

from __future__ import annotations
from dataclasses import dataclass, field


@dataclass(frozen=True)
class BigInt:
    negative: bool
    digits: tuple[int, ...]    # little-endian base-10; empty = zero

    # --- construction ---
    @staticmethod
    def zero() -> "BigInt":
        return BigInt(False, ())

    @staticmethod
    def from_int(n: int) -> "BigInt":
        if n == 0:
            return BigInt.zero()
        neg = n < 0
        n = abs(n)
        digs: list[int] = []
        while n:
            digs.append(n % 10)
            n //= 10
        return BigInt(neg, tuple(digs))

    @staticmethod
    def from_string(s: str) -> "BigInt":
        s = s.strip()
        if not s:
            raise ValueError("empty string")
        neg = False
        if s[0] in "+-":
            neg = s[0] == "-"
            s = s[1:]
        if not s or any(c not in "0123456789" for c in s):
            raise ValueError(f"not a decimal integer: {s!r}")
        digs = [int(c) for c in reversed(s)]
        # Canonicalize: strip leading zeros (at the high end = tuple tail).
        while len(digs) > 1 and digs[-1] == 0:
            digs.pop()
        if digs == [0]:
            return BigInt.zero()
        return BigInt(neg, tuple(digs))

    # --- predicates ---
    def is_zero(self) -> bool:
        return len(self.digits) == 0

    def __bool__(self) -> bool:
        return not self.is_zero()

    # --- formatting ---
    def __str__(self) -> str:
        if self.is_zero():
            return "0"
        body = "".join(str(d) for d in reversed(self.digits))
        return ("-" if self.negative else "") + body

    def __repr__(self) -> str:
        return f"BigInt({self})"

    # --- comparison ---
    def _cmp_magnitude(self, other: "BigInt") -> int:
        """Compare absolute values. Returns -1/0/1."""
        if len(self.digits) != len(other.digits):
            return -1 if len(self.digits) < len(other.digits) else 1
        for a, b in zip(reversed(self.digits), reversed(other.digits)):
            if a != b:
                return -1 if a < b else 1
        return 0

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, BigInt):
            return NotImplemented
        return self.negative == other.negative and self.digits == other.digits

    def __hash__(self) -> int:
        return hash((self.negative, self.digits))

    def __lt__(self, other: "BigInt") -> bool:
        if self.is_zero() and other.is_zero():
            return False
        if self.negative != other.negative:
            return self.negative
        mag = self._cmp_magnitude(other)
        return mag > 0 if self.negative else mag < 0

    def __le__(self, other: "BigInt") -> bool: return self < other or self == other
    def __gt__(self, other: "BigInt") -> bool: return other < self
    def __ge__(self, other: "BigInt") -> bool: return self > other or self == other

    # --- sign operations ---
    def neg(self) -> "BigInt":
        if self.is_zero():
            return self
        return BigInt(not self.negative, self.digits)

    def abs(self) -> "BigInt":
        return BigInt(False, self.digits) if self.negative else self

    # --- addition / subtraction ---
    def add(self, other: "BigInt") -> "BigInt":
        if self.negative == other.negative:
            return BigInt(self.negative, _add_digits(self.digits, other.digits)) \
                if not self.is_zero() or not other.is_zero() else BigInt.zero()
        # Different signs: subtract smaller-magnitude from larger, keep sign of larger.
        cmp = self._cmp_magnitude(other)
        if cmp == 0:
            return BigInt.zero()
        if cmp > 0:
            return BigInt(self.negative, _sub_digits(self.digits, other.digits))
        return BigInt(other.negative, _sub_digits(other.digits, self.digits))

    def sub(self, other: "BigInt") -> "BigInt":
        return self.add(other.neg())

    def __add__(self, other: "BigInt") -> "BigInt": return self.add(other)
    def __sub__(self, other: "BigInt") -> "BigInt": return self.sub(other)
    def __neg__(self) -> "BigInt": return self.neg()
    def __abs__(self) -> "BigInt": return self.abs()

    # --- multiplication ---
    def mul(self, other: "BigInt") -> "BigInt":
        if self.is_zero() or other.is_zero():
            return BigInt.zero()
        sign = self.negative != other.negative
        prod = _mul_digits(self.digits, other.digits)
        return BigInt(sign, prod)

    def __mul__(self, other: "BigInt") -> "BigInt": return self.mul(other)


# --- low-level digit operations (magnitude only) ---

def _add_digits(a: tuple[int, ...], b: tuple[int, ...]) -> tuple[int, ...]:
    """Schoolbook add: digit + digit + carry, write digit, carry the rest."""
    if not a: return b
    if not b: return a
    out: list[int] = []
    carry = 0
    for i in range(max(len(a), len(b))):
        da = a[i] if i < len(a) else 0
        db = b[i] if i < len(b) else 0
        s = da + db + carry
        out.append(s % 10)
        carry = s // 10
    if carry:
        out.append(carry)
    # No leading zeros to strip; addition of magnitudes can only grow.
    return tuple(out)


def _sub_digits(a: tuple[int, ...], b: tuple[int, ...]) -> tuple[int, ...]:
    """Schoolbook subtract: assumes |a| >= |b|. Result may have leading zeros to strip."""
    out: list[int] = []
    borrow = 0
    for i in range(len(a)):
        da = a[i]
        db = b[i] if i < len(b) else 0
        d = da - db - borrow
        if d < 0:
            d += 10
            borrow = 1
        else:
            borrow = 0
        out.append(d)
    assert borrow == 0, "sub_digits called with |a| < |b|"
    while len(out) > 1 and out[-1] == 0:
        out.pop()
    if out == [0]:
        return ()
    return tuple(out)


def _mul_digits(a: tuple[int, ...], b: tuple[int, ...]) -> tuple[int, ...]:
    """Schoolbook multiply: each partial product shifted, then summed."""
    out = [0] * (len(a) + len(b))
    for i, da in enumerate(a):
        carry = 0
        for j, db in enumerate(b):
            s = out[i + j] + da * db + carry
            out[i + j] = s % 10
            carry = s // 10
        out[i + len(b)] += carry
    while len(out) > 1 and out[-1] == 0:
        out.pop()
    if out == [0]:
        return ()
    return tuple(out)


# --- demo ---
if __name__ == "__main__":
    a = BigInt.from_string("123456789012345678901234567890")
    b = BigInt.from_string("-98765432109876543210")
    print(f"a            = {a}")
    print(f"b            = {b}")
    print(f"a + b        = {a + b}")
    print(f"a - b        = {a - b}")
    print(f"a * b        = {a * b}")
    print(f"|b|          = {abs(b)}")
    print(f"a > b        = {a > b}")

    # Factorial of 30 by schoolbook multiplication.
    fact = BigInt.from_int(1)
    for k in range(1, 31):
        fact = fact * BigInt.from_int(k)
    print(f"30!          = {fact}")

    # Round-trip test against native ints.
    native = 123456789012345678901234567890 * -98765432109876543210
    assert str(a * b) == str(native), f"mismatch: {a * b} vs {native}"
    print("round-trip against native int: OK")
