-- Shape Area and Perimeter — one type, many constructors.
--
-- Lean's most idiomatic polymorphism for a FIXED set of alternatives is an
-- inductive sum type. The `Shape` type enumerates every shape; functions
-- pattern-match on the constructor. This is **closed polymorphism**: adding
-- a new shape requires extending the inductive type and every dispatching
-- function. The compiler enforces exhaustivity — you CANNOT forget a case.

namespace Classes

inductive Shape where
  | circle (radius : Float)
  | rectangle (width height : Float)
  | triangle (a b c : Float)
  | regularPolygon (sides : Nat) (sideLength : Float)
  deriving Repr

namespace Shape

private def pi : Float := 3.14159265358979323846

/-- Smart constructor with triangle-inequality validation. -/
def mkTriangle (a b c : Float) : Except String Shape :=
  if a ≤ 0 || b ≤ 0 || c ≤ 0 then
    .error s!"non-positive side: {a} {b} {c}"
  else if ¬ (a + b > c && a + c > b && b + c > a) then
    .error s!"triangle inequality violated: {a} {b} {c}"
  else
    .ok <| Shape.triangle a b c

/-- Smart constructor for regular polygons. -/
def mkRegularPolygon (sides : Nat) (sideLength : Float) : Except String Shape :=
  if sides < 3 then
    .error s!"polygon needs >= 3 sides, got {sides}"
  else if sideLength < 0 then
    .error s!"negative side: {sideLength}"
  else
    .ok <| Shape.regularPolygon sides sideLength

def mkCircle (radius : Float) : Except String Shape :=
  if radius < 0 then .error s!"negative radius: {radius}"
  else .ok (Shape.circle radius)

def mkRectangle (w h : Float) : Except String Shape :=
  if w < 0 || h < 0 then .error s!"negative dimension: {w}x{h}"
  else .ok (Shape.rectangle w h)

/-- Area of a shape. Exhaustive pattern match — compiler refuses to compile
    if a new `Shape` constructor is added without a case here. That's the
    power of closed polymorphism. -/
def area : Shape → Float
  | .circle r => pi * r * r
  | .rectangle w h => w * h
  | .triangle a b c =>
    let s := (a + b + c) / 2.0
    Float.sqrt (s * (s - a) * (s - b) * (s - c))
  | .regularPolygon n s =>
    let nf := n.toFloat
    nf * s * s / (4.0 * Float.tan (pi / nf))

def perimeter : Shape → Float
  | .circle r => 2.0 * pi * r
  | .rectangle w h => 2.0 * (w + h)
  | .triangle a b c => a + b + c
  | .regularPolygon n s => n.toFloat * s

def name : Shape → String
  | .circle _ => "Circle"
  | .rectangle w h => if w == h then "Square" else "Rectangle"
  | .triangle _ _ _ => "Triangle"
  | .regularPolygon n _ => s!"Regular-{n}-gon"

end Shape

-- --- heterogeneous collection operations ---

def totalArea (shapes : List Shape) : Float :=
  shapes.foldl (fun acc s => acc + s.area) 0.0

def totalPerimeter (shapes : List Shape) : Float :=
  shapes.foldl (fun acc s => acc + s.perimeter) 0.0

def largestByArea : List Shape → Option Shape
  | [] => none
  | s :: ss => some <| ss.foldl
      (fun best x => if x.area > best.area then x else best) s

def sortByPerimeter (shapes : List Shape) : List Shape :=
  shapes.mergeSort (fun a b => a.perimeter ≤ b.perimeter)

end Classes
