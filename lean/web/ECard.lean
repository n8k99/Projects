-- E-Card Generator — template + data rendering with slot validation.
-- Pure-functional: every operation returns a new Gallery.

namespace Web.EC

inductive SlotKind where | text | longText | imageUrl | color | date
  deriving Repr, DecidableEq, BEq

structure Slot where
  name : String
  kind : SlotKind := .text
  required : Bool := false
  default : Option String := none
  deriving Repr

structure Template where
  id : Nat
  name : String
  category : String
  slots : List Slot
  htmlTemplate : String
  deriving Repr

structure Card where
  id : Nat
  templateId : Nat
  filled : List (String × String)
  recipient : String
  sender : String
  createdAtMs : Nat
  deriving Repr

inductive ValidationError where
  | unknownTemplate (id : Nat)
  | missingRequired (name : String)
  | unknownSlot (name : String)
  deriving Repr

structure Gallery where
  templates : List Template := []
  cards : List Card := []
  nextTemplateId : Nat := 1
  nextCardId : Nat := 1
  deriving Repr

def Gallery.empty : Gallery := {}

def Gallery.addTemplate (g : Gallery) (name category : String)
    (slots : List Slot) (htmlTemplate : String) : Gallery × Nat :=
  let id := g.nextTemplateId
  let t : Template := { id, name, category, slots, htmlTemplate }
  ({ g with templates := g.templates ++ [t], nextTemplateId := id + 1 }, id)

def Gallery.template (g : Gallery) (id : Nat) : Option Template :=
  g.templates.find? (·.id == id)

def Gallery.templatesInCategory (g : Gallery) (category : String) : List Template :=
  g.templates.filter (·.category == category)

private def findPair (xs : List (String × String)) (key : String)
    : Option String :=
  (xs.find? (·.1 == key)).map (·.2)

def Gallery.createCard (g : Gallery) (templateId : Nat)
    (filled : List (String × String))
    (recipient sender : String) (nowMs : Nat)
    : Except ValidationError (Gallery × Nat) := do
  let template ← (g.template templateId).toExcept (.unknownTemplate templateId)
  let known := template.slots.map (·.name)
  -- Unknown slot?
  for pair in filled do
    if ! known.contains pair.1 then throw (.unknownSlot pair.1)
  -- Missing required?
  for s in template.slots do
    if s.required && (filled.find? (·.1 == s.name)).isNone then
      throw (.missingRequired s.name)
  -- Apply defaults.
  let mut final := filled
  for s in template.slots do
    if (final.find? (·.1 == s.name)).isNone then
      match s.default with
      | some d => final := final ++ [(s.name, d)]
      | none => pure ()
  let id := g.nextCardId
  let c : Card := {
    id, templateId, filled := final,
    recipient, sender, createdAtMs := nowMs
  }
  pure ({ g with cards := g.cards ++ [c], nextCardId := id + 1 }, id)

def Gallery.card (g : Gallery) (id : Nat) : Option Card :=
  g.cards.find? (·.id == id)

private def replaceAll (text needle replacement : String) : String :=
  text.splitOn needle |>.intersperse replacement |> String.join

def Gallery.render (g : Gallery) (cardId : Nat) : Option String := do
  let card ← g.card cardId
  let template ← g.template card.templateId
  let mut out := template.htmlTemplate
  for (k, v) in card.filled do
    out := replaceAll out s!"\{\{{k}}}" v
  out := replaceAll out "{{_recipient}}" card.recipient
  out := replaceAll out "{{_sender}}" card.sender
  return out

end Web.EC
