-- Image Map Generator — clickable regions on an image with hit testing.

namespace Files.IM

inductive Shape where
  | rect (x y w h : Int)
  | circle (cx cy r : Int)
  | polygon (points : List (Int × Int))
  deriving Repr

partial def pointInPolygon (x y : Int) (points : List (Int × Int)) : Bool :=
  let n := points.length
  if n < 3 then false
  else
    let rec go (i : Nat) (inside : Bool) : Bool :=
      if i ≥ n then inside
      else
        let j := if i = 0 then n - 1 else i - 1
        let (xi, yi) := points[i]!
        let (xj, yj) := points[j]!
        let crosses := (yi > y) != (yj > y)
        let newInside :=
          if crosses then
            let xIntersect := (xj - xi) * (y - yi) / (yj - yi) + xi
            if x < xIntersect then !inside else inside
          else inside
        go (i + 1) newInside
    go 0 false

def Shape.contains : Shape → Int → Int → Bool
  | .rect rx ry w h, x, y =>
    x ≥ rx ∧ x < rx + w ∧ y ≥ ry ∧ y < ry + h
  | .circle cx cy r, x, y =>
    let dx := x - cx
    let dy := y - cy
    dx * dx + dy * dy ≤ r * r
  | .polygon points, x, y => pointInPolygon x y points

structure Region where
  id : Nat
  shape : Shape
  action : String
  alt : String
  z : Int := 0
  deriving Repr

structure ImageMap where
  regions : List Region := []
  deriving Repr

def ImageMap.add (m : ImageMap) (r : Region) : ImageMap :=
  { m with regions := m.regions ++ [r] }

def ImageMap.hitTest (m : ImageMap) (x y : Int) : Option Region :=
  let sorted := m.regions.mergeSort (fun a b => a.z > b.z)
  sorted.find? (fun r => r.shape.contains x y)

def ImageMap.hitTestAll (m : ImageMap) (x y : Int) : List Region :=
  let sorted := m.regions.mergeSort (fun a b => a.z > b.z)
  sorted.filter (fun r => r.shape.contains x y)

def Shape.toHtmlParts : Shape → (String × String)
  | .rect x y w h => ("rect", s!"{x},{y},{x + w},{y + h}")
  | .circle cx cy r => ("circle", s!"{cx},{cy},{r}")
  | .polygon points =>
    let coords := ",".intercalate (points.flatMap (fun (x, y) => [toString x, toString y]))
    ("poly", coords)

def ImageMap.toHtml (m : ImageMap) (name : String) : String :=
  let opener := s!"<map name=\"{name}\">\n"
  let areas := m.regions.foldl (init := "") fun acc r =>
    let (sname, coords) := r.shape.toHtmlParts
    acc ++ s!"  <area shape=\"{sname}\" coords=\"{coords}\" href=\"{r.action}\" alt=\"{r.alt}\">\n"
  opener ++ areas ++ "</map>\n"

end Files.IM
