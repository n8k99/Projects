-- Template Maker — meta-level tool for building and validating templates.
-- Pure-functional: every operation returns a new TemplateMaker.

import Web.ECard

namespace Web.TM

open Web.EC

structure Builder where
  id : Nat
  name : String
  category : String
  slots : List Slot := []
  htmlDraft : String := ""
  sampleData : List (String × String) := []
  deriving Repr

inductive IssueKind where
  | unusedSlot | undeclaredPlaceholder | missingSample
  | duplicateSlot | invalidSlotName
  deriving Repr, DecidableEq, BEq

structure Issue where
  kind : IssueKind
  name : String
  deriving Repr

structure Maker where
  builders : List Builder := []
  nextId : Nat := 1
  deriving Repr

def Maker.empty : Maker := {}

def Maker.newTemplate (m : Maker) (name category : String) : Maker × Nat :=
  let id := m.nextId
  let b : Builder := { id, name, category }
  ({ m with builders := m.builders ++ [b], nextId := id + 1 }, id)

def Maker.builder (m : Maker) (id : Nat) : Option Builder :=
  m.builders.find? (·.id == id)

private def updateBuilder (m : Maker) (id : Nat) (f : Builder → Builder) : Maker :=
  { m with builders := m.builders.map fun b => if b.id == id then f b else b }

def Maker.addSlot (m : Maker) (id : Nat) (slot : Slot) : Except String Maker := do
  match m.builder id with
  | none => throw s!"unknown builder: {id}"
  | some b =>
    if b.slots.any (·.name == slot.name) then
      throw s!"slot already exists: {slot.name}"
    else
      pure (updateBuilder m id fun x => { x with slots := x.slots ++ [slot] })

def Maker.updateHtml (m : Maker) (id : Nat) (html : String) : Maker :=
  updateBuilder m id fun b => { b with htmlDraft := html }

def Maker.setSample (m : Maker) (id : Nat) (slotName value : String)
    : Except String Maker := do
  match m.builder id with
  | none => throw s!"unknown builder: {id}"
  | some b =>
    if ! b.slots.any (·.name == slotName) then
      throw s!"slot not found: {slotName}"
    else
      pure (updateBuilder m id fun x =>
        { x with sampleData := (x.sampleData.filter (·.1 ≠ slotName)) ++ [(slotName, value)] })

private partial def extractPlaceholders (s : String) : List String :=
  let rec go (idx : Nat) (acc : List String) : List String :=
    if idx + 1 ≥ s.length then acc.reverse
    else
      let c1 := s.get ⟨idx⟩
      let c2 := s.get ⟨idx + 1⟩
      if c1 == '{' && c2 == '{' then
        let start := idx + 2
        let rec findEnd (j : Nat) : Option Nat :=
          if j + 1 ≥ s.length then none
          else if s.get ⟨j⟩ == '}' && s.get ⟨j + 1⟩ == '}' then some j
          else findEnd (j + 1)
        match findEnd start with
        | some j =>
          let name := s.extract ⟨start⟩ ⟨j⟩
          if name.isEmpty then go (j + 2) acc
          else go (j + 2) (name :: acc)
        | none => acc.reverse
      else go (idx + 1) acc
  go 0 []

private def validSlotName (name : String) : Bool :=
  !name.isEmpty && name.all fun c => c.isAlphanum || c == '_'

def Maker.validate (m : Maker) (id : Nat) : List Issue :=
  match m.builder id with
  | none => []
  | some b =>
    let mut issues : List Issue := []
    let mut seen : List String := []
    for slot in b.slots do
      if !validSlotName slot.name then
        issues := issues ++ [{ kind := .invalidSlotName, name := slot.name }]
      if seen.contains slot.name then
        issues := issues ++ [{ kind := .duplicateSlot, name := slot.name }]
      seen := seen ++ [slot.name]
    let declared := b.slots.map (·.name)
    let referenced := extractPlaceholders b.htmlDraft
    for slot in b.slots do
      if ! referenced.contains slot.name then
        issues := issues ++ [{ kind := .unusedSlot, name := slot.name }]
    for ref in referenced do
      if ! ref.startsWith "_" && ! declared.contains ref then
        issues := issues ++ [{ kind := .undeclaredPlaceholder, name := ref }]
    for slot in b.slots do
      if slot.required && ! b.sampleData.any (·.1 == slot.name) then
        issues := issues ++ [{ kind := .missingSample, name := slot.name }]
    issues

end Web.TM
