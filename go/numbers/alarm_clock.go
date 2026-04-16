package numbers

import "fmt"

// Alarm Clock — schedule and trigger time-based alerts.

// AlarmEntry holds a single alarm's time and label.
type AlarmEntry struct {
	Hour   int
	Minute int
	Second int
	Label  string
}

// AlarmClock manages a collection of time-based alarms.
type AlarmClock struct {
	alarms []AlarmEntry
}

// NewAlarmClock creates an empty alarm clock.
func NewAlarmClock() *AlarmClock {
	return &AlarmClock{}
}

// SetAlarm schedules an alarm for the given time with an optional label.
func (ac *AlarmClock) SetAlarm(hour, minute, second int, label string) error {
	if hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 59 {
		return fmt.Errorf("invalid time: %02d:%02d:%02d", hour, minute, second)
	}
	ac.alarms = append(ac.alarms, AlarmEntry{
		Hour: hour, Minute: minute, Second: second, Label: label,
	})
	return nil
}

// Check compares all pending alarms against the given time (h, m, s).
// Returns the list of triggered alarms and removes them from pending.
func (ac *AlarmClock) Check(nowH, nowM, nowS int) []AlarmEntry {
	nowSecs := nowH*3600 + nowM*60 + nowS
	var triggered []AlarmEntry
	var remaining []AlarmEntry

	for _, a := range ac.alarms {
		alarmSecs := a.Hour*3600 + a.Minute*60 + a.Second
		if nowSecs >= alarmSecs {
			triggered = append(triggered, a)
		} else {
			remaining = append(remaining, a)
		}
	}
	ac.alarms = remaining
	return triggered
}

// Pending returns all alarms that have not yet triggered.
func (ac *AlarmClock) Pending() []AlarmEntry {
	result := make([]AlarmEntry, len(ac.alarms))
	copy(result, ac.alarms)
	return result
}

// Cancel removes the first pending alarm with the given label.
// Returns true if an alarm was removed, false otherwise.
func (ac *AlarmClock) Cancel(label string) bool {
	for i, a := range ac.alarms {
		if a.Label == label {
			ac.alarms = append(ac.alarms[:i], ac.alarms[i+1:]...)
			return true
		}
	}
	return false
}

// Demo usage (in a main package):
//
//	func main() {
//		clock := numbers.NewAlarmClock()
//		clock.SetAlarm(0, 0, 0, "midnight")
//		clock.SetAlarm(23, 59, 59, "end-of-day")
//		clock.SetAlarm(12, 0, 0, "noon")
//
//		fmt.Printf("Pending: %d\n", len(clock.Pending()))
//		clock.Cancel("noon")
//		triggered := clock.Check(12, 0, 0)
//		fmt.Printf("Triggered: %d\n", len(triggered))
//	}
