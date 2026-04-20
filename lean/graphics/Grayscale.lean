-- Grayscale Converter — pixel buffers + named conversion algorithms.

namespace Graphics.Gray

structure Rgb where
  r : Nat
  g : Nat
  b : Nat
  deriving Repr, BEq

structure Image where
  width : Nat
  height : Nat
  pixels : List Rgb := []
  deriving Repr

structure GrayImage where
  width : Nat
  height : Nat
  pixels : List Nat := []
  deriving Repr

inductive Algorithm where
  | luminosity | average | lightness | red | green | blue
  deriving Repr, BEq

def pixelToGray (p : Rgb) : Algorithm → Nat
  | .luminosity =>
    let v := 0.2126 * p.r.toFloat + 0.7152 * p.g.toFloat + 0.0722 * p.b.toFloat
    let rounded := (v + 0.5).toUInt32.toNat
    if rounded > 255 then 255 else rounded
  | .average => (p.r + p.g + p.b) / 3
  | .lightness =>
    let mx := max p.r (max p.g p.b)
    let mn := min p.r (min p.g p.b)
    (mx + mn) / 2
  | .red => p.r
  | .green => p.g
  | .blue => p.b

def convert (img : Image) (algo : Algorithm) : GrayImage :=
  { width := img.width, height := img.height,
    pixels := img.pixels.map (fun p => pixelToGray p algo) }

def histogram (g : GrayImage) : List Nat :=
  let bins := Array.mkArray 256 (0 : Nat)
  (g.pixels.foldl (init := bins) fun acc v =>
    acc.set! v (acc[v]! + 1)).toList

def meanBrightness (g : GrayImage) : Float :=
  if g.pixels.isEmpty then 0.0
  else
    let sum := (g.pixels.foldl (init := (0 : Nat)) (· + ·)).toFloat
    sum / g.pixels.length.toFloat

def contrastStddev (g : GrayImage) : Float :=
  if g.pixels.isEmpty then 0.0
  else
    let m := meanBrightness g
    let var := (g.pixels.foldl (init := (0.0 : Float)) fun acc v =>
                  acc + ((v.toFloat - m) * (v.toFloat - m))) / g.pixels.length.toFloat
    var.sqrt

def dynamicRange (g : GrayImage) : Option (Nat × Nat) :=
  match g.pixels with
  | [] => none
  | first :: rest =>
    let (mn, mx) := rest.foldl (init := (first, first)) fun (mn, mx) v =>
      (min mn v, max mx v)
    some (mn, mx)

end Graphics.Gray
