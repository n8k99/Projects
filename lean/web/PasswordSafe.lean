-- Password Safe — encrypted-at-rest credential store with session FSM.
-- Not real crypto; byte-sum + XOR demonstrates structure.

namespace Web.PS

inductive Strength where
  | weak | fair | strong | veryStrong
  deriving Repr, DecidableEq

structure GeneratePolicy where
  length : Nat := 16
  lower : Bool := true
  upper : Bool := true
  digits : Bool := true
  symbols : Bool := true
  deriving Repr

structure Entry where
  id : Nat
  service : String
  username : String
  encrypted : ByteArray           -- XOR of password with key
  notes : String
  createdAtMs : Nat
  modifiedAtMs : Nat
  deriving Repr

structure Summary where
  id : Nat
  service : String
  username : String
  notes : String
  deriving Repr

structure Safe where
  salt : ByteArray
  verifier : ByteArray
  entries : List Entry := []
  nextId : Nat := 1
  key : Option ByteArray := none
  lastActivityMs : Nat := 0
  autoLockAfterMs : Nat
  deriving Repr

private def verifierHash (master : String) (salt : ByteArray) : ByteArray :=
  let bytes := master.toUTF8
  (ByteArray.mk (Array.range 32 |>.map fun r =>
    let mut h : UInt64 := r.toUInt64
    for i in [0:bytes.size] do
      h := h * 131 + bytes.get! i |>.toUInt64
    for i in [0:salt.size] do
      h := h * 131 + salt.get! i |>.toUInt64
    (h &&& 0xFF).toUInt8))

private def deriveKey (master : String) (salt : ByteArray) : ByteArray :=
  let bytes := master.toUTF8
  (ByteArray.mk (Array.range 64 |>.map fun i =>
    let mut h : UInt64 := (128 + i).toUInt64
    for j in [0:bytes.size] do
      h := h * 197 + bytes.get! j |>.toUInt64
    for j in [0:salt.size] do
      h := h * 197 + salt.get! j |>.toUInt64
    (h &&& 0xFF).toUInt8))

private def xor (data key : ByteArray) : ByteArray :=
  ByteArray.mk (Array.range data.size |>.map fun i =>
    data.get! i ^^^ key.get! (i % key.size))

private def saltBytes : ByteArray :=
  ByteArray.mk (Array.range 16 |>.map (·.toUInt8))

def Safe.mk! (master : String) (autoLockAfterMs : Nat) : Safe :=
  let salt := saltBytes
  { salt, verifier := verifierHash master salt, autoLockAfterMs }

def Safe.isLocked (s : Safe) : Bool := s.key.isNone

def Safe.unlock (s : Safe) (master : String) (nowMs : Nat) : Safe × Bool :=
  let v := verifierHash master s.salt
  if v == s.verifier then
    ({ s with key := some (deriveKey master s.salt), lastActivityMs := nowMs }, true)
  else (s, false)

def Safe.lock (s : Safe) : Safe := { s with key := none }

def Safe.checkAutoLock (s : Safe) (nowMs : Nat) : Safe :=
  match s.key with
  | some _ =>
    if nowMs ≥ s.lastActivityMs + s.autoLockAfterMs then s.lock else s
  | none => s

private def touch (s : Safe) (nowMs : Nat) : Safe × Option ByteArray :=
  match s.key with
  | none => (s, none)
  | some k => ({ s with lastActivityMs := nowMs }, some k)

def Safe.addEntry (s : Safe) (service username password notes : String)
    (nowMs : Nat) : Safe × Option Nat :=
  let (s, k?) := touch s nowMs
  match k? with
  | none => (s, none)
  | some k =>
    let id := s.nextId
    let e : Entry := {
      id, service, username,
      encrypted := xor password.toUTF8 k,
      notes,
      createdAtMs := nowMs, modifiedAtMs := nowMs
    }
    ({ s with entries := s.entries ++ [e], nextId := id + 1 }, some id)

def Safe.getPassword (s : Safe) (id nowMs : Nat) : Safe × Option String :=
  let (s, k?) := touch s nowMs
  match k? with
  | none => (s, none)
  | some k =>
    match s.entries.find? (·.id == id) with
    | none => (s, none)
    | some e => (s, some (String.fromUTF8? (xor e.encrypted k)).getD "")

def Safe.getSummary (s : Safe) (id : Nat) : Option Summary :=
  (s.entries.find? (·.id == id)).map fun e =>
    { id := e.id, service := e.service, username := e.username, notes := e.notes }

def passwordStrength (pw : String) : Strength :=
  let chars := pw.toList
  let len := chars.length
  let hasLower := chars.any (fun c => 'a' ≤ c && c ≤ 'z')
  let hasUpper := chars.any (fun c => 'A' ≤ c && c ≤ 'Z')
  let hasDigit := chars.any (fun c => '0' ≤ c && c ≤ '9')
  let hasSymbol := chars.any (fun c => !c.isAlphanum)
  let classes := (if hasLower then 1 else 0) + (if hasUpper then 1 else 0)
                + (if hasDigit then 1 else 0) + (if hasSymbol then 1 else 0)
  if len < 8 then .weak
  else if len < 12 && classes < 3 then .weak
  else if len ≥ 20 then .veryStrong
  else if len ≥ 14 && classes ≥ 3 then .strong
  else if classes ≥ 4 then .strong
  else .fair

end Web.PS
