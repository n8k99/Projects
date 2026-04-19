// Student Grade Book — the join record carries a measurement. Aggregation lives on the edge.
package classes

import (
	"errors"
	"fmt"
)

// Student enrolls in courses and earns grades.
type Student struct {
	ID   string
	Name string
}

// Course is a unit of study with credits.
type Course struct {
	ID      string
	Name    string
	Credits float64
}

// Assignment is a graded task within a course.
type Assignment struct {
	ID        string
	CourseID  string
	Name      string
	MaxPoints float64
	Weight    float64
}

// Grade is the valued join: a student's score on an assignment.
type Grade struct {
	StudentID    string
	AssignmentID string
	Points       float64
	Note         string
}

// Grade book errors.
var (
	ErrUnknownStudent    = errors.New("unknown student")
	ErrUnknownCourse     = errors.New("unknown course")
	ErrUnknownAssignment = errors.New("unknown assignment")
	ErrNegativePoints    = errors.New("points must be non-negative")
)

type letterBand struct {
	cutoff float64
	letter string
	gpa    float64
}

var letterScale = []letterBand{
	{93, "A", 4.0},
	{90, "A-", 3.7},
	{87, "B+", 3.3},
	{83, "B", 3.0},
	{80, "B-", 2.7},
	{77, "C+", 2.3},
	{73, "C", 2.0},
	{70, "C-", 1.7},
	{67, "D+", 1.3},
	{63, "D", 1.0},
	{60, "D-", 0.7},
}

// PercentToLetter maps a percentage to a letter grade and GPA point value.
func PercentToLetter(percent float64) (string, float64) {
	for _, b := range letterScale {
		if percent >= b.cutoff {
			return b.letter, b.gpa
		}
	}
	return "F", 0.0
}

// GradeKey keys a grade by (student, assignment), making record_grade idempotent.
type GradeKey struct {
	StudentID    string
	AssignmentID string
}

// GradeBook ties students, courses, assignments, and grades.
type GradeBook struct {
	Students    map[string]Student
	Courses     map[string]Course
	Assignments map[string]Assignment
	Grades      map[GradeKey]Grade
}

// NewGradeBook builds an empty grade book.
func NewGradeBook() *GradeBook {
	return &GradeBook{
		Students:    make(map[string]Student),
		Courses:     make(map[string]Course),
		Assignments: make(map[string]Assignment),
		Grades:      make(map[GradeKey]Grade),
	}
}

// AddStudent registers a student.
func (g *GradeBook) AddStudent(s Student) error {
	if _, ok := g.Students[s.ID]; ok {
		return fmt.Errorf("%w: student %s", ErrDuplicate, s.ID)
	}
	g.Students[s.ID] = s
	return nil
}

// AddCourse registers a course.
func (g *GradeBook) AddCourse(c Course) error {
	if _, ok := g.Courses[c.ID]; ok {
		return fmt.Errorf("%w: course %s", ErrDuplicate, c.ID)
	}
	g.Courses[c.ID] = c
	return nil
}

// AddAssignment registers an assignment under an existing course.
func (g *GradeBook) AddAssignment(a Assignment) error {
	if _, ok := g.Courses[a.CourseID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownCourse, a.CourseID)
	}
	if _, ok := g.Assignments[a.ID]; ok {
		return fmt.Errorf("%w: assignment %s", ErrDuplicate, a.ID)
	}
	g.Assignments[a.ID] = a
	return nil
}

// RecordGrade records or overwrites a student's grade on an assignment.
func (g *GradeBook) RecordGrade(studentID, assignmentID string, points float64, note string) error {
	if _, ok := g.Students[studentID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownStudent, studentID)
	}
	if _, ok := g.Assignments[assignmentID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownAssignment, assignmentID)
	}
	if points < 0 {
		return ErrNegativePoints
	}
	g.Grades[GradeKey{studentID, assignmentID}] = Grade{
		StudentID:    studentID,
		AssignmentID: assignmentID,
		Points:       points,
		Note:         note,
	}
	return nil
}

// GradeOf returns the recorded grade or (zero, false).
func (g *GradeBook) GradeOf(studentID, assignmentID string) (Grade, bool) {
	grade, ok := g.Grades[GradeKey{studentID, assignmentID}]
	return grade, ok
}

// CourseAssignments lists assignments belonging to a course.
func (g *GradeBook) CourseAssignments(courseID string) []Assignment {
	var out []Assignment
	for _, a := range g.Assignments {
		if a.CourseID == courseID {
			out = append(out, a)
		}
	}
	return out
}

// Missing returns a student's ungraded assignments, optionally scoped to a course.
func (g *GradeBook) Missing(studentID string, courseID string) []Assignment {
	var out []Assignment
	for _, a := range g.Assignments {
		if courseID != "" && a.CourseID != courseID {
			continue
		}
		if _, ok := g.Grades[GradeKey{studentID, a.ID}]; !ok {
			out = append(out, a)
		}
	}
	return out
}

// CourseAverage is the weighted percentage across the course's assignments.
// missingAsZero=true penalizes ungraded assignments; false skips them.
// Returns (0, false) if no weighted data is available.
func (g *GradeBook) CourseAverage(studentID, courseID string, missingAsZero bool) (float64, bool) {
	assigns := g.CourseAssignments(courseID)
	if len(assigns) == 0 {
		return 0, false
	}
	weightedSum := 0.0
	weightTotal := 0.0
	for _, a := range assigns {
		var percent float64
		grade, ok := g.GradeOf(studentID, a.ID)
		switch {
		case ok && a.MaxPoints > 0:
			percent = grade.Points / a.MaxPoints * 100.0
		case ok:
			percent = 0
		case !ok && missingAsZero:
			percent = 0
		default:
			continue
		}
		weightedSum += percent * a.Weight
		weightTotal += a.Weight
	}
	if weightTotal == 0 {
		return 0, false
	}
	return weightedSum / weightTotal, true
}

// GPA is the credit-weighted average on a 4.0 scale.
func (g *GradeBook) GPA(studentID string, missingAsZero bool) (float64, bool) {
	totalCredits := 0.0
	weighted := 0.0
	for _, c := range g.Courses {
		avg, ok := g.CourseAverage(studentID, c.ID, missingAsZero)
		if !ok {
			continue
		}
		_, pts := PercentToLetter(avg)
		weighted += pts * c.Credits
		totalCredits += c.Credits
	}
	if totalCredits == 0 {
		return 0, false
	}
	return weighted / totalCredits, true
}

// ClassAverage is the mean percentage across all recorded grades for one assignment.
func (g *GradeBook) ClassAverage(assignmentID string) (float64, bool) {
	a, ok := g.Assignments[assignmentID]
	if !ok || a.MaxPoints <= 0 {
		return 0, false
	}
	total := 0.0
	count := 0
	for _, grade := range g.Grades {
		if grade.AssignmentID == assignmentID {
			total += grade.Points
			count++
		}
	}
	if count == 0 {
		return 0, false
	}
	return total / float64(count) / a.MaxPoints * 100.0, true
}

// Distribution returns a letter-grade histogram for a course.
func (g *GradeBook) Distribution(courseID string) map[string]int {
	hist := make(map[string]int)
	for _, s := range g.Students {
		avg, ok := g.CourseAverage(s.ID, courseID, false)
		if !ok {
			continue
		}
		letter, _ := PercentToLetter(avg)
		hist[letter]++
	}
	return hist
}
