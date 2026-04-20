-- Quick Launcher — ranked launchable items with frecency and file round-trip.

namespace Files.QL

structure Entry where
  id : Nat
  name : String
  action : String
  keywords : List String := []
  count : Nat := 0
  last : Nat := 0
  deriving Repr

structure Launcher where
  entries : List Entry := []
  frecencyWeight : Float := 0.1
  deriving Repr

structure Match where
  entryId : Nat
  score : Float
  deriving Repr

def addEntry (l : Launcher) (e : Entry) : Launcher :=
  let filtered := l.entries.filter (fun x => x.id ≠ e.id)
  { l with entries := e :: filtered }

def getEntry (l : Launcher) (id : Nat) : Option Entry :=
  l.entries.find? (fun e => e.id == id)

def size (l : Launcher) : Nat := l.entries.length

/-- Text match: exact > prefix > substring > keyword. Zero = filter out. -/
def textMatch (needle : String) (e : Entry) : Float :=
  if needle.isEmpty then 0.5
  else
    let nameLc := e.name.toLower
    if nameLc == needle then 1.0
    else if nameLc.startsWith needle then 0.8
    else if nameLc.splitOn needle |>.length > 1 then 0.6
    else
      let rec check (ks : List String) : Float :=
        match ks with
        | [] => 0.0
        | k :: rest =>
          let klc := k.toLower
          if klc == needle then 0.5
          else if klc.splitOn needle |>.length > 1 then 0.4
          else check rest
      check e.keywords

/-- Frecency: log(1+count) / (1+age_hours). -/
def frecency (e : Entry) (nowMs : Nat) : Float :=
  if e.count == 0 then 0.0
  else
    let ageMs := if nowMs ≥ e.last then nowMs - e.last else 0
    let ageHours := ageMs.toFloat / 3600000.0
    (1.0 + e.count.toFloat).log / (1.0 + ageHours)

def query (l : Launcher) (text : String) (nowMs : Nat) : List Match :=
  let needle := text.trim.toLower
  let scored := l.entries.filterMap fun e =>
    let tm := textMatch needle e
    if tm == 0.0 then none
    else
      let fr := frecency e nowMs
      some { entryId := e.id, score := tm + l.frecencyWeight * fr : Match }
  -- Sort desc by score, tie-break asc by entryId
  scored.mergeSort (fun a b =>
    if a.score ≠ b.score then a.score > b.score
    else a.entryId < b.entryId)

def recordLaunch (l : Launcher) (id : Nat) (nowMs : Nat) : Launcher :=
  { l with entries := l.entries.map fun e =>
      if e.id == id then { e with count := e.count + 1, last := nowMs } else e }

def toText (l : Launcher) : String :=
  let sorted := l.entries.mergeSort (fun a b => a.id < b.id)
  let body := sorted.foldl (init := "") fun acc e =>
    let header := s!"---\nid: {e.id}\nname: {e.name}\naction: {e.action}\n"
    let kws := e.keywords.foldl (init := "") (fun a k => a ++ s!"keyword: {k}\n")
    let trail := s!"count: {e.count}\nlast: {e.last}\n"
    acc ++ header ++ kws ++ trail
  "LAUNCHER\n" ++ body

end Files.QL
