-- Student Grade Book — the join record carries a measurement. Aggregation lives on the edge.

namespace Classes

structure Student where
  id : String
  name : String
  deriving Repr

structure Course where
  id : String
  name : String
  credits : Float := 1.0
  deriving Repr

structure Assignment where
  id : String
  courseId : String
  name : String
  maxPoints : Float := 100.0
  weight : Float := 1.0
  deriving Repr

structure Grade where
  studentId : String
  assignmentId : String
  points : Float
  note : String := ""
  deriving Repr

structure GradeBook where
  students : List Student := []
  courses : List Course := []
  assignments : List Assignment := []
  grades : List Grade := []
  deriving Repr

def GradeBook.empty : GradeBook := { }

-- US letter scale → 4.0 GPA.
def percentToLetter (percent : Float) : String × Float :=
  if percent ≥ 93.0 then ("A",  4.0)
  else if percent ≥ 90.0 then ("A-", 3.7)
  else if percent ≥ 87.0 then ("B+", 3.3)
  else if percent ≥ 83.0 then ("B",  3.0)
  else if percent ≥ 80.0 then ("B-", 2.7)
  else if percent ≥ 77.0 then ("C+", 2.3)
  else if percent ≥ 73.0 then ("C",  2.0)
  else if percent ≥ 70.0 then ("C-", 1.7)
  else if percent ≥ 67.0 then ("D+", 1.3)
  else if percent ≥ 63.0 then ("D",  1.0)
  else if percent ≥ 60.0 then ("D-", 0.7)
  else ("F", 0.0)

def GradeBook.findStudent (gb : GradeBook) (id : String) : Option Student :=
  gb.students.find? (·.id == id)

def GradeBook.findCourse (gb : GradeBook) (id : String) : Option Course :=
  gb.courses.find? (·.id == id)

def GradeBook.findAssignment (gb : GradeBook) (id : String) : Option Assignment :=
  gb.assignments.find? (·.id == id)

def GradeBook.addStudent (gb : GradeBook) (s : Student) : Except String GradeBook :=
  if gb.students.any (·.id == s.id)
  then .error s!"duplicate student: {s.id}"
  else .ok { gb with students := gb.students ++ [s] }

def GradeBook.addCourse (gb : GradeBook) (c : Course) : Except String GradeBook :=
  if gb.courses.any (·.id == c.id)
  then .error s!"duplicate course: {c.id}"
  else .ok { gb with courses := gb.courses ++ [c] }

def GradeBook.addAssignment (gb : GradeBook) (a : Assignment) : Except String GradeBook :=
  if (gb.findCourse a.courseId).isNone then
    .error s!"unknown course: {a.courseId}"
  else if gb.assignments.any (·.id == a.id) then
    .error s!"duplicate assignment: {a.id}"
  else
    .ok { gb with assignments := gb.assignments ++ [a] }

def GradeBook.gradeOf (gb : GradeBook) (studentId assignmentId : String) : Option Grade :=
  gb.grades.find? fun g => g.studentId == studentId && g.assignmentId == assignmentId

def GradeBook.recordGrade (gb : GradeBook) (studentId assignmentId : String)
    (points : Float) (note : String := "") : Except String GradeBook :=
  if (gb.findStudent studentId).isNone then
    .error s!"unknown student: {studentId}"
  else if (gb.findAssignment assignmentId).isNone then
    .error s!"unknown assignment: {assignmentId}"
  else if points < 0.0 then
    .error "points must be non-negative"
  else
    let g : Grade := { studentId, assignmentId, points, note }
    let without := gb.grades.filter fun x =>
      ¬ (x.studentId == studentId && x.assignmentId == assignmentId)
    .ok { gb with grades := without ++ [g] }

def GradeBook.courseAssignments (gb : GradeBook) (courseId : String) : List Assignment :=
  gb.assignments.filter (·.courseId == courseId)

def GradeBook.missing (gb : GradeBook) (studentId : String) (courseId : Option String := none)
    : List Assignment :=
  let pool := match courseId with
    | some c => gb.courseAssignments c
    | none => gb.assignments
  pool.filter fun a => (gb.gradeOf studentId a.id).isNone

/-- Weighted percentage across the course's assignments, or none if no weighted data. -/
def GradeBook.courseAverage (gb : GradeBook) (studentId courseId : String)
    (missingAsZero : Bool := false) : Option Float :=
  let assigns := gb.courseAssignments courseId
  if assigns.isEmpty then none
  else
    let (weightedSum, weightTotal) := assigns.foldl
      (fun (acc : Float × Float) (a : Assignment) =>
        match gb.gradeOf studentId a.id with
        | some g =>
          let percent := if a.maxPoints > 0.0 then g.points / a.maxPoints * 100.0 else 0.0
          (acc.1 + percent * a.weight, acc.2 + a.weight)
        | none =>
          if missingAsZero then (acc.1, acc.2 + a.weight)
          else acc)
      (0.0, 0.0)
    if weightTotal == 0.0 then none else some (weightedSum / weightTotal)

def GradeBook.gpa (gb : GradeBook) (studentId : String) (missingAsZero : Bool := false)
    : Option Float :=
  let (weighted, totalCredits) := gb.courses.foldl
    (fun (acc : Float × Float) (c : Course) =>
      match gb.courseAverage studentId c.id missingAsZero with
      | some avg =>
        let (_, pts) := percentToLetter avg
        (acc.1 + pts * c.credits, acc.2 + c.credits)
      | none => acc)
    (0.0, 0.0)
  if totalCredits == 0.0 then none else some (weighted / totalCredits)

def GradeBook.classAverage (gb : GradeBook) (assignmentId : String) : Option Float :=
  match gb.findAssignment assignmentId with
  | none => none
  | some a =>
    if a.maxPoints ≤ 0.0 then none
    else
      let grades := gb.grades.filter (·.assignmentId == assignmentId)
      if grades.isEmpty then none
      else
        let total := grades.foldl (fun acc g => acc + g.points) 0.0
        some (total / grades.length.toFloat / a.maxPoints * 100.0)

end Classes
