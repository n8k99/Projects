-- File Explorer — navigable virtual filesystem with path resolution.

namespace Files.FE

inductive Kind where | dir | file
  deriving Repr, DecidableEq, BEq

structure Node where
  kind : Kind
  size : Nat := 0
  mtime : Nat := 0
  children : List (String × Node) := []
  deriving Repr

def Node.dir : Node := { kind := .dir }
def Node.file (size : Nat) (mtime : Nat) : Node := { kind := .file, size, mtime }

structure Listing where
  name : String
  kind : Kind
  size : Nat
  mtime : Nat
  deriving Repr

structure Explorer where
  root : Node := Node.dir
  cwd : List String := []

def Explorer.cwdString (e : Explorer) : String :=
  "/" ++ "/".intercalate e.cwd

/-- Resolve a path to a canonical segment list. None for empty input.
    Root escape is prevented — `..` at root is a no-op. -/
def resolve (e : Explorer) (path : String) : Option (List String) :=
  if path.isEmpty then none
  else
    let segsStart := if path.get 0 == '/' then [] else e.cwd
    let parts := path.splitOn "/"
    let final := parts.foldl (init := segsStart) fun acc seg =>
      if seg == "" || seg == "." then acc
      else if seg == ".." then acc.dropLast
      else acc ++ [seg]
    some final

/-- Walk to a node given segments, or none. -/
partial def nodeAt (root : Node) (segs : List String) : Option Node :=
  match segs with
  | [] => some root
  | s :: rest =>
    match root.kind with
    | .dir =>
      match root.children.find? (fun (n, _) => n == s) with
      | some (_, child) => nodeAt child rest
      | none => none
    | _ => none

partial def sizeOf (n : Node) : Nat :=
  match n.kind with
  | .file => n.size
  | .dir => n.children.foldl (init := 0) fun acc (_, c) => acc + sizeOf c

/-- Listing for a directory's direct children — dirs first, then alphabetical. -/
def listDir (node : Node) : List Listing :=
  let children := node.children.map fun (name, c) =>
    { name, kind := c.kind,
      size := if c.kind == .file then c.size else 0,
      mtime := if c.kind == .file then c.mtime else 0 : Listing }
  children.mergeSort fun a b =>
    if a.kind != b.kind then a.kind == .dir
    else a.name < b.name

end Files.FE
