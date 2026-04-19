-- Online White Board — shared canvas of strokes with spatial queries.
-- Pure-functional: every operation returns a new Whiteboard.

namespace Web.WB

structure Point where
  x : Float
  y : Float
  deriving Repr

def Point.distance (a b : Point) : Float :=
  let dx := a.x - b.x
  let dy := a.y - b.y
  (dx * dx + dy * dy).sqrt

abbrev StrokeId := Nat

structure Stroke where
  id : StrokeId
  author : String
  color : String
  thickness : Float
  points : List Point
  deriving Repr

def Stroke.bbox (s : Stroke) : Option (Point × Point) :=
  match s.points with
  | [] => none
  | p :: rest =>
    let folded := rest.foldl (init := (p, p)) fun (lo, hi) q =>
      let lo' : Point := { x := min lo.x q.x, y := min lo.y q.y }
      let hi' : Point := { x := max hi.x q.x, y := max hi.y q.y }
      (lo', hi')
    some folded

def Stroke.distanceTo (s : Stroke) (p : Point) : Float :=
  s.points.foldl (init := Float.inf) fun best q =>
    let d := q.distance p
    if d < best then d else best

structure Whiteboard where
  strokes : List Stroke := []
  nextId : StrokeId := 1
  deriving Repr

def Whiteboard.empty : Whiteboard := {}

def Whiteboard.addStroke (b : Whiteboard) (author color : String)
    (thickness : Float) (points : List Point) : Whiteboard × StrokeId :=
  let id := b.nextId
  let s : Stroke := { id, author, color, thickness, points }
  ({ b with strokes := b.strokes ++ [s], nextId := id + 1 }, id)

def Whiteboard.removeStroke (b : Whiteboard) (id : StrokeId)
    : Whiteboard × Bool :=
  if b.strokes.any (·.id == id) then
    ({ b with strokes := b.strokes.filter (·.id ≠ id) }, true)
  else (b, false)

def Whiteboard.clear (b : Whiteboard) : Whiteboard :=
  { b with strokes := [] }

def Whiteboard.strokesByAuthor (b : Whiteboard) (author : String) : List Stroke :=
  b.strokes.filter (·.author == author)

def Whiteboard.strokesNear (b : Whiteboard) (p : Point) (radius : Float)
    : List Stroke :=
  b.strokes.filter (·.distanceTo p ≤ radius)

def Whiteboard.boundingBox (b : Whiteboard) : Option (Point × Point) :=
  let boxes := b.strokes.filterMap (·.bbox)
  match boxes with
  | [] => none
  | (lo, hi) :: rest =>
    let folded := rest.foldl (init := (lo, hi)) fun (accLo, accHi) (l, h) =>
      let newLo : Point := { x := min accLo.x l.x, y := min accLo.y l.y }
      let newHi : Point := { x := max accHi.x h.x, y := max accHi.y h.y }
      (newLo, newHi)
    some folded

end Web.WB
