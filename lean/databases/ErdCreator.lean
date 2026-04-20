-- ERD Creator — entity-relationship diagram with cardinality + topological sort.

namespace Databases.ERD

inductive Cardinality where | oneToOne | oneToMany | manyToMany
  deriving Repr, BEq

def Cardinality.dotArrow : Cardinality → String
  | .oneToOne => "none"
  | .oneToMany => "crow"
  | .manyToMany => "crowodot"

structure Attribute where
  name : String
  ty : String
  primaryKey : Bool := false
  nullable : Bool := false
  deriving Repr

structure Entity where
  name : String
  attributes : List Attribute := []
  deriving Repr

structure Relationship where
  fromEntity : String
  fromAttr : String
  toEntity : String
  toAttr : String
  cardinality : Cardinality
  deriving Repr

structure Erd where
  entities : List (String × Entity) := []
  relationships : List Relationship := []
  deriving Repr

def Erd.addEntity (erd : Erd) (e : Entity) : Erd :=
  { erd with entities := (erd.entities.filter (fun p => p.1 ≠ e.name)) ++ [(e.name, e)] }

def Erd.addRelationship (erd : Erd) (r : Relationship) : Erd :=
  { erd with relationships := erd.relationships ++ [r] }

def Erd.entityNames (erd : Erd) : List String :=
  erd.entities.map Prod.fst

def Erd.dependenciesOf (erd : Erd) (name : String) : List String :=
  let deps := erd.relationships.filterMap fun r =>
    if r.fromEntity == name ∧ r.toEntity ≠ name then some r.toEntity else none
  deps.eraseDups.mergeSort (· < ·)

-- Compute in-degree for each entity based on outgoing FK edges.
def inDegrees (erd : Erd) : List (String × Nat) :=
  let names := erd.entityNames
  let known := fun n => erd.entities.any (fun p => p.1 == n)
  names.map fun n =>
    let deg := (erd.relationships.filter fun r =>
      r.fromEntity == n ∧ r.toEntity ≠ n ∧ known r.toEntity).length
    (n, deg)

partial def Erd.topoSort (erd : Erd) : Option (List String) :=
  let rec go (inDeg : List (String × Nat)) (out : List String) : Option (List String) :=
    let zero := inDeg.filter (fun p => p.2 == 0) |>.map Prod.fst
    if zero.isEmpty then
      if inDeg.isEmpty then some out else none
    else
      -- Pick alphabetically first zero-degree node, decrement its referrers.
      let sorted := zero.mergeSort (· < ·)
      let n := sorted.head!
      let remaining := inDeg.filter (fun p => p.1 ≠ n)
      let updated := remaining.map fun (name, deg) =>
        let incoming := erd.relationships.filter fun r =>
          r.toEntity == n ∧ r.fromEntity == name ∧ r.fromEntity ≠ r.toEntity
        (name, if deg ≥ incoming.length then deg - incoming.length else 0)
      go updated (out ++ [n])
  go (inDegrees erd) []

def Erd.validate (erd : Erd) : List String :=
  erd.relationships.mapIdx fun i r =>
    let fromEnt := erd.entities.find? (fun p => p.1 == r.fromEntity)
    let toEnt := erd.entities.find? (fun p => p.1 == r.toEntity)
    let issues : List String :=
      (match fromEnt with
       | some (_, e) =>
         if e.attributes.any (fun a => a.name == r.fromAttr) then []
         else [s!"relationship {i}: {r.fromEntity}.{r.fromAttr} — attribute missing"]
       | none => [s!"relationship {i}: unknown from_entity {r.fromEntity}"]) ++
      (match toEnt with
       | some (_, e) =>
         if e.attributes.any (fun a => a.name == r.toAttr) then []
         else [s!"relationship {i}: {r.toEntity}.{r.toAttr} — attribute missing"]
       | none => [s!"relationship {i}: unknown to_entity {r.toEntity}"])
    issues
  |>.flatten

end Databases.ERD
