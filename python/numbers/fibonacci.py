"""Fibonacci sequence — generate to a count or up to a value."""

from typing import Iterator


def fibonacci(n: int) -> list[int]:
    """Return the first n Fibonacci numbers."""
    if n < 0:
        raise ValueError("n must be non-negative")
    if n == 0:
        return []
    if n == 1:
        return [0]

    seq = [0, 1]
    for _ in range(n - 2):
        seq.append(seq[-1] + seq[-2])
    return seq


def fibonacci_up_to(limit: int) -> list[int]:
    """Return all Fibonacci numbers up to and including limit."""
    if limit < 0:
        return []
    seq = [0]
    if limit == 0:
        return seq
    a, b = 0, 1
    while b <= limit:
        seq.append(b)
        a, b = b, a + b
    return seq


def fibonacci_gen() -> Iterator[int]:
    """Infinite Fibonacci generator."""
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b


def fib(n: int) -> int:
    """Return the nth Fibonacci number (0-indexed)."""
    if n < 0:
        raise ValueError("n must be non-negative")
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
