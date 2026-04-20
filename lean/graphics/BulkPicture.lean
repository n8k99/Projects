-- Bulk Picture Manipulator — transformation pipeline applied over a batch.

import Graphics.Grayscale

namespace Graphics.Bulk

open Graphics.Gray

inductive Op where
  | resize : Nat → Nat → Op
  | grayscale : Algorithm → Op
  | rotate90 | rotate180 | rotate270
  | flipH | flipV
  | crop : Nat → Nat → Nat → Nat → Op  -- x, y, w, h
  | brightness : Int → Op
  | contrast : Float → Op
  deriving Repr

inductive OpError where | cropOutOfBounds | zeroDimension
  deriving Repr, BEq

abbrev Pipeline := List Op

def clampU8 (v : Int) : Nat :=
  if v < 0 then 0 else if v > 255 then 255 else v.toNat

def adjBrightness (p : Rgb) (delta : Int) : Rgb :=
  { r := clampU8 (p.r.toInt + delta),
    g := clampU8 (p.g.toInt + delta),
    b := clampU8 (p.b.toInt + delta) }

def adjContrast (p : Rgb) (factor : Float) : Rgb :=
  let adj (v : Nat) : Nat :=
    let x := (v.toFloat - 128.0) * factor + 128.0
    let rounded := (x + 0.5).toInt64
    if rounded < 0 then 0
    else if rounded > 255 then 255
    else rounded.toNat
  { r := adj p.r, g := adj p.g, b := adj p.b }

def mapPixels (img : Image) (f : Rgb → Rgb) : Image :=
  { width := img.width, height := img.height, pixels := img.pixels.map f }

def getPixel (img : Image) (x y : Nat) : Rgb :=
  img.pixels.get! (y * img.width + x)

def grayscaleImg (img : Image) (algo : Algorithm) : Image :=
  mapPixels img (fun p =>
    let v := pixelToGray p algo
    { r := v, g := v, b := v })

def buildImage (w h : Nat) (cell : Nat → Nat → Rgb) : Image :=
  let rows := (List.range h).foldl (init := #[]) fun acc y =>
    (List.range w).foldl (init := acc) fun acc2 x => acc2.push (cell x y)
  { width := w, height := h, pixels := rows.toList }

def resizeNearest (img : Image) (nw nh : Nat) : Image :=
  buildImage nw nh fun x y =>
    let sx := (x * img.width) / nw
    let sy := (y * img.height) / nh
    let sx' := if sx ≥ img.width then img.width - 1 else sx
    let sy' := if sy ≥ img.height then img.height - 1 else sy
    getPixel img sx' sy'

def rotate90 (img : Image) : Image :=
  let w := img.width
  let h := img.height
  buildImage h w fun nx ny => getPixel img ny (h - 1 - nx)

def rotate180 (img : Image) : Image :=
  let w := img.width
  let h := img.height
  buildImage w h fun nx ny => getPixel img (w - 1 - nx) (h - 1 - ny)

def rotate270 (img : Image) : Image :=
  let w := img.width
  let h := img.height
  buildImage h w fun nx ny => getPixel img (w - 1 - ny) nx

def flipH (img : Image) : Image :=
  let w := img.width
  let h := img.height
  buildImage w h fun nx ny => getPixel img (w - 1 - nx) ny

def flipV (img : Image) : Image :=
  let w := img.width
  let h := img.height
  buildImage w h fun nx ny => getPixel img nx (h - 1 - ny)

def cropImg (img : Image) (x0 y0 w h : Nat) : Image :=
  buildImage w h fun nx ny => getPixel img (x0 + nx) (y0 + ny)

def applyOp (img : Image) (op : Op) : Except OpError Image :=
  match op with
  | .resize w h =>
    if w == 0 ∨ h == 0 then .error .zeroDimension
    else .ok (resizeNearest img w h)
  | .grayscale algo => .ok (grayscaleImg img algo)
  | .rotate90 => .ok (rotate90 img)
  | .rotate180 => .ok (rotate180 img)
  | .rotate270 => .ok (rotate270 img)
  | .flipH => .ok (flipH img)
  | .flipV => .ok (flipV img)
  | .crop x y w h =>
    if w == 0 ∨ h == 0 then .error .zeroDimension
    else if x + w > img.width ∨ y + h > img.height then .error .cropOutOfBounds
    else .ok (cropImg img x y w h)
  | .brightness d => .ok (mapPixels img (fun p => adjBrightness p d))
  | .contrast f => .ok (mapPixels img (fun p => adjContrast p f))

def applyPipeline (img : Image) (pipeline : Pipeline) : Except OpError Image :=
  pipeline.foldlM (init := img) applyOp

inductive BatchResult where
  | ok : Nat → Image → BatchResult
  | err : Nat → OpError → BatchResult
  deriving Repr

def runBatch (images : List Image) (pipeline : Pipeline) : List BatchResult :=
  (images.zipIdx).map fun ⟨img, i⟩ =>
    match applyPipeline img pipeline with
    | .ok out => .ok i out
    | .error e => .err i e

def dryRunDims (start : Nat × Nat) (pipeline : Pipeline)
    : Except OpError (List (Nat × Nat)) :=
  let step : (List (Nat × Nat) × (Nat × Nat)) → Op
             → Except OpError (List (Nat × Nat) × (Nat × Nat))
    := fun ⟨acc, cur⟩ op =>
    match op with
    | .resize w h =>
      if w == 0 ∨ h == 0 then .error .zeroDimension
      else .ok (acc ++ [(w, h)], (w, h))
    | .grayscale _ | .flipH | .flipV | .rotate180 | .brightness _ | .contrast _ =>
      .ok (acc ++ [cur], cur)
    | .rotate90 | .rotate270 => .ok (acc ++ [(cur.2, cur.1)], (cur.2, cur.1))
    | .crop x y w h =>
      if w == 0 ∨ h == 0 then .error .zeroDimension
      else if x + w > cur.1 ∨ y + h > cur.2 then .error .cropOutOfBounds
      else .ok (acc ++ [(w, h)], (w, h))
  match pipeline.foldlM (init := ([start], start)) step with
  | .ok ⟨acc, _⟩ => .ok acc
  | .error e => .error e

end Graphics.Bulk
