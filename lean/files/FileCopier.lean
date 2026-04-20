-- File Copy Utility — recursive tree copy with policy + filter + progress events.

namespace Files.FC

inductive Item where
  | file (size : Nat)
  | dir
  deriving Repr, BEq

inductive Overwrite where | skip | overwrite | renameWithSuffix
  deriving Repr, BEq

structure Filter where
  include : List String := []
  exclude : List String := []
  deriving Repr

def Filter.allowAll : Filter := {}

def Filter.accepts (f : Filter) (path : String) : Bool :=
  let includeOk := f.include.isEmpty ∨
    f.include.any (fun pat => (path.splitOn pat).length > 1)
  let excludeOk := !f.exclude.any (fun pat => (path.splitOn pat).length > 1)
  includeOk ∧ excludeOk

inductive EventKind where | copied | skipped | renamed | createdDir
  deriving Repr, BEq

structure CopyEvent where
  kind : EventKind
  source : String
  dest : String
  bytes : Nat := 0
  deriving Repr

structure Fs where
  items : List (String × Item) := []

def Fs.paths (fs : Fs) : List String :=
  (fs.items.map Prod.fst).mergeSort (· < ·)

def Fs.get (fs : Fs) (path : String) : Option Item :=
  (fs.items.find? (fun p => p.1 == path)).map Prod.snd

def Fs.set (fs : Fs) (path : String) (item : Item) : Fs :=
  let filtered := fs.items.filter (fun p => p.1 ≠ path)
  { items := filtered ++ [(path, item)] }

partial def nextAvailable (fs : Fs) (base : String) : String :=
  let rec loop (n : Nat) : String :=
    let candidate := s!"{base}.{n}"
    if fs.get candidate |>.isSome then loop (n + 1)
    else candidate
  loop 1

def Fs.copyTree (fs : Fs) (srcRoot destRoot : String)
    (policy : Overwrite) (filter : Filter)
    : Except String (Fs × List CopyEvent) := do
  let srcPaths := fs.items.filterMap fun (p, _) =>
    if p == srcRoot ∨ p.startsWith (srcRoot ++ "/") then some p else none
  if srcPaths.isEmpty then
    throw s!"source not found: {srcRoot}"
  let sortedPaths := srcPaths.mergeSort (· < ·)
  let mut current := fs
  let mut events : List CopyEvent := []
  for srcPath in sortedPaths do
    match current.get srcPath with
    | none => pure ()
    | some item =>
      if filter.accepts srcPath then
        let rel := if srcPath == srcRoot then "" else srcPath.drop (srcRoot.length + 1)
        let destPath := if rel == "" then destRoot else destRoot ++ "/" ++ rel
        let mut finalDest := destPath
        let mut skip := false
        if current.get destPath |>.isSome then
          match policy with
          | .skip =>
            events := events ++ [{ kind := .skipped, source := srcPath, dest := destPath }]
            skip := true
          | .renameWithSuffix =>
            let renamed := nextAvailable current destPath
            events := events ++ [{ kind := .renamed, source := srcPath, dest := renamed }]
            finalDest := renamed
          | _ => pure ()
        if !skip then
          match item with
          | .dir =>
            current := current.set finalDest .dir
            events := events ++ [{ kind := .createdDir, source := srcPath, dest := finalDest }]
          | .file size =>
            current := current.set finalDest (.file size)
            events := events ++ [{ kind := .copied, source := srcPath, dest := finalDest, bytes := size }]
  return (current, events)

def totalSizeCopied (events : List CopyEvent) : Nat :=
  events.foldl (init := 0) fun acc e =>
    if e.kind == .copied then acc + e.bytes else acc

end Files.FC
