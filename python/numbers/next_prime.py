"""Next prime number — continuously find primes on demand."""

from typing import Iterator


def is_prime(n: int) -> bool:
    """Test whether n is prime."""
    if n < 2:
        return False
    if n < 4:
        return True
    if n % 2 == 0 or n % 3 == 0:
        return False
    d = 5
    while d * d <= n:
        if n % d == 0 or n % (d + 2) == 0:
            return False
        d += 6
    return True


def next_prime(n: int) -> int:
    """Return the smallest prime greater than n."""
    candidate = n + 1 if n >= 2 else 2
    if candidate == 2:
        return 2
    if candidate % 2 == 0:
        candidate += 1
    while not is_prime(candidate):
        candidate += 2
    return candidate


def primes() -> Iterator[int]:
    """Infinite prime generator."""
    yield 2
    candidate = 3
    while True:
        if is_prime(candidate):
            yield candidate
        candidate += 2


def nth_prime(n: int) -> int:
    """Return the nth prime (1-indexed: nth_prime(1) = 2)."""
    if n < 1:
        raise ValueError("n must be positive")
    gen = primes()
    for _ in range(n - 1):
        next(gen)
    return next(gen)
