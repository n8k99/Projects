-- Travel Planner System — leg-chained itineraries with integrity validation.

namespace Databases.TP

inductive TransportMode where | flight | train | bus | car | walk
  deriving Repr, BEq

structure Leg where
  id : Int
  mode : TransportMode := .flight
  origin : String
  destination : String
  departMs : Int
  arriveMs : Int
  costCents : Int
  carrier : String := ""
  deriving Repr

def Leg.durationMs (l : Leg) : Int := l.arriveMs - l.departMs

structure Trip where
  id : Int
  title : String
  legs : List Leg := []
  minLayoverMinutes : Nat := 30
  deriving Repr

def Trip.addLeg (t : Trip) (l : Leg) : Trip :=
  { t with legs := t.legs ++ [l] }

def Trip.totalSpanMs (t : Trip) : Int :=
  match t.legs, t.legs.getLast? with
  | [], _ => 0
  | first :: _, some last => last.arriveMs - first.departMs
  | _, _ => 0

def Trip.inTransitMs (t : Trip) : Int :=
  t.legs.foldl (init := (0 : Int)) (fun acc l => acc + l.durationMs)

def Trip.totalCostCents (t : Trip) : Int :=
  t.legs.foldl (init := (0 : Int)) (fun acc l => acc + l.costCents)

def Trip.totalLayoverMs (t : Trip) : Int :=
  t.totalSpanMs - t.inTransitMs

structure Layover where
  location : String
  arriveMs : Int
  departMs : Int
  durationMs : Int
  deriving Repr

def layovers (trip : Trip) : List Layover :=
  let pairs := trip.legs.zip trip.legs.tail!
  pairs.map fun (prev, next) =>
    { location := prev.destination,
      arriveMs := prev.arriveMs,
      departMs := next.departMs,
      durationMs := next.departMs - prev.arriveMs : Layover }

inductive IssueSeverity where | info | warning | error
  deriving Repr, BEq

inductive IssueKind where
  | locationMismatch | timeConflict | tightLayover
  | zeroDurationLeg | negativeDurationLeg
  deriving Repr, BEq

structure Issue where
  severity : IssueSeverity
  kind : IssueKind
  legIdx : Nat
  detail : String := ""
  deriving Repr

def validateTrip (trip : Trip) : List Issue := Id.run do
  let mut out : List Issue := []
  for (i, leg) in trip.legs.mapIdx (fun i l => (i, l)) do
    let d := leg.durationMs
    if d == 0 then
      out := out ++ [{ severity := .warning, kind := .zeroDurationLeg, legIdx := i }]
    else if d < 0 then
      out := out ++ [{ severity := .error, kind := .negativeDurationLeg, legIdx := i }]

  let rec chainCheck (i : Nat) (prev : Leg) (rest : List Leg) : List Issue :=
    match rest with
    | [] => []
    | next :: tail =>
      let idx := i + 1
      let mut acc : List Issue := []
      if prev.destination ≠ next.origin then
        acc := acc ++ [{ severity := .error, kind := .locationMismatch,
                          legIdx := idx,
                          detail := s!"{prev.destination} -> {next.origin}" }]
      if next.departMs < prev.arriveMs then
        acc := acc ++ [{ severity := .error, kind := .timeConflict,
                          legIdx := idx,
                          detail := s!"arrive={prev.arriveMs} depart={next.departMs}" }]
      else
        let layoverMs := next.departMs - prev.arriveMs
        let layoverMin := layoverMs / 60000
        if layoverMs > 0 ∧ layoverMin < trip.minLayoverMinutes then
          acc := acc ++ [{ severity := .warning, kind := .tightLayover,
                            legIdx := idx,
                            detail := s!"{layoverMin} min" }]
      acc ++ chainCheck idx next tail

  match trip.legs with
  | [] => pure ()
  | first :: rest => out := out ++ chainCheck 0 first rest

  return out

end Databases.TP
