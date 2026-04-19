//! Student Grade Book — the join record carries a measurement. Aggregation lives on the edge.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Student {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub credits: f64,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub id: String,
    pub course_id: String,
    pub name: String,
    pub max_points: f64,
    pub weight: f64,
}

/// The valued join: a student's score on an assignment.
#[derive(Debug, Clone)]
pub struct Grade {
    pub student_id: String,
    pub assignment_id: String,
    pub points: f64,
    pub note: String,
}

#[derive(Debug)]
pub enum GradeBookError {
    UnknownStudent(String),
    UnknownCourse(String),
    UnknownAssignment(String),
    Duplicate(String),
    NegativePoints,
}

/// Standard US letter-scale → 4.0 GPA.
fn percent_to_letter(percent: f64) -> (&'static str, f64) {
    let scale: &[(f64, &str, f64)] = &[
        (93.0, "A",  4.0),
        (90.0, "A-", 3.7),
        (87.0, "B+", 3.3),
        (83.0, "B",  3.0),
        (80.0, "B-", 2.7),
        (77.0, "C+", 2.3),
        (73.0, "C",  2.0),
        (70.0, "C-", 1.7),
        (67.0, "D+", 1.3),
        (63.0, "D",  1.0),
        (60.0, "D-", 0.7),
    ];
    for (cut, letter, gpa) in scale {
        if percent >= *cut {
            return (*letter, *gpa);
        }
    }
    ("F", 0.0)
}

pub struct GradeBook {
    pub students: HashMap<String, Student>,
    pub courses: HashMap<String, Course>,
    pub assignments: HashMap<String, Assignment>,
    /// Keyed by (student_id, assignment_id) so record_grade is idempotent.
    pub grades: HashMap<(String, String), Grade>,
}

impl GradeBook {
    pub fn new() -> Self {
        Self {
            students: HashMap::new(),
            courses: HashMap::new(),
            assignments: HashMap::new(),
            grades: HashMap::new(),
        }
    }

    pub fn add_student(&mut self, s: Student) -> Result<(), GradeBookError> {
        if self.students.contains_key(&s.id) {
            return Err(GradeBookError::Duplicate(s.id));
        }
        self.students.insert(s.id.clone(), s);
        Ok(())
    }

    pub fn add_course(&mut self, c: Course) -> Result<(), GradeBookError> {
        if self.courses.contains_key(&c.id) {
            return Err(GradeBookError::Duplicate(c.id));
        }
        self.courses.insert(c.id.clone(), c);
        Ok(())
    }

    pub fn add_assignment(&mut self, a: Assignment) -> Result<(), GradeBookError> {
        if !self.courses.contains_key(&a.course_id) {
            return Err(GradeBookError::UnknownCourse(a.course_id));
        }
        if self.assignments.contains_key(&a.id) {
            return Err(GradeBookError::Duplicate(a.id));
        }
        self.assignments.insert(a.id.clone(), a);
        Ok(())
    }

    pub fn record_grade(
        &mut self,
        student_id: &str,
        assignment_id: &str,
        points: f64,
        note: &str,
    ) -> Result<(), GradeBookError> {
        if !self.students.contains_key(student_id) {
            return Err(GradeBookError::UnknownStudent(student_id.to_string()));
        }
        if !self.assignments.contains_key(assignment_id) {
            return Err(GradeBookError::UnknownAssignment(assignment_id.to_string()));
        }
        if points < 0.0 {
            return Err(GradeBookError::NegativePoints);
        }
        self.grades.insert(
            (student_id.to_string(), assignment_id.to_string()),
            Grade {
                student_id: student_id.to_string(),
                assignment_id: assignment_id.to_string(),
                points,
                note: note.to_string(),
            },
        );
        Ok(())
    }

    pub fn grade_of(&self, student_id: &str, assignment_id: &str) -> Option<&Grade> {
        self.grades.get(&(student_id.to_string(), assignment_id.to_string()))
    }

    pub fn course_assignments(&self, course_id: &str) -> Vec<&Assignment> {
        self.assignments.values().filter(|a| a.course_id == course_id).collect()
    }

    pub fn missing(&self, student_id: &str, course_id: Option<&str>) -> Vec<&Assignment> {
        self.assignments
            .values()
            .filter(|a| match course_id {
                Some(c) => a.course_id == c,
                None => true,
            })
            .filter(|a| {
                !self
                    .grades
                    .contains_key(&(student_id.to_string(), a.id.clone()))
            })
            .collect()
    }

