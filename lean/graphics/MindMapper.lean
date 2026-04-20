-- Mind Mapper — radial layout + SVG emission.

namespace Graphics.MM

structure MindNode where
  id : Int
  label : String
  color : String := "#3b82f6"
  collapsed : Bool := false
  children : List MindNode := []
  deriving Repr

partial def MindNode.find (n : MindNode) (id : Int) : Option MindNode :=
  if n.id == id then some n
  else n.children.findSome? (fun c => c.find id)

partial def MindNode.visibleCount (n : MindNode) : Nat :=
  if n.collapsed then 1
  else 1 + n.children.foldl (init := 0) (fun acc c => acc + c.visibleCount)

partial def MindNode.totalCount (n : MindNode) : Nat :=
  1 + n.children.foldl (init := 0) (fun acc c => acc + c.totalCount)

partial def MindNode.maxDepth (n : MindNode) : Nat :=
  if n.children.isEmpty ∨ n.collapsed then 0
  else 1 + (n.children.map MindNode.maxDepth).foldl Nat.max 0

structure LaidOutNode where
  id : Int
  label : String
  color : String
  depth : Nat
  x : Float
  y : Float
  parentId : Option Int := none
  collapsed : Bool := false
  deriving Repr

partial def layoutChildren (parent : MindNode)
    (arcStart arcEnd : Float) (depth : Nat)
    (radiusStep cx cy : Float) : List LaidOutNode :=
  let n := parent.children.length
  if n == 0 then []
  else
    let slice := (arcEnd - arcStart) / n.toFloat
    parent.children.mapIdx (fun i child =>
      let a0 := arcStart + slice * i.toFloat
      let a1 := arcStart + slice * (i + 1).toFloat
      let angle := (a0 + a1) / 2.0
      let r := depth.toFloat * radiusStep
      let x := cx + r * angle.cos
      let y := cy + r * angle.sin
      let selfNode : LaidOutNode :=
        { id := child.id, label := child.label, color := child.color,
          depth := depth, x := x, y := y,
          parentId := some parent.id, collapsed := child.collapsed }
      let subtree :=
        if child.collapsed then []
        else layoutChildren child a0 a1 (depth + 1) radiusStep cx cy
      selfNode :: subtree) |>.flatten

def radialLayout (root : MindNode) (cx cy radiusStep : Float) : List LaidOutNode :=
  let rootNode : LaidOutNode :=
    { id := root.id, label := root.label, color := root.color,
      depth := 0, x := cx, y := cy, parentId := none,
      collapsed := root.collapsed }
  if root.collapsed then [rootNode]
  else rootNode :: layoutChildren root 0.0 (2.0 * 3.14159265358979323846) 1 radiusStep cx cy

partial def traverseDFS (root : MindNode) : List Int :=
  if root.collapsed then [root.id]
  else root.id :: (root.children.bind traverseDFS)

partial def traverseBFS (root : MindNode) : List Int :=
  let rec bfs (queue : List MindNode) (acc : List Int) : List Int :=
    match queue with
    | [] => acc.reverse
    | n :: rest =>
      let acc' := n.id :: acc
      let queue' := if n.collapsed then rest else rest ++ n.children
      bfs queue' acc'
  bfs [root] []

def escapeXML (s : String) : String :=
  s.replace "&" "&amp;" |>.replace "<" "&lt;" |>.replace ">" "&gt;" |>.replace "\"" "&quot;"

end Graphics.MM
