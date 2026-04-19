-- Chart Making — a visualization pipeline as composed functions.
--
-- data → scale → encode → layout → render. Each stage is independently testable.

namespace Classes

structure DataPoint where
  label : String
  value : Float
  series : String := ""
  deriving Repr

/-- A function from a numeric domain to a numeric range. -/
structure LinearScale where
  domainMin : Float
  domainMax : Float
  rangeMin : Float
  rangeMax : Float
  deriving Repr

/-- Evaluate the scale at `x`. Degenerate domains return the range midpoint. -/
def LinearScale.at (s : LinearScale) (x : Float) : Float :=
  if s.domainMax == s.domainMin then
    (s.rangeMin + s.rangeMax) / 2.0
  else
    let t := (x - s.domainMin) / (s.domainMax - s.domainMin)
    s.rangeMin + t * (s.rangeMax - s.rangeMin)

/-- Fit a scale to a list of values. -/
def LinearScale.fit (values : List Float) (rangeMin rangeMax : Float)
    (includeZero : Bool := true) : LinearScale :=
  match values with
  | [] => { domainMin := 0.0, domainMax := 1.0, rangeMin, rangeMax }
  | v :: rest =>
    let lo0 := rest.foldl min v
    let hi0 := rest.foldl max v
    let lo := if includeZero then min lo0 0.0 else lo0
    let hi := if includeZero then max hi0 0.0 else hi0
    let (lo, hi) := if lo == hi then (lo - 1.0, hi + 1.0) else (lo, hi)
    { domainMin := lo, domainMax := hi, rangeMin, rangeMax }

structure OrdinalScale where
  categories : List String
  rangeMin : Int
  rangeMax : Int
  deriving Repr

def OrdinalScale.at (s : OrdinalScale) (cat : String) : Option Int :=
  match s.categories.idxOf? cat with
  | none => none
  | some idx =>
    let n := s.categories.length
    if n ≤ 1 then some ((s.rangeMin + s.rangeMax) / 2)
    else
      let step := (s.rangeMax - s.rangeMin).toFloat / (n - 1).toFloat
      some (s.rangeMin + (idx.toFloat * step).toInt64.toInt)

structure BarChart where
  title : String
  points : List DataPoint
  width : Nat := 40
  deriving Repr

private def repeatChar (c : Char) (n : Nat) : String :=
  (List.replicate n c).asString

private def maxLabelWidth (points : List DataPoint) : Nat :=
  points.foldl (fun acc p => max acc p.label.length) 0

/-- Render the bar chart as text. -/
def BarChart.render (c : BarChart) : String :=
  if c.points.isEmpty then
    s!"{c.title}\n(no data)"
  else
    let values := c.points.map (·.value)
    let scale := LinearScale.fit values 0.0 c.width.toFloat true
    let labelW := maxLabelWidth c.points
    let sepLen := labelW + 4 + c.width + 10
    let header := s!"{c.title}\n{repeatChar '=' sepLen}"
    let rows := c.points.map fun p =>
      let barLen := (scale.at p.value).round.toUInt64.toNat
      let bar := repeatChar '█' (min barLen c.width)
      let pad := repeatChar ' ' (labelW - p.label.length)
      let barPad := repeatChar ' ' (c.width - min barLen c.width)
      s!"  {p.label}{pad}  {bar}{barPad}  {p.value}"
    header ++ "\n" ++ "\n".intercalate rows

structure LineChart where
  title : String
  points : List DataPoint
  width : Nat := 40
  height : Nat := 12
  deriving Repr

/-- A mutable grid represented as Array (Array Char). -/
private def emptyGrid (h w : Nat) : Array (Array Char) :=
  Array.mkArray h (Array.mkArray w ' ')

private def gridSet (g : Array (Array Char)) (r c : Nat) (ch : Char)
    : Array (Array Char) :=
  if r ≥ g.size then g
  else
    let row := g.get! r
    if c ≥ row.size then g
    else g.set! r (row.set! c ch)

private def gridGet (g : Array (Array Char)) (r c : Nat) : Char :=
  if r ≥ g.size then ' '
  else
    let row := g.get! r
    if c ≥ row.size then ' ' else row.get! c

/-- Render the line chart as text. -/
def LineChart.render (c : LineChart) : String :=
  if c.points.isEmpty then
    s!"{c.title}\n(no data)"
  else
    let n := c.points.length
    let values := c.points.map (·.value)
    let xScale : LinearScale := {
      domainMin := 0.0,
      domainMax := max 1.0 ((n - 1).toFloat),
      rangeMin := 0.0,
      rangeMax := (c.width - 1).toFloat
    }
    let yScale := LinearScale.fit values 0.0 (c.height - 1).toFloat false

    -- Plot points only (skip interpolation to keep Lean code tight).
    let g0 := emptyGrid c.height c.width
    let grid := c.points.zipIdx.foldl (fun acc (p, i) =>
      let col := (xScale.at i.toFloat).round.toUInt64.toNat
      let row := c.height - 1 - (yScale.at p.value).round.toUInt64.toNat
      gridSet acc row col '●') g0

    let rows := (List.range c.height).map fun r =>
      let left :=
        if r == 0 then s!"{yScale.domainMax} ┤"
        else if r == c.height - 1 then s!"{yScale.domainMin} ┤"
        else "        │"
      let body := (List.range c.width).foldl (fun acc col =>
        acc.push (gridGet grid r col)) ""
      left ++ body
    let axis := "        └" ++ repeatChar '─' c.width
    s!"{c.title}\n" ++ "\n".intercalate rows ++ "\n" ++ axis

end Classes
