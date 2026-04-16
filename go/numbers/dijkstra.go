package numbers

// Dijkstra's Algorithm — shortest paths in weighted graphs.

import (
	"container/heap"
	"math"
)

// DijkstraEdge represents a weighted edge to a destination node.
type DijkstraEdge struct {
	To     string
	Weight float64
}

// DijkstraGraph is a weighted directed graph stored as an adjacency list.
type DijkstraGraph struct {
	adj map[string][]DijkstraEdge
}

// NewDijkstraGraph creates an empty graph.
func NewDijkstraGraph() *DijkstraGraph {
	return &DijkstraGraph{adj: make(map[string][]DijkstraEdge)}
}

// AddEdge adds a weighted edge from u to v. If undirected, adds both directions.
func (g *DijkstraGraph) AddEdge(u, v string, weight float64, undirected bool) {
	g.adj[u] = append(g.adj[u], DijkstraEdge{To: v, Weight: weight})
	if _, ok := g.adj[v]; !ok {
		g.adj[v] = nil
	}
	if undirected {
		g.adj[v] = append(g.adj[v], DijkstraEdge{To: u, Weight: weight})
	}
}

// ShortestDistances returns the shortest distance from start to every reachable node.
func (g *DijkstraGraph) ShortestDistances(start string) map[string]float64 {
	dist := make(map[string]float64, len(g.adj))
	for node := range g.adj {
		dist[node] = math.Inf(1)
	}
	dist[start] = 0

	pq := &dijkstraPQ{{cost: 0, node: start}}
	heap.Init(pq)

	for pq.Len() > 0 {
		cur := heap.Pop(pq).(dijkstraItem)
		if cur.cost > dist[cur.node] {
			continue
		}
		for _, e := range g.adj[cur.node] {
			nd := cur.cost + e.Weight
			if nd < dist[e.To] {
				dist[e.To] = nd
				heap.Push(pq, dijkstraItem{cost: nd, node: e.To})
			}
		}
	}

	return dist
}

// DijkstraResult holds the shortest path distance and the sequence of nodes.
type DijkstraResult struct {
	Distance float64
	Path     []string
}

// ShortestPath returns the shortest path from start to end.
// Returns nil if no path exists.
func (g *DijkstraGraph) ShortestPath(start, end string) *DijkstraResult {
	dist := make(map[string]float64, len(g.adj))
	prev := make(map[string]string, len(g.adj))
	for node := range g.adj {
		dist[node] = math.Inf(1)
	}
	dist[start] = 0

	pq := &dijkstraPQ{{cost: 0, node: start}}
	heap.Init(pq)

	for pq.Len() > 0 {
		cur := heap.Pop(pq).(dijkstraItem)
		if cur.cost > dist[cur.node] {
			continue
		}
		if cur.node == end {
			break
		}
		for _, e := range g.adj[cur.node] {
			nd := cur.cost + e.Weight
			if nd < dist[e.To] {
				dist[e.To] = nd
				prev[e.To] = cur.node
				heap.Push(pq, dijkstraItem{cost: nd, node: e.To})
			}
		}
	}

	d, ok := dist[end]
	if !ok || math.IsInf(d, 1) {
		return nil
	}

	// Reconstruct path.
	var path []string
	for node := end; node != ""; node = prev[node] {
		path = append([]string{node}, path...)
		if node == start {
			break
		}
	}

	return &DijkstraResult{Distance: d, Path: path}
}

// --- priority queue for container/heap ---

type dijkstraItem struct {
	cost float64
	node string
}

type dijkstraPQ []dijkstraItem

func (pq dijkstraPQ) Len() int            { return len(pq) }
func (pq dijkstraPQ) Less(i, j int) bool   { return pq[i].cost < pq[j].cost }
func (pq dijkstraPQ) Swap(i, j int)        { pq[i], pq[j] = pq[j], pq[i] }
func (pq *dijkstraPQ) Push(x interface{})  { *pq = append(*pq, x.(dijkstraItem)) }
func (pq *dijkstraPQ) Pop() interface{} {
	old := *pq
	n := len(old)
	item := old[n-1]
	*pq = old[:n-1]
	return item
}
