-- Address Book — contacts with normalised-identifier deduplication and merge.

namespace Databases.AB

structure Contact where
  id : Int
  name : String
  createdMs : Int
  emails : List String := []
  phones : List String := []
  tags : List String := []
  address : String := ""
  notes : String := ""
  deriving Repr

def canonicalEmail (email : String) : String :=
  let lower := email.trim.toLower
  match lower.splitOn "@" with
  | [local, domain] =>
    let local := match local.splitOn "+" with
      | l :: _ => l
      | [] => local
    if domain == "gmail.com" ∨ domain == "googlemail.com" then
      (local.replace "." "") ++ "@gmail.com"
    else
      local ++ "@" ++ domain
  | _ => lower

def canonicalPhone (phone : String) : String :=
  let digits := String.mk <| phone.toList.filter Char.isDigit
  if digits.length == 11 ∧ digits.front == '1' then
    digits.drop 1
  else
    digits

def Contact.canonicalEmails (c : Contact) : List String :=
  c.emails.map canonicalEmail

def Contact.canonicalPhones (c : Contact) : List String :=
  c.phones.map canonicalPhone

structure AddressBook where
  contacts : List (Int × Contact) := []
  nextId : Int := 1
  deriving Repr

def AddressBook.add (ab : AddressBook) (c : Contact) : AddressBook :=
  let filtered := ab.contacts.filter (fun p => p.1 ≠ c.id)
  { contacts := filtered ++ [(c.id, c)],
    nextId := if c.id ≥ ab.nextId then c.id + 1 else ab.nextId }

def AddressBook.newId (ab : AddressBook) : (Int × AddressBook) :=
  (ab.nextId, { ab with nextId := ab.nextId + 1 })

def AddressBook.get (ab : AddressBook) (id : Int) : Option Contact :=
  (ab.contacts.find? (fun p => p.1 == id)).map Prod.snd

-- Simple equivalence-class builder: for each identifier, merge the IDs
-- that share it into a single bucket. We use nested lists — not the most
-- efficient but sufficient for the Rosetta Stone.
partial def unifyGroups (groups : List (List Int)) : List (List Int) :=
  let rec merge1 (g : List Int) (rest : List (List Int)) (acc : List (List Int))
      : (List Int × List (List Int)) :=
    match rest with
    | [] => (g, acc.reverse)
    | h :: t =>
      if g.any (fun x => h.contains x) then
        merge1 (g ++ h.filter (fun x => !g.contains x)) t acc
      else merge1 g t (h :: acc)
  match groups with
  | [] => []
  | g :: rest =>
    let (g', rest') := merge1 g rest []
    g' :: unifyGroups rest'

def AddressBook.findDuplicates (ab : AddressBook) : List (List Int) :=
  let emailGroups := (ab.contacts.foldl (init := ([] : List (String × List Int))) fun acc (id, c) =>
    c.canonicalEmails.foldl (init := acc) fun acc e =>
      if e.isEmpty then acc
      else
        match acc.findIdx? (fun p => p.1 == e) with
        | some i => acc.mapIdx fun idx p =>
            if idx == i then (e, p.2 ++ [id]) else p
        | none => acc ++ [(e, [id])])
    |>.map Prod.snd
  let phoneGroups := (ab.contacts.foldl (init := ([] : List (String × List Int))) fun acc (id, c) =>
    c.canonicalPhones.foldl (init := acc) fun acc p =>
      if p.isEmpty then acc
      else
        match acc.findIdx? (fun q => q.1 == p) with
        | some i => acc.mapIdx fun idx q =>
            if idx == i then (p, q.2 ++ [id]) else q
        | none => acc ++ [(p, [id])])
    |>.map Prod.snd
  let initGroups := ab.contacts.map (fun p => [p.1])
  let allGroups := initGroups ++ emailGroups ++ phoneGroups
  let unified := unifyGroups allGroups
  (unified.filter (fun g => g.length ≥ 2)).map (fun g =>
    g.mergeSort (· < ·)) |>.mergeSort (fun a b =>
    match a, b with
    | ha :: _, hb :: _ => ha < hb
    | _, _ => false)

def AddressBook.mergeContacts (ab : AddressBook) (ids : List Int)
    : Except String (Int × AddressBook) := do
  if ids.length < 2 then throw "need at least 2 ids to merge"
  let sources ← ids.mapM fun id =>
    match ab.get id with
    | some c => pure c
    | none => throw s!"id {id} not found"
  let (mergedId, ab1) := ab.newId
  let earliest := sources.foldl (init := (sources.head!).createdMs) fun acc c =>
    if c.createdMs < acc then c.createdMs else acc
  let names := sources.map Contact.name |>.filter (fun n => !n.isEmpty)
  let longest := names.foldl (init := "") fun best n =>
    if n.length > best.length then n else best
  let dedup (xs : List String) : List String :=
    (xs.filter (fun s => !s.isEmpty)).eraseDups.mergeSort (· < ·)
  let merged : Contact := {
    id := mergedId,
    name := longest,
    createdMs := earliest,
    emails := dedup (sources.bind Contact.emails),
    phones := dedup (sources.bind Contact.phones),
    tags := dedup (sources.bind Contact.tags),
    address := " | ".intercalate
      ((sources.map Contact.address).filter (fun s => !s.isEmpty)),
    notes := "\n---\n".intercalate
      ((sources.map Contact.notes).filter (fun s => !s.isEmpty))
  }
  let filtered := ab1.contacts.filter (fun p => !(ids.contains p.1))
  return (mergedId, { ab1 with contacts := filtered ++ [(mergedId, merged)] })

end Databases.AB
