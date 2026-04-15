"""Prime factorization — find all prime factors of a number."""


def prime_factors(n: int) -> list[int]:
    """Return the prime factors of n in ascending order, with repetition."""
    if n < 2:
        return []

    factors = []
    d = 2
    while d * d <= n:
        while n % d == 0:
            factors.append(d)
            n //= d
        d += 1
    if n > 1:
        factors.append(n)
    return factors


def prime_factorization(n: int) -> dict[int, int]:
    """Return prime factorization as {prime: exponent}."""
    factors = prime_factors(n)
    result = {}
    for f in factors:
        result[f] = result.get(f, 0) + 1
    return result
