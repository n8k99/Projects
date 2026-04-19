// Library Catalog — three-layer identity, hierarchical subjects, FIFO holds, renewal.
package classes

import (
	"errors"
	"fmt"
	"sort"
	"strings"
	"time"
)

// CopyStatus enumerates a physical copy's lifecycle.
type CopyStatus string

const (
	CopyAvailable  CopyStatus = "available"
	CopyCheckedOut CopyStatus = "checked_out"
	CopyLost       CopyStatus = "lost"
	CopyWithdrawn  CopyStatus = "withdrawn"
)

// HoldStatus enumerates a hold's lifecycle.
type HoldStatus string

const (
	HoldWaiting   HoldStatus = "waiting"
	HoldFulfilled HoldStatus = "fulfilled"
	HoldCancelled HoldStatus = "cancelled"
)

// Title is the abstract work — "1984" by Orwell.
type Title struct {
	ID          string
	Name        string
	Authors     []string
	SubjectPath string // e.g. "500.512.5"
	ISBN        string
}

// LibCopy is a physical item with its own identity, distinct from its Title.
// Named LibCopy to avoid package-level collision.
type LibCopy struct {
	ID       string
	TitleID  string
	Location string
	Status   CopyStatus
}

// Patron borrows copies.
type Patron struct {
	ID   string
	Name string
}

// LibLoan is a temporal claim on a specific copy by a patron.
type LibLoan struct {
	ID            uint64
	CopyID        string
	PatronID      string
	CheckedOutAt  time.Time
	DueAt         time.Time
	ReturnedAt    *time.Time
	RenewalCount  uint32
}

// IsActive reports whether the loan is still checked out.
func (l LibLoan) IsActive() bool { return l.ReturnedAt == nil }

// IsOverdue reports whether an active loan has passed its due date.
func (l LibLoan) IsOverdue(now time.Time) bool {
	return l.IsActive() && now.After(l.DueAt)
}

// Hold is a patron's FIFO place in line for a title.
type Hold struct {
	ID               uint64
	TitleID          string
	PatronID         string
	PlacedAt         time.Time
	Status           HoldStatus
	FulfilledCopyID  string
}

// Catalog errors.
var (
	ErrUnknownTitle         = errors.New("unknown title")
	ErrUnknownCopy          = errors.New("unknown copy")
	ErrUnknownPatron        = errors.New("unknown patron")
	ErrUnknownLoan          = errors.New("unknown loan")
	ErrUnknownHold          = errors.New("unknown hold")
	ErrCopyAlreadyOut       = errors.New("copy already out")
	ErrLoanAlreadyReturned  = errors.New("loan already returned")
	ErrHoldNotWaiting       = errors.New("hold is not waiting")
	ErrRenewalMaxed         = errors.New("loan at max renewals")
	ErrRenewalBlockedByHold = errors.New("cannot renew: other patrons have holds")
)

// DefaultLoanDays is the default checkout term.
const DefaultLoanDays = 21

// MaxRenewals is the per-loan renewal cap.
const MaxRenewals uint32 = 2

// LibraryCatalog coordinates titles, copies, patrons, loans, and holds.
type LibraryCatalog struct {
	Titles     map[string]Title
	Copies     map[string]*LibCopy
	Patrons    map[string]Patron
	Loans      map[uint64]*LibLoan
	Holds      map[uint64]*Hold
	nextLoanID uint64
	nextHoldID uint64
}

// NewLibraryCatalog builds an empty catalog.
func NewLibraryCatalog() *LibraryCatalog {
	return &LibraryCatalog{
		Titles:     make(map[string]Title),
		Copies:     make(map[string]*LibCopy),
		Patrons:    make(map[string]Patron),
		Loans:      make(map[uint64]*LibLoan),
		Holds:      make(map[uint64]*Hold),
		nextLoanID: 1,
		nextHoldID: 1,
	}
}

// AddTitle registers a title.
func (c *LibraryCatalog) AddTitle(t Title) error {
	if _, ok := c.Titles[t.ID]; ok {
		return fmt.Errorf("%w: title %s", ErrDuplicate, t.ID)
	}
	c.Titles[t.ID] = t
	return nil
}

// AddCopy registers a physical copy.
func (c *LibraryCatalog) AddCopy(cp LibCopy) error {
	if _, ok := c.Titles[cp.TitleID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownTitle, cp.TitleID)
	}
	if _, ok := c.Copies[cp.ID]; ok {
		return fmt.Errorf("%w: copy %s", ErrDuplicate, cp.ID)
	}
	if cp.Status == "" {
		cp.Status = CopyAvailable
	}
	c.Copies[cp.ID] = &cp
	return nil
}

// AddPatron registers a patron.
func (c *LibraryCatalog) AddPatron(p Patron) error {
	if _, ok := c.Patrons[p.ID]; ok {
		return fmt.Errorf("%w: patron %s", ErrDuplicate, p.ID)
	}
	c.Patrons[p.ID] = p
	return nil
}

// CheckOut hands a copy to a patron.
func (c *LibraryCatalog) CheckOut(copyID, patronID string, duration time.Duration, now time.Time) (*LibLoan, error) {
	if _, ok := c.Patrons[patronID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownPatron, patronID)
	}
	copy, ok := c.Copies[copyID]
	if !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownCopy, copyID)
	}
	if copy.Status != CopyAvailable {
		return nil, fmt.Errorf("%w: %s (status: %s)", ErrCopyAlreadyOut, copyID, copy.Status)
	}
	copy.Status = CopyCheckedOut
	id := c.nextLoanID
	c.nextLoanID++
	l := &LibLoan{
		ID: id, CopyID: copyID, PatronID: patronID,
		CheckedOutAt: now, DueAt: now.Add(duration),
	}
	c.Loans[id] = l
	return l, nil
}