    /// Weighted percentage across the course's assignments.
    pub fn course_average(
        &self,
        student_id: &str,
        course_id: &str,
        missing_as_zero: bool,
    ) -> Option<f64> {
        let assigns = self.course_assignments(course_id);
        if assigns.is_empty() {
            return None;
        }
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        for a in &assigns {
            let percent = match self.grade_of(student_id, &a.id) {
                Some(g) if a.max_points > 0.0 => g.points / a.max_points * 100.0,
                Some(_) => 0.0,
                None => {
                    if !missing_as_zero {
                        continue;
                    }
                    0.0
                }
            };
            weighted_sum += percent * a.weight;
            weight_total += a.weight;
        }
        if weight_total == 0.0 {
            None
        } else {
            Some(weighted_sum / weight_total)
        }
    }

    /// Credit-weighted GPA on a 4.0 scale.
    pub fn gpa(&self, student_id: &str, missing_as_zero: bool) -> Option<f64> {
        let mut total_credits = 0.0;
        let mut weighted = 0.0;
        for course in self.courses.values() {
            if let Some(avg) = self.course_average(student_id, &course.id, missing_as_zero) {
                let (_, gpa_points) = percent_to_letter(avg);
                weighted += gpa_points * course.credits;
                total_credits += course.credits;
            }
        }
        if total_credits == 0.0 {
            None
        } else {
            Some(weighted / total_credits)
        }
    }

    pub fn class_average(&self, assignment_id: &str) -> Option<f64> {
        let a = self.assignments.get(assignment_id)?;
        let recorded: Vec<&Grade> = self
            .grades
            .values()
            .filter(|g| g.assignment_id == assignment_id)
            .collect();
        if recorded.is_empty() || a.max_points <= 0.0 {
            return None;
        }
        let total: f64 = recorded.iter().map(|g| g.points).sum();
        Some(total / recorded.len() as f64 / a.max_points * 100.0)
    }

    pub fn distribution(&self, course_id: &str) -> HashMap<&'static str, u32> {
        let mut hist: HashMap<&'static str, u32> = HashMap::new();
        for s in self.students.values() {
            if let Some(avg) = self.course_average(&s.id, course_id, false) {
                let (letter, _) = percent_to_letter(avg);
                *hist.entry(letter).or_insert(0) += 1;
            }
        }
        hist
    }
}

impl Default for GradeBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> GradeBook {
        let mut gb = GradeBook::new();
        gb.add_student(Student { id: "S1".into(), name: "Alice".into() }).unwrap();
        gb.add_course(Course { id: "C1".into(), name: "Intro".into(), credits: 3.0 }).unwrap();
        gb.add_assignment(Assignment {
            id: "A1".into(), course_id: "C1".into(), name: "HW".into(),
            max_points: 100.0, weight: 1.0,
        }).unwrap();
        gb.add_assignment(Assignment {
            id: "A2".into(), course_id: "C1".into(), name: "Final".into(),
            max_points: 100.0, weight: 3.0,
        }).unwrap();
        gb
    }

    #[test]
    fn weighted_course_average() {
        let mut gb = setup();
        gb.record_grade("S1", "A1", 80.0, "").unwrap();
        gb.record_grade("S1", "A2", 100.0, "").unwrap();
        // (80*1 + 100*3) / 4 = 95
        let avg = gb.course_average("S1", "C1", false).unwrap();
        assert!((avg - 95.0).abs() < 1e-9);
    }

    #[test]
    fn missing_excluded_by_default_included_as_zero_on_request() {
        let mut gb = setup();
        gb.record_grade("S1", "A1", 80.0, "").unwrap();
        let partial = gb.course_average("S1", "C1", false).unwrap();
        assert!((partial - 80.0).abs() < 1e-9); // only A1 counted
        let projected = gb.course_average("S1", "C1", true).unwrap();
        // (80*1 + 0*3) / 4 = 20
        assert!((projected - 20.0).abs() < 1e-9);
    }

    #[test]
    fn missing_lists_ungraded_assignments() {
        let mut gb = setup();
        gb.record_grade("S1", "A1", 80.0, "").unwrap();
        let miss: Vec<&str> = gb.missing("S1", Some("C1")).iter().map(|a| a.id.as_str()).collect();
        assert_eq!(miss, vec!["A2"]);
    }

    #[test]
    fn gpa_reflects_letter_conversion() {
        let mut gb = setup();
        gb.record_grade("S1", "A1", 95.0, "").unwrap();
        gb.record_grade("S1", "A2", 95.0, "").unwrap();
        let g = gb.gpa("S1", false).unwrap();
        assert!((g - 4.0).abs() < 1e-9);
    }
}
