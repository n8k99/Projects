-- Bulk Renamer and Organizer — pattern-driven rename + preview + undo log.

namespace Files.BRN

inductive Rule where
  | replace (find : String) (replace : String)
  | prefix (prefix : String)
  | suffixBeforeExt (suffix : String)
  | sequence (template : String)
  deriving Repr

structure RenameOp where
  from' : String
  to' : String
  deriving Repr, BEq

inductive RenameError where
  | fromMissing (name : String)
  | toExists (name : String)
  | collision (msg : String)
  deriving Repr

structure BrnDir where
  entries : List String := []
  deriving Repr

def BrnDir.names (d : BrnDir) : List String :=
  d.entries.mergeSort (· < ·)

def String.replaceFirst (s : String) (find replaceStr : String) : String :=
  match s.splitOn find with
  | [] => s
  | [_] => s
  | first :: rest => first ++ replaceStr ++ (find.intercalate rest)

def applyRule (name : String) (rule : Rule) (index : Nat) : String :=
  match rule with
  | .replace find rep => name.replaceFirst find rep
  | .prefix p => p ++ name
  | .suffixBeforeExt suf =>
    match name.splitOn "." with
    | [] => name ++ suf
    | [_] => name ++ suf
    | parts =>
      let ext := parts.getLast!
      let stem := ".".intercalate parts.dropLast
      stem ++ suf ++ "." ++ ext
  | .sequence tmpl => tmpl.replace "{n}" (toString (index + 1))

def BrnDir.preview (d : BrnDir) (rule : Rule) : Except RenameError (List RenameOp) :=
  let named := d.names.mapIdx fun i name => (name, applyRule name rule i)
  let filtered := named.filter (fun p => p.1 ≠ p.2)
  -- Check collisions: no two sources map to same target
  let targets := filtered.map (·.2)
  let deduped := targets.eraseDups
  if targets.length ≠ deduped.length then
    .error (.collision "two sources map to same target")
  else
    let ops := filtered.map fun (from', to') => { from', to' : RenameOp }
    let renameSources := ops.map (·.from')
    match ops.find? (fun op =>
      d.entries.contains op.to' ∧ ¬renameSources.contains op.to') with
    | some op => .error (.toExists op.to')
    | none => .ok ops

def BrnDir.apply (d : BrnDir) (ops : List RenameOp) : Except RenameError (BrnDir × List RenameOp) :=
  match ops.find? (fun op => ¬d.entries.contains op.from') with
  | some op => .error (.fromMissing op.from')
  | none =>
    let afterRemove := ops.foldl (init := d.entries) fun acc op =>
      acc.filter (· ≠ op.from')
    let afterAdd := ops.foldl (init := afterRemove) fun acc op =>
      acc ++ [op.to']
    let undoLog := ops.map fun op => { from' := op.to', to' := op.from' }
    .ok ({ entries := afterAdd }, undoLog)

end Files.BRN
