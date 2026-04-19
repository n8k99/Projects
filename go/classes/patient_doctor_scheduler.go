// Patient / Doctor Scheduler — dual-resource availability, rescheduling, slot search.
package classes

import (
	"errors"
	"fmt"
	"sort"
	"time"
)

// AppointmentStatus enumerates an appointment's lifecycle.
type AppointmentStatus string

const (
	ApptScheduled AppointmentStatus = "scheduled"
	ApptCompleted AppointmentStatus = "completed"
	ApptCancelled AppointmentStatus = "cancelled"
	ApptNoShow    AppointmentStatus = "no_show"
)

// Doctor provides care within a specialty.
type Doctor struct {
	ID        string
	Name      string
	Specialty string
}

// ConsultRoom is a physical exam room.
type ConsultRoom struct {
	ID    string
	Label string
}

// SchedulerPatient is a scheduled patient.
type SchedulerPatient struct {
	ID              string
	Name            string
	PrimaryDoctorID string
}

// Appointment occupies both a doctor's calendar and a room's calendar.
type Appointment struct {
	ID        uint64
	PatientID string
	DoctorID  string
	RoomID    string
	Start     time.Time
	End       time.Time
	Reason    string
	Status    AppointmentStatus
}

// IsActive reports whether the appointment is still scheduled.
func (a Appointment) IsActive() bool { return a.Status == ApptScheduled }

// Overlaps reports half-open interval overlap.
func (a Appointment) Overlaps(start, end time.Time) bool {
	return a.Start.Before(end) && start.Before(a.End)
}

// Scheduler errors.
var (
	ErrUnknownDoctor      = errors.New("unknown doctor")
	ErrUnknownRoom        = errors.New("unknown room")
	ErrUnknownPatient     = errors.New("unknown patient")
	ErrUnknownAppointment = errors.New("unknown appointment")
	ErrDoctorConflict     = errors.New("doctor conflict")
	ErrRoomConflict       = errors.New("room conflict")
	ErrAppointmentClosed  = errors.New("appointment no longer scheduled")
	ErrNoSlotFound        = errors.New("no slot found in window")
)

// Scheduler coordinates doctors, rooms, patients, and appointments.
type Scheduler struct {
	Doctors      map[string]Doctor
	Rooms        map[string]ConsultRoom
	Patients     map[string]SchedulerPatient
	Appointments map[uint64]*Appointment
	nextID       uint64
}

// NewScheduler builds an empty scheduler.
func NewScheduler() *Scheduler {
	return &Scheduler{
		Doctors:      make(map[string]Doctor),
		Rooms:        make(map[string]ConsultRoom),
		Patients:     make(map[string]SchedulerPatient),
		Appointments: make(map[uint64]*Appointment),
		nextID:       1,
	}
}

// AddDoctor registers a doctor.
func (s *Scheduler) AddDoctor(d Doctor) error {
	if _, ok := s.Doctors[d.ID]; ok {
		return fmt.Errorf("%w: doctor %s", ErrDuplicate, d.ID)
	}
	s.Doctors[d.ID] = d
	return nil
}

// AddRoom registers a room.
func (s *Scheduler) AddRoom(r ConsultRoom) error {
	if _, ok := s.Rooms[r.ID]; ok {
		return fmt.Errorf("%w: room %s", ErrDuplicate, r.ID)
	}
	s.Rooms[r.ID] = r
	return nil
}

// AddSchedulerPatient registers a patient.
func (s *Scheduler) AddSchedulerPatient(p SchedulerPatient) error {
	if _, ok := s.Patients[p.ID]; ok {
		return fmt.Errorf("%w: patient %s", ErrDuplicate, p.ID)
	}
	if p.PrimaryDoctorID != "" {
		if _, ok := s.Doctors[p.PrimaryDoctorID]; !ok {
			return fmt.Errorf("%w: %s", ErrUnknownDoctor, p.PrimaryDoctorID)
		}
	}
	s.Patients[p.ID] = p
	return nil
}

// DoctorConflicts returns active appointments on the doctor's calendar that
// overlap [start, end), optionally excluding a specific appointment id.
func (s *Scheduler) DoctorConflicts(doctorID string, start, end time.Time, exclude uint64) ([]uint64, error) {
	if _, ok := s.Doctors[doctorID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownDoctor, doctorID)
	}
	var out []uint64
	for _, a := range s.Appointments {
		if a.DoctorID == doctorID && a.IsActive() && a.ID != exclude && a.Overlaps(start, end) {
			out = append(out, a.ID)
		}
	}
	return out, nil
}

// RoomConflicts is the room-side analog of DoctorConflicts.
func (s *Scheduler) RoomConflicts(roomID string, start, end time.Time, exclude uint64) ([]uint64, error) {
	if _, ok := s.Rooms[roomID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownRoom, roomID)
	}
	var out []uint64
	for _, a := range s.Appointments {
		if a.RoomID == roomID && a.IsActive() && a.ID != exclude && a.Overlaps(start, end) {
			out = append(out, a.ID)
		}
	}
	return out, nil
}

