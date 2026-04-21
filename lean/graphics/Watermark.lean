-- Watermarking Application — LSB steganography + visible overlay + position ADT.

import Graphics.Grayscale

namespace Graphics.Watermark

open Graphics.Gray

inductive Position where
  | topLeft | topRight | bottomLeft | bottomRight | center | tiled
  deriving Repr, BEq

inductive WatermarkError where
  | payloadTooLarge | invalidBitsPerChannel | truncatedData
  deriving Repr, BEq

def lsbCapacity (image : Image) (bitsPerChannel : Nat) : Nat :=
  if bitsPerChannel == 0 ∨ bitsPerChannel > 4 then 0
  else
    let totalBits := image.pixels.length * 3 * bitsPerChannel
    let totalBytes := totalBits / 8
    if totalBytes < 4 then 0 else totalBytes - 4

def byteToBits (b : Nat) : List Nat :=
  (List.range 8).map fun i => (b >>> (7 - i)) &&& 1

def bitsToByte (bits : List Nat) : Nat :=
  bits.foldl (fun acc b => (acc <<< 1) ||| b) 0

def embedLsb (image : Image) (payload : List Nat) (bitsPerChannel : Nat)
    : Except WatermarkError Image :=
  if bitsPerChannel == 0 ∨ bitsPerChannel > 4 then .error .invalidBitsPerChannel
  else if payload.length > lsbCapacity image bitsPerChannel then .error .payloadTooLarge
  else
    let n := payload.length
    let lenBytes : List Nat := [(n >>> 24) &&& 0xff, (n >>> 16) &&& 0xff,
                                  (n >>> 8) &&& 0xff, n &&& 0xff]
    let bits : List Nat := (lenBytes ++ payload).flatMap byteToBits
    let mask := (1 <<< bitsPerChannel) - 1  -- low bits mask
    let invMask := 255 - mask
    let bitsArr := bits.toArray
    let totalBits := bitsArr.size
    let pixels := image.pixels.toArray
    let consumeChunk (bitIdx : Nat) : Nat × Nat :=
      let mut chunk : Nat := 0
      let mut idx := bitIdx
      for _ in [0:bitsPerChannel] do
        if idx < totalBits then
          chunk := (chunk <<< 1) ||| bitsArr[idx]!
          idx := idx + 1
      (chunk, idx)
    let mut bitIdx := 0
    let mut newPixels : Array Rgb := #[]
    for p in pixels do
      if bitIdx ≥ totalBits then
        newPixels := newPixels.push p
      else
        let (rChunk, idx1) := consumeChunk bitIdx
        let newR := (p.r &&& invMask) ||| rChunk
        bitIdx := idx1
        let (newG, idx2) :=
          if bitIdx < totalBits then
            let (gChunk, idx) := consumeChunk bitIdx
            (((p.g &&& invMask) ||| gChunk), idx)
          else (p.g, bitIdx)
        bitIdx := idx2
        let (newB, idx3) :=
          if bitIdx < totalBits then
            let (bChunk, idx) := consumeChunk bitIdx
            (((p.b &&& invMask) ||| bChunk), idx)
          else (p.b, bitIdx)
        bitIdx := idx3
        newPixels := newPixels.push { r := newR, g := newG, b := newB }
    .ok { width := image.width, height := image.height, pixels := newPixels.toList }

def extractLsb (image : Image) (bitsPerChannel : Nat)
    : Except WatermarkError (List Nat) :=
  if bitsPerChannel == 0 ∨ bitsPerChannel > 4 then .error .invalidBitsPerChannel
  else
    let mask := (1 <<< bitsPerChannel) - 1
    let bits := image.pixels.foldl (init := #[]) fun acc p =>
      let collect (acc' : Array Nat) (ch : Nat) : Array Nat :=
        let chunk := ch &&& mask
        (List.range bitsPerChannel).foldl (init := acc') fun a i =>
          a.push ((chunk >>> (bitsPerChannel - 1 - i)) &&& 1)
      let acc1 := collect acc p.r
      let acc2 := collect acc1 p.g
      collect acc2 p.b
    let bitsList := bits.toList
    if bits.size < 32 then .error .truncatedData
    else
      let headerBits := (bitsList.take 32)
      let payloadLen := headerBits.foldl (fun acc b => (acc <<< 1) ||| b) 0
      let totalBitsNeeded := 32 + payloadLen * 8
      if bits.size < totalBitsNeeded then .error .truncatedData
      else
        let payloadBits := (bitsList.drop 32).take (payloadLen * 8)
        let result := (List.range payloadLen).map fun byteI =>
          let chunk := payloadBits.drop (byteI * 8) |>.take 8
          chunk.foldl (fun acc b => (acc <<< 1) ||| b) 0
        .ok result

def resolvePosition (base wm : Image) (pos : Position) : Int × Int :=
  let bw : Int := base.width
  let bh : Int := base.height
  let ww : Int := wm.width
  let wh : Int := wm.height
  match pos with
  | .topLeft => (0, 0)
  | .topRight => (max 0 (bw - ww), 0)
  | .bottomLeft => (0, max 0 (bh - wh))
  | .bottomRight => (max 0 (bw - ww), max 0 (bh - wh))
  | .center => ((bw - ww) / 2, (bh - wh) / 2)
  | .tiled => (0, 0)

end Graphics.Watermark
