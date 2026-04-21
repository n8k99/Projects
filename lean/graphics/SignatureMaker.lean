-- Signature Maker — stroke model + Catmull-Rom + normalize + similarity.

namespace Graphics.Signature

structure Point where
  x : Int
  y : Int
  pressure : Nat := 0
  tMs : Nat := 0
  deriving Repr, BEq

structure Stroke where
  points : List Point := []
  deriving Repr

structure Signature where
  strokes : List Stroke := []
  deriving Repr

def Signature.new : Signature := {}

def Signature.addStroke (sig : Signature) (s : Stroke) : Signature :=
  { sig with strokes := sig.strokes ++ [s] }

def Signature.pointCount (sig : Signature) : Nat :=
  sig.strokes.foldl (init := 0) (fun acc s => acc + s.points.length)

def Signature.boundingBox (sig : Signature) : Option (Int × Int × Int × Int) :=
  let points := sig.strokes.flatMap (·.points)
  match points with
  | [] => none
  | first :: rest =>
    let init := (first.x, first.y, first.x, first.y)
    let (minX, minY, maxX, maxY) := rest.foldl (init := init) fun (minX, minY, maxX, maxY) p =>
      (min minX p.x, min minY p.y, max maxX p.x, max maxY p.y)
    some (minX, minY, maxX, maxY)

def crCompute (a b c d tt t2 t3 : Int) : Int :=
  let termA := 2 * b
  let termB := (-a + c) * tt
  let termC := (2 * a - 5 * b + 4 * c - d) * t2
  let termD := (-a + 3 * b - 3 * c + d) * t3
  let sum := termA * 1000000 + termB * 1000 + termC + termD / 1000
  sum / 2000000

def catmullRom (p0 p1 p2 p3 : Int × Int) (tt : Int) : Int × Int :=
  let t2 := tt * tt
  let t3 := t2 * tt / 1000
  let x := crCompute p0.1 p1.1 p2.1 p3.1 tt t2 t3
  let y := crCompute p0.2 p1.2 p2.2 p3.2 tt t2 t3
  (x, y)

partial def smoothStroke (stroke : Stroke) (steps : Nat) : Stroke :=
  let pts := stroke.points
  if pts.length < 4 ∨ steps == 0 then stroke
  else
    let arr := pts.toArray
    let indices := List.range (pts.length - 3)
    let interpolated := indices.foldl (init := [pts.get! 0]) fun acc i =>
      let p0 := (arr.get! i).x |> (·, (arr.get! i).y)
      let p1 := ((arr.get! (i+1)).x, (arr.get! (i+1)).y)
      let p2 := ((arr.get! (i+2)).x, (arr.get! (i+2)).y)
      let p3 := ((arr.get! (i+3)).x, (arr.get! (i+3)).y)
      let steps' := List.range steps
      steps'.foldl (init := acc) fun acc2 s =>
        let tt := ((s + 1) * 1000 / steps).toInt
        let (x, y) := catmullRom p0 p1 p2 p3 tt
        let pp := arr.get! (i+1)
        let pn := arr.get! (i+2)
        let pressureDiff : Int := Int.ofNat pn.pressure - Int.ofNat pp.pressure
        let pressureRaw : Int := Int.ofNat pp.pressure + pressureDiff * tt / 1000
        let pressure : Nat := if pressureRaw < 0 then 0
                              else if pressureRaw > 1024 then 1024
                              else pressureRaw.toNat
        let tDiff : Int := Int.ofNat pn.tMs - Int.ofNat pp.tMs
        let tRaw : Int := Int.ofNat pp.tMs + tDiff * tt / 1000
        let tMs : Nat := if tRaw < 0 then 0 else tRaw.toNat
        acc2 ++ [{ x, y, pressure, tMs }]
    { points := interpolated ++ [pts.get! (pts.length - 1)] }

def normalize (sig : Signature) (targetW targetH : Int) : Signature :=
  match sig.boundingBox with
  | none => sig
  | some (minX, minY, maxX, maxY) =>
    let w := max 1 (maxX - minX)
    let h := max 1 (maxY - minY)
    let scaleX := targetW * 1000 / w
    let scaleY := targetH * 1000 / h
    { strokes := sig.strokes.map fun s =>
        { points := s.points.map fun p =>
            { x := (p.x - minX) * scaleX / 1000,
              y := (p.y - minY) * scaleY / 1000,
              pressure := p.pressure,
              tMs := p.tMs }
        }
    }

def similarityDistance (a b : Signature) : Nat :=
  let allB := b.strokes.flatMap (·.points)
  let allA := a.strokes.flatMap (·.points)
  allA.foldl (init := 0) fun acc p =>
    let minD2 := allB.foldl (init := (10 ^ 30 : Nat)) fun accD q =>
      let dx := p.x - q.x
      let dy := p.y - q.y
      let d2 := (dx * dx + dy * dy).natAbs
      min accD d2
    acc + minD2

end Graphics.Signature
