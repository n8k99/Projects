// Bank Account Manager — double-entry ledger, atomic transactions, balance as aggregate.
package classes

import (
	"errors"
	"fmt"
	"time"
)

// AccountKind enumerates supported account types.
type AccountKind string

const (
	AccountChecking AccountKind = "checking"
	AccountSavings  AccountKind = "savings"
	AccountCredit   AccountKind = "credit"
)

// AccountStatus enumerates account lifecycle states.
type AccountStatus string

const (
	AccountOpen   AccountStatus = "open"
	AccountFrozen AccountStatus = "frozen"
	AccountClosed AccountStatus = "closed"
)

// TxKind enumerates transaction categories.
type TxKind string

const (
	TxDeposit    TxKind = "deposit"
	TxWithdrawal TxKind = "withdrawal"
	TxTransfer   TxKind = "transfer"
	TxFee        TxKind = "fee"
	TxInterest   TxKind = "interest"
)

// CustomerB is a bank customer. Named CustomerB to avoid collision with
// Customer in the same package.
type CustomerB struct {
	ID   string
	Name string
}

// Account is a ledger account owned by a customer.
type Account struct {
	ID       string
	HolderID string
	Kind     AccountKind
	Status   AccountStatus
	OpenedAt time.Time
}

// Entry is one side of a transaction; positive credits, negative debits.
type Entry struct {
	AccountID   string
	AmountCents int64
}

// Transaction is an atomic group of entries.
type Transaction struct {
	ID      uint64
	Kind    TxKind
	Entries []Entry
	Memo    string
	At      time.Time
}

// Bank errors.
var (
	ErrUnknownAccount     = errors.New("unknown account")
	ErrInvalidAmount      = errors.New("amount must be positive cents")
	ErrInsufficientFunds  = errors.New("insufficient funds")
	ErrAccountFrozen      = errors.New("account frozen")
	ErrAccountClosed      = errors.New("account closed")
	ErrNonZeroBalance     = errors.New("cannot close with non-zero balance")
	ErrUnbalancedTransfer = errors.New("transfer entries must sum to zero")
	ErrSameAccountXfer    = errors.New("transfer requires distinct accounts")
)

// Bank is a ledger-first manager. Balances are projections over the transaction log.
type Bank struct {
	Customers    map[string]CustomerB
	Accounts     map[string]*Account
	Transactions map[uint64]Transaction
	nextTxID     uint64
	nextAcctSeq  uint64
}

// NewBank builds an empty bank.
func NewBank() *Bank {
	return &Bank{
		Customers:    make(map[string]CustomerB),
		Accounts:     make(map[string]*Account),
		Transactions: make(map[uint64]Transaction),
		nextTxID:     1,
		nextAcctSeq:  1,
	}
}

// AddCustomerB registers a bank customer.
func (b *Bank) AddCustomerB(c CustomerB) error {
	if _, ok := b.Customers[c.ID]; ok {
		return fmt.Errorf("%w: customer %s", ErrDuplicate, c.ID)
	}
	b.Customers[c.ID] = c
	return nil
}

// OpenAccount opens a new account for an existing customer.
// Pass "" to auto-generate an account id.
func (b *Bank) OpenAccount(holderID string, kind AccountKind, accountID string) (*Account, error) {
	if _, ok := b.Customers[holderID]; !ok {
		return nil, fmt.Errorf("%w: customer %s", ErrUnknownCustomer, holderID)
	}
	id := accountID
	if id == "" {
		id = fmt.Sprintf("A%06d", b.nextAcctSeq)
		b.nextAcctSeq++
	}
	if _, ok := b.Accounts[id]; ok {
		return nil, fmt.Errorf("%w: account %s", ErrDuplicate, id)
	}
	a := &Account{
		ID:       id,
		HolderID: holderID,
		Kind:     kind,
		Status:   AccountOpen,
		OpenedAt: time.Now(),
	}
	b.Accounts[id] = a
	return a, nil
}

// CloseAccount closes an account with zero balance.
func (b *Bank) CloseAccount(accountID string) error {
	bal, err := b.Balance(accountID)
	if err != nil {
		return err
	}
	if bal != 0 {
		return fmt.Errorf("%w: %s has %d cents", ErrNonZeroBalance, accountID, bal)
	}
	a := b.Accounts[accountID]
	a.Status = AccountClosed
	return nil
}

// Freeze marks an account frozen, blocking new transactions.
func (b *Bank) Freeze(accountID string) error {
	a, ok := b.Accounts[accountID]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownAccount, accountID)
	}
	a.Status = AccountFrozen
	return nil
}

