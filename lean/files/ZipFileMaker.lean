-- Zip File Maker — archive multiple named files with per-entry RLE compression.

namespace Files.ZFM

inductive Method where | store | rle
  deriving Repr, BEq

structure Entry where
  name : String
  method : Method
  sizeOriginal : Nat
  data : ByteArray
  deriving Repr

structure Archive where
  entries : List Entry := []
  deriving Repr

partial def rleEncode (data : ByteArray) : ByteArray :=
  let n := data.size
  let rec loop (i : Nat) (out : ByteArray) : ByteArray :=
    if i ≥ n then out
    else
      let b := data[i]!
      let rec extend (run : Nat) : Nat :=
        if i + run < n ∧ data[i + run]! == b ∧ run < 255 then extend (run + 1)
        else run
      let run := extend 1
      let out := out.push run.toUInt8 |>.push b
      loop (i + run) out
  loop 0 ByteArray.empty

partial def rleDecode (encoded : ByteArray) : ByteArray :=
  let n := encoded.size
  let rec loop (i : Nat) (out : ByteArray) : ByteArray :=
    if i + 1 ≥ n then out
    else
      let count := encoded[i]!.toNat
      let b := encoded[i+1]!
      let rec emit (k : Nat) (o : ByteArray) : ByteArray :=
        if k == 0 then o else emit (k - 1) (o.push b)
      loop (i + 2) (emit count out)
  loop 0 ByteArray.empty

def Archive.addFile (a : Archive) (name : String) (data : ByteArray) : Archive :=
  let rle := rleEncode data
  let useRle := rle.size < data.size
  let entry : Entry :=
    { name,
      method := if useRle then .rle else .store,
      sizeOriginal := data.size,
      data := if useRle then rle else data }
  { a with entries := a.entries ++ [entry] }

def Archive.addStored (a : Archive) (name : String) (data : ByteArray) : Archive :=
  { a with entries := a.entries ++ [{
      name, method := .store, sizeOriginal := data.size, data := data }] }

def Archive.extract (a : Archive) (name : String) : Option ByteArray :=
  match a.entries.find? (fun e => e.name == name) with
  | some e => some (match e.method with
                     | .store => e.data
                     | .rle => rleDecode e.data)
  | none => none

def Archive.names (a : Archive) : List String :=
  a.entries.map Entry.name

def Archive.compressedSize (a : Archive) : Nat :=
  a.entries.foldl (init := 0) fun acc e => acc + e.data.size

def Archive.compressionRatio (a : Archive) : Float :=
  let orig := a.entries.foldl (init := 0) fun acc e => acc + e.sizeOriginal
  if orig == 0 then 1.0
  else a.compressedSize.toFloat / orig.toFloat

end Files.ZFM
