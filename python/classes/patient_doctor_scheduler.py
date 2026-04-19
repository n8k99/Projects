"""Patient / Doctor Scheduler — dual-resource availability, rescheduling, slot search.

An appointment requires TWO independent calendars to both be free: the doctor's
and the room's. Conflict is the conjunction. `find_slot` searches a doctor's
calendar for the first gap and then picks any room free across that window.

Rescheduling introduces self-exclusion: when moving appointment X, the conflict
check must ignore X itself — otherwise the appointment conflicts with itself and
no move is ever legal.
"""

from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from enum import Enum
from typing import Optional
import itertools


class AppointmentStatus(Enum):
    SCHEDULED = "scheduled"
    COMPLETED = "completed"
    CANCELLED = "cancelled"
    NO_SHOW = "no_show"


@dataclass
class Doctor:
    id: str
    name: str
    specialty: str = ""


@dataclass
class Room:
    id: str
    label: str = ""
    equipment: set[str] = field(default_factory=set)


@dataclass
class Patient:
    id: str
    name: str
    primary_doctor_id: Optional[str] = None


@dataclass
class Appointment:
    id: int
    patient_id: str
    doctor_id: str
    room_id: str
    start: datetime
    end: datetime
    reason: str = ""
    status: AppointmentStatus = AppointmentStatus.SCHEDULED

    @property
    def is_active(self) -> bool:
        return self.status == AppointmentStatus.SCHEDULED

    def overlaps(self, start: datetime, end: datetime) -> bool:
        """Half-open overlap (same convention as G050)."""
        return self.start < end and start < self.end


class SchedulerError(Exception):
    pass


class UnknownEntity(SchedulerError): ...
class InvalidInterval(SchedulerError): ...
class DoctorConflict(SchedulerError): ...
class RoomConflict(SchedulerError): ...
class NoSlotFound(SchedulerError): ...
class AppointmentClosed(SchedulerError): ...