// Unfreeze reopens a frozen account.
func (b *Bank) Unfreeze(accountID string) error {
	a, ok := b.Accounts[accountID]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownAccount, accountID)
	}
	if a.Status == AccountFrozen {
		a.Status = AccountOpen
	}
	return nil
}

func (b *Bank) requireActive(accountID string) error {
	a, ok := b.Accounts[accountID]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownAccount, accountID)
	}
	switch a.Status {
	case AccountFrozen:
		return fmt.Errorf("%w: %s", ErrAccountFrozen, accountID)
	case AccountClosed:
		return fmt.Errorf("%w: %s", ErrAccountClosed, accountID)
	}
	return nil
}

// Deposit credits an account.
func (b *Bank) Deposit(accountID string, amountCents int64, memo string) (uint64, error) {
	if amountCents <= 0 {
		return 0, ErrInvalidAmount
	}
	if err := b.requireActive(accountID); err != nil {
		return 0, err
	}
	return b.commit(TxDeposit, []Entry{{AccountID: accountID, AmountCents: amountCents}}, memo), nil
}

// Withdraw debits an account, failing if overdraft would result.
func (b *Bank) Withdraw(accountID string, amountCents int64, memo string) (uint64, error) {
	if amountCents <= 0 {
		return 0, ErrInvalidAmount
	}
	if err := b.requireActive(accountID); err != nil {
		return 0, err
	}
	bal, _ := b.Balance(accountID)
	if bal < amountCents {
		return 0, fmt.Errorf("%w: %s has %d, requested %d", ErrInsufficientFunds, accountID, bal, amountCents)
	}
	return b.commit(TxWithdrawal, []Entry{{AccountID: accountID, AmountCents: -amountCents}}, memo), nil
}

// Transfer moves funds atomically between two accounts.
func (b *Bank) Transfer(fromID, toID string, amountCents int64, memo string) (uint64, error) {
	if amountCents <= 0 {
		return 0, ErrInvalidAmount
	}
	if fromID == toID {
		return 0, ErrSameAccountXfer
	}
	if err := b.requireActive(fromID); err != nil {
		return 0, err
	}
	if err := b.requireActive(toID); err != nil {
		return 0, err
	}
	bal, _ := b.Balance(fromID)
	if bal < amountCents {
		return 0, fmt.Errorf("%w: %s has %d, requested %d", ErrInsufficientFunds, fromID, bal, amountCents)
	}
	entries := []Entry{
		{AccountID: fromID, AmountCents: -amountCents},
		{AccountID: toID, AmountCents: amountCents},
	}
	var sum int64
	for _, e := range entries {
		sum += e.AmountCents
	}
	if sum != 0 {
		return 0, ErrUnbalancedTransfer
	}
	return b.commit(TxTransfer, entries, memo), nil
}

func (b *Bank) commit(kind TxKind, entries []Entry, memo string) uint64 {
	id := b.nextTxID
	b.nextTxID++
	b.Transactions[id] = Transaction{
		ID: id, Kind: kind, Entries: entries, Memo: memo, At: time.Now(),
	}
	return id
}

// Balance is the signed sum of all entries for an account.
func (b *Bank) Balance(accountID string) (int64, error) {
	if _, ok := b.Accounts[accountID]; !ok {
		return 0, fmt.Errorf("%w: %s", ErrUnknownAccount, accountID)
	}
	var total int64
	for _, tx := range b.Transactions {
		for _, e := range tx.Entries {
			if e.AccountID == accountID {
				total += e.AmountCents
			}
		}
	}
	return total, nil
}

// TotalAssets sums all balances for a customer's accounts.
func (b *Bank) TotalAssets(customerID string) (int64, error) {
	if _, ok := b.Customers[customerID]; !ok {
		return 0, fmt.Errorf("%w: %s", ErrUnknownCustomer, customerID)
	}
	var total int64
	for _, a := range b.Accounts {
		if a.HolderID == customerID {
			bal, _ := b.Balance(a.ID)
			total += bal
		}
	}
	return total, nil
}

// LedgerIsBalanced verifies that every transfer transaction sums to zero.
func (b *Bank) LedgerIsBalanced() bool {
	for _, tx := range b.Transactions {
		if tx.Kind != TxTransfer {
			continue
		}
		var sum int64
		for _, e := range tx.Entries {
			sum += e.AmountCents
		}
		if sum != 0 {
			return false
		}
	}
	return true
}
