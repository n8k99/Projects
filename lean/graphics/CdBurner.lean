-- CD Burning App — multi-session disc model + FFD packing + ISO names.

namespace Graphics.Burn

def leadinBytes : Nat := 13000000
def leadoutBytes : Nat := 22000000

structure BurnFile where
  name : String
  sizeBytes : Nat
  deriving Repr

inductive SessionState where | open' | closed
  deriving Repr, BEq

structure Session where
  tracks : List BurnFile := []
  state : SessionState := .open'
  deriving Repr

def Session.overheadBytes (_ : Session) : Nat := leadinBytes + leadoutBytes

def Session.totalTrackBytes (s : Session) : Nat :=
  s.tracks.foldl (init := 0) (fun acc f => acc + f.sizeBytes)

def Session.totalBytes (s : Session) : Nat := s.totalTrackBytes + s.overheadBytes

inductive DiscState where
  | blank | sessionOpen | sessionClosed | finalized
  deriving Repr, BEq

structure Disc where
  capacityBytes : Nat
  sessions : List Session := []
  state : DiscState := .blank
  deriving Repr

def Disc.usedBytes (d : Disc) : Nat :=
  d.sessions.foldl (init := 0) (fun acc s => acc + s.totalBytes)

def Disc.remainingBytes (d : Disc) : Nat :=
  if d.usedBytes ≥ d.capacityBytes then 0 else d.capacityBytes - d.usedBytes

inductive BurnError where
  | discFinalized | noOpenSession | sessionAlreadyOpen | notEnoughSpace
  deriving Repr, BEq

def Disc.openSession (d : Disc) : Except BurnError Disc :=
  match d.state with
  | .finalized => .error .discFinalized
  | .sessionOpen => .error .sessionAlreadyOpen
  | _ => .ok { d with sessions := d.sessions ++ [{}], state := .sessionOpen }

def replaceLast {α : Type} (xs : List α) (x : α) : List α :=
  match xs.reverse with
  | [] => [x]
  | _ :: rest => (x :: rest).reverse

def Disc.addFile (d : Disc) (f : BurnFile) : Except BurnError Disc :=
  if d.state == .finalized then .error .discFinalized
  else if d.state != .sessionOpen then .error .noOpenSession
  else if f.sizeBytes > d.remainingBytes then .error .notEnoughSpace
  else
    match d.sessions.reverse with
    | [] => .error .noOpenSession
    | last :: rest =>
      let newSession := { last with tracks := last.tracks ++ [f] }
      .ok { d with sessions := (newSession :: rest).reverse }

def Disc.closeSession (d : Disc) : Except BurnError Disc :=
  if d.state == .finalized then .error .discFinalized
  else if d.state != .sessionOpen then .error .noOpenSession
  else
    match d.sessions.reverse with
    | [] => .error .noOpenSession
    | last :: rest =>
      let newSession := { last with state := .closed }
      .ok { d with sessions := (newSession :: rest).reverse, state := .sessionClosed }

def Disc.finalize (d : Disc) : Except BurnError Disc :=
  if d.state == .finalized then .error .discFinalized
  else if d.state == .sessionOpen then
    match d.closeSession with
    | .ok d' => .ok { d' with state := .finalized }
    | .error e => .error e
  else .ok { d with state := .finalized }

structure PackResult where
  discs : List (List BurnFile)
  overflow : List BurnFile
  deriving Repr

partial def insertSortedDesc (f : BurnFile) : List BurnFile → List BurnFile
  | [] => [f]
  | g :: rest => if f.sizeBytes ≥ g.sizeBytes then f :: g :: rest
                  else g :: insertSortedDesc f rest

def sortDesc (fs : List BurnFile) : List BurnFile :=
  fs.foldl (init := []) (fun acc f => insertSortedDesc f acc)

def packFilesFFD (files : List BurnFile) (discCapacity : Nat)
    (numDiscs : Nat) : PackResult := Id.run do
  let sorted := sortDesc files
  let usable := if discCapacity ≥ leadinBytes + leadoutBytes
                then discCapacity - (leadinBytes + leadoutBytes) else 0
  let mut discs : Array (List BurnFile) := Array.mkArray numDiscs []
  let mut remaining : Array Nat := Array.mkArray numDiscs usable
  let mut overflow : List BurnFile := []
  for f in sorted do
    let mut placed := false
    let mut i := 0
    while i < numDiscs ∧ !placed do
      if f.sizeBytes ≤ remaining[i]! then
        remaining := remaining.set! i (remaining[i]! - f.sizeBytes)
        discs := discs.set! i (discs[i]! ++ [f])
        placed := true
      i := i + 1
    if !placed then overflow := overflow ++ [f]
  pure { discs := discs.toList, overflow }

def isCleanChar (c : Char) : Bool :=
  let u := c.toUpper
  (u.isAlpha ∧ u.isUpper) ∨ u.isDigit ∨ u == '_'

def cleanChars (s : String) : String :=
  s.foldl (init := "") fun acc c =>
    let u := c.toUpper
    if isCleanChar c then acc.push u else acc

def splitExt (name : String) : String × String :=
  match name.revFind? (· == '.') with
  | none => (name, "")
  | some i =>
    let stem := name.extract 0 i
    let ext := name.extract (name.next i) name.endPos
    if stem.isEmpty then (name, "") else (stem, ext)

def canonicalizeIsoName (name : String) (existing : List String) : String :=
  let (stem, ext) := splitExt name
  let stemClean := cleanChars stem
  let extClean := cleanChars ext
  let extTrunc := extClean.extract 0 ⟨min 3 extClean.length⟩
  let stemBase := stemClean.extract 0 ⟨min 8 stemClean.length⟩
  let base := if extTrunc.isEmpty then stemBase else stemBase ++ "." ++ extTrunc
  if !existing.contains base then base
  else Id.run do
    for n in [1:1000] do
      let suffix := "~" ++ toString n
      let room := if suffix.length ≥ 8 then 0 else 8 - suffix.length
      let short := stemClean.extract 0 ⟨min room stemClean.length⟩
      let candidate := if extTrunc.isEmpty then short ++ suffix
                        else short ++ suffix ++ "." ++ extTrunc
      if !existing.contains candidate then return candidate
    return base

end Graphics.Burn
