-- WYSIWYG Editor — document model where edit and render are the same structure.
--
-- Pure-functional document model: every operation returns a new Document.
-- Runs merge when adjacent and attribute-equal; split when attributes diverge.

namespace Web

inductive Attr where
  | bold | italic | underline | h1 | h2
  deriving Repr, DecidableEq, Ord, BEq

structure Run where
  text : String
  attrs : List Attr := []
  deriving Repr

structure Document where
  runs : List Run := []
  deriving Repr

def Document.empty : Document := {}

def Document.length (d : Document) : Nat :=
  d.runs.foldl (init := 0) (fun acc r => acc + r.text.length)

private def splitRuns (runs : List Run) (pos : Nat) : List Run × List Run :=
  let rec loop (rs : List Run) (seen : Nat) (leftAcc : List Run)
      : List Run × List Run :=
    match rs with
    | [] => (leftAcc.reverse, [])
    | r :: rest =>
      let n := r.text.length
      if pos ≥ seen + n then
        loop rest (seen + n) (r :: leftAcc)
      else if pos ≤ seen then
        (leftAcc.reverse, r :: rest)
      else
        let split := pos - seen
        let l : Run := { text := r.text.take split, attrs := r.attrs }
        let rt : Run := { text := r.text.drop split, attrs := r.attrs }
        ((l :: leftAcc).reverse, rt :: rest)
  loop runs 0 []

private def normalize (runs : List Run) : List Run :=
  runs.foldl (init := []) (fun acc r =>
    if r.text.isEmpty then acc
    else match acc.reverse with
      | last :: rest =>
        if last.attrs == r.attrs then
          ({ last with text := last.text ++ r.text } :: rest).reverse
        else acc ++ [r]
      | [] => [r]
  )

def Document.insert (d : Document) (pos : Nat) (text : String) (attrs : List Attr)
    : Document :=
  let (left, right) := splitRuns d.runs pos
  { runs := normalize (left ++ [{ text, attrs }] ++ right) }

def Document.delete (d : Document) (start stop : Nat) : Document :=
  if start ≥ stop then d
  else
    let (left, rest) := splitRuns d.runs start
    let (_, right) := splitRuns rest (stop - start)
    { runs := normalize (left ++ right) }

private def modifyAttrs (d : Document) (start stop : Nat)
    (fn : List Attr → List Attr) : Document :=
  if start ≥ stop then d
  else
    let (left, rest) := splitRuns d.runs start
    let (middle, right) := splitRuns rest (stop - start)
    let newMid := middle.map fun r => { r with attrs := fn r.attrs }
    { runs := normalize (left ++ newMid ++ right) }

def Document.applyStyle (d : Document) (start stop : Nat) (a : Attr) : Document :=
  modifyAttrs d start stop fun attrs =>
    if attrs.contains a then attrs else attrs ++ [a]

def Document.removeStyle (d : Document) (start stop : Nat) (a : Attr) : Document :=
  modifyAttrs d start stop fun attrs => attrs.filter (· ≠ a)

def Document.toPlain (d : Document) : String :=
  d.runs.foldl (init := "") (fun acc r => acc ++ r.text)

end Web
