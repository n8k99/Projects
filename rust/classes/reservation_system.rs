//! Reservation System — bookable resources with time intervals and conflict detection.
//!
//! Models airline seats and hotel rooms uniformly. Overlap is the core primitive:
//! two reservations on the same resource collide iff their half-open intervals
//! [start, end) overlap.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Resource {
    pub id: String,
    pub kind: String,
    pub label: String,
}

impl Resource {
    pub fn new(id: &str, kind: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            kind: kind.to_string(),
            label: label.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Customer {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReservationStatus {
    Booked,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Reservation {
    pub id: u64,
    pub resource_id: String,
    pub customer_id: String,
    /// Unix seconds.
    pub start: u64,
    pub end: u64,
    pub status: ReservationStatus,
}

impl Reservation {
    pub fn is_active(&self) -> bool {
        self.status == ReservationStatus::Booked
    }

    /// Half-open overlap: reservations touching at a boundary do not conflict.
    pub fn overlaps(&self, start: u64, end: u64) -> bool {
        self.start < end && start < self.end
    }
}

#[derive(Debug)]
pub enum ReservationError {
    UnknownResource(String),
    UnknownCustomer(String),
    UnknownReservation(u64),
    InvalidInterval,
    Conflict { resource_id: String, colliding: Vec<u64> },
    AlreadyCancelled(u64),
    Duplicate(String),
}

pub struct ReservationSystem {
    pub resources: HashMap<String, Resource>,
    pub customers: HashMap<String, Customer>,
    pub reservations: HashMap<u64, Reservation>,
    next_id: u64,
}

impl ReservationSystem {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            customers: HashMap::new(),
            reservations: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_resource(&mut self, resource: Resource) -> Result<(), ReservationError> {
        if self.resources.contains_key(&resource.id) {
            return Err(ReservationError::Duplicate(resource.id));
        }
        self.resources.insert(resource.id.clone(), resource);
        Ok(())
    }

    pub fn add_customer(&mut self, customer: Customer) -> Result<(), ReservationError> {
        if self.customers.contains_key(&customer.id) {
            return Err(ReservationError::Duplicate(customer.id));
        }
        self.customers.insert(customer.id.clone(), customer);
        Ok(())
    }

    pub fn conflicts(
        &self,
        resource_id: &str,
        start: u64,
        end: u64,
    ) -> Result<Vec<&Reservation>, ReservationError> {
        if !self.resources.contains_key(resource_id) {
            return Err(ReservationError::UnknownResource(resource_id.to_string()));
        }
        if start >= end {
            return Err(ReservationError::InvalidInterval);
        }
        Ok(self
            .reservations
            .values()
            .filter(|r| r.resource_id == resource_id && r.is_active() && r.overlaps(start, end))
            .collect())
    }

    pub fn book(
        &mut self,
        resource_id: &str,
        customer_id: &str,
        start: u64,
        end: u64,
    ) -> Result<u64, ReservationError> {
        if !self.resources.contains_key(resource_id) {
            return Err(ReservationError::UnknownResource(resource_id.to_string()));
        }
        if !self.customers.contains_key(customer_id) {
            return Err(ReservationError::UnknownCustomer(customer_id.to_string()));
        }
        if start >= end {
            return Err(ReservationError::InvalidInterval);
        }
        let colliding: Vec<u64> = self
            .reservations
            .values()
            .filter(|r| r.resource_id == resource_id && r.is_active() && r.overlaps(start, end))
            .map(|r| r.id)
            .collect();
        if !colliding.is_empty() {
            return Err(ReservationError::Conflict {
                resource_id: resource_id.to_string(),
                colliding,
            });
        }
        let id = self.next_id;
        self.next_id += 1;
        self.reservations.insert(
            id,
            Reservation {
                id,
                resource_id: resource_id.to_string(),
                customer_id: customer_id.to_string(),
                start,
                end,
                status: ReservationStatus::Booked,
            },
        );
        Ok(id)
    }

    pub fn cancel(&mut self, reservation_id: u64) -> Result<(), ReservationError> {
        let r = self
            .reservations
            .get_mut(&reservation_id)
            .ok_or(ReservationError::UnknownReservation(reservation_id))?;
        if r.status == ReservationStatus::Cancelled {
            return Err(ReservationError::AlreadyCancelled(reservation_id));
        }
        r.status = ReservationStatus::Cancelled;
        Ok(())
    }

    pub fn available(
        &self,
        start: u64,
        end: u64,
        kind: Option<&str>,
    ) -> Result<Vec<&Resource>, ReservationError> {
        if start >= end {
            return Err(ReservationError::InvalidInterval);
        }
        let mut out = Vec::new();
        for res in self.resources.values() {
            if let Some(k) = kind {
                if res.kind != k {
                    continue;
                }
            }
            let collides = self
                .reservations
                .values()
                .any(|r| r.resource_id == res.id && r.is_active() && r.overlaps(start, end));
            if !collides {
                out.push(res);
            }
        }
        Ok(out)
    }

    pub fn occupancy(&self, resource_id: &str) -> Result<Vec<&Reservation>, ReservationError> {
        if !self.resources.contains_key(resource_id) {
            return Err(ReservationError::UnknownResource(resource_id.to_string()));
        }
        let mut out: Vec<&Reservation> = self
            .reservations
            .values()
            .filter(|r| r.resource_id == resource_id && r.is_active())
            .collect();
        out.sort_by_key(|r| r.start);
        Ok(out)
    }

    pub fn customer_bookings(&self, customer_id: &str) -> Result<Vec<&Reservation>, ReservationError> {
        if !self.customers.contains_key(customer_id) {
            return Err(ReservationError::UnknownCustomer(customer_id.to_string()));
        }
        Ok(self
            .reservations
            .values()
            .filter(|r| r.customer_id == customer_id)
            .collect())
    }
}

impl Default for ReservationSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> ReservationSystem {
        let mut s = ReservationSystem::new();
        s.add_resource(Resource::new("12A", "seat", "window")).unwrap();
        s.add_resource(Resource::new("12B", "seat", "aisle")).unwrap();
        s.add_resource(Resource::new("304", "room", "king")).unwrap();
        s.add_customer(Customer { id: "C1".into(), name: "A".into() }).unwrap();
        s.add_customer(Customer { id: "C2".into(), name: "B".into() }).unwrap();
        s
    }

    #[test]
    fn non_overlapping_bookings_succeed() {
        let mut s = setup();
        s.book("12A", "C1", 1000, 2000).unwrap();
        s.book("12A", "C2", 2000, 3000).unwrap(); // touching boundary is not overlap
    }

    #[test]
    fn overlapping_booking_errors() {
        let mut s = setup();
        s.book("12A", "C1", 1000, 2000).unwrap();
        assert!(matches!(
            s.book("12A", "C2", 1500, 2500),
            Err(ReservationError::Conflict { .. })
        ));
    }

    #[test]
    fn cancel_frees_the_slot() {
        let mut s = setup();
        let id = s.book("12A", "C1", 1000, 2000).unwrap();
        s.cancel(id).unwrap();
        s.book("12A", "C2", 1500, 2500).unwrap();
    }

    #[test]
    fn available_filters_by_kind_and_conflict() {
        let mut s = setup();
        s.book("12A", "C1", 1000, 2000).unwrap();
        let seats = s.available(1000, 2000, Some("seat")).unwrap();
        let ids: Vec<&str> = seats.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec!["12B"]);
    }

    #[test]
    fn invalid_interval_rejected() {
        let mut s = setup();
        assert!(matches!(
            s.book("12A", "C1", 2000, 1000),
            Err(ReservationError::InvalidInterval)
        ));
    }
}
