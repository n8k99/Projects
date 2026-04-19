-- Reservation System — bookable resources with time intervals and conflict detection.
--
-- Models airline seats and hotel rooms uniformly. Overlap is the core primitive:
-- two reservations on the same resource collide iff their half-open intervals
-- [start, end) overlap.

namespace Classes

structure Resource where
  id : String
  kind : String
  label : String := ""
  deriving Repr

structure ResCustomer where
  id : String
  name : String
  deriving Repr

inductive ReservationStatus
  | booked
  | cancelled
  | completed
  deriving Repr, DecidableEq

structure Reservation where
  id : Nat
  resourceId : String
  customerId : String
  start : Nat        -- unix seconds
  endAt : Nat
  status : ReservationStatus := .booked
  deriving Repr

def Reservation.isActive (r : Reservation) : Bool :=
  r.status == .booked

/-- Half-open overlap: touching boundaries do not conflict. -/
def Reservation.overlaps (r : Reservation) (s e : Nat) : Bool :=
  r.start < e && s < r.endAt

structure ReservationSystem where
  resources : List Resource := []
  customers : List ResCustomer := []
  reservations : List Reservation := []
  nextId : Nat := 1
  deriving Repr

def ReservationSystem.empty : ReservationSystem := { }

def ReservationSystem.findResource (s : ReservationSystem) (id : String) : Option Resource :=
  s.resources.find? (·.id == id)

def ReservationSystem.findCustomer (s : ReservationSystem) (id : String) : Option ResCustomer :=
  s.customers.find? (·.id == id)

def ReservationSystem.addResource (s : ReservationSystem) (r : Resource)
    : Except String ReservationSystem :=
  if s.resources.any (·.id == r.id)
  then .error s!"duplicate resource id: {r.id}"
  else .ok { s with resources := s.resources ++ [r] }

def ReservationSystem.addCustomer (s : ReservationSystem) (c : ResCustomer)
    : Except String ReservationSystem :=
  if s.customers.any (·.id == c.id)
  then .error s!"duplicate customer id: {c.id}"
  else .ok { s with customers := s.customers ++ [c] }

def ReservationSystem.conflicts (sys : ReservationSystem) (resourceId : String) (s e : Nat)
    : Except String (List Reservation) :=
  if (sys.findResource resourceId).isNone then
    .error s!"unknown resource: {resourceId}"
  else if s ≥ e then
    .error "start must be before end"
  else
    .ok <| sys.reservations.filter fun r =>
      r.resourceId == resourceId && r.isActive && r.overlaps s e

def ReservationSystem.book (sys : ReservationSystem)
    (resourceId customerId : String) (s e : Nat)
    : Except String (ReservationSystem × Reservation) :=
  if (sys.findResource resourceId).isNone then
    .error s!"unknown resource: {resourceId}"
  else if (sys.findCustomer customerId).isNone then
    .error s!"unknown customer: {customerId}"
  else if s ≥ e then
    .error "start must be before end"
  else
    let colliding := sys.reservations.filter fun r =>
      r.resourceId == resourceId && r.isActive && r.overlaps s e
    if colliding.length > 0 then
      .error s!"conflict: {resourceId} overlaps {colliding.length} existing reservation(s)"
    else
      let r : Reservation := {
        id := sys.nextId, resourceId, customerId,
        start := s, endAt := e, status := .booked
      }
      .ok ({ sys with reservations := sys.reservations ++ [r], nextId := sys.nextId + 1 }, r)

def ReservationSystem.cancel (sys : ReservationSystem) (reservationId : Nat)
    : Except String ReservationSystem :=
  match sys.reservations.find? (·.id == reservationId) with
  | none => .error s!"unknown reservation: {reservationId}"
  | some r =>
    if r.status == .cancelled then
      .error s!"reservation {reservationId} already cancelled"
    else
      let updated := sys.reservations.map fun x =>
        if x.id == reservationId then { x with status := .cancelled } else x
      .ok { sys with reservations := updated }

def ReservationSystem.available (sys : ReservationSystem) (s e : Nat) (kind : Option String := none)
    : Except String (List Resource) :=
  if s ≥ e then
    .error "start must be before end"
  else
    .ok <| sys.resources.filter fun res =>
      (match kind with | none => true | some k => res.kind == k)
      && ¬ (sys.reservations.any fun r =>
             r.resourceId == res.id && r.isActive && r.overlaps s e)

def ReservationSystem.occupancy (sys : ReservationSystem) (resourceId : String)
    : List Reservation :=
  sys.reservations.filter fun r => r.resourceId == resourceId && r.isActive

def ReservationSystem.customerBookings (sys : ReservationSystem) (customerId : String)
    : List Reservation :=
  sys.reservations.filter (·.customerId == customerId)

end Classes
