-- Library Catalog — three-layer identity, hierarchical subjects, FIFO holds, renewal.

namespace Classes

inductive CopyStatus
  | available
  | checkedOut
  | lost
  | withdrawn
  deriving Repr, DecidableEq

inductive HoldStatus
  | waiting
  | fulfilled
  | cancelled
  deriving Repr, DecidableEq

structure Title where
  id : String
  name : String
  authors : List String := []
  subjectPath : String := ""
  isbn : String := ""
  deriving Repr

structure LibCopy where
  id : String
  titleId : String
  location : String := ""
  status : CopyStatus := .available
  deriving Repr

structure Patron where
  id : String
  name : String
  deriving Repr

structure LibLoan where
  id : Nat
  copyId : String
  patronId : String
  checkedOutAt : Nat
  dueAt : Nat
  returnedAt : Option Nat := none
  renewalCount : Nat := 0
  deriving Repr

def LibLoan.isActive (l : LibLoan) : Bool := l.returnedAt.isNone

def LibLoan.isOverdue (l : LibLoan) (now : Nat) : Bool :=
  l.isActive && now > l.dueAt

structure Hold where
  id : Nat
  titleId : String
  patronId : String
  placedAt : Nat
  status : HoldStatus := .waiting
  fulfilledCopyId : Option String := none
  deriving Repr

structure LibraryCatalog where
  titles : List Title := []
  copies : List LibCopy := []
  patrons : List Patron := []
  loans : List LibLoan := []
  holds : List Hold := []
  nextLoanId : Nat := 1
  nextHoldId : Nat := 1
  deriving Repr

def LibraryCatalog.empty : LibraryCatalog := { }

def secondsPerDayLib : Nat := 86400

def LibraryCatalog.findTitle (c : LibraryCatalog) (id : String) : Option Title :=
  c.titles.find? (·.id == id)

def LibraryCatalog.findCopy (c : LibraryCatalog) (id : String) : Option LibCopy :=
  c.copies.find? (·.id == id)

def LibraryCatalog.findPatron (c : LibraryCatalog) (id : String) : Option Patron :=
  c.patrons.find? (·.id == id)

def LibraryCatalog.addTitle (c : LibraryCatalog) (t : Title) : Except String LibraryCatalog :=
  if c.titles.any (·.id == t.id)
  then .error s!"duplicate title: {t.id}"
  else .ok { c with titles := c.titles ++ [t] }

def LibraryCatalog.addCopy (c : LibraryCatalog) (cp : LibCopy) : Except String LibraryCatalog :=
  if (c.findTitle cp.titleId).isNone then
    .error s!"unknown title: {cp.titleId}"
  else if c.copies.any (·.id == cp.id) then
    .error s!"duplicate copy: {cp.id}"
  else
    .ok { c with copies := c.copies ++ [cp] }

def LibraryCatalog.addPatron (c : LibraryCatalog) (p : Patron) : Except String LibraryCatalog :=
  if c.patrons.any (·.id == p.id)
  then .error s!"duplicate patron: {p.id}"
  else .ok { c with patrons := c.patrons ++ [p] }

private def updateCopy (cs : List LibCopy) (id : String) (status : CopyStatus) : List LibCopy :=
  cs.map fun c => if c.id == id then { c with status := status } else c

def LibraryCatalog.checkOut (c : LibraryCatalog) (copyId patronId : String)
    (durationDays now : Nat) : Except String (LibraryCatalog × LibLoan) := do
  if (c.findPatron patronId).isNone then
    throw s!"unknown patron: {patronId}"
  let cp ← (c.findCopy copyId).elim (throw s!"unknown copy: {copyId}") pure
  if cp.status ≠ .available then
    throw s!"copy {copyId} not available"
  let l : LibLoan := {
    id := c.nextLoanId, copyId, patronId,
    checkedOutAt := now, dueAt := now + durationDays * secondsPerDayLib,
    returnedAt := none, renewalCount := 0
  }
  pure ({ c with
    copies := updateCopy c.copies copyId .checkedOut,
    loans := c.loans ++ [l],
    nextLoanId := c.nextLoanId + 1
  }, l)

