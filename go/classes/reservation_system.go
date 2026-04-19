// Reservation System — bookable resources with time intervals and conflict detection.
//
// Models airline seats and hotel rooms uniformly. Overlap is the core primitive:
// two reservations on the same resource collide iff their half-open intervals
// [start, end) overlap.
package classes

import (
	"errors"
	"fmt"
	"sort"
	"time"
)

// Resource is a specific bookable thing: "12A" or "Room 304".
type Resource struct {
	ID    string
	Kind  string // "seat", "room", "slot", ...
	Label string
}

// CustomerR is a person making reservations. Named CustomerR to avoid collision
// with movie_store.Customer in the same package.
type CustomerR struct {
	ID   string
	Name string
}

// ReservationStatus enumerates reservation lifecycle states.
type ReservationStatus string

const (
	ReservationBooked    ReservationStatus = "booked"
	ReservationCancelled ReservationStatus = "cancelled"
	ReservationCompleted ReservationStatus = "completed"
)

// Reservation is an interval claim on a resource by a customer.
type Reservation struct {
	ID         uint64
	ResourceID string
	CustomerID string
	Start      time.Time
	End        time.Time
	Status     ReservationStatus
	CreatedAt  time.Time
}

// IsActive reports whether the reservation is still booked.
func (r Reservation) IsActive() bool { return r.Status == ReservationBooked }

// Overlaps reports half-open interval overlap; touching boundaries do not conflict.
func (r Reservation) Overlaps(start, end time.Time) bool {
	return r.Start.Before(end) && start.Before(r.End)
}

// Reservation errors.
var (
	ErrUnknownResource    = errors.New("unknown resource")
	ErrUnknownReservation = errors.New("unknown reservation")
	ErrInvalidInterval    = errors.New("start must be before end")
	ErrConflict           = errors.New("reservation conflict")
	ErrAlreadyCancelled   = errors.New("reservation already cancelled")
)

// ReservationSystem allocates resources across time intervals.
type ReservationSystem struct {
	Resources    map[string]Resource
	Customers    map[string]CustomerR
	Reservations map[uint64]*Reservation
	nextID       uint64
}

// NewReservationSystem builds an empty system.
func NewReservationSystem() *ReservationSystem {
	return &ReservationSystem{
		Resources:    make(map[string]Resource),
		Customers:    make(map[string]CustomerR),
		Reservations: make(map[uint64]*Reservation),
		nextID:       1,
	}
}

// AddResource registers a resource.
func (s *ReservationSystem) AddResource(r Resource) error {
	if _, ok := s.Resources[r.ID]; ok {
		return fmt.Errorf("%w: resource %s", ErrDuplicate, r.ID)
	}
	s.Resources[r.ID] = r
	return nil
}

// AddCustomerR registers a customer.
func (s *ReservationSystem) AddCustomerR(c CustomerR) error {
	if _, ok := s.Customers[c.ID]; ok {
		return fmt.Errorf("%w: customer %s", ErrDuplicate, c.ID)
	}
	s.Customers[c.ID] = c
	return nil
}

// Conflicts returns active reservations on resourceID that overlap [start, end).
func (s *ReservationSystem) Conflicts(resourceID string, start, end time.Time) ([]*Reservation, error) {
	if _, ok := s.Resources[resourceID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownResource, resourceID)
	}
	if !start.Before(end) {
		return nil, ErrInvalidInterval
	}
	var out []*Reservation
	for _, r := range s.Reservations {
		if r.ResourceID == resourceID && r.IsActive() && r.Overlaps(start, end) {
			out = append(out, r)
		}
	}
	return out, nil
}

// Book creates a reservation, failing if the interval conflicts with existing bookings.
func (s *ReservationSystem) Book(resourceID, customerID string, start, end time.Time) (*Reservation, error) {
	if _, ok := s.Resources[resourceID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownResource, resourceID)
	}
	if _, ok := s.Customers[customerID]; !ok {
		return nil, fmt.Errorf("%w: customer %s", ErrUnknownCustomer, customerID)
	}
	if !start.Before(end) {
		return nil, ErrInvalidInterval
	}
	colliding, _ := s.Conflicts(resourceID, start, end)
	if len(colliding) > 0 {
		ids := make([]uint64, len(colliding))
		for i, c := range colliding {
			ids[i] = c.ID
		}
		return nil, fmt.Errorf("%w: %s overlaps %v", ErrConflict, resourceID, ids)
	}
	id := s.nextID
	s.nextID++
	r := &Reservation{
		ID:         id,
		ResourceID: resourceID,
		CustomerID: customerID,
		Start:      start,
		End:        end,
		Status:     ReservationBooked,
		CreatedAt:  time.Now(),
	}
	s.Reservations[id] = r
	return r, nil
}

// Cancel marks a reservation cancelled, freeing its interval.
func (s *ReservationSystem) Cancel(reservationID uint64) error {
	r, ok := s.Reservations[reservationID]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownReservation, reservationID)
	}
	if r.Status == ReservationCancelled {
		return fmt.Errorf("%w: %d", ErrAlreadyCancelled, reservationID)
	}
	r.Status = ReservationCancelled
	return nil
}

// Available returns resources (optionally filtered by kind) with no overlapping
// active reservation across [start, end).
func (s *ReservationSystem) Available(start, end time.Time, kind string) ([]Resource, error) {
	if !start.Before(end) {
		return nil, ErrInvalidInterval
	}
	var out []Resource
	for _, res := range s.Resources {
		if kind != "" && res.Kind != kind {
			continue
		}
		collides := false
		for _, r := range s.Reservations {
			if r.ResourceID == res.ID && r.IsActive() && r.Overlaps(start, end) {
				collides = true
				break
			}
		}
		if !collides {
			out = append(out, res)
		}
	}
	return out, nil
}

// Occupancy returns active reservations on a resource, sorted by start.
func (s *ReservationSystem) Occupancy(resourceID string) ([]*Reservation, error) {
	if _, ok := s.Resources[resourceID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownResource, resourceID)
	}
	var out []*Reservation
	for _, r := range s.Reservations {
		if r.ResourceID == resourceID && r.IsActive() {
			out = append(out, r)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Start.Before(out[j].Start) })
	return out, nil
}

// CustomerBookings returns every reservation a customer has made.
func (s *ReservationSystem) CustomerBookings(customerID string) ([]*Reservation, error) {
	if _, ok := s.Customers[customerID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownCustomer, customerID)
	}
	var out []*Reservation
	for _, r := range s.Reservations {
		if r.CustomerID == customerID {
			out = append(out, r)
		}
	}
	return out, nil
}
