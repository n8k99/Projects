// Vending Machine — a finite state machine with coin-constrained change-making.
package classes

import (
	"errors"
	"fmt"
	"sort"
)

// CoinDenominations lists the accepted coin values in cents (high → low).
var CoinDenominations = []int64{100, 25, 10, 5, 1}

// VendingState enumerates the machine's two states.
type VendingState string

const (
	VendIdle      VendingState = "idle"
	VendAccepting VendingState = "accepting"
)

// VendingSlot is one product slot in the machine.
type VendingSlot struct {
	SlotID     string
	Product    string
	PriceCents int64
	Quantity   int64
}

// Vending errors.
var (
	ErrVendUnknownSlot       = errors.New("unknown slot")
	ErrVendDuplicateSlot     = errors.New("duplicate slot")
	ErrVendInvalidPrice      = errors.New("invalid price")
	ErrVendSoldOut           = errors.New("sold out")
	ErrVendInsufficientFunds = errors.New("insufficient funds")
	ErrVendCannotMakeChange  = errors.New("cannot make change")
	ErrVendBadState          = errors.New("bad state")
	ErrVendInvalidCoin       = errors.New("invalid coin")
	ErrVendInvalidCount      = errors.New("invalid count")
)

// VendingMachine is an FSM with coin inventory and product slots.
type VendingMachine struct {
	Slots          map[string]*VendingSlot
	CoinInventory  map[int64]int64
	InsertedCents  int64
	InsertedCoins  map[int64]int64
	State          VendingState
}

// NewVendingMachine builds an empty machine in the Idle state.
func NewVendingMachine() *VendingMachine {
	coins := make(map[int64]int64)
	ins := make(map[int64]int64)
	for _, d := range CoinDenominations {
		coins[d] = 0
		ins[d] = 0
	}
	return &VendingMachine{
		Slots:         make(map[string]*VendingSlot),
		CoinInventory: coins,
		InsertedCoins: ins,
		State:         VendIdle,
	}
}

// AddSlot registers a product slot.
func (m *VendingMachine) AddSlot(s VendingSlot) error {
	if _, ok := m.Slots[s.SlotID]; ok {
		return fmt.Errorf("%w: %s", ErrVendDuplicateSlot, s.SlotID)
	}
	if s.PriceCents <= 0 {
		return ErrVendInvalidPrice
	}
	m.Slots[s.SlotID] = &s
	return nil
}

func isValidCoin(d int64) bool {
	for _, v := range CoinDenominations {
		if v == d {
			return true
		}
	}
	return false
}

// LoadChange increases the machine's coin inventory for one denomination.
func (m *VendingMachine) LoadChange(denom, count int64) error {
	if !isValidCoin(denom) {
		return fmt.Errorf("%w: %d", ErrVendInvalidCoin, denom)
	}
	if count < 0 {
		return ErrVendInvalidCount
	}
	m.CoinInventory[denom] += count
	return nil
}

// InsertCoin accepts a coin, transitioning Idle → Accepting on first insert.
func (m *VendingMachine) InsertCoin(denom int64) (int64, error) {
	if !isValidCoin(denom) {
		return 0, fmt.Errorf("%w: %d", ErrVendInvalidCoin, denom)
	}
	m.InsertedCents += denom
	m.InsertedCoins[denom]++
	m.State = VendAccepting
	return m.InsertedCents, nil
}

// Refund returns all inserted coins and resets to Idle.
func (m *VendingMachine) Refund() map[int64]int64 {
	out := make(map[int64]int64)
	for d, c := range m.InsertedCoins {
		if c > 0 {
			out[d] = c
		}
		m.InsertedCoins[d] = 0
	}
	m.InsertedCents = 0
	m.State = VendIdle
	return out
}

// Select atomically dispenses a product + change, or fails without mutating.
func (m *VendingMachine) Select(slotID string) (string, map[int64]int64, error) {
	if m.State != VendAccepting {
		return "", nil, fmt.Errorf("%w: cannot select while %s", ErrVendBadState, m.State)
	}
	slot, ok := m.Slots[slotID]
	if !ok {
		return "", nil, fmt.Errorf("%w: %s", ErrVendUnknownSlot, slotID)
	}
	if slot.Quantity <= 0 {
		return "", nil, fmt.Errorf("%w: %s", ErrVendSoldOut, slotID)
	}
	if m.InsertedCents < slot.PriceCents {
		return "", nil, fmt.Errorf("%w: %s is %d, inserted %d",
			ErrVendInsufficientFunds, slot.Product, slot.PriceCents, m.InsertedCents)
	}
	changeOwed := m.InsertedCents - slot.PriceCents

	// Pool = machine coins + inserted coins (inserted become the machine's at commit).
	pool := make(map[int64]int64)
	for d, c := range m.CoinInventory {
		pool[d] = c
	}
	for d, c := range m.InsertedCoins {
		pool[d] += c
	}

	change := greedyChange(changeOwed, pool)
	if change == nil {
		return "", nil, fmt.Errorf("%w: %d cents owed", ErrVendCannotMakeChange, changeOwed)
	}

	// Commit.
	for d, c := range change {
		pool[d] -= c
	}
	m.CoinInventory = pool
	slot.Quantity--
	m.InsertedCents = 0
	for d := range m.InsertedCoins {
		m.InsertedCoins[d] = 0
	}
	m.State = VendIdle

	out := make(map[int64]int64)
	for d, c := range change {
		if c > 0 {
			out[d] = c
		}
	}
	return slot.Product, out, nil
}

func greedyChange(amount int64, pool map[int64]int64) map[int64]int64 {
	result := make(map[int64]int64)
	remaining := amount
	denoms := make([]int64, 0, len(pool))
	for d := range pool {
		denoms = append(denoms, d)
	}
	sort.Slice(denoms, func(i, j int) bool { return denoms[i] > denoms[j] })
	for _, d := range denoms {
		if d > remaining {
			continue
		}
		avail := pool[d]
		use := remaining / d
		if use > avail {
			use = avail
		}
		if use > 0 {
			result[d] = use
			remaining -= d * use
		}
		if remaining == 0 {
			break
		}
	}
	if remaining != 0 {
		return nil
	}
	return result
}
