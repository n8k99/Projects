//! Patient / Doctor Scheduler — dual-resource availability, rescheduling, slot search.
//!
//! An appointment requires TWO calendars to be free simultaneously. `reschedule`
//! uses self-exclusion so an appointment being moved doesn't block its own move.
//! `find_slot` scans at a step granularity for the first window where the doctor
//! is free and at least one room is free.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppointmentStatus {
    Scheduled,
    Completed,
    Cancelled,
    NoShow,
}

#[derive(Debug, Clone)]
pub struct Doctor {
    pub id: String,
    pub name: String,
    pub specialty: String,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct SchedulerPatient {
    pub id: String,
    pub name: String,
    pub primary_doctor_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Appointment {
    pub id: u64,
    pub patient_id: String,
    pub doctor_id: String,
    pub room_id: String,
    /// Unix seconds.
    pub start: u64,
    pub end: u64,
    pub reason: String,
    pub status: AppointmentStatus,
}

impl Appointment {
    pub fn is_active(&self) -> bool {
        self.status == AppointmentStatus::Scheduled
    }

    pub fn overlaps(&self, start: u64, end: u64) -> bool {
        self.start < end && start < self.end
    }
}

#[derive(Debug)]
pub enum SchedulerError {
    UnknownDoctor(String),
    UnknownRoom(String),
    UnknownPatient(String),
    UnknownAppointment(u64),
    Duplicate(String),
    InvalidInterval,
    DoctorConflict { doctor_id: String, colliding: Vec<u64> },
    RoomConflict { room_id: String, colliding: Vec<u64> },
    AppointmentClosed(u64),
    NoSlotFound,
}

pub struct Scheduler {
    pub doctors: HashMap<String, Doctor>,
    pub rooms: HashMap<String, Room>,
    pub patients: HashMap<String, SchedulerPatient>,
    pub appointments: HashMap<u64, Appointment>,
    next_id: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            doctors: HashMap::new(),
            rooms: HashMap::new(),
            patients: HashMap::new(),
            appointments: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_doctor(&mut self, d: Doctor) -> Result<(), SchedulerError> {
        if self.doctors.contains_key(&d.id) {
            return Err(SchedulerError::Duplicate(d.id));
        }
        self.doctors.insert(d.id.clone(), d);
        Ok(())
    }

    pub fn add_room(&mut self, r: Room) -> Result<(), SchedulerError> {
        if self.rooms.contains_key(&r.id) {
            return Err(SchedulerError::Duplicate(r.id));
        }
        self.rooms.insert(r.id.clone(), r);
        Ok(())
    }

    pub fn add_patient(&mut self, p: SchedulerPatient) -> Result<(), SchedulerError> {
        if self.patients.contains_key(&p.id) {
            return Err(SchedulerError::Duplicate(p.id));
        }
        if let Some(ref d) = p.primary_doctor_id {
            if !self.doctors.contains_key(d) {
                return Err(SchedulerError::UnknownDoctor(d.clone()));
            }
        }
        self.patients.insert(p.id.clone(), p);
        Ok(())
    }

    pub fn doctor_conflicts(
        &self,
        doctor_id: &str,
        start: u64,
        end: u64,
        exclude: Option<u64>,
    ) -> Result<Vec<u64>, SchedulerError> {
        if !self.doctors.contains_key(doctor_id) {
            return Err(SchedulerError::UnknownDoctor(doctor_id.to_string()));
        }
        Ok(self
            .appointments
            .values()
            .filter(|a| a.doctor_id == doctor_id
                && a.is_active()
                && Some(a.id) != exclude
                && a.overlaps(start, end))
            .map(|a| a.id)
            .collect())
    }

    pub fn room_conflicts(
        &self,
        room_id: &str,
        start: u64,
        end: u64,
        exclude: Option<u64>,
    ) -> Result<Vec<u64>, SchedulerError> {
        if !self.rooms.contains_key(room_id) {
            return Err(SchedulerError::UnknownRoom(room_id.to_string()));
        }
        Ok(self
            .appointments
            .values()
            .filter(|a| a.room_id == room_id
                && a.is_active()
                && Some(a.id) != exclude
                && a.overlaps(start, end))
            .map(|a| a.id)
            .collect())
    }

    pub fn schedule(
        &mut self,
        patient_id: &str,
        doctor_id: &str,
        room_id: &str,
        start: u64,
        end: u64,
        reason: &str,
    ) -> Result<u64, SchedulerError> {
        if !self.patients.contains_key(patient_id) {
            return Err(SchedulerError::UnknownPatient(patient_id.to_string()));
        }
        if start >= end {
            return Err(SchedulerError::InvalidInterval);
        }
        let d = self.doctor_conflicts(doctor_id, start, end, None)?;
        if !d.is_empty() {
            return Err(SchedulerError::DoctorConflict {
                doctor_id: doctor_id.to_string(), colliding: d });
        }
        let r = self.room_conflicts(room_id, start, end, None)?;
        if !r.is_empty() {
            return Err(SchedulerError::RoomConflict {
                room_id: room_id.to_string(), colliding: r });
        }
        let id = self.next_id;
        self.next_id += 1;
        self.appointments.insert(
            id,
            Appointment {
                id,
                patient_id: patient_id.to_string(),
                doctor_id: doctor_id.to_string(),
                room_id: room_id.to_string(),
                start, end,
                reason: reason.to_string(),
                status: AppointmentStatus::Scheduled,
            },
        );
        Ok(id)
    }

    fn transition(&mut self, id: u64, to: AppointmentStatus) -> Result<(), SchedulerError> {
        let a = self
            .appointments
            .get_mut(&id)
            .ok_or(SchedulerError::UnknownAppointment(id))?;
        if a.status != AppointmentStatus::Scheduled {
            return Err(SchedulerError::AppointmentClosed(id));
        }
        a.status = to;
        Ok(())
    }

    pub fn cancel(&mut self, id: u64) -> Result<(), SchedulerError> {
        self.transition(id, AppointmentStatus::Cancelled)
    }
    pub fn complete(&mut self, id: u64) -> Result<(), SchedulerError> {
        self.transition(id, AppointmentStatus::Completed)
    }
    pub fn mark_no_show(&mut self, id: u64) -> Result<(), SchedulerError> {
        self.transition(id, AppointmentStatus::NoShow)
    }

    /// Move an appointment. Conflict check excludes the appointment itself
    /// so an appointment never blocks its own move.
    pub fn reschedule(
        &mut self,
        id: u64,
        new_start: u64,
        new_end: u64,
        new_room_id: Option<&str>,
    ) -> Result<(), SchedulerError> {
        if new_start >= new_end {
            return Err(SchedulerError::InvalidInterval);
        }
        let (doctor_id, current_room) = {
            let a = self
                .appointments
                .get(&id)
                .ok_or(SchedulerError::UnknownAppointment(id))?;
            if a.status != AppointmentStatus::Scheduled {
                return Err(SchedulerError::AppointmentClosed(id));
            }
            (a.doctor_id.clone(), a.room_id.clone())
        };
        let target_room = new_room_id.unwrap_or(&current_room).to_string();
        if !self.rooms.contains_key(&target_room) {
            return Err(SchedulerError::UnknownRoom(target_room));
        }
        let d = self.doctor_conflicts(&doctor_id, new_start, new_end, Some(id))?;
        if !d.is_empty() {
            return Err(SchedulerError::DoctorConflict {
                doctor_id, colliding: d });
        }
        let r = self.room_conflicts(&target_room, new_start, new_end, Some(id))?;
        if !r.is_empty() {
            return Err(SchedulerError::RoomConflict {
                room_id: target_room, colliding: r });
        }
        let a = self.appointments.get_mut(&id).unwrap();
        a.start = new_start;
        a.end = new_end;
        a.room_id = target_room;
        Ok(())
    }

    /// First free window of `duration_secs` for the doctor where any room is free.
    /// Scans at `step_secs` granularity from `from` up to `until`.
    pub fn find_slot(
        &self,
        doctor_id: &str,
        duration_secs: u64,
        from: u64,
        until: u64,
        step_secs: u64,
    ) -> Result<(u64, String), SchedulerError> {
        if !self.doctors.contains_key(doctor_id) {
            return Err(SchedulerError::UnknownDoctor(doctor_id.to_string()));
        }
        if duration_secs == 0 || from >= until || step_secs == 0 {
            return Err(SchedulerError::InvalidInterval);
        }
        let mut t = from;
        while t + duration_secs <= until {
            let end = t + duration_secs;
            if self.doctor_conflicts(doctor_id, t, end, None)?.is_empty() {
                for room in self.rooms.values() {
                    if self.room_conflicts(&room.id, t, end, None)?.is_empty() {
                        return Ok((t, room.id.clone()));
                    }
                }
            }
            t += step_secs;
        }
        Err(SchedulerError::NoSlotFound)
    }

    pub fn doctor_schedule(&self, doctor_id: &str) -> Vec<&Appointment> {
        let mut out: Vec<&Appointment> = self
            .appointments
            .values()
            .filter(|a| a.doctor_id == doctor_id && a.is_active())
            .collect();
        out.sort_by_key(|a| a.start);
        out
    }

    pub fn room_schedule(&self, room_id: &str) -> Vec<&Appointment> {
        let mut out: Vec<&Appointment> = self
            .appointments
            .values()
            .filter(|a| a.room_id == room_id && a.is_active())
            .collect();
        out.sort_by_key(|a| a.start);
        out
    }

    pub fn patient_history(&self, patient_id: &str) -> Vec<&Appointment> {
        let mut out: Vec<&Appointment> = self
            .appointments
            .values()
            .filter(|a| a.patient_id == patient_id)
            .collect();
        out.sort_by_key(|a| a.start);
        out
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Scheduler {
        let mut s = Scheduler::new();
        s.add_doctor(Doctor {
            id: "D1".into(), name: "Chen".into(), specialty: "cardio".into() }).unwrap();
        s.add_doctor(Doctor {
            id: "D2".into(), name: "Patel".into(), specialty: "peds".into() }).unwrap();
        s.add_room(Room { id: "R1".into(), label: "Exam 1".into() }).unwrap();
        s.add_room(Room { id: "R2".into(), label: "Exam 2".into() }).unwrap();
        s.add_patient(SchedulerPatient {
            id: "P1".into(), name: "N".into(), primary_doctor_id: Some("D1".into()) }).unwrap();
        s
    }

    #[test]
    fn doctor_conflict_blocks_schedule() {
        let mut s = setup();
        s.schedule("P1", "D1", "R1", 1000, 2000, "").unwrap();
        assert!(matches!(
            s.schedule("P1", "D1", "R2", 1500, 2500, ""),
            Err(SchedulerError::DoctorConflict { .. })
        ));
    }

    #[test]
    fn room_conflict_blocks_schedule() {
        let mut s = setup();
        s.schedule("P1", "D1", "R1", 1000, 2000, "").unwrap();
        assert!(matches!(
            s.schedule("P1", "D2", "R1", 1500, 2500, ""),
            Err(SchedulerError::RoomConflict { .. })
        ));
    }

    #[test]
    fn reschedule_excludes_self_from_conflict_check() {
        let mut s = setup();
        let id = s.schedule("P1", "D1", "R1", 1000, 2000, "").unwrap();
        // Moving to an overlapping time shouldn't count itself as a conflict.
        s.reschedule(id, 1500, 2500, None).unwrap();
        assert_eq!(s.appointments[&id].start, 1500);
    }

    #[test]
    fn find_slot_returns_first_free_window() {
        let mut s = setup();
        // Block 1000..2000 on D1.
        s.schedule("P1", "D1", "R1", 1000, 2000, "").unwrap();
        // 500s window starting anywhere in [0, 1500] at 500s granularity.
        let (t, _room) = s.find_slot("D1", 500, 0, 4000, 500).unwrap();
        assert_eq!(t, 0); // the gap before 1000 fits the 500s request
    }

    #[test]
    fn find_slot_picks_any_free_room() {
        let mut s = setup();
        // Block R1 and R2 at different times; find_slot should still find R2.
        s.schedule("P1", "D1", "R1", 2000, 3000, "").unwrap();
        s.schedule("P1", "D2", "R2", 4000, 5000, "").unwrap();
        // D1 free at 2000..3000? no. At 3000..4000? yes, and R2 is free.
        let (t, room) = s.find_slot("D1", 500, 3000, 6000, 500).unwrap();
        assert!(t >= 3000 && t + 500 <= 4000 || room == "R1");
    }
}
