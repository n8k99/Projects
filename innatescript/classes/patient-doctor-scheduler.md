# G054 — Patient / Doctor Scheduler

> Dual-resource availability. A single operation must validate across two independent calendars. Rescheduling excludes the moving record from its own conflict check. `find_slot` turns availability from a yes/no check into a search.

```yaml
id: G054
title: Patient / Doctor Scheduler
category: classes
requires: [G050-reservation-system, G053-library-catalog]
provides: [dual-resource-availability, self-exclusion-on-reschedule, slot-search]
```

## Insight: Dual-Resource Availability Is a Conjunction, Not a Sum

G050's reservations checked one calendar per booking. A seat is free, you book it. A room is free, you book it. G054 requires both at once: an appointment occupies the *doctor's* calendar AND the *room's* calendar simultaneously. The booking fails if either is busy. Availability is the conjunction:

```
schedule(p, d, r, [s, e))  <-  doctor_free(d, [s, e))  ∧  room_free(r, [s, e))
```

This is the first *conjunctive* scheduling constraint in the Rosetta Stone. Generalizes immediately: a build needs both a runner and a cache lock; a meeting needs every attendee's calendar AND a conference room; a deploy needs the CI pipeline AND the production window. The noosphere's choreographies routinely involve this shape — most real work requires multiple resources in alignment.

Critically, the two conflict queries are **independent** (they run against different tables, no shared state) but the commit is **joint** (both must pass before either side is marked busy). This is a small transaction in the G052 sense: the write happens atomically after both reads pass.

## Insight: Reschedule Requires Self-Exclusion

Moving an existing appointment is not "cancel + rebook" — if it were, you'd pay the cost of losing the slot and racing for it back. It is a single mutation: the appointment's start and end move. But the conflict check must exclude the appointment's *own current state*, or it conflicts with itself and no move is ever legal.

```
reschedule(x, [s', e'))
  <- doctor_free(x.doctor, [s', e'), except x)
  ∧ room_free(x.room, [s', e'), except x)
```

The `except x` clause is new. Previous conflict checks were global ("anything that overlaps"). Reschedule introduces the *exclusion predicate* on the entity being updated.

This generalizes to every in-place update of a temporal obligation. A deploy window being shifted excludes itself from its own conflict check. A team member's shift being moved excludes itself. An event being rescheduled on the calendar excludes itself. "Ignore yourself when checking if you'd collide with someone" is the shape of *move-in-place* in any system with temporal uniqueness.

## Insight: `find_slot` Turns Availability Into a Search

G050's `available(start, end)` asked: given this window, what's free? G054's `find_slot(doctor, duration, within)` asks: within this window, where is *something* free? The second is a search, not a check. The scheduler walks time forward at some step granularity, evaluating the dual-resource constraint at each step, and returns the first point where both calendars agree.

This is the first project in the Rosetta Stone where the answer to an availability question is *computed by scanning* rather than *retrieved by lookup*. Every "when can we meet" dialog box in every calendar app is running this algorithm. Every agent scheduler in the noosphere that takes a task and finds a free window does the same scan.

**Granularity matters.** A 15-minute step won't find 10-minute slots that start on the 7's. A 5-second step is wasteful for hour-long appointments. The step is a policy parameter that trades recall against work; the scheduler exposes it rather than hardcoding. In InnateScript, this is the first explicit cost/precision knob on a resolver native. Expect more.

## Insight: One Appointment, Three Projections

An appointment is one record. But it participates in three distinct schedules: the doctor's, the room's, the patient's. `doctor_schedule(d)`, `room_schedule(r)`, `patient_history(p)` all query the same underlying list with different filters. One source, three views — the database-view pattern from G052 elevated from "two accounts per transaction" to "three indexes per appointment."

This is the shape of every vault record that participates in multiple indexes. A task is in the Tasks index, the Project index, the Agent-assigned index. A conversation is in the Thread index, the Participants index, the Topic index. A daily note is in the Temporal index, the Narrative index, the Ghost-activity index. The scheduler is the first project where one record's membership in multiple indexes is explicit and ergonomic — a primary list with filtered projections.

