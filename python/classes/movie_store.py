"""Movie Store — two entities joined by a rental, with due dates and temporal obligations."""

from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from typing import Optional
import itertools


@dataclass
class Movie:
    """A movie title. Physical copies are counted, not individually tracked."""
    id: str
    title: str
    director: str = ""
    year: int = 0
    genre: str = ""
    copies_total: int = 1


@dataclass
class Customer:
    id: str
    name: str


@dataclass
class Rental:
    """A time-bounded obligation: customer borrows a movie until due_date."""
    id: int
    movie_id: str
    customer_id: str
    rented_at: datetime
    due_date: datetime
    returned_at: Optional[datetime] = None

    @property
    def is_active(self) -> bool:
        return self.returned_at is None

    def is_overdue(self, now: datetime) -> bool:
        return self.is_active and now > self.due_date


class MovieStoreError(Exception):
    """Base for store errors."""


class NotAvailable(MovieStoreError):
    """Requested movie has no copies available."""


class UnknownEntity(MovieStoreError):
    """Referenced id not registered."""


class AlreadyReturned(MovieStoreError):
    """Rental was already returned."""


class MovieStore:
    """A rental store. Rentals are the join table; availability is aggregated."""

    DEFAULT_RENTAL_DAYS = 7

    def __init__(self) -> None:
        self.movies: dict[str, Movie] = {}
        self.customers: dict[str, Customer] = {}
        self.rentals: dict[int, Rental] = {}
        self._rental_ids = itertools.count(1)

    # --- registration ---
    def add_movie(self, movie: Movie) -> None:
        if movie.id in self.movies:
            raise ValueError(f"movie id already registered: {movie.id}")
        self.movies[movie.id] = movie

    def add_customer(self, customer: Customer) -> None:
        if customer.id in self.customers:
            raise ValueError(f"customer id already registered: {customer.id}")
        self.customers[customer.id] = customer

    def _movie(self, movie_id: str) -> Movie:
        if movie_id not in self.movies:
            raise UnknownEntity(f"unknown movie: {movie_id}")
        return self.movies[movie_id]

    def _customer(self, customer_id: str) -> Customer:
        if customer_id not in self.customers:
            raise UnknownEntity(f"unknown customer: {customer_id}")
        return self.customers[customer_id]

    # --- rentals ---
    def rent(self, movie_id: str, customer_id: str,
             duration_days: int = DEFAULT_RENTAL_DAYS,
             now: Optional[datetime] = None) -> Rental:
        self._movie(movie_id)
        self._customer(customer_id)
        if self.copies_available(movie_id) <= 0:
            raise NotAvailable(f"no copies available: {movie_id}")
        at = now or datetime.now(timezone.utc)
        rental = Rental(
            id=next(self._rental_ids),
            movie_id=movie_id,
            customer_id=customer_id,
            rented_at=at,
            due_date=at + timedelta(days=duration_days),
        )
        self.rentals[rental.id] = rental
        return rental

    def return_movie(self, rental_id: int, now: Optional[datetime] = None) -> Rental:
        if rental_id not in self.rentals:
            raise UnknownEntity(f"unknown rental: {rental_id}")
        rental = self.rentals[rental_id]
        if rental.returned_at is not None:
            raise AlreadyReturned(f"rental {rental_id} already returned")
        rental.returned_at = now or datetime.now(timezone.utc)
        return rental

    # --- aggregates ---
    def copies_available(self, movie_id: str) -> int:
        movie = self._movie(movie_id)
        active = sum(1 for r in self.rentals.values()
                     if r.movie_id == movie_id and r.is_active)
        return movie.copies_total - active

    def active_rentals(self, customer_id: str) -> list[Rental]:
        self._customer(customer_id)
        return [r for r in self.rentals.values()
                if r.customer_id == customer_id and r.is_active]

    def overdue(self, as_of: Optional[datetime] = None) -> list[Rental]:
        now = as_of or datetime.now(timezone.utc)
        return [r for r in self.rentals.values() if r.is_overdue(now)]

    def movie_history(self, movie_id: str) -> list[Rental]:
        self._movie(movie_id)
        return [r for r in self.rentals.values() if r.movie_id == movie_id]

    def customer_history(self, customer_id: str) -> list[Rental]:
        self._customer(customer_id)
        return [r for r in self.rentals.values() if r.customer_id == customer_id]

    def by_genre(self, genre: str) -> list[Movie]:
        return [m for m in self.movies.values() if m.genre == genre]

    def summary(self, as_of: Optional[datetime] = None) -> dict:
        now = as_of or datetime.now(timezone.utc)
        return {
            "movies": len(self.movies),
            "customers": len(self.customers),
            "rentals_total": len(self.rentals),
            "rentals_active": sum(1 for r in self.rentals.values() if r.is_active),
            "overdue": sum(1 for r in self.rentals.values() if r.is_overdue(now)),
        }


# --- demo ---
if __name__ == "__main__":
    store = MovieStore()
    store.add_movie(Movie("M001", "Casablanca", "Curtiz", 1942, "drama", copies_total=2))
    store.add_movie(Movie("M002", "Blade Runner", "Scott", 1982, "sci-fi", copies_total=1))
    store.add_customer(Customer("C001", "Nathan"))
    store.add_customer(Customer("C002", "Korrallan"))

    # Six days ago — a rental that's now overdue
    past = datetime.now(timezone.utc) - timedelta(days=10)
    r1 = store.rent("M001", "C001", duration_days=7, now=past)
    r2 = store.rent("M002", "C002")
    print(store.summary())
    print(f"  overdue: {[r.id for r in store.overdue()]}")
    store.return_movie(r1.id)
    print(f"  after return: {store.summary()}")
