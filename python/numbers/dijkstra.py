"""Dijkstra's Algorithm — shortest paths in weighted graphs."""

import heapq
from collections import defaultdict


class Graph:
    """Weighted directed graph with Dijkstra's shortest-path support."""

    def __init__(self):
        self._adj = defaultdict(list)

    def add_edge(self, u: str, v: str, weight: float, undirected: bool = False):
        """Add a weighted edge from u to v. If undirected, add both directions."""
        self._adj[u].append((v, weight))
        # Ensure v exists even if it has no outgoing edges.
        if v not in self._adj:
            self._adj[v] = []
        if undirected:
            self._adj[v].append((u, weight))

    def shortest_distances(self, start: str) -> dict[str, float]:
        """Return shortest distance from start to every reachable node."""
        dist = {node: float("inf") for node in self._adj}
        dist[start] = 0.0
        # Priority queue: (distance, node)
        pq = [(0.0, start)]

        while pq:
            d, u = heapq.heappop(pq)
            if d > dist[u]:
                continue
            for v, w in self._adj[u]:
                nd = d + w
                if nd < dist[v]:
                    dist[v] = nd
                    heapq.heappush(pq, (nd, v))

        return dist

    def shortest_path(self, start: str, end: str) -> tuple[float, list[str]]:
        """Return (distance, path) for the shortest path from start to end.

        Returns (inf, []) if no path exists.
        """
        dist = {node: float("inf") for node in self._adj}
        prev = {node: None for node in self._adj}
        dist[start] = 0.0
        pq = [(0.0, start)]

        while pq:
            d, u = heapq.heappop(pq)
            if d > dist[u]:
                continue
            if u == end:
                break
            for v, w in self._adj[u]:
                nd = d + w
                if nd < dist[v]:
                    dist[v] = nd
                    prev[v] = u
                    heapq.heappush(pq, (nd, v))

        if dist[end] == float("inf"):
            return (float("inf"), [])

        # Reconstruct path.
        path = []
        node = end
        while node is not None:
            path.append(node)
            node = prev[node]
        path.reverse()
        return (dist[end], path)


if __name__ == "__main__":
    g = Graph()
    # Build a small road network.
    g.add_edge("A", "B", 4)
    g.add_edge("A", "C", 2)
    g.add_edge("B", "D", 3)
    g.add_edge("B", "E", 1)
    g.add_edge("C", "B", 1)
    g.add_edge("C", "D", 5)
    g.add_edge("D", "E", 2)
    g.add_edge("D", "F", 6)
    g.add_edge("E", "F", 3)

    start, end = "A", "F"
    dist, path = g.shortest_path(start, end)
    print(f"{start} → {end}: distance {dist}, path {' → '.join(path)}")

    print("\nAll distances from A:")
    for node, d in sorted(g.shortest_distances("A").items()):
        print(f"  A → {node}: {d}")
