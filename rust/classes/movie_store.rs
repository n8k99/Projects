//! Movie Store — two entities joined by a rental, with due dates and temporal obligations.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Movie {
    pub id: String,
    pub title: String,
    pub director: String,
    pub year: u32,
    pub genre: String,
    pub copies_total: u32,
}

impl Movie {
    pub fn new(id: &str, title: &str, copies_total: u32) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            director: String::new(),
            year: 0,
            genre: String::new(),
            copies_total,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Customer {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Rental {
    pub id: u64,
    pub movie_id: String,
    pub customer_id: String,
    /// Unix seconds.
    pub rented_at: u64,
    pub due_at: u64,
    /// None while active.
    pub returned_at: Option<u64>,
}

impl Rental {
    pub fn is_active(&self) -> bool {
        self.returned_at.is_none()
    }

    pub fn is_overdue(&self, now: u64) -> bool {
        self.is_active() && now > self.due_at
    }
}

#[derive(Debug)]
pub enum StoreError {
    UnknownMovie(String),
    UnknownCustomer(String),
    UnknownRental(u64),
    NotAvailable(String),
    AlreadyReturned(u64),
    Duplicate(String),
}

pub struct MovieStore {
    pub movies: HashMap<String, Movie>,
    pub customers: HashMap<String, Customer>,
    pub rentals: HashMap<u64, Rental>,
    next_rental_id: u64,
}

impl MovieStore {
    pub const DEFAULT_RENTAL_DAYS: u64 = 7;
    const SECONDS_PER_DAY: u64 = 86_400;

    pub fn new() -> Self {
        Self {
            movies: HashMap::new(),
            customers: HashMap::new(),
            rentals: HashMap::new(),
            next_rental_id: 1,
        }
    }

    pub fn add_movie(&mut self, movie: Movie) -> Result<(), StoreError> {
        if self.movies.contains_key(&movie.id) {
            return Err(StoreError::Duplicate(movie.id));
        }
        self.movies.insert(movie.id.clone(), movie);
        Ok(())
    }

    pub fn add_customer(&mut self, customer: Customer) -> Result<(), StoreError> {
        if self.customers.contains_key(&customer.id) {
            return Err(StoreError::Duplicate(customer.id));
        }
        self.customers.insert(customer.id.clone(), customer);
        Ok(())
    }

    pub fn rent(
        &mut self,
        movie_id: &str,
        customer_id: &str,
        duration_days: u64,
        now: u64,
    ) -> Result<u64, StoreError> {
        if !self.movies.contains_key(movie_id) {
            return Err(StoreError::UnknownMovie(movie_id.to_string()));
        }
        if !self.customers.contains_key(customer_id) {
            return Err(StoreError::UnknownCustomer(customer_id.to_string()));
        }
        if self.copies_available(movie_id) == 0 {
            return Err(StoreError::NotAvailable(movie_id.to_string()));
        }
        let id = self.next_rental_id;
        self.next_rental_id += 1;
        self.rentals.insert(
            id,
            Rental {
                id,
                movie_id: movie_id.to_string(),
                customer_id: customer_id.to_string(),
                rented_at: now,
                due_at: now + duration_days * Self::SECONDS_PER_DAY,
                returned_at: None,
            },
        );
        Ok(id)
    }

    pub fn return_movie(&mut self, rental_id: u64, now: u64) -> Result<(), StoreError> {
        let rental = self
            .rentals
            .get_mut(&rental_id)
            .ok_or(StoreError::UnknownRental(rental_id))?;
        if rental.returned_at.is_some() {
            return Err(StoreError::AlreadyReturned(rental_id));
        }
        rental.returned_at = Some(now);
        Ok(())
    }

    pub fn copies_available(&self, movie_id: &str) -> u32 {
        let Some(movie) = self.movies.get(movie_id) else {
            return 0;
        };
        let active: u32 = self
            .rentals
            .values()
            .filter(|r| r.movie_id == movie_id && r.is_active())
            .count() as u32;
        movie.copies_total.saturating_sub(active)
    }

    pub fn active_rentals(&self, customer_id: &str) -> Vec<&Rental> {
        self.rentals
            .values()
            .filter(|r| r.customer_id == customer_id && r.is_active())
            .collect()
    }

    pub fn overdue(&self, now: u64) -> Vec<&Rental> {
        self.rentals.values().filter(|r| r.is_overdue(now)).collect()
    }

    pub fn movie_history(&self, movie_id: &str) -> Vec<&Rental> {
        self.rentals.values().filter(|r| r.movie_id == movie_id).collect()
    }

    pub fn customer_history(&self, customer_id: &str) -> Vec<&Rental> {
        self.rentals.values().filter(|r| r.customer_id == customer_id).collect()
    }

    pub fn by_genre(&self, genre: &str) -> Vec<&Movie> {
        self.movies.values().filter(|m| m.genre == genre).collect()
    }
}

impl Default for MovieStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> MovieStore {
        let mut s = MovieStore::new();
        let mut m = Movie::new("M1", "Casablanca", 2);
        m.genre = "drama".into();
        s.add_movie(m).unwrap();
        s.add_customer(Customer { id: "C1".into(), name: "Nathan".into() }).unwrap();
        s.add_customer(Customer { id: "C2".into(), name: "Korrallan".into() }).unwrap();
        s
    }

    #[test]
    fn copies_decrement_while_rented() {
        let mut s = setup();
        assert_eq!(s.copies_available("M1"), 2);
        s.rent("M1", "C1", 7, 1000).unwrap();
        assert_eq!(s.copies_available("M1"), 1);
        s.rent("M1", "C2", 7, 1000).unwrap();
        assert_eq!(s.copies_available("M1"), 0);
    }

    #[test]
    fn no_rent_when_out_of_stock() {
        let mut s = setup();
        s.rent("M1", "C1", 7, 1000).unwrap();
        s.rent("M1", "C2", 7, 1000).unwrap();
        assert!(matches!(s.rent("M1", "C1", 7, 1000), Err(StoreError::NotAvailable(_))));
    }

    #[test]
    fn overdue_detects_past_due_active_rentals() {
        let mut s = setup();
        s.rent("M1", "C1", 7, 1000).unwrap();
        let after_due = 1000 + 8 * 86_400;
        let overdue = s.overdue(after_due);
        assert_eq!(overdue.len(), 1);
    }

    #[test]
    fn return_frees_a_copy() {
        let mut s = setup();
        let id = s.rent("M1", "C1", 7, 1000).unwrap();
        s.return_movie(id, 2000).unwrap();
        assert_eq!(s.copies_available("M1"), 2);
    }
}
