"""Bank Account Manager — double-entry ledger, atomic transactions, balance as aggregate.

A transaction is a list of entries, each a signed amount against one account.
Transfers have two entries whose amounts sum to zero (conservation). Deposits
and withdrawals have one entry — money crossing the bank's boundary. Balance
is not stored; it is the signed sum of all entries for an account (same
event-sourcing pattern as G048, now with multi-sided events).

Money is integer cents throughout. No float arithmetic on money, ever.
"""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Optional
import itertools


class AccountKind(Enum):
    CHECKING = "checking"
    SAVINGS = "savings"
    CREDIT = "credit"


class AccountStatus(Enum):
    OPEN = "open"
    FROZEN = "frozen"
    CLOSED = "closed"


class TxKind(Enum):
    DEPOSIT = "deposit"
    WITHDRAWAL = "withdrawal"
    TRANSFER = "transfer"
    FEE = "fee"
    INTEREST = "interest"


@dataclass
class Customer:
    id: str
    name: str


@dataclass
class Account:
    id: str
    holder_id: str
    kind: AccountKind
    status: AccountStatus = AccountStatus.OPEN
    opened_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))


@dataclass
class Entry:
    """One side of a transaction. Positive credits the account, negative debits it."""
    account_id: str
    amount_cents: int


@dataclass
class Transaction:
    id: int
    kind: TxKind
    entries: list[Entry]
    memo: str = ""
    at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))


class BankError(Exception):
    """Base for bank errors."""


class UnknownEntity(BankError): ...
class InsufficientFunds(BankError): ...
class AccountFrozen(BankError): ...
class AccountClosed(BankError): ...
class NonZeroBalance(BankError): ...
class UnbalancedTransfer(BankError): ...


