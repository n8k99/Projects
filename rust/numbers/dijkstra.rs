//! Dijkstra's Algorithm — shortest paths in weighted graphs.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Weighted directed graph stored as an adjacency list.
pub struct Graph {
    adj: HashMap<String, Vec<(String, f64)>>,
}

/// Wrapper for min-heap ordering (BinaryHeap is a max-heap by default).
#[derive(PartialEq)]
struct State {
    cost: f64,
    node: String,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Graph {
    /// Create an empty graph.
    pub fn new() -> Self {
        Graph {
            adj: HashMap::new(),
        }
    }

    /// Add a weighted edge from u to v. If undirected, add both directions.
    pub fn add_edge(&mut self, u: &str, v: &str, weight: f64, undirected: bool) {
        self.adj
            .entry(u.to_string())
            .or_default()
            .push((v.to_string(), weight));
        self.adj.entry(v.to_string()).or_default();
        if undirected {
            self.adj
                .entry(v.to_string())
                .or_default()
                .push((u.to_string(), weight));
        }
    }

    /// Return shortest distance from start to every reachable node.
    pub fn shortest_distances(&self, start: &str) -> HashMap<String, f64> {
        let mut dist: HashMap<String, f64> = self
            .adj
            .keys()
            .map(|k| (k.clone(), f64::INFINITY))
            .collect();
        dist.insert(start.to_string(), 0.0);

        let mut heap = BinaryHeap::new();
        heap.push(State {
            cost: 0.0,
            node: start.to_string(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            if cost > *dist.get(&node).unwrap_or(&f64::INFINITY) {
                continue;
            }
            if let Some(edges) = self.adj.get(&node) {
                for (v, w) in edges {
                    let nd = cost + w;
                    if nd < *dist.get(v).unwrap_or(&f64::INFINITY) {
                        dist.insert(v.clone(), nd);
                        heap.push(State {
                            cost: nd,
                            node: v.clone(),
                        });
                    }
                }
            }
        }

        dist
    }

    /// Return (distance, path) for the shortest path from start to end.
    /// Returns None if no path exists.
    pub fn shortest_path(&self, start: &str, end: &str) -> Option<(f64, Vec<String>)> {
        let mut dist: HashMap<String, f64> = self
            .adj
            .keys()
            .map(|k| (k.clone(), f64::INFINITY))
            .collect();
        let mut prev: HashMap<String, Option<String>> =
            self.adj.keys().map(|k| (k.clone(), None)).collect();

        dist.insert(start.to_string(), 0.0);

        let mut heap = BinaryHeap::new();
        heap.push(State {
            cost: 0.0,
            node: start.to_string(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            if cost > *dist.get(&node).unwrap_or(&f64::INFINITY) {
                continue;
            }
            if node == end {
                break;
            }
            if let Some(edges) = self.adj.get(&node) {
                for (v, w) in edges {
                    let nd = cost + w;
                    if nd < *dist.get(v).unwrap_or(&f64::INFINITY) {
                        dist.insert(v.clone(), nd);
                        prev.insert(v.clone(), Some(node.clone()));
                        heap.push(State {
                            cost: nd,
                            node: v.clone(),
                        });
                    }
                }
            }
        }

        let end_dist = *dist.get(end)?;
        if end_dist == f64::INFINITY {
            return None;
        }

        let mut path = Vec::new();
        let mut cur = Some(end.to_string());
        while let Some(ref node) = cur {
            path.push(node.clone());
            cur = prev.get(node).and_then(|p| p.clone());
        }
        path.reverse();
        Some((end_dist, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> Graph {
        let mut g = Graph::new();
        g.add_edge("A", "B", 4.0, false);
        g.add_edge("A", "C", 2.0, false);
        g.add_edge("B", "D", 3.0, false);
        g.add_edge("B", "E", 1.0, false);
        g.add_edge("C", "B", 1.0, false);
        g.add_edge("C", "D", 5.0, false);
        g.add_edge("D", "E", 2.0, false);
        g.add_edge("D", "F", 6.0, false);
        g.add_edge("E", "F", 3.0, false);
        g
    }

    #[test]
    fn test_shortest_path_a_to_f() {
        let g = sample_graph();
        let (dist, path) = g.shortest_path("A", "F").unwrap();
        assert!((dist - 7.0).abs() < 1e-9);
        assert_eq!(path, vec!["A", "C", "B", "E", "F"]);
    }

    #[test]
    fn test_shortest_distances() {
        let g = sample_graph();
        let dists = g.shortest_distances("A");
        assert!((dists["A"] - 0.0).abs() < 1e-9);
        assert!((dists["C"] - 2.0).abs() < 1e-9);
        assert!((dists["B"] - 3.0).abs() < 1e-9);
        assert!((dists["E"] - 4.0).abs() < 1e-9);
        assert!((dists["F"] - 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_no_path() {
        let mut g = Graph::new();
        g.add_edge("A", "B", 1.0, false);
        g.add_edge("C", "D", 1.0, false);
        assert!(g.shortest_path("A", "D").is_none());
    }

    #[test]
    fn test_undirected() {
        let mut g = Graph::new();
        g.add_edge("A", "B", 5.0, true);
        g.add_edge("B", "C", 3.0, true);
        let (dist, path) = g.shortest_path("C", "A").unwrap();
        assert!((dist - 8.0).abs() < 1e-9);
        assert_eq!(path, vec!["C", "B", "A"]);
    }
}
