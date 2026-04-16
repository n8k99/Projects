/- Alarm Clock — schedule and trigger time-based alerts. -/

namespace Numbers

/-- A time of day, stored as hour/minute/second. -/
structure TimeOfDay where
  hour   : Nat
  minute : Nat
  second : Nat
deriving BEq, Repr

/-- A single alarm with a target time and label. -/
structure Alarm where
  time  : TimeOfDay
  label : String
deriving BEq, Repr

/-- The state of an alarm clock: a list of pending alarms. -/
structure AlarmState where
  alarms : List Alarm
deriving Repr

/-- Create an empty alarm state. -/
def AlarmState.empty : AlarmState :=
  { alarms := [] }

/-- Convert a time of day to total seconds for comparison. -/
def TimeOfDay.toSeconds (t : TimeOfDay) : Nat :=
  t.hour * 3600 + t.minute * 60 + t.second

/-- Set a new alarm. Returns the updated state. -/
def setAlarm (state : AlarmState) (hour minute second : Nat) (label : String) : AlarmState :=
  let alarm : Alarm := { time := { hour, minute, second }, label }
  { alarms := state.alarms ++ [alarm] }

/-- Check which alarms have triggered at the given time.
    Returns (triggered alarms, new state with only pending alarms). -/
def checkAlarms (state : AlarmState) (now : TimeOfDay) : List Alarm × AlarmState :=
  let nowSecs := now.toSeconds
  let (triggered, remaining) := state.alarms.partition fun a =>
    a.time.toSeconds <= nowSecs
  (triggered, { alarms := remaining })

/-- Return all pending alarms. -/
def pendingAlarms (state : AlarmState) : List Alarm :=
  state.alarms

/-- Cancel the first alarm with the given label.
    Returns (success, updated state). -/
def cancelAlarm (state : AlarmState) (label : String) : Bool × AlarmState :=
  let rec go (before after : List Alarm) : Bool × List Alarm :=
    match after with
    | [] => (false, before.reverse)
    | a :: rest =>
      if a.label == label then
        (true, before.reverse ++ rest)
      else
        go (a :: before) rest
  let (found, newAlarms) := go [] state.alarms
  (found, { alarms := newAlarms })

end Numbers
