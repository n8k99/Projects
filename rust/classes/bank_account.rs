//! Bank Account Manager — double-entry ledger, atomic transactions, balance as aggregate.
//!
//! A transaction is a list of entries, each a signed amount against one account.
//! Transfers sum to zero (conservation). Deposits and withdrawals cross the bank
//! boundary as single entries. Balance is the signed sum of an account's entries.
//! Money is integer cents throughout.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountKind {
    Checking,
    Savings,
    Credit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountStatus {
    Open,
    Frozen,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxKind {
    Deposit,
    Withdrawal,
    Transfer,
    Fee,
    Interest,
}

#[derive(Debug, Clone)]
pub struct Customer {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub holder_id: String,
    pub kind: AccountKind,
    pub status: AccountStatus,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub account_id: String,
    pub amount_cents: i64,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u64,
    pub kind: TxKind,
    pub entries: Vec<Entry>,
    pub memo: String,
}

#[derive(Debug)]
pub enum BankError {
    UnknownCustomer(String),
    UnknownAccount(String),
    Duplicate(String),
    InvalidAmount,
    InsufficientFunds { account_id: String, balance: i64, requested: i64 },
    AccountFrozen(String),
    AccountClosed(String),
    NonZeroBalance { account_id: String, balance: i64 },
    UnbalancedTransfer,
    SameAccountTransfer,
}

pub struct Bank {
    pub customers: HashMap<String, Customer>,
    pub accounts: HashMap<String, Account>,
    pub transactions: HashMap<u64, Transaction>,
    next_tx_id: u64,
    next_account_seq: u64,
}

impl Bank {
    pub fn new() -> Self {
        Self {
            customers: HashMap::new(),
            accounts: HashMap::new(),
            transactions: HashMap::new(),
            next_tx_id: 1,
            next_account_seq: 1,
        }
    }

    pub fn add_customer(&mut self, customer: Customer) -> Result<(), BankError> {
        if self.customers.contains_key(&customer.id) {
            return Err(BankError::Duplicate(customer.id));
        }
        self.customers.insert(customer.id.clone(), customer);
        Ok(())
    }

    pub fn open_account(
        &mut self,
        holder_id: &str,
        kind: AccountKind,
        account_id: Option<&str>,
    ) -> Result<String, BankError> {
        if !self.customers.contains_key(holder_id) {
            return Err(BankError::UnknownCustomer(holder_id.to_string()));
        }
        let id = match account_id {
            Some(s) => s.to_string(),
            None => {
                let n = self.next_account_seq;
                self.next_account_seq += 1;
                format!("A{:06}", n)
            }
        };
        if self.accounts.contains_key(&id) {
            return Err(BankError::Duplicate(id));
        }
        self.accounts.insert(
            id.clone(),
            Account {
                id: id.clone(),
                holder_id: holder_id.to_string(),
                kind,
                status: AccountStatus::Open,
            },
        );
        Ok(id)
    }

    pub fn close_account(&mut self, account_id: &str) -> Result<(), BankError> {
        let balance = self.balance(account_id)?;
        if balance != 0 {
            return Err(BankError::NonZeroBalance {
                account_id: account_id.to_string(),
                balance,
            });
        }
        let account = self
            .accounts
            .get_mut(account_id)
            .ok_or_else(|| BankError::UnknownAccount(account_id.to_string()))?;
        account.status = AccountStatus::Closed;
        Ok(())
    }

    pub fn freeze(&mut self, account_id: &str) -> Result<(), BankError> {
        let account = self
            .accounts
            .get_mut(account_id)
            .ok_or_else(|| BankError::UnknownAccount(account_id.to_string()))?;
        account.status = AccountStatus::Frozen;
        Ok(())
    }

    pub fn unfreeze(&mut self, account_id: &str) -> Result<(), BankError> {
        let account = self
            .accounts
            .get_mut(account_id)
            .ok_or_else(|| BankError::UnknownAccount(account_id.to_string()))?;
        if account.status == AccountStatus::Frozen {
            account.status = AccountStatus::Open;
        }
        Ok(())
    }

    fn require_active(&self, account_id: &str) -> Result<(), BankError> {
        let account = self
            .accounts
            .get(account_id)
            .ok_or_else(|| BankError::UnknownAccount(account_id.to_string()))?;
        match account.status {
            AccountStatus::Open => Ok(()),
            AccountStatus::Frozen => Err(BankError::AccountFrozen(account_id.to_string())),
            AccountStatus::Closed => Err(BankError::AccountClosed(account_id.to_string())),
        }
    }

    pub fn deposit(&mut self, account_id: &str, amount_cents: i64, memo: &str) -> Result<u64, BankError> {
        if amount_cents <= 0 {
            return Err(BankError::InvalidAmount);
        }
        self.require_active(account_id)?;
        Ok(self.commit(
            TxKind::Deposit,
            vec![Entry { account_id: account_id.to_string(), amount_cents }],
            memo,
        ))
    }

    pub fn withdraw(&mut self, account_id: &str, amount_cents: i64, memo: &str) -> Result<u64, BankError> {
        if amount_cents <= 0 {
            return Err(BankError::InvalidAmount);
        }
        self.require_active(account_id)?;
        let balance = self.balance(account_id)?;
        if balance < amount_cents {
            return Err(BankError::InsufficientFunds {
                account_id: account_id.to_string(),
                balance,
                requested: amount_cents,
            });
        }
        Ok(self.commit(
            TxKind::Withdrawal,
            vec![Entry { account_id: account_id.to_string(), amount_cents: -amount_cents }],
            memo,
        ))
    }

    pub fn transfer(
        &mut self,
        from_id: &str,
        to_id: &str,
        amount_cents: i64,
        memo: &str,
    ) -> Result<u64, BankError> {
        if amount_cents <= 0 {
            return Err(BankError::InvalidAmount);
        }
        if from_id == to_id {
            return Err(BankError::SameAccountTransfer);
        }
        self.require_active(from_id)?;
        self.require_active(to_id)?;
        let balance = self.balance(from_id)?;
        if balance < amount_cents {
            return Err(BankError::InsufficientFunds {
                account_id: from_id.to_string(),
                balance,
                requested: amount_cents,
            });
        }
        let entries = vec![
            Entry { account_id: from_id.to_string(), amount_cents: -amount_cents },
            Entry { account_id: to_id.to_string(), amount_cents },
        ];
        let sum: i64 = entries.iter().map(|e| e.amount_cents).sum();
        if sum != 0 {
            return Err(BankError::UnbalancedTransfer);
        }
        Ok(self.commit(TxKind::Transfer, entries, memo))
    }

    fn commit(&mut self, kind: TxKind, entries: Vec<Entry>, memo: &str) -> u64 {
        let id = self.next_tx_id;
        self.next_tx_id += 1;
        self.transactions.insert(
            id,
            Transaction { id, kind, entries, memo: memo.to_string() },
        );
        id
    }

    pub fn balance(&self, account_id: &str) -> Result<i64, BankError> {
        if !self.accounts.contains_key(account_id) {
            return Err(BankError::UnknownAccount(account_id.to_string()));
        }
        Ok(self
            .transactions
            .values()
            .flat_map(|t| t.entries.iter())
            .filter(|e| e.account_id == account_id)
            .map(|e| e.amount_cents)
            .sum())
    }

    pub fn total_assets(&self, customer_id: &str) -> Result<i64, BankError> {
        if !self.customers.contains_key(customer_id) {
            return Err(BankError::UnknownCustomer(customer_id.to_string()));
        }
        let mut total = 0;
        for a in self.accounts.values() {
            if a.holder_id == customer_id {
                total += self.balance(&a.id)?;
            }
        }
        Ok(total)
    }

    /// Global invariant: all transfer transactions must sum to zero.
    pub fn ledger_is_balanced(&self) -> bool {
        self.transactions
            .values()
            .filter(|t| t.kind == TxKind::Transfer)
            .all(|t| t.entries.iter().map(|e| e.amount_cents).sum::<i64>() == 0)
    }
}

impl Default for Bank {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Bank, String, String, String) {
        let mut b = Bank::new();
        b.add_customer(Customer { id: "C1".into(), name: "A".into() }).unwrap();
        b.add_customer(Customer { id: "C2".into(), name: "B".into() }).unwrap();
        let a1 = b.open_account("C1", AccountKind::Checking, Some("A1")).unwrap();
        let a2 = b.open_account("C1", AccountKind::Savings, Some("A2")).unwrap();
        let a3 = b.open_account("C2", AccountKind::Checking, Some("A3")).unwrap();
        (b, a1, a2, a3)
    }

    #[test]
    fn deposit_increases_balance() {
        let (mut b, a1, _, _) = setup();
        b.deposit(&a1, 10_000, "").unwrap();
        assert_eq!(b.balance(&a1).unwrap(), 10_000);
    }

    #[test]
    fn withdraw_reduces_balance() {
        let (mut b, a1, _, _) = setup();
        b.deposit(&a1, 10_000, "").unwrap();
        b.withdraw(&a1, 3_000, "").unwrap();
        assert_eq!(b.balance(&a1).unwrap(), 7_000);
    }

    #[test]
    fn withdraw_refuses_overdraft() {
        let (mut b, a1, _, _) = setup();
        b.deposit(&a1, 1_000, "").unwrap();
        assert!(matches!(
            b.withdraw(&a1, 2_000, ""),
            Err(BankError::InsufficientFunds { .. })
        ));
    }

    #[test]
    fn transfer_conserves_total() {
        let (mut b, a1, a2, _) = setup();
        b.deposit(&a1, 10_000, "").unwrap();
        b.transfer(&a1, &a2, 4_000, "").unwrap();
        assert_eq!(b.balance(&a1).unwrap(), 6_000);
        assert_eq!(b.balance(&a2).unwrap(), 4_000);
        assert!(b.ledger_is_balanced());
    }

    #[test]
    fn frozen_account_rejects_transactions() {
        let (mut b, a1, _, _) = setup();
        b.deposit(&a1, 1_000, "").unwrap();
        b.freeze(&a1).unwrap();
        assert!(matches!(b.deposit(&a1, 100, ""), Err(BankError::AccountFrozen(_))));
        assert!(matches!(b.withdraw(&a1, 100, ""), Err(BankError::AccountFrozen(_))));
    }

    #[test]
    fn close_requires_zero_balance() {
        let (mut b, a1, _, _) = setup();
        b.deposit(&a1, 100, "").unwrap();
        assert!(matches!(
            b.close_account(&a1),
            Err(BankError::NonZeroBalance { .. })
        ));
        b.withdraw(&a1, 100, "").unwrap();
        b.close_account(&a1).unwrap();
        assert!(matches!(b.deposit(&a1, 100, ""), Err(BankError::AccountClosed(_))));
    }
}
