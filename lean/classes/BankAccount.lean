-- Bank Account Manager — double-entry ledger, atomic transactions, balance as aggregate.

namespace Classes

inductive AccountKind
  | checking
  | savings
  | credit
  deriving Repr, DecidableEq

inductive AccountStatus
  | open
  | frozen
  | closed
  deriving Repr, DecidableEq

inductive TxKind
  | deposit
  | withdrawal
  | transfer
  | fee
  | interest
  deriving Repr, DecidableEq

structure BankCustomer where
  id : String
  name : String
  deriving Repr

structure Account where
  id : String
  holderId : String
  kind : AccountKind
  status : AccountStatus := .open
  deriving Repr

structure Entry where
  accountId : String
  amountCents : Int
  deriving Repr

structure Transaction where
  id : Nat
  kind : TxKind
  entries : List Entry
  memo : String := ""
  at : Nat := 0
  deriving Repr

structure Bank where
  customers : List BankCustomer := []
  accounts : List Account := []
  transactions : List Transaction := []
  nextTxId : Nat := 1
  nextAcctSeq : Nat := 1
  deriving Repr

def Bank.empty : Bank := { }

def Bank.findCustomer (b : Bank) (id : String) : Option BankCustomer :=
  b.customers.find? (·.id == id)

def Bank.findAccount (b : Bank) (id : String) : Option Account :=
  b.accounts.find? (·.id == id)

def Bank.addCustomer (b : Bank) (c : BankCustomer) : Except String Bank :=
  if b.customers.any (·.id == c.id)
  then .error s!"duplicate customer: {c.id}"
  else .ok { b with customers := b.customers ++ [c] }

def Bank.openAccount (b : Bank) (holderId : String) (kind : AccountKind)
    (accountId : Option String := none) : Except String (Bank × Account) :=
  if (b.findCustomer holderId).isNone then
    .error s!"unknown customer: {holderId}"
  else
    let id := match accountId with
      | some s => s
      | none => s!"A{b.nextAcctSeq}"
    if b.accounts.any (·.id == id) then
      .error s!"duplicate account id: {id}"
    else
      let a : Account := { id, holderId, kind, status := .open }
      let nextSeq := if accountId.isSome then b.nextAcctSeq else b.nextAcctSeq + 1
      .ok ({ b with accounts := b.accounts ++ [a], nextAcctSeq := nextSeq }, a)

def Bank.balance (b : Bank) (accountId : String) : Int :=
  b.transactions.foldl
    (fun acc tx => acc + tx.entries.foldl
      (fun a e => if e.accountId == accountId then a + e.amountCents else a) 0)
    0

private def Bank.requireActive (b : Bank) (accountId : String) : Except String Unit :=
  match b.findAccount accountId with
  | none => .error s!"unknown account: {accountId}"
  | some a =>
    match a.status with
    | .open => .ok ()
    | .frozen => .error s!"account frozen: {accountId}"
    | .closed => .error s!"account closed: {accountId}"

private def Bank.commit (b : Bank) (kind : TxKind) (entries : List Entry) (memo : String)
    : Bank × Transaction :=
  let tx : Transaction := { id := b.nextTxId, kind, entries, memo, at := 0 }
  ({ b with transactions := b.transactions ++ [tx], nextTxId := b.nextTxId + 1 }, tx)

def Bank.deposit (b : Bank) (accountId : String) (amountCents : Int) (memo : String := "")
    : Except String (Bank × Transaction) := do
  if amountCents ≤ 0 then throw "deposit amount must be positive cents"
  b.requireActive accountId
  pure (b.commit .deposit [{ accountId, amountCents }] memo)

def Bank.withdraw (b : Bank) (accountId : String) (amountCents : Int) (memo : String := "")
    : Except String (Bank × Transaction) := do
  if amountCents ≤ 0 then throw "withdraw amount must be positive cents"
  b.requireActive accountId
  let bal := b.balance accountId
  if bal < amountCents then
    throw s!"insufficient funds: {accountId} has {bal}, requested {amountCents}"
  pure (b.commit .withdrawal [{ accountId, amountCents := -amountCents }] memo)

def Bank.transfer (b : Bank) (fromId toId : String) (amountCents : Int) (memo : String := "")
    : Except String (Bank × Transaction) := do
  if amountCents ≤ 0 then throw "transfer amount must be positive cents"
  if fromId == toId then throw "transfer requires distinct accounts"
  b.requireActive fromId
  b.requireActive toId
  let bal := b.balance fromId
  if bal < amountCents then
    throw s!"insufficient funds: {fromId} has {bal}, requested {amountCents}"
  let entries : List Entry := [
    { accountId := fromId, amountCents := -amountCents },
    { accountId := toId, amountCents }
  ]
  let sum := entries.foldl (fun a e => a + e.amountCents) 0
  if sum ≠ 0 then throw "transfer entries must sum to zero"
  pure (b.commit .transfer entries memo)

def Bank.freeze (b : Bank) (accountId : String) : Except String Bank :=
  if (b.findAccount accountId).isNone then
    .error s!"unknown account: {accountId}"
  else
    .ok { b with accounts := b.accounts.map fun a =>
      if a.id == accountId then { a with status := .frozen } else a }

def Bank.unfreeze (b : Bank) (accountId : String) : Except String Bank :=
  if (b.findAccount accountId).isNone then
    .error s!"unknown account: {accountId}"
  else
    .ok { b with accounts := b.accounts.map fun a =>
      if a.id == accountId && a.status == .frozen then { a with status := .open } else a }

def Bank.closeAccount (b : Bank) (accountId : String) : Except String Bank :=
  if (b.findAccount accountId).isNone then
    .error s!"unknown account: {accountId}"
  else
    let bal := b.balance accountId
    if bal ≠ 0 then
      .error s!"cannot close {accountId} with balance {bal}"
    else
      .ok { b with accounts := b.accounts.map fun a =>
        if a.id == accountId then { a with status := .closed } else a }

def Bank.statement (b : Bank) (accountId : String) : List Transaction :=
  b.transactions.filter fun tx => tx.entries.any (·.accountId == accountId)

def Bank.totalAssets (b : Bank) (customerId : String) : Int :=
  b.accounts.foldl
    (fun acc a => if a.holderId == customerId then acc + b.balance a.id else acc) 0

def Bank.ledgerBalanced (b : Bank) : Bool :=
  b.transactions.all fun tx =>
    match tx.kind with
    | .transfer => (tx.entries.foldl (fun a e => a + e.amountCents) 0) == 0
    | _ => true

end Classes
