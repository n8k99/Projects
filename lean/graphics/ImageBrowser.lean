-- Image Browser — 2D grid navigation + LRU cache + filter/sort query.

namespace Graphics.Browser

structure ImageEntry where
  id : Nat
  name : String
  path : String
  width : Nat
  height : Nat
  sizeBytes : Nat
  tags : List String := []
  createdMs : Nat := 0
  deriving Repr

inductive FilterPredicate where
  | nameContains (s : String)
  | tagHas (s : String)
  | sizeUnder (n : Nat)
  | sizeOver (n : Nat)
  | widthAtLeast (n : Nat)
  | createdAfter (n : Nat)
  deriving Repr

def FilterPredicate.matches (p : FilterPredicate) (e : ImageEntry) : Bool :=
  match p with
  | .nameContains s => s.isPrefixOf e.name ∨ e.name.splitOn s |>.length ≥ 2
  | .tagHas s => e.tags.contains s
  | .sizeUnder n => e.sizeBytes < n
  | .sizeOver n => e.sizeBytes > n
  | .widthAtLeast n => e.width ≥ n
  | .createdAfter n => e.createdMs > n

inductive SortBy where | name | size | created | width
  deriving Repr, BEq

inductive Order where | asc | desc
  deriving Repr, BEq

structure Query where
  filters : List FilterPredicate := []
  sortBy : SortBy := .name
  order : Order := .asc
  deriving Repr

def applyQuery (entries : List ImageEntry) (q : Query) : List ImageEntry :=
  let filtered := entries.filter (fun e => q.filters.all (·.matches e))
  let cmp := fun (a b : ImageEntry) =>
    let natLt : Nat → Nat → Bool := fun x y => x < y
    let res := match q.sortBy with
      | .name => a.name < b.name
      | .size => natLt a.sizeBytes b.sizeBytes
      | .created => natLt a.createdMs b.createdMs
      | .width => natLt a.width b.width
    if q.order == .desc then !res else res
  filtered.mergeSort cmp |>.toList

inductive GridMove where | left | right | up | down | home | end'
  deriving Repr, BEq

structure Grid where
  cols : Nat
  total : Nat
  selected : Nat := 0
  deriving Repr

def Grid.rows (g : Grid) : Nat :=
  if g.cols == 0 ∨ g.total == 0 then 0
  else (g.total + g.cols - 1) / g.cols

def Grid.rowCol (g : Grid) : Nat × Nat := (g.selected / g.cols, g.selected % g.cols)

def Grid.move (g : Grid) (mv : GridMove) : Grid :=
  if g.total == 0 then g
  else
    let (row, col) := g.rowCol
    match mv with
    | .left => if g.selected > 0 then { g with selected := g.selected - 1 } else g
    | .right => if g.selected + 1 < g.total then { g with selected := g.selected + 1 } else g
    | .up => if row > 0 then { g with selected := (row - 1) * g.cols + col } else g
    | .down =>
      let nxt := (row + 1) * g.cols + col
      if nxt < g.total then { g with selected := nxt }
      else if row + 1 < g.rows then { g with selected := g.total - 1 }
      else g
    | .home => { g with selected := 0 }
    | .end' => if g.total > 0 then { g with selected := g.total - 1 } else g

-- LRU cache via list: head = LRU, tail = MRU
structure LruCache (V : Type) where
  capacity : Nat
  entries : List (Nat × V) := []
  deriving Repr

def LruCache.contains {V} (c : LruCache V) (k : Nat) : Bool :=
  c.entries.any (fun p => p.1 == k)

def LruCache.len {V} (c : LruCache V) : Nat := c.entries.length

def LruCache.get {V} (c : LruCache V) (k : Nat) : LruCache V × Option V :=
  match c.entries.find? (fun p => p.1 == k) with
  | none => (c, none)
  | some p =>
    let rest := c.entries.filter (fun q => q.1 != k)
    ({ c with entries := rest ++ [p] }, some p.2)

def LruCache.put {V} (c : LruCache V) (k : Nat) (v : V) : LruCache V × Option Nat :=
  if c.entries.any (fun p => p.1 == k) then
    let rest := c.entries.filter (fun q => q.1 != k)
    ({ c with entries := rest ++ [(k, v)] }, none)
  else if c.entries.length ≥ c.capacity then
    match c.entries with
    | [] => ({ c with entries := [(k, v)] }, none)
    | (evictedKey, _) :: rest =>
      ({ c with entries := rest ++ [(k, v)] }, some evictedKey)
  else
    ({ c with entries := c.entries ++ [(k, v)] }, none)

structure Browser where
  gridCols : Nat
  breadcrumbs : List String := []
  allEntries : List ImageEntry := []
  query : Query := {}
  grid : Grid
  thumbnailCache : LruCache (List UInt8)
  deriving Repr

def Browser.new (gridCols thumbCap : Nat) : Browser :=
  { gridCols, grid := { cols := gridCols, total := 0 },
    thumbnailCache := { capacity := thumbCap } }

def Browser.setEntries (b : Browser) (entries : List ImageEntry) : Browser :=
  let filtered := applyQuery entries b.query
  { b with allEntries := entries, grid := { cols := b.gridCols, total := filtered.length } }

def Browser.setQuery (b : Browser) (q : Query) : Browser :=
  let filtered := applyQuery b.allEntries q
  { b with query := q, grid := { cols := b.gridCols, total := filtered.length } }

def Browser.filteredEntries (b : Browser) : List ImageEntry :=
  applyQuery b.allEntries b.query

def Browser.selectedEntry (b : Browser) : Option ImageEntry :=
  b.filteredEntries.get? b.grid.selected

def Browser.navigateInto (b : Browser) (subdir : String) : Browser :=
  { b with breadcrumbs := b.breadcrumbs ++ [subdir] }

def Browser.navigateUp (b : Browser) : Browser × Bool :=
  if b.breadcrumbs.isEmpty then (b, false)
  else ({ b with breadcrumbs := b.breadcrumbs.dropLast }, true)

def Browser.currentPath (b : Browser) : String :=
  "/" ++ String.intercalate "/" b.breadcrumbs

def Browser.moveCursor (b : Browser) (mv : GridMove) : Browser :=
  { b with grid := b.grid.move mv }

end Graphics.Browser
