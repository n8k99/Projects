-- Movie Store — two entities joined by a rental, with due dates and temporal obligations.

namespace Classes

structure Movie where
  id : String
  title : String
  director : String := ""
  year : Nat := 0
  genre : String := ""
  copiesTotal : Nat := 1
  deriving Repr

structure Customer where
  id : String
  name : String
  deriving Repr

structure Rental where
  id : Nat
  movieId : String
  customerId : String
  rentedAt : Nat        -- unix seconds
  dueAt : Nat
  returnedAt : Option Nat := none
  deriving Repr

def Rental.isActive (r : Rental) : Bool :=
  r.returnedAt.isNone

def Rental.isOverdue (r : Rental) (now : Nat) : Bool :=
  r.isActive && now > r.dueAt

structure MovieStore where
  movies : List Movie := []
  customers : List Customer := []
  rentals : List Rental := []
  nextId : Nat := 1
  deriving Repr

def MovieStore.empty : MovieStore := { }

def MovieStore.findMovie (s : MovieStore) (id : String) : Option Movie :=
  s.movies.find? (·.id == id)

def MovieStore.findCustomer (s : MovieStore) (id : String) : Option Customer :=
  s.customers.find? (·.id == id)

def MovieStore.addMovie (s : MovieStore) (m : Movie) : Except String MovieStore :=
  if s.movies.any (·.id == m.id)
  then .error s!"duplicate movie id: {m.id}"
  else .ok { s with movies := s.movies ++ [m] }

def MovieStore.addCustomer (s : MovieStore) (c : Customer) : Except String MovieStore :=
  if s.customers.any (·.id == c.id)
  then .error s!"duplicate customer id: {c.id}"
  else .ok { s with customers := s.customers ++ [c] }

def MovieStore.copiesAvailable (s : MovieStore) (movieId : String) : Nat :=
  match s.findMovie movieId with
  | none   => 0
  | some m =>
    let active := (s.rentals.filter (fun r => r.movieId == movieId && r.isActive)).length
    if active > m.copiesTotal then 0 else m.copiesTotal - active

def secondsPerDay : Nat := 86400

def MovieStore.rent (s : MovieStore) (movieId customerId : String)
    (durationDays now : Nat) : Except String (MovieStore × Rental) :=
  if (s.findMovie movieId).isNone then
    .error s!"unknown movie: {movieId}"
  else if (s.findCustomer customerId).isNone then
    .error s!"unknown customer: {customerId}"
  else if s.copiesAvailable movieId == 0 then
    .error s!"no copies available: {movieId}"
  else
    let r : Rental := {
      id := s.nextId,
      movieId, customerId,
      rentedAt := now,
      dueAt := now + durationDays * secondsPerDay,
      returnedAt := none
    }
    .ok ({ s with rentals := s.rentals ++ [r], nextId := s.nextId + 1 }, r)

def MovieStore.returnMovie (s : MovieStore) (rentalId now : Nat)
    : Except String MovieStore :=
  match s.rentals.find? (·.id == rentalId) with
  | none => .error s!"unknown rental: {rentalId}"
  | some r =>
    if r.returnedAt.isSome then
      .error s!"rental {rentalId} already returned"
    else
      let updated := s.rentals.map fun x =>
        if x.id == rentalId then { x with returnedAt := some now } else x
      .ok { s with rentals := updated }

def MovieStore.activeRentals (s : MovieStore) (customerId : String) : List Rental :=
  s.rentals.filter fun r => r.customerId == customerId && r.isActive

def MovieStore.overdue (s : MovieStore) (now : Nat) : List Rental :=
  s.rentals.filter (·.isOverdue now)

def MovieStore.movieHistory (s : MovieStore) (movieId : String) : List Rental :=
  s.rentals.filter (·.movieId == movieId)

def MovieStore.customerHistory (s : MovieStore) (customerId : String) : List Rental :=
  s.rentals.filter (·.customerId == customerId)

def MovieStore.byGenre (s : MovieStore) (genre : String) : List Movie :=
  s.movies.filter (·.genre == genre)

end Classes
