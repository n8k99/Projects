"""Student Grade Book — the join record carries a measurement. Aggregation lives on the edge.

Weighted mean at two levels: assignment weight inside a course, course credit inside a GPA.
Missing grades are distinct from zeros — the same dataset supports two policies
(current_average ignores missing; projected_average treats missing as zero).
"""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class Student:
    id: str
    name: str


@dataclass
class Course:
    id: str
    name: str
    credits: float = 1.0


@dataclass
class Assignment:
    id: str
    course_id: str
    name: str
    max_points: float = 100.0
    weight: float = 1.0


@dataclass
class Grade:
    """The valued join: a student's score on an assignment."""
    student_id: str
    assignment_id: str
    points: float
    note: str = ""


class GradeBookError(Exception):
    pass


class UnknownEntity(GradeBookError):
    pass


# Classic 4.0-scale letter mapping.
_LETTER_SCALE = [
    (93, "A",  4.0),
    (90, "A-", 3.7),
    (87, "B+", 3.3),
    (83, "B",  3.0),
    (80, "B-", 2.7),
    (77, "C+", 2.3),
    (73, "C",  2.0),
    (70, "C-", 1.7),
    (67, "D+", 1.3),
    (63, "D",  1.0),
    (60, "D-", 0.7),
    (0,  "F",  0.0),
]


def percent_to_letter(percent: float) -> tuple[str, float]:
    """(letter, gpa-points) for a percentage score."""
    for cutoff, letter, gpa in _LETTER_SCALE:
        if percent >= cutoff:
            return (letter, gpa)
    return ("F", 0.0)


class GradeBook:
    """Students, courses, assignments, and the grades that join them."""

    def __init__(self) -> None:
        self.students: dict[str, Student] = {}
        self.courses: dict[str, Course] = {}
        self.assignments: dict[str, Assignment] = {}
        # keyed by (student_id, assignment_id) so record_grade is idempotent
        self.grades: dict[tuple[str, str], Grade] = {}

    # --- registration ---
    def add_student(self, student: Student) -> None:
        if student.id in self.students:
            raise ValueError(f"student already registered: {student.id}")
        self.students[student.id] = student

    def add_course(self, course: Course) -> None:
        if course.id in self.courses:
            raise ValueError(f"course already registered: {course.id}")
        self.courses[course.id] = course

    def add_assignment(self, assignment: Assignment) -> None:
        if assignment.course_id not in self.courses:
            raise UnknownEntity(f"unknown course: {assignment.course_id}")
        if assignment.id in self.assignments:
            raise ValueError(f"assignment already registered: {assignment.id}")
        self.assignments[assignment.id] = assignment

    # --- grading ---
    def record_grade(self, student_id: str, assignment_id: str,
                     points: float, note: str = "") -> Grade:
        if student_id not in self.students:
            raise UnknownEntity(f"unknown student: {student_id}")
        if assignment_id not in self.assignments:
            raise UnknownEntity(f"unknown assignment: {assignment_id}")
        if points < 0:
            raise ValueError("points must be non-negative")
        grade = Grade(student_id=student_id, assignment_id=assignment_id,
                      points=points, note=note)
        self.grades[(student_id, assignment_id)] = grade
        return grade

    def grade_of(self, student_id: str, assignment_id: str) -> Optional[Grade]:
        return self.grades.get((student_id, assignment_id))

    # --- queries on assignments ---
    def course_assignments(self, course_id: str) -> list[Assignment]:
        if course_id not in self.courses:
            raise UnknownEntity(f"unknown course: {course_id}")
        return [a for a in self.assignments.values() if a.course_id == course_id]

    def missing(self, student_id: str, course_id: Optional[str] = None) -> list[Assignment]:
        """Assignments this student has no grade for (optionally scoped to a course)."""
        if student_id not in self.students:
            raise UnknownEntity(f"unknown student: {student_id}")
        pool = (self.course_assignments(course_id) if course_id
                else list(self.assignments.values()))
        return [a for a in pool if (student_id, a.id) not in self.grades]

    # --- aggregation: course average ---
    def course_average(self, student_id: str, course_id: str,
                       missing_as_zero: bool = False) -> Optional[float]:
        """Weighted percentage across the course's assignments, or None if no data."""
        if student_id not in self.students:
            raise UnknownEntity(f"unknown student: {student_id}")
        if course_id not in self.courses:
            raise UnknownEntity(f"unknown course: {course_id}")
        assigns = self.course_assignments(course_id)
        if not assigns:
            return None
        weighted_sum = 0.0
        weight_total = 0.0
        for a in assigns:
            g = self.grade_of(student_id, a.id)
            if g is None:
                if not missing_as_zero:
                    continue
                percent = 0.0
            else:
                percent = (g.points / a.max_points * 100.0) if a.max_points > 0 else 0.0
            weighted_sum += percent * a.weight
            weight_total += a.weight
        if weight_total == 0:
            return None
        return weighted_sum / weight_total

    # --- aggregation: GPA across courses ---
    def gpa(self, student_id: str, missing_as_zero: bool = False) -> Optional[float]:
        """Credit-weighted GPA on a 4.0 scale, or None if no courses have data."""
        if student_id not in self.students:
            raise UnknownEntity(f"unknown student: {student_id}")
        total_credits = 0.0
        weighted = 0.0
        for course in self.courses.values():
            avg = self.course_average(student_id, course.id, missing_as_zero=missing_as_zero)
            if avg is None:
                continue
            _, gpa_points = percent_to_letter(avg)
            weighted += gpa_points * course.credits
            total_credits += course.credits
        if total_credits == 0:
            return None
        return weighted / total_credits

    # --- aggregation: per-assignment class view ---
    def class_average(self, assignment_id: str) -> Optional[float]:
        if assignment_id not in self.assignments:
            raise UnknownEntity(f"unknown assignment: {assignment_id}")
        a = self.assignments[assignment_id]
        recorded = [g for g in self.grades.values() if g.assignment_id == assignment_id]
        if not recorded:
            return None
        if a.max_points <= 0:
            return 0.0
        return sum(g.points for g in recorded) / len(recorded) / a.max_points * 100.0

    def distribution(self, course_id: str) -> dict[str, int]:
        """Letter-grade histogram for a course, across all students."""
        if course_id not in self.courses:
            raise UnknownEntity(f"unknown course: {course_id}")
        hist: dict[str, int] = {}
        for student in self.students.values():
            avg = self.course_average(student.id, course_id)
            if avg is None:
                continue
            letter, _ = percent_to_letter(avg)
            hist[letter] = hist.get(letter, 0) + 1
        return hist


