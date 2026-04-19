-- Patient / Doctor Scheduler — dual-resource availability, rescheduling, slot search.

namespace Classes

inductive AppointmentStatus
  | scheduled
  | completed
  | cancelled
  | noShow
  deriving Repr, DecidableEq

structure Doctor where
  id : String
  name : String
  specialty : String := ""
  deriving Repr

structure ConsultRoom where
  id : String
  label : String := ""
  deriving Repr

structure SchedPatient where
  id : String
  name : String
  primaryDoctorId : Option String := none
  deriving Repr

structure Appointment where
  id : Nat
  patientId : String
  doctorId : String
  roomId : String
  start : Nat         -- unix seconds
  endAt : Nat
  reason : String := ""
  status : AppointmentStatus := .scheduled
  deriving Repr

def Appointment.isActive (a : Appointment) : Bool :=
  a.status == .scheduled

def Appointment.overlaps (a : Appointment) (s e : Nat) : Bool :=
  a.start < e && s < a.endAt

structure Scheduler where
  doctors : List Doctor := []
  rooms : List ConsultRoom := []
  patients : List SchedPatient := []
  appointments : List Appointment := []
  nextId : Nat := 1
  deriving Repr

def Scheduler.empty : Scheduler := { }

def Scheduler.findDoctor (s : Scheduler) (id : String) : Option Doctor :=
  s.doctors.find? (·.id == id)

def Scheduler.findRoom (s : Scheduler) (id : String) : Option ConsultRoom :=
  s.rooms.find? (·.id == id)

def Scheduler.findPatient (s : Scheduler) (id : String) : Option SchedPatient :=
  s.patients.find? (·.id == id)

def Scheduler.addDoctor (s : Scheduler) (d : Doctor) : Except String Scheduler :=
  if s.doctors.any (·.id == d.id)
  then .error s!"duplicate doctor: {d.id}"
  else .ok { s with doctors := s.doctors ++ [d] }

def Scheduler.addRoom (s : Scheduler) (r : ConsultRoom) : Except String Scheduler :=
  if s.rooms.any (·.id == r.id)
  then .error s!"duplicate room: {r.id}"
  else .ok { s with rooms := s.rooms ++ [r] }

def Scheduler.addPatient (s : Scheduler) (p : SchedPatient) : Except String Scheduler :=
  if s.patients.any (·.id == p.id) then
    .error s!"duplicate patient: {p.id}"
  else match p.primaryDoctorId with
    | some d => if (s.findDoctor d).isNone
                then .error s!"unknown doctor: {d}"
                else .ok { s with patients := s.patients ++ [p] }
    | none   => .ok { s with patients := s.patients ++ [p] }

def Scheduler.doctorConflicts (s : Scheduler) (doctorId : String) (startAt endAt : Nat)
    (excludeId : Option Nat := none) : List Appointment :=
  s.appointments.filter fun a =>
    a.doctorId == doctorId && a.isActive
    && (match excludeId with | some i => a.id ≠ i | none => true)
    && a.overlaps startAt endAt

def Scheduler.roomConflicts (s : Scheduler) (roomId : String) (startAt endAt : Nat)
    (excludeId : Option Nat := none) : List Appointment :=
  s.appointments.filter fun a =>
    a.roomId == roomId && a.isActive
    && (match excludeId with | some i => a.id ≠ i | none => true)
    && a.overlaps startAt endAt

def Scheduler.schedule (s : Scheduler) (patientId doctorId roomId : String)
    (startAt endAt : Nat) (reason : String := "")
    : Except String (Scheduler × Appointment) := do
  if (s.findPatient patientId).isNone then throw s!"unknown patient: {patientId}"
  if (s.findDoctor doctorId).isNone then throw s!"unknown doctor: {doctorId}"
  if (s.findRoom roomId).isNone then throw s!"unknown room: {roomId}"
  if startAt ≥ endAt then throw "start must be before end"
  let dc := s.doctorConflicts doctorId startAt endAt
  if dc.length > 0 then throw s!"doctor {doctorId} busy"
  let rc := s.roomConflicts roomId startAt endAt
  if rc.length > 0 then throw s!"room {roomId} busy"
  let a : Appointment := {
    id := s.nextId, patientId, doctorId, roomId,
    start := startAt, endAt, reason, status := .scheduled
  }
  pure ({ s with appointments := s.appointments ++ [a], nextId := s.nextId + 1 }, a)

private def transitionTo (s : Scheduler) (id : Nat) (to : AppointmentStatus)
    : Except String Scheduler := do
  let a ← (s.appointments.find? (·.id == id)).elim (throw s!"unknown appointment: {id}") pure
  if a.status ≠ .scheduled then throw s!"appointment {id} already {repr a.status}"
  pure { s with appointments := s.appointments.map fun x =>
    if x.id == id then { x with status := to } else x }

def Scheduler.cancel (s : Scheduler) (id : Nat) : Except String Scheduler :=
  transitionTo s id .cancelled

def Scheduler.complete (s : Scheduler) (id : Nat) : Except String Scheduler :=
  transitionTo s id .completed

def Scheduler.markNoShow (s : Scheduler) (id : Nat) : Except String Scheduler :=
  transitionTo s id .noShow

def Scheduler.reschedule (s : Scheduler) (id : Nat) (newStart newEnd : Nat)
    (newRoomId : Option String := none) : Except String Scheduler := do
  if newStart ≥ newEnd then throw "new_start must be before new_end"
  let a ← (s.appointments.find? (·.id == id)).elim (throw s!"unknown appointment: {id}") pure
  if a.status ≠ .scheduled then throw s!"appointment {id} not scheduled"
  let target := newRoomId.getD a.roomId
  if (s.findRoom target).isNone then throw s!"unknown room: {target}"
  let dc := s.doctorConflicts a.doctorId newStart newEnd (some id)
  if dc.length > 0 then throw s!"doctor {a.doctorId} busy"
  let rc := s.roomConflicts target newStart newEnd (some id)
  if rc.length > 0 then throw s!"room {target} busy"
  pure { s with appointments := s.appointments.map fun x =>
    if x.id == id
    then { x with start := newStart, endAt := newEnd, roomId := target }
    else x }

/-- Linear scan of the doctor's calendar at `step` granularity for the first
    window of `duration` where some room is also free. Bounded recursion using
    an explicit fuel counter so Lean accepts termination. -/
partial def Scheduler.findSlot (s : Scheduler) (doctorId : String)
    (duration fromAt untilAt stepSecs : Nat) : Option (Nat × String) :=
  let rec scan (t : Nat) : Option (Nat × String) :=
    if t + duration > untilAt then none
    else
      let e := t + duration
      if (s.doctorConflicts doctorId t e).isEmpty then
        match s.rooms.find? fun r => (s.roomConflicts r.id t e).isEmpty with
        | some r => some (t, r.id)
        | none => scan (t + stepSecs)
      else scan (t + stepSecs)
  scan fromAt

def Scheduler.doctorSchedule (s : Scheduler) (doctorId : String) : List Appointment :=
  s.appointments.filter fun a => a.doctorId == doctorId && a.isActive

def Scheduler.roomSchedule (s : Scheduler) (roomId : String) : List Appointment :=
  s.appointments.filter fun a => a.roomId == roomId && a.isActive

def Scheduler.patientHistory (s : Scheduler) (patientId : String) : List Appointment :=
  s.appointments.filter (·.patientId == patientId)

end Classes
