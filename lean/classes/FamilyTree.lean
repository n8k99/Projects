-- Family Tree Creator — the first self-referential graph with cycle prevention.
--
-- A `Person` refers to other persons (as parents). The structure is a DAG,
-- not a tree — "family tree" is a misnomer. A person has at most two parents,
-- but joining two unrelated branches of a large family yields multiple paths
-- to shared ancestors. The code treats it as a graph throughout.
--
-- `setParents` runs a transitive closure check: a candidate parent must not
-- already be a descendant of the child. First Classes project where an
-- invariant spans the entire reachable structure.

namespace Classes

abbrev PersonId := Nat

structure Person where
  id : PersonId
  name : String
  birth : Option Int := none
  death : Option Int := none
  parents : List PersonId := []
  deriving Repr

structure FamilyTree where
  persons : List Person := []
  nextId : PersonId := 1
  deriving Repr

def FamilyTree.empty : FamilyTree := {}

def FamilyTree.find (t : FamilyTree) (id : PersonId) : Option Person :=
  t.persons.find? (·.id == id)

def FamilyTree.addPerson (t : FamilyTree) (name : String)
    (birth death : Option Int := none) : FamilyTree × PersonId :=
  let p : Person := { id := t.nextId, name, birth, death }
  ({ t with persons := t.persons ++ [p], nextId := t.nextId + 1 }, t.nextId)

/-- Children of `id` (direct, one generation). -/
def FamilyTree.childrenOf (t : FamilyTree) (id : PersonId) : List PersonId :=
  (t.persons.filter fun p => p.parents.contains id).map (·.id)

/-- BFS reachable via `step`, bounded by `maxDepth` (none = unbounded).
    Excludes `start`. -/
partial def bfs (start : PersonId) (maxDepth : Option Nat)
    (step : PersonId → List PersonId) : List PersonId :=
  let rec loop (queue : List (PersonId × Nat)) (seen : List PersonId)
      : List PersonId :=
    match queue with
    | [] => seen.reverse
    | (cur, d) :: rest =>
      let atLimit := match maxDepth with | some m => d ≥ m | none => false
      if atLimit then loop rest seen
      else
        let nexts := step cur
        let (newSeen, newQ) := nexts.foldl (init := (seen, rest))
          fun (acc : List PersonId × List (PersonId × Nat)) n =>
            if acc.1.contains n then acc
            else (n :: acc.1, acc.2 ++ [(n, d + 1)])
        loop newQ newSeen
  loop [(start, 0)] []

def FamilyTree.ancestors (t : FamilyTree) (id : PersonId)
    (maxDepth : Option Nat := none) : List PersonId :=
  bfs id maxDepth fun x =>
    (t.find x).map (·.parents) |>.getD []

def FamilyTree.descendants (t : FamilyTree) (id : PersonId)
    (maxDepth : Option Nat := none) : List PersonId :=
  bfs id maxDepth (t.childrenOf)

def FamilyTree.commonAncestors (t : FamilyTree) (a b : PersonId)
    : List PersonId :=
  let aa := t.ancestors a
  let bb := t.ancestors b
  aa.filter (fun x => bb.contains x)

def FamilyTree.siblings (t : FamilyTree) (id : PersonId) : List PersonId :=
  match t.find id with
  | none => []
  | some p =>
    let all := p.parents.flatMap (t.childrenOf)
    -- Dedup and drop self.
    (all.filter (· ≠ id)).eraseDups

def FamilyTree.setParents (t : FamilyTree) (child : PersonId)
    (newParents : List PersonId) : Except String FamilyTree := do
  if (t.find child).isNone then throw s!"unknown person: {child}"
  if newParents.length > 2 then throw s!"too many parents: {newParents.length}"
  if newParents.length ≠ newParents.eraseDups.length then throw "duplicate parent"
  for p in newParents do
    if p == child then throw "self-parent not allowed"
    if (t.find p).isNone then throw s!"unknown person: {p}"
  let desc := t.descendants child
  for p in newParents do
    if desc.contains p then
      throw s!"would create cycle: {p} is a descendant of {child}"
  let updated : List Person := t.persons.map fun pr =>
    if pr.id == child then { pr with parents := newParents } else pr
  pure { t with persons := updated }

end Classes