# --- demo ---
if __name__ == "__main__":
    gb = GradeBook()
    gb.add_student(Student("S001", "Alice"))
    gb.add_student(Student("S002", "Bob"))
    gb.add_course(Course("CS101", "Intro to CS", credits=4))
    gb.add_course(Course("MATH201", "Calculus II", credits=3))

    gb.add_assignment(Assignment("CS101-HW1", "CS101", "Homework 1", max_points=100, weight=1))
    gb.add_assignment(Assignment("CS101-MID", "CS101", "Midterm",    max_points=100, weight=2))
    gb.add_assignment(Assignment("CS101-FIN", "CS101", "Final",      max_points=100, weight=3))
    gb.add_assignment(Assignment("MATH201-Q1", "MATH201", "Quiz 1",  max_points=20,  weight=1))
    gb.add_assignment(Assignment("MATH201-FIN","MATH201", "Final",   max_points=100, weight=4))

    # Alice: strong
    gb.record_grade("S001", "CS101-HW1", 95)
    gb.record_grade("S001", "CS101-MID", 88)
    gb.record_grade("S001", "CS101-FIN", 92)
    gb.record_grade("S001", "MATH201-Q1", 18)
    gb.record_grade("S001", "MATH201-FIN", 85)
    # Bob: missing the CS101 final
    gb.record_grade("S002", "CS101-HW1", 75)
    gb.record_grade("S002", "CS101-MID", 70)
    gb.record_grade("S002", "MATH201-Q1", 14)
    gb.record_grade("S002", "MATH201-FIN", 68)

    for sid in ("S001", "S002"):
        cs  = gb.course_average(sid, "CS101")
        mth = gb.course_average(sid, "MATH201")
        g   = gb.gpa(sid)
        miss = [a.id for a in gb.missing(sid)]
        print(f"{sid}: CS101 {cs:.1f}%, MATH201 {mth:.1f}%, GPA {g:.2f}, missing {miss}")
    print("CS101 distribution:", gb.distribution("CS101"))
