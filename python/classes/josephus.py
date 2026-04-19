"""Josephus Problem — circular elimination and the survivor.

N people stand in a circle (0..n). Starting from position 0, count `k`
and remove that person; continue from the next remaining position.
The last one left is the survivor.

Two algorithms, two different queries on the same process:
- ``josephus(n, k)`` — O(n) recurrence ``J(n,k) = (J(n-1,k) + k) % n``,
  returns the survivor only.
- ``josephus_simulate(n, k)`` — O(n²) list simulation, returns both the
  survivor and the full elimination order.

The recurrence is the first closed-form shortcut in Rosetta Stone Classes:
a problem whose answer can be computed without materializing the process.
"""

from __future__ import annotations


def josephus(n: int, k: int) -> int:
    """Return the 0-indexed survivor via the O(n) recurrence."""
    if n < 1:
        raise ValueError("n must be >= 1")
    if k < 1:
        raise ValueError("k must be >= 1")
    survivor = 0
    for m in range(2, n + 1):
        survivor = (survivor + k) % m
    return survivor


def josephus_simulate(n: int, k: int) -> tuple[int, list[int]]:
    """Simulate the elimination. Returns (survivor, elimination_order)."""
    if n < 1:
        raise ValueError("n must be >= 1")
    if k < 1:
        raise ValueError("k must be >= 1")
    ring = list(range(n))
    order: list[int] = []
    pos = 0
    while len(ring) > 1:
        pos = (pos + k - 1) % len(ring)
        order.append(ring.pop(pos))
    return ring[0], order


if __name__ == "__main__":
    # Flavius Josephus's legend: 41 men in a circle, every 3rd killed.
    # Survivor: position 30 (0-indexed).
    n, k = 41, 3
    print(f"josephus(n={n}, k={k}) = {josephus(n, k)}")
    survivor, order = josephus_simulate(n, k)
    print(f"simulate  = {survivor}; first 10 eliminated: {order[:10]}")

    # Small hand-traceable case.
    print()
    print(f"josephus(7, 3) = {josephus(7, 3)}  (expected 3)")
    s, o = josephus_simulate(7, 3)
    print(f"simulate  = {s}; order = {o}  (expected [2, 5, 1, 6, 4, 0])")

    # Cross-check the recurrence against the simulation for a range.
    mismatches = [
        (n, k)
        for n in range(1, 30)
        for k in range(1, 7)
        if josephus(n, k) != josephus_simulate(n, k)[0]
    ]
    print()
    print(f"recurrence vs simulation: {len(mismatches)} mismatches "
          f"across 29*6 = 174 cases")
