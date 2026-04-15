"""Find PI to the Nth digit using arbitrary precision."""

from decimal import Decimal, getcontext


def pi(n: int) -> str:
    """Return PI to n decimal places as a string.

    Uses the Chudnovsky algorithm — fastest known series for PI,
    converging at ~14 digits per term.
    """
    if n < 0:
        raise ValueError("n must be non-negative")
    if n == 0:
        return "3"

    # Extra precision to avoid rounding errors in final digits
    getcontext().prec = n + 20

    C = Decimal(426880) * Decimal(10005).sqrt()
    K = Decimal(6)
    M = Decimal(1)
    X = Decimal(1)
    L = Decimal(13591409)
    S = Decimal(13591409)

    for i in range(1, n // 14 + 2):
        M = M * (K**3 - 16 * K) / Decimal(i**3)
        K += 12
        L += 545140134
        X *= -262537412640768000
        S += Decimal(M * L) / X

    result = str(C / S)

    # Format to exactly n decimal places
    if '.' in result:
        integer, decimal = result.split('.')
        return f"{integer}.{decimal[:n]}"
    return result


if __name__ == "__main__":
    import sys
    digits = int(sys.argv[1]) if len(sys.argv) > 1 else 50
    print(pi(digits))