private def fulfillNextHold (c : LibraryCatalog) (titleId copyId : String)
    : LibraryCatalog × Option Hold :=
  let waiting := c.holds.filter fun h => h.titleId == titleId && h.status == .waiting
  let sorted := waiting.mergeSort (·.placedAt ≤ ·.placedAt)
  match sorted.head? with
  | none => (c, none)
  | some first =>
    let updatedHolds := c.holds.map fun h =>
      if h.id == first.id then
        { h with status := .fulfilled, fulfilledCopyId := some copyId }
      else h
    let updatedFirst : Hold :=
      { first with status := .fulfilled, fulfilledCopyId := some copyId }
    ({ c with holds := updatedHolds }, some updatedFirst)

def LibraryCatalog.returnCopy (c : LibraryCatalog) (loanId now : Nat)
    : Except String (LibraryCatalog × Option Hold) := do
  let l ← (c.loans.find? (·.id == loanId)).elim (throw s!"unknown loan: {loanId}") pure
  if l.returnedAt.isSome then
    throw s!"loan {loanId} already returned"
  let cp ← (c.findCopy l.copyId).elim (throw s!"unknown copy: {l.copyId}") pure
  let updatedLoans := c.loans.map fun x =>
    if x.id == loanId then { x with returnedAt := some now } else x
  let afterReturn : LibraryCatalog := {
    c with
    loans := updatedLoans,
    copies := updateCopy c.copies l.copyId .available
  }
  pure (fulfillNextHold afterReturn cp.titleId cp.id)

def LibraryCatalog.renew (c : LibraryCatalog) (loanId additionalDays : Nat)
    : Except String LibraryCatalog := do
  let l ← (c.loans.find? (·.id == loanId)).elim (throw s!"unknown loan: {loanId}") pure
  if ¬ l.isActive then throw s!"loan {loanId} is not active"
  if l.renewalCount ≥ 2 then throw s!"loan {loanId} at max renewals"
  let cp ← (c.findCopy l.copyId).elim (throw "unknown copy") pure
  let hasWaiting := c.holds.any fun h => h.titleId == cp.titleId && h.status == .waiting
  if hasWaiting then throw "cannot renew: other patrons have holds"
  let updated := c.loans.map fun x =>
    if x.id == loanId
    then { x with
           dueAt := x.dueAt + additionalDays * secondsPerDayLib,
           renewalCount := x.renewalCount + 1 }
    else x
  pure { c with loans := updated }

def LibraryCatalog.placeHold (c : LibraryCatalog) (titleId patronId : String) (now : Nat)
    : Except String (LibraryCatalog × Hold) := do
  if (c.findTitle titleId).isNone then throw s!"unknown title: {titleId}"
  if (c.findPatron patronId).isNone then throw s!"unknown patron: {patronId}"
  let h : Hold := {
    id := c.nextHoldId, titleId, patronId, placedAt := now,
    status := .waiting, fulfilledCopyId := none
  }
  pure ({ c with holds := c.holds ++ [h], nextHoldId := c.nextHoldId + 1 }, h)

def LibraryCatalog.holdQueue (c : LibraryCatalog) (titleId : String) : List Hold :=
  (c.holds.filter fun h => h.titleId == titleId && h.status == .waiting).mergeSort
    (·.placedAt ≤ ·.placedAt)

def LibraryCatalog.copiesOf (c : LibraryCatalog) (titleId : String) : List LibCopy :=
  c.copies.filter (·.titleId == titleId)

def LibraryCatalog.availableCopies (c : LibraryCatalog) (titleId : String) : List LibCopy :=
  c.copies.filter fun x => x.titleId == titleId && x.status == .available

def LibraryCatalog.isAvailable (c : LibraryCatalog) (titleId : String) : Bool :=
  ¬ (c.availableCopies titleId).isEmpty

private def subjectPrefixMatch (path prefix : String) : Bool :=
  path == prefix || (path.startsWith (prefix ++ "."))

def LibraryCatalog.searchBySubject (c : LibraryCatalog) (prefix : String) : List Title :=
  c.titles.filter fun t => subjectPrefixMatch t.subjectPath prefix

def LibraryCatalog.searchByAuthor (c : LibraryCatalog) (author : String) : List Title :=
  c.titles.filter fun t => t.authors.contains author

def LibraryCatalog.overdue (c : LibraryCatalog) (now : Nat) : List LibLoan :=
  c.loans.filter (·.isOverdue now)

end Classes