class Scheduler:
    """Coordinates patients, doctors, rooms, and appointments across two calendars."""

    def __init__(self) -> None:
        self.doctors: dict[str, Doctor] = {}
        self.rooms: dict[str, Room] = {}
        self.patients: dict[str, Patient] = {}
        self.appointments: dict[int, Appointment] = {}
        self._ids = itertools.count(1)

    # --- registration ---
    def add_doctor(self, doctor: Doctor) -> None:
        if doctor.id in self.doctors:
            raise ValueError(f"doctor already registered: {doctor.id}")
        self.doctors[doctor.id] = doctor

    def add_room(self, room: Room) -> None:
        if room.id in self.rooms:
            raise ValueError(f"room already registered: {room.id}")
        self.rooms[room.id] = room

    def add_patient(self, patient: Patient) -> None:
        if patient.id in self.patients:
            raise ValueError(f"patient already registered: {patient.id}")
        if patient.primary_doctor_id and patient.primary_doctor_id not in self.doctors:
            raise UnknownEntity(f"unknown doctor: {patient.primary_doctor_id}")
        self.patients[patient.id] = patient

    # --- core: dual-resource conflict check ---
    def doctor_conflicts(self, doctor_id: str, start: datetime, end: datetime,
                         exclude_appointment_id: Optional[int] = None) -> list[Appointment]:
        if doctor_id not in self.doctors:
            raise UnknownEntity(f"unknown doctor: {doctor_id}")
        return [a for a in self.appointments.values()
                if a.doctor_id == doctor_id
                and a.is_active
                and a.id != exclude_appointment_id
                and a.overlaps(start, end)]

    def room_conflicts(self, room_id: str, start: datetime, end: datetime,
                       exclude_appointment_id: Optional[int] = None) -> list[Appointment]:
        if room_id not in self.rooms:
            raise UnknownEntity(f"unknown room: {room_id}")
        return [a for a in self.appointments.values()
                if a.room_id == room_id
                and a.is_active
                and a.id != exclude_appointment_id
                and a.overlaps(start, end)]

    # --- schedule / cancel / reschedule / complete ---
    def schedule(self, patient_id: str, doctor_id: str, room_id: str,
                 start: datetime, end: datetime, reason: str = "") -> Appointment:
        if patient_id not in self.patients:
            raise UnknownEntity(f"unknown patient: {patient_id}")
        if start >= end:
            raise InvalidInterval("start must be before end")
        # Both conflict checks run — error naming preserves which calendar blocked.
        d_conflicts = self.doctor_conflicts(doctor_id, start, end)
        if d_conflicts:
            raise DoctorConflict(
                f"{doctor_id} busy: conflicts {[a.id for a in d_conflicts]}")
        r_conflicts = self.room_conflicts(room_id, start, end)
        if r_conflicts:
            raise RoomConflict(
                f"{room_id} busy: conflicts {[a.id for a in r_conflicts]}")
        appointment = Appointment(
            id=next(self._ids), patient_id=patient_id, doctor_id=doctor_id,
            room_id=room_id, start=start, end=end, reason=reason,
        )
        self.appointments[appointment.id] = appointment
        return appointment

    def cancel(self, appointment_id: int) -> Appointment:
        appt = self._require_appointment(appointment_id)
        if appt.status != AppointmentStatus.SCHEDULED:
            raise AppointmentClosed(f"appointment {appointment_id} is {appt.status.value}")
        appt.status = AppointmentStatus.CANCELLED
        return appt

    def complete(self, appointment_id: int) -> Appointment:
        appt = self._require_appointment(appointment_id)
        if appt.status != AppointmentStatus.SCHEDULED:
            raise AppointmentClosed(f"appointment {appointment_id} is {appt.status.value}")
        appt.status = AppointmentStatus.COMPLETED
        return appt

    def mark_no_show(self, appointment_id: int) -> Appointment:
        appt = self._require_appointment(appointment_id)
        if appt.status != AppointmentStatus.SCHEDULED:
            raise AppointmentClosed(f"appointment {appointment_id} is {appt.status.value}")
        appt.status = AppointmentStatus.NO_SHOW
        return appt

    def reschedule(self, appointment_id: int,
                   new_start: datetime, new_end: datetime,
                   new_room_id: Optional[str] = None) -> Appointment:
        """Move an existing appointment. Conflict check excludes the appointment itself."""
        appt = self._require_appointment(appointment_id)
        if appt.status != AppointmentStatus.SCHEDULED:
            raise AppointmentClosed(f"appointment {appointment_id} is {appt.status.value}")
        if new_start >= new_end:
            raise InvalidInterval("new_start must be before new_end")
        target_room = new_room_id or appt.room_id
        if target_room not in self.rooms:
            raise UnknownEntity(f"unknown room: {target_room}")
        d_conflicts = self.doctor_conflicts(
            appt.doctor_id, new_start, new_end, exclude_appointment_id=appt.id)
        if d_conflicts:
            raise DoctorConflict(
                f"{appt.doctor_id} busy: conflicts {[a.id for a in d_conflicts]}")
        r_conflicts = self.room_conflicts(
            target_room, new_start, new_end, exclude_appointment_id=appt.id)
        if r_conflicts:
            raise RoomConflict(
                f"{target_room} busy: conflicts {[a.id for a in r_conflicts]}")
        appt.start = new_start
        appt.end = new_end
        appt.room_id = target_room
        return appt

    def _require_appointment(self, appointment_id: int) -> Appointment:
        if appointment_id not in self.appointments:
            raise UnknownEntity(f"unknown appointment: {appointment_id}")
        return self.appointments[appointment_id]

    # --- find the first slot where both doctor and some room are free ---
    def find_slot(self, doctor_id: str, duration: timedelta,
                  search_from: datetime, search_until: datetime,
                  step: timedelta = timedelta(minutes=15)) -> tuple[datetime, Room]:
        """Scan the doctor's calendar at `step` granularity for the first window
        of `duration` where the doctor is free AND at least one room is free.

        Returns (start_time, room). Raises NoSlotFound if nothing fits.
        """
        if doctor_id not in self.doctors:
            raise UnknownEntity(f"unknown doctor: {doctor_id}")
        if duration <= timedelta(0):
            raise InvalidInterval("duration must be positive")
        if search_from >= search_until:
            raise InvalidInterval("search_from must be before search_until")
        t = search_from
        while t + duration <= search_until:
            end = t + duration
            if not self.doctor_conflicts(doctor_id, t, end):
                for room in self.rooms.values():
                    if not self.room_conflicts(room.id, t, end):
                        return (t, room)
            t += step
        raise NoSlotFound(
            f"no {duration} slot for {doctor_id} between {search_from} and {search_until}")

    # --- views ---
    def doctor_schedule(self, doctor_id: str) -> list[Appointment]:
        if doctor_id not in self.doctors:
            raise UnknownEntity(f"unknown doctor: {doctor_id}")
        return sorted(
            (a for a in self.appointments.values()
             if a.doctor_id == doctor_id and a.is_active),
            key=lambda a: a.start)

    def room_schedule(self, room_id: str) -> list[Appointment]:
        if room_id not in self.rooms:
            raise UnknownEntity(f"unknown room: {room_id}")
        return sorted(
            (a for a in self.appointments.values()
             if a.room_id == room_id and a.is_active),
            key=lambda a: a.start)

    def patient_history(self, patient_id: str) -> list[Appointment]:
        if patient_id not in self.patients:
            raise UnknownEntity(f"unknown patient: {patient_id}")
        return sorted(
            (a for a in self.appointments.values() if a.patient_id == patient_id),
            key=lambda a: a.start)


# --- demo ---
if __name__ == "__main__":
    s = Scheduler()
    s.add_doctor(Doctor("D-CHEN", "Dr. Chen", "cardiology"))
    s.add_doctor(Doctor("D-PATEL", "Dr. Patel", "pediatrics"))
    s.add_room(Room("R-101", "Exam 101"))
    s.add_room(Room("R-102", "Exam 102"))
    s.add_patient(Patient("P-001", "Nathan", primary_doctor_id="D-CHEN"))

    t = datetime(2026, 5, 4, 9, 0, tzinfo=timezone.utc)
    a1 = s.schedule("P-001", "D-CHEN", "R-101", t, t + timedelta(minutes=30),
                    reason="annual checkup")
    print(f"scheduled {a1.id}: {a1.start.time()} - {a1.end.time()}")

    # Doctor conflict:
    try:
        s.schedule("P-001", "D-CHEN", "R-102",
                   t + timedelta(minutes=15), t + timedelta(minutes=45))
    except DoctorConflict as e:
        print(f"refused (doctor): {e}")

    # Room conflict (same room, different doctor):
    try:
        s.schedule("P-001", "D-PATEL", "R-101",
                   t + timedelta(minutes=15), t + timedelta(minutes=45))
    except RoomConflict as e:
        print(f"refused (room):   {e}")

    # Reschedule — conflict check excludes self.
    a1_moved = s.reschedule(a1.id,
                            t + timedelta(hours=2),
                            t + timedelta(hours=2, minutes=30))
    print(f"moved  {a1.id} -> {a1_moved.start.time()}")

    # Slot search.
    slot_start, slot_room = s.find_slot(
        "D-CHEN", timedelta(minutes=30), t, t + timedelta(hours=4))
    print(f"first free 30-min slot for D-CHEN: {slot_start.time()} in {slot_room.id}")