// ReturnCopy marks a loan returned. If any hold is waiting for the title,
// it is fulfilled and its id returned; otherwise the returned id is 0.
func (c *LibraryCatalog) ReturnCopy(loanID uint64, now time.Time) (uint64, error) {
	l, ok := c.Loans[loanID]
	if !ok {
		return 0, fmt.Errorf("%w: %d", ErrUnknownLoan, loanID)
	}
	if l.ReturnedAt != nil {
		return 0, fmt.Errorf("%w: %d", ErrLoanAlreadyReturned, loanID)
	}
	l.ReturnedAt = &now
	copy := c.Copies[l.CopyID]
	copy.Status = CopyAvailable
	return c.fulfillNextHold(copy.TitleID, copy.ID), nil
}

func (c *LibraryCatalog) fulfillNextHold(titleID, copyID string) uint64 {
	var candidates []*Hold
	for _, h := range c.Holds {
		if h.TitleID == titleID && h.Status == HoldWaiting {
			candidates = append(candidates, h)
		}
	}
	if len(candidates) == 0 {
		return 0
	}
	sort.Slice(candidates, func(i, j int) bool {
		return candidates[i].PlacedAt.Before(candidates[j].PlacedAt)
	})
	h := candidates[0]
	h.Status = HoldFulfilled
	h.FulfilledCopyID = copyID
	return h.ID
}

// Renew extends a loan's due date. Fails if other patrons have waiting holds
// on the title, or if the per-loan renewal cap is reached.
func (c *LibraryCatalog) Renew(loanID uint64, additional time.Duration) error {
	l, ok := c.Loans[loanID]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownLoan, loanID)
	}
	if !l.IsActive() {
		return fmt.Errorf("%w: %d", ErrLoanAlreadyReturned, loanID)
	}
	if l.RenewalCount >= MaxRenewals {
		return fmt.Errorf("%w: %d", ErrRenewalMaxed, loanID)
	}
	copy := c.Copies[l.CopyID]
	for _, h := range c.Holds {
		if h.TitleID == copy.TitleID && h.Status == HoldWaiting {
			return ErrRenewalBlockedByHold
		}
	}
	l.DueAt = l.DueAt.Add(additional)
	l.RenewalCount++
	return nil
}

// PlaceHold queues a patron's claim when no copies are available.
func (c *LibraryCatalog) PlaceHold(titleID, patronID string, now time.Time) (*Hold, error) {
	if _, ok := c.Titles[titleID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownTitle, titleID)
	}
	if _, ok := c.Patrons[patronID]; !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownPatron, patronID)
	}
	id := c.nextHoldID
	c.nextHoldID++
	h := &Hold{
		ID: id, TitleID: titleID, PatronID: patronID,
		PlacedAt: now, Status: HoldWaiting,
	}
	c.Holds[id] = h
	return h, nil
}

// CancelHold retracts a waiting hold.
func (c *LibraryCatalog) CancelHold(holdID uint64) error {
	h, ok := c.Holds[holdID]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownHold, holdID)
	}
	if h.Status != HoldWaiting {
		return fmt.Errorf("%w: %d", ErrHoldNotWaiting, holdID)
	}
	h.Status = HoldCancelled
	return nil
}

// HoldQueue returns waiting holds for a title in FIFO order.
func (c *LibraryCatalog) HoldQueue(titleID string) []*Hold {
	var out []*Hold
	for _, h := range c.Holds {
		if h.TitleID == titleID && h.Status == HoldWaiting {
			out = append(out, h)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].PlacedAt.Before(out[j].PlacedAt) })
	return out
}

// CopiesOf returns all copies of a title.
func (c *LibraryCatalog) CopiesOf(titleID string) []*LibCopy {
	var out []*LibCopy
	for _, cp := range c.Copies {
		if cp.TitleID == titleID {
			out = append(out, cp)
		}
	}
	return out
}

// AvailableCopies returns copies currently on the shelf.
func (c *LibraryCatalog) AvailableCopies(titleID string) []*LibCopy {
	var out []*LibCopy
	for _, cp := range c.Copies {
		if cp.TitleID == titleID && cp.Status == CopyAvailable {
			out = append(out, cp)
		}
	}
	return out
}

// IsAvailable reports whether at least one copy is available.
func (c *LibraryCatalog) IsAvailable(titleID string) bool {
	return len(c.AvailableCopies(titleID)) > 0
}

// SearchBySubject runs a dot-prefix tree query over SubjectPath.
func (c *LibraryCatalog) SearchBySubject(prefix string) []Title {
	var out []Title
	for _, t := range c.Titles {
		if t.SubjectPath == prefix || strings.HasPrefix(t.SubjectPath, prefix+".") {
			out = append(out, t)
		}
	}
	return out
}

// SearchByAuthor returns titles by an author.
func (c *LibraryCatalog) SearchByAuthor(author string) []Title {
	var out []Title
	for _, t := range c.Titles {
		for _, a := range t.Authors {
			if a == author {
				out = append(out, t)
				break
			}
		}
	}
	return out
}

// Overdue returns all active loans past their due date.
func (c *LibraryCatalog) Overdue(now time.Time) []*LibLoan {
	var out []*LibLoan
	for _, l := range c.Loans {
		if l.IsOverdue(now) {
			out = append(out, l)
		}
	}
	return out
}