## Choreographic Case: Urgent Slot Request

```innate
(@find-next-available){
  @request <- {
    doctor:   "D-CHEN"
    duration: 30_min
    within:   @today..@today + 3d
    priority: "urgent"
  }

  @slot <- @scheduler/find-slot{
    doctor-id: @request.doctor,
    duration:  @request.duration,
    from:      @request.within.start,
    until:     @request.within.end,
    step:      15_min
  }

  concurrent {
    @sarah/notify-patient{slot: @slot}
    @kathryn/confirm-resources{slot: @slot}
  } join as @confirmed

  @appt <- @scheduler/schedule{
    patient-id: @patient.id,
    doctor-id:  @request.doctor,
    room-id:    @slot.room,
    start:      @slot.start,
    end:        @slot.start + @request.duration
  }

  where {
    slot_found:         @slot != nil
    no_double_book:     @appt.status == scheduled
    patient_notified:   @confirmed.sarah == true
  }
}
```

`find_slot` returns a candidate; `schedule` commits it. Between the two, other choreographies might book the slot — so the `schedule` call is the real source of truth, and the `where` catches the race. `find_slot` is advisory; `schedule` is authoritative.

## Structures

```innate
(defstruct doctor
  id        : String
  name      : String
  specialty : String)

(defstruct room
  id    : String
  label : String)

(defstruct appointment
  id         : Nat
  patient-id : String
  doctor-id  : String
  room-id    : String
  start      : Timestamp
  end        : Timestamp
  reason     : String
  status     : "scheduled" | "completed" | "cancelled" | "no-show")

(defstruct scheduler
  doctors      : {String -> Doctor}
  rooms        : {String -> Room}
  patients     : {String -> Patient}
  appointments : {Nat -> Appointment})
```

## Resolver Natives

```innate
@scheduler{}                                                     -> Scheduler
@scheduler/add-doctor{doctor}                                    -> Scheduler
@scheduler/add-room{room}                                        -> Scheduler
@scheduler/schedule{patient, doctor, room, start, end}           -> Appointment
@scheduler/cancel{id}                                            -> Appointment
@scheduler/reschedule{id, new-start, new-end, new-room?}         -> Appointment
@scheduler/doctor-conflicts{doctor-id, start, end, except?}      -> [Appointment]
@scheduler/room-conflicts{room-id, start, end, except?}          -> [Appointment]
@scheduler/find-slot{doctor-id, duration, from, until, step?}    -> (Time, Room)?
@scheduler/doctor-schedule{doctor-id}                            -> [Appointment]
@scheduler/room-schedule{room-id}                                -> [Appointment]
@scheduler/patient-history{patient-id}                           -> [Appointment]
```

## Demo

```innate
(@demo){
  @s <- @scheduler{}
    .add-doctor{id: "D-CHEN", specialty: "cardiology"}
    .add-room{id: "R-101"}
    .add-room{id: "R-102"}
    .add-patient{id: "P-001", name: "Nathan", primary-doctor-id: "D-CHEN"}

  @a1 <- @s/schedule{patient: "P-001", doctor: "D-CHEN", room: "R-101",
                     start: @t, end: @t + 30_min}

  ;; doctor conflict:
  @s/schedule{patient: "P-001", doctor: "D-CHEN", room: "R-102",
              start: @t + 15_min, end: @t + 45_min}
  ;; => error: doctor busy

  ;; self-excluding reschedule:
  @a1-moved <- @s/reschedule{id: @a1.id, new-start: @t + 2h, new-end: @t + 2h30m}

  @slot <- @s/find-slot{doctor-id: "D-CHEN", duration: 30_min,
                        from: @t, until: @t + 4h}
}
```

## Where

The dual-resource constraint MUST be a conjunction — both calendars checked, both required free. Reschedule MUST exclude the moving appointment from its own conflict check. `find_slot` MUST return the first window where BOTH the doctor and at least one room are free, not the first doctor-free window. Those three rules are what makes a scheduler a scheduler rather than two independent calendars that happen to share a page.