// Schedule books an appointment if both calendars are free.
func (s *Scheduler) Schedule(patientID, doctorID, roomID string, start, end time.Time, reason string) (*Appointment, error) {
	if _, ok := s.Patients[patientID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownPatient, patientID)
	}
	if !start.Before(end) {
		return nil, ErrInvalidInterval
	}
	d, err := s.DoctorConflicts(doctorID, start, end, 0)
	if err != nil {
		return nil, err
	}
	if len(d) > 0 {
		return nil, fmt.Errorf("%w: %s busy (appts %v)", ErrDoctorConflict, doctorID, d)
	}
	r, err := s.RoomConflicts(roomID, start, end, 0)
	if err != nil {
		return nil, err
	}
	if len(r) > 0 {
		return nil, fmt.Errorf("%w: %s busy (appts %v)", ErrRoomConflict, roomID, r)
	}
	id := s.nextID
	s.nextID++
	a := &Appointment{
		ID: id, PatientID: patientID, DoctorID: doctorID, RoomID: roomID,
		Start: start, End: end, Reason: reason, Status: ApptScheduled,
	}
	s.Appointments[id] = a
	return a, nil
}

func (s *Scheduler) transition(id uint64, to AppointmentStatus) error {
	a, ok := s.Appointments[id]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownAppointment, id)
	}
	if a.Status != ApptScheduled {
		return fmt.Errorf("%w: %d", ErrAppointmentClosed, id)
	}
	a.Status = to
	return nil
}

// Cancel marks an appointment cancelled.
func (s *Scheduler) Cancel(id uint64) error { return s.transition(id, ApptCancelled) }

// Complete marks an appointment completed.
func (s *Scheduler) Complete(id uint64) error { return s.transition(id, ApptCompleted) }

// MarkNoShow marks an appointment no_show.
func (s *Scheduler) MarkNoShow(id uint64) error { return s.transition(id, ApptNoShow) }

// Reschedule moves an appointment, excluding itself from the conflict check.
// Pass newRoomID="" to keep the existing room.
func (s *Scheduler) Reschedule(id uint64, newStart, newEnd time.Time, newRoomID string) error {
	if !newStart.Before(newEnd) {
		return ErrInvalidInterval
	}
	a, ok := s.Appointments[id]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownAppointment, id)
	}
	if a.Status != ApptScheduled {
		return fmt.Errorf("%w: %d", ErrAppointmentClosed, id)
	}
	targetRoom := a.RoomID
	if newRoomID != "" {
		targetRoom = newRoomID
	}
	if _, ok := s.Rooms[targetRoom]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownRoom, targetRoom)
	}
	d, err := s.DoctorConflicts(a.DoctorID, newStart, newEnd, id)
	if err != nil {
		return err
	}
	if len(d) > 0 {
		return fmt.Errorf("%w: %s busy (appts %v)", ErrDoctorConflict, a.DoctorID, d)
	}
	r, err := s.RoomConflicts(targetRoom, newStart, newEnd, id)
	if err != nil {
		return err
	}
	if len(r) > 0 {
		return fmt.Errorf("%w: %s busy (appts %v)", ErrRoomConflict, targetRoom, r)
	}
	a.Start = newStart
	a.End = newEnd
	a.RoomID = targetRoom
	return nil
}

// FindSlot scans at `step` granularity for the first window of `duration`
// where the doctor is free AND at least one room is free.
func (s *Scheduler) FindSlot(doctorID string, duration, step time.Duration, from, until time.Time) (time.Time, string, error) {
	if _, ok := s.Doctors[doctorID]; !ok {
		return time.Time{}, "", fmt.Errorf("%w: %s", ErrUnknownDoctor, doctorID)
	}
	if duration <= 0 || step <= 0 || !from.Before(until) {
		return time.Time{}, "", ErrInvalidInterval
	}
	for t := from; !t.Add(duration).After(until); t = t.Add(step) {
		end := t.Add(duration)
		d, _ := s.DoctorConflicts(doctorID, t, end, 0)
		if len(d) > 0 {
			continue
		}
		for _, room := range s.Rooms {
			r, _ := s.RoomConflicts(room.ID, t, end, 0)
			if len(r) == 0 {
				return t, room.ID, nil
			}
		}
	}
	return time.Time{}, "", ErrNoSlotFound
}

// DoctorSchedule returns active appointments for a doctor, sorted by start.
func (s *Scheduler) DoctorSchedule(doctorID string) []*Appointment {
	var out []*Appointment
	for _, a := range s.Appointments {
		if a.DoctorID == doctorID && a.IsActive() {
			out = append(out, a)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Start.Before(out[j].Start) })
	return out
}

// RoomSchedule returns active appointments for a room, sorted by start.
func (s *Scheduler) RoomSchedule(roomID string) []*Appointment {
	var out []*Appointment
	for _, a := range s.Appointments {
		if a.RoomID == roomID && a.IsActive() {
			out = append(out, a)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Start.Before(out[j].Start) })
	return out
}

// PatientHistory returns every appointment a patient ever had.
func (s *Scheduler) PatientHistory(patientID string) []*Appointment {
	var out []*Appointment
	for _, a := range s.Appointments {
		if a.PatientID == patientID {
			out = append(out, a)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Start.Before(out[j].Start) })
	return out
}
