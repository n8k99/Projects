// Movie Store — two entities joined by a rental, with due dates and temporal obligations.
package classes

import (
	"errors"
	"fmt"
	"time"
)

// Movie is a title with a copy count; individual copies are not tracked separately.
type Movie struct {
	ID           string
	Title        string
	Director     string
	Year         int
	Genre        string
	CopiesTotal  int
}

// Customer borrows movies.
type Customer struct {
	ID   string
	Name string
}

// Rental is a time-bounded obligation joining Customer and Movie.
type Rental struct {
	ID         uint64
	MovieID    string
	CustomerID string
	RentedAt   time.Time
	DueAt      time.Time
	ReturnedAt *time.Time // nil while active
}

// IsActive reports whether the rental is still checked out.
func (r Rental) IsActive() bool { return r.ReturnedAt == nil }

// IsOverdue reports whether an active rental has passed its due date.
func (r Rental) IsOverdue(now time.Time) bool {
	return r.IsActive() && now.After(r.DueAt)
}

// Store errors.
var (
	ErrUnknownMovie    = errors.New("unknown movie")
	ErrUnknownCustomer = errors.New("unknown customer")
	ErrUnknownRental   = errors.New("unknown rental")
	ErrNotAvailable    = errors.New("no copies available")
	ErrAlreadyReturned = errors.New("rental already returned")
	ErrDuplicate       = errors.New("duplicate id")
)

// DefaultRentalDays is the default rental term.
const DefaultRentalDays = 7

// MovieStore holds movies, customers, and the rental ledger.
type MovieStore struct {
	Movies    map[string]Movie
	Customers map[string]Customer
	Rentals   map[uint64]*Rental
	nextID    uint64
}

// NewMovieStore builds an empty store.
func NewMovieStore() *MovieStore {
	return &MovieStore{
		Movies:    make(map[string]Movie),
		Customers: make(map[string]Customer),
		Rentals:   make(map[uint64]*Rental),
		nextID:    1,
	}
}

// AddMovie registers a movie.
func (s *MovieStore) AddMovie(m Movie) error {
	if _, ok := s.Movies[m.ID]; ok {
		return fmt.Errorf("%w: movie %s", ErrDuplicate, m.ID)
	}
	s.Movies[m.ID] = m
	return nil
}

// AddCustomer registers a customer.
func (s *MovieStore) AddCustomer(c Customer) error {
	if _, ok := s.Customers[c.ID]; ok {
		return fmt.Errorf("%w: customer %s", ErrDuplicate, c.ID)
	}
	s.Customers[c.ID] = c
	return nil
}

// Rent creates a rental, failing if no copy is available.
func (s *MovieStore) Rent(movieID, customerID string, duration time.Duration, now time.Time) (*Rental, error) {
	if _, ok := s.Movies[movieID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownMovie, movieID)
	}
	if _, ok := s.Customers[customerID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownCustomer, customerID)
	}
	if s.CopiesAvailable(movieID) <= 0 {
		return nil, fmt.Errorf("%w: %s", ErrNotAvailable, movieID)
	}
	id := s.nextID
	s.nextID++
	r := &Rental{
		ID:         id,
		MovieID:    movieID,
		CustomerID: customerID,
		RentedAt:   now,
		DueAt:      now.Add(duration),
	}
	s.Rentals[id] = r
	return r, nil
}

// ReturnMovie marks a rental returned.
func (s *MovieStore) ReturnMovie(rentalID uint64, now time.Time) error {
	r, ok := s.Rentals[rentalID]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownRental, rentalID)
	}
	if r.ReturnedAt != nil {
		return fmt.Errorf("%w: %d", ErrAlreadyReturned, rentalID)
	}
	r.ReturnedAt = &now
	return nil
}

// CopiesAvailable is total copies minus active rentals.
func (s *MovieStore) CopiesAvailable(movieID string) int {
	m, ok := s.Movies[movieID]
	if !ok {
		return 0
	}
	active := 0
	for _, r := range s.Rentals {
		if r.MovieID == movieID && r.IsActive() {
			active++
		}
	}
	return m.CopiesTotal - active
}

// ActiveRentals returns a customer's open rentals.
func (s *MovieStore) ActiveRentals(customerID string) []*Rental {
	var out []*Rental
	for _, r := range s.Rentals {
		if r.CustomerID == customerID && r.IsActive() {
			out = append(out, r)
		}
	}
	return out
}

// Overdue returns all active rentals past due as of the given time.
func (s *MovieStore) Overdue(now time.Time) []*Rental {
	var out []*Rental
	for _, r := range s.Rentals {
		if r.IsOverdue(now) {
			out = append(out, r)
		}
	}
	return out
}

// MovieHistory returns every rental that ever involved a movie.
func (s *MovieStore) MovieHistory(movieID string) []*Rental {
	var out []*Rental
	for _, r := range s.Rentals {
		if r.MovieID == movieID {
			out = append(out, r)
		}
	}
	return out
}

// CustomerHistory returns every rental a customer has ever made.
func (s *MovieStore) CustomerHistory(customerID string) []*Rental {
	var out []*Rental
	for _, r := range s.Rentals {
		if r.CustomerID == customerID {
			out = append(out, r)
		}
	}
	return out
}

// ByGenre filters movies.
func (s *MovieStore) ByGenre(genre string) []Movie {
	var out []Movie
	for _, m := range s.Movies {
		if m.Genre == genre {
			out = append(out, m)
		}
	}
	return out
}