class Bank:
    """Ledger-first bank. Balances are projections over the transaction log."""

    def __init__(self) -> None:
        self.customers: dict[str, Customer] = {}
        self.accounts: dict[str, Account] = {}
        self.transactions: dict[int, Transaction] = {}
        self._tx_ids = itertools.count(1)
        self._acct_ids = itertools.count(1)

    # --- registration ---
    def add_customer(self, customer: Customer) -> None:
        if customer.id in self.customers:
            raise ValueError(f"customer already registered: {customer.id}")
        self.customers[customer.id] = customer

    def open_account(self, holder_id: str, kind: AccountKind,
                     account_id: Optional[str] = None) -> Account:
        if holder_id not in self.customers:
            raise UnknownEntity(f"unknown customer: {holder_id}")
        aid = account_id or f"A{next(self._acct_ids):06d}"
        if aid in self.accounts:
            raise ValueError(f"account id already in use: {aid}")
        account = Account(id=aid, holder_id=holder_id, kind=kind)
        self.accounts[aid] = account
        return account

    def close_account(self, account_id: str) -> Account:
        account = self._require_account(account_id)
        if self.balance(account_id) != 0:
            raise NonZeroBalance(
                f"cannot close {account_id} with balance {self.balance(account_id)} cents"
            )
        account.status = AccountStatus.CLOSED
        return account

    def freeze(self, account_id: str) -> Account:
        account = self._require_account(account_id)
        account.status = AccountStatus.FROZEN
        return account

    def unfreeze(self, account_id: str) -> Account:
        account = self._require_account(account_id)
        if account.status == AccountStatus.FROZEN:
            account.status = AccountStatus.OPEN
        return account

    def _require_account(self, account_id: str) -> Account:
        if account_id not in self.accounts:
            raise UnknownEntity(f"unknown account: {account_id}")
        return self.accounts[account_id]

    def _require_active(self, account_id: str) -> Account:
        account = self._require_account(account_id)
        if account.status == AccountStatus.FROZEN:
            raise AccountFrozen(f"account frozen: {account_id}")
        if account.status == AccountStatus.CLOSED:
            raise AccountClosed(f"account closed: {account_id}")
        return account

    # --- transactions ---
    def deposit(self, account_id: str, amount_cents: int, memo: str = "") -> Transaction:
        if amount_cents <= 0:
            raise ValueError("deposit amount must be positive cents")
        self._require_active(account_id)
        return self._commit(TxKind.DEPOSIT, [Entry(account_id, amount_cents)], memo)

    def withdraw(self, account_id: str, amount_cents: int, memo: str = "") -> Transaction:
        if amount_cents <= 0:
            raise ValueError("withdraw amount must be positive cents")
        self._require_active(account_id)
        if self.balance(account_id) < amount_cents:
            raise InsufficientFunds(
                f"{account_id}: balance {self.balance(account_id)}, requested {amount_cents}"
            )
        return self._commit(TxKind.WITHDRAWAL, [Entry(account_id, -amount_cents)], memo)

    def transfer(self, from_id: str, to_id: str, amount_cents: int,
                 memo: str = "") -> Transaction:
        if amount_cents <= 0:
            raise ValueError("transfer amount must be positive cents")
        if from_id == to_id:
            raise ValueError("transfer requires distinct accounts")
        self._require_active(from_id)
        self._require_active(to_id)
        if self.balance(from_id) < amount_cents:
            raise InsufficientFunds(
                f"{from_id}: balance {self.balance(from_id)}, requested {amount_cents}"
            )
        entries = [Entry(from_id, -amount_cents), Entry(to_id, amount_cents)]
        if sum(e.amount_cents for e in entries) != 0:
            raise UnbalancedTransfer("transfer entries must sum to zero")
        return self._commit(TxKind.TRANSFER, entries, memo)

    def _commit(self, kind: TxKind, entries: list[Entry], memo: str) -> Transaction:
        tx = Transaction(id=next(self._tx_ids), kind=kind, entries=entries, memo=memo)
        self.transactions[tx.id] = tx
        return tx

    # --- aggregates ---
    def balance(self, account_id: str) -> int:
        self._require_account(account_id)
        total = 0
        for tx in self.transactions.values():
            for e in tx.entries:
                if e.account_id == account_id:
                    total += e.amount_cents
        return total

    def statement(self, account_id: str,
                  from_time: Optional[datetime] = None,
                  to_time: Optional[datetime] = None) -> list[Transaction]:
        self._require_account(account_id)
        result: list[Transaction] = []
        for tx in self.transactions.values():
            if not any(e.account_id == account_id for e in tx.entries):
                continue
            if from_time is not None and tx.at < from_time:
                continue
            if to_time is not None and tx.at > to_time:
                continue
            result.append(tx)
        return sorted(result, key=lambda t: t.at)

    def total_assets(self, customer_id: str) -> int:
        if customer_id not in self.customers:
            raise UnknownEntity(f"unknown customer: {customer_id}")
        return sum(self.balance(a.id) for a in self.accounts.values()
                   if a.holder_id == customer_id)

    def ledger_is_balanced(self) -> bool:
        """Global invariant: the sum of all transfer entries is zero.

        Deposits and withdrawals cross the bank boundary and do not balance
        internally; transfers must. This check isolates transfers.
        """
        total = 0
        for tx in self.transactions.values():
            if tx.kind == TxKind.TRANSFER:
                total += sum(e.amount_cents for e in tx.entries)
        return total == 0


# --- demo ---
if __name__ == "__main__":
    bank = Bank()
    bank.add_customer(Customer("C001", "Nathan"))
    bank.add_customer(Customer("C002", "Korrallan"))

    checking = bank.open_account("C001", AccountKind.CHECKING, account_id="ACC-CHK-1")
    savings  = bank.open_account("C001", AccountKind.SAVINGS,  account_id="ACC-SAV-1")
    joint    = bank.open_account("C002", AccountKind.CHECKING, account_id="ACC-CHK-2")

    bank.deposit(checking.id, 500_00, memo="opening deposit")
    bank.transfer(checking.id, savings.id, 200_00, memo="monthly savings")
    bank.transfer(checking.id, joint.id,   75_00,  memo="groceries split")

    try:
        bank.withdraw(checking.id, 1_000_00, memo="big ATM")
    except InsufficientFunds as e:
        print(f"refused: {e}")

    print(f"checking: {bank.balance(checking.id)} cents")
    print(f"savings:  {bank.balance(savings.id)}  cents")
    print(f"joint:    {bank.balance(joint.id)}    cents")
    print(f"Nathan total assets: {bank.total_assets('C001')} cents")
    print(f"ledger balanced (transfers sum to zero): {bank.ledger_is_balanced()}")
