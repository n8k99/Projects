"""Binary to Decimal and Back — convert between number systems."""


def to_binary(n: int) -> str:
    """Convert a non-negative integer to its binary string representation."""
    if n < 0:
        raise ValueError("n must be non-negative")
    if n == 0:
        return "0"
    bits = []
    while n > 0:
        bits.append(str(n & 1))
        n >>= 1
    return "".join(reversed(bits))


def to_decimal(binary: str) -> int:
    """Convert a binary string to its decimal integer value."""
    if not binary:
        raise ValueError("empty string")
    if any(c not in "01" for c in binary):
        raise ValueError(f"invalid binary string: {binary}")
    result = 0
    for bit in binary:
        result = (result << 1) | int(bit)
    return result


def to_base(n: int, base: int) -> str:
    """Convert a non-negative integer to an arbitrary base (2-36)."""
    if base < 2 or base > 36:
        raise ValueError("base must be between 2 and 36")
    if n < 0:
        raise ValueError("n must be non-negative")
    if n == 0:
        return "0"
    digits = "0123456789abcdefghijklmnopqrstuvwxyz"
    result = []
    while n > 0:
        result.append(digits[n % base])
        n //= base
    return "".join(reversed(result))


def from_base(s: str, base: int) -> int:
    """Convert a string in an arbitrary base (2-36) to decimal."""
    if base < 2 or base > 36:
        raise ValueError("base must be between 2 and 36")
    if not s:
        raise ValueError("empty string")
    digits = "0123456789abcdefghijklmnopqrstuvwxyz"
    result = 0
    for c in s.lower():
        val = digits.index(c)
        if val >= base:
            raise ValueError(f"digit '{c}' invalid for base {base}")
        result = result * base + val
    return result


if __name__ == "__main__":
    print(f"42 -> binary: {to_binary(42)}")
    print(f"101010 -> decimal: {to_decimal('101010')}")
    print(f"255 -> hex: {to_base(255, 16)}")
    print(f"ff -> decimal: {from_base('ff', 16)}")
