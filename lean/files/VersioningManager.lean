-- Versioning Manager — content-addressed version store, minimal Git-like.

namespace Files.VM

def fnv1aHash (data : ByteArray) : UInt64 :=
  let h := data.foldl (init := 0xcbf29ce484222325) fun acc b =>
    (acc ^^^ b.toUInt64) * 0x100000001b3
  h

structure Blob where
  data : ByteArray
  deriving Repr

structure Tree where
  entries : List (String × UInt64) := []  -- sorted by name
  deriving Repr

structure Commit where
  treeHash : UInt64
  parent : Option UInt64
  message : String
  timestampMs : Nat
  deriving Repr

inductive DiffKind where | added | removed | changed
  deriving Repr, BEq

structure DiffOp where
  kind : DiffKind
  name : String
  fromHash : UInt64 := 0
  toHash : UInt64 := 0
  deriving Repr

structure Repo where
  blobs : List (UInt64 × Blob) := []
  trees : List (UInt64 × Tree) := []
  commits : List (UInt64 × Commit) := []
  head : Option UInt64 := none

def stringToBytes (s : String) : ByteArray :=
  s.toUTF8

def Repo.putBlob (r : Repo) (data : ByteArray) : (Repo × UInt64) :=
  let h := fnv1aHash data
  if (r.blobs.find? (fun p => p.1 == h)).isSome then (r, h)
  else ({ r with blobs := r.blobs ++ [(h, { data })] }, h)

def Repo.putTree (r : Repo) (entries : List (String × UInt64)) : (Repo × UInt64) :=
  let sorted := entries.mergeSort (fun a b => a.1 < b.1)
  let tree : Tree := { entries := sorted }
  let canonical := stringToBytes <|
    "tree\n" ++ String.join (sorted.map fun (n, h) => s!"{n}\t{h.toNat}\n")
  let h := fnv1aHash canonical
  if (r.trees.find? (fun p => p.1 == h)).isSome then (r, h)
  else ({ r with trees := r.trees ++ [(h, tree)] }, h)

def Repo.commit (r : Repo) (files : List (String × ByteArray)) (message : String)
    (timestamp : Nat) : (Repo × UInt64) :=
  let (r1, entries) := files.foldl (init := (r, ([] : List (String × UInt64)))) fun (r, acc) (name, data) =>
    let (r', h) := r.putBlob data
    (r', acc ++ [(name, h)])
  let (r2, treeHash) := r1.putTree entries
  let c : Commit := { treeHash, parent := r.head, message, timestampMs := timestamp }
  let canonicalStr :=
    "commit\ntree " ++ toString c.treeHash ++ "\n" ++
    (match c.parent with | some p => "parent " ++ toString p ++ "\n" | none => "") ++
    s!"ts {timestamp}\nmsg {message}\n"
  let h := fnv1aHash (stringToBytes canonicalStr)
  ({ r2 with commits := r2.commits ++ [(h, c)], head := some h }, h)

def Repo.checkout (r : Repo) (commitHash : UInt64) : Option (List (String × ByteArray)) := do
  let (_, c) ← r.commits.find? (fun p => p.1 == commitHash)
  let (_, tree) ← r.trees.find? (fun p => p.1 == c.treeHash)
  let mut out : List (String × ByteArray) := []
  for (name, bh) in tree.entries do
    let (_, blob) ← r.blobs.find? (fun p => p.1 == bh)
    out := out ++ [(name, blob.data)]
  return out

def Repo.diff (r : Repo) (from' to' : UInt64) : Option (List DiffOp) := do
  let (_, fromC) ← r.commits.find? (fun p => p.1 == from')
  let (_, toC) ← r.commits.find? (fun p => p.1 == to')
  let (_, fromTree) ← r.trees.find? (fun p => p.1 == fromC.treeHash)
  let (_, toTree) ← r.trees.find? (fun p => p.1 == toC.treeHash)
  let mut ops : List DiffOp := []
  for (name, th) in toTree.entries do
    match fromTree.entries.find? (fun p => p.1 == name) with
    | none => ops := ops ++ [{ kind := .added, name, toHash := th }]
    | some (_, fh) =>
      if fh != th then
        ops := ops ++ [{ kind := .changed, name, fromHash := fh, toHash := th }]
  for (name, fh) in fromTree.entries do
    if (toTree.entries.find? (fun p => p.1 == name)).isNone then
      ops := ops ++ [{ kind := .removed, name, fromHash := fh }]
  return ops.mergeSort (fun a b =>
    let ka := (match a.kind with | .added => "A" | .removed => "R" | .changed => "C") ++ a.name
    let kb := (match b.kind with | .added => "A" | .removed => "R" | .changed => "C") ++ b.name
    ka < kb)

partial def Repo.log (r : Repo) : List (UInt64 × Commit) :=
  let rec walk (cursor : Option UInt64) (acc : List (UInt64 × Commit)) : List (UInt64 × Commit) :=
    match cursor with
    | none => acc
    | some h =>
      match r.commits.find? (fun p => p.1 == h) with
      | some (_, c) => walk c.parent (acc ++ [(h, c)])
      | none => acc
  walk r.head []

end Files.VM
