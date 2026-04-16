/- Dijkstra's Algorithm — shortest paths in weighted graphs. -/

namespace Numbers

/-- A weighted directed graph as an adjacency list. -/
structure DGraph where
  adj : Std.HashMap String (List (String × Float)) := {}

/-- Create an empty graph. -/
def DGraph.empty : DGraph := { adj := {} }

/-- Add a weighted edge from u to v. If undirected, add both directions. -/
def DGraph.addEdge (g : DGraph) (u v : String) (weight : Float) (undirected : Bool := false) : DGraph :=
  let edges := g.adj.getD u []
  let g := { g with adj := g.adj.insert u ((v, weight) :: edges) }
  -- Ensure v exists.
  let g := if g.adj.contains v then g else { g with adj := g.adj.insert v [] }
  if undirected then
    let edges2 := g.adj.getD v []
    { g with adj := g.adj.insert v ((u, weight) :: edges2) }
  else
    g

/-- Priority queue entry. -/
private structure PQEntry where
  cost : Float
  node : String

/-- Insert into a sorted worklist (ascending by cost). -/
private def insertSorted (entry : PQEntry) : List PQEntry → List PQEntry
  | [] => [entry]
  | h :: t => if entry.cost ≤ h.cost then entry :: h :: t else h :: insertSorted entry t

/-- Compute shortest distances from start to all reachable nodes. -/
def shortestDistances (graph : DGraph) (start : String) : Std.HashMap String Float :=
  let init : Std.HashMap String Float := ({} : Std.HashMap String Float).insert start 0.0
  let pq : List PQEntry := [{ cost := 0.0, node := start }]
  go pq init
where
  go : List PQEntry → Std.HashMap String Float → Std.HashMap String Float
  | [], dist => dist
  | { cost, node } :: rest, dist =>
    let curDist := dist.getD node Float.inf
    if cost > curDist then go rest dist
    else
      let edges := graph.adj.getD node []
      let (pq', dist') := edges.foldl (fun (pq, d) (v, w) =>
        let nd := cost + w
        let vd := d.getD v Float.inf
        if nd < vd then
          (insertSorted { cost := nd, node := v } pq, d.insert v nd)
        else
          (pq, d)
      ) (rest, dist)
      go pq' dist'

/-- Compute shortest path from start to goal. Returns Option (distance, path). -/
def shortestPath (graph : DGraph) (start goal : String) : Option (Float × List String) :=
  let initDist : Std.HashMap String Float := ({} : Std.HashMap String Float).insert start 0.0
  let initPrev : Std.HashMap String String := {}
  let pq : List PQEntry := [{ cost := 0.0, node := start }]
  match go pq initDist initPrev with
  | none => none
  | some (d, prev) => some (d, reconstruct prev goal [])
where
  go : List PQEntry → Std.HashMap String Float → Std.HashMap String String
     → Option (Float × Std.HashMap String String)
  | [], dist, prev =>
    match dist.get? goal with
    | some d => if d == Float.inf then none else some (d, prev)
    | none => none
  | { cost, node } :: rest, dist, prev =>
    let curDist := dist.getD node Float.inf
    if cost > curDist then go rest dist prev
    else if node == goal then some (cost, prev)
    else
      let edges := graph.adj.getD node []
      let (pq', dist', prev') := edges.foldl (fun (pq, d, p) (v, w) =>
        let nd := cost + w
        let vd := d.getD v Float.inf
        if nd < vd then
          (insertSorted { cost := nd, node := v } pq, d.insert v nd, p.insert v node)
        else
          (pq, d, p)
      ) (rest, dist, prev)
      go pq' dist' prev'
  reconstruct (prev : Std.HashMap String String) (node : String) (acc : List String) : List String :=
    let acc := node :: acc
    match prev.get? node with
    | some p => reconstruct prev p acc
    | none => acc

end Numbers
