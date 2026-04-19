# G051 — Student Grade Book

> The join record carries a measurement. Aggregation lives on the edge, and composes through nested weighted means.

```yaml
id: G051
title: Student Grade Book
category: classes
requires: [G048-product-inventory, G049-movie-store, G006-mortgage]
provides: [valued-join, nested-weighted-aggregation, missing-vs-zero-policy]
```

## Insight: The Join Carries a Value, Not Just a Status

G049's rental had a status: active or returned. G050's reservation had a status: booked, cancelled, completed. G051's grade has a *number*. The relationship between a student and an assignment is not just "did it happen" — it's "how well did it happen, expressed as a measurement."

The valued join is more important than either endpoint. A student without grades is unscored. An assignment without grades is ungraded. The edge is the primary object; the nodes are just its endpoints. This is the shape of every performance record in the noosphere: the conversation carries the tone, the task completion carries the quality, the forex trade carries the P&L. The *relationship* is where the performance data lives.

## Insight: Aggregation Lives on the Edge

Because the join is valued, you can aggregate over it. Student's course average is the weighted mean of their grades in that course. Class average is the mean grade on one assignment across students. Distribution is the histogram of course averages across students. All three aggregations are folds over the grade table, grouped differently.

G048 aggregated a single entity's events (product → quantity on hand). G051 aggregates the *edges* between two entity sets. This is the primary analytical operation over a relational model: group by one endpoint, fold over the join, produce a per-endpoint measurement. Every dashboard in the noosphere is this shape.

## Insight: Nested Weighted Aggregation

Course average = `Σ(weight × grade) / Σ(weight)` over assignments.
GPA = `Σ(credit × course_gpa_points) / Σ(credit)` over courses.

Two levels of weighted mean. This is the first project where aggregation *composes*. And it generalizes cleanly: the temporal compression chain is the same shape.

- Daily pace = weighted average of today's task scores.
- Weekly pace = credit-weighted average of the seven daily paces.
- Monthly pace = day-weighted average of the four weekly paces.
- Quarterly, yearly — same structure, more levels.

Every `pace_check` in the noosphere is a course average; every GPA-style summary at a wider horizon is a nested weighted mean over those pace checks. G006 (mortgage) introduced the idea of amortized obligation; G051 gives the precise arithmetic. The temporal compression chain is a GPA calculator that runs forever.

## Insight: Missing Is Distinct From Zero

A student who hasn't submitted assignment #3 doesn't have a zero. They have *unknown*. Two policies both matter:

- **current_average**: fold only graded assignments. Answers: how are they doing on the work they've done?
- **projected_average (missing_as_zero)**: fold all assignments, treating missing as zero. Answers: if nothing else gets submitted, where do they land?

Both queries are valid — they answer different questions. The grade book must support both, because absence itself is informational, and the interpretation depends on the context of the question.

This applies throughout the noosphere. If Kathryn made no forex trades today, is today missing data (exclude from the month's P&L average) or a zero-P&L day (include)? Depends: was she expected to trade? Did market conditions prevent it? The `missing_as_zero` parameter is not an implementation detail — it is the shape of a question about expectations.

## Choreographic Case: Term-End Report Card

```innate
(@term-end-reports){
  @for each student in @students {
    @transcript <- {
      courses: @courses.map(c => {
        id: c.id,
        average: @gb/course-average{student: @student.id, course: c.id},
        missing: @gb/missing{student: @student.id, course: c.id}
      })
      gpa: @gb/gpa{student: @student.id}
      projected_gpa: @gb/gpa{student: @student.id, missing_as_zero: true}
    }

    concurrent {
      @teacher/review{transcript: @transcript}
      @advisor/flag_risk{transcript: @transcript}
    } join as @reviewed

    where {
      no_missing_in_final_report: @transcript.courses.every(.missing == [])
      gpa_matches_projected:      @transcript.gpa == @transcript.projected_gpa
      reviewed_by_two_parties:    @reviewed.teacher && @reviewed.advisor
    }
  }
}
```

The `where` fires on a specific failure mode: if `gpa != projected_gpa`, missing grades are inflating the current average. That's a flag, not an error — the advisor acts on it. The two aggregates viewed side by side turn a missing grade into a signal.

## Structures

```innate
(defstruct student
  id   : String
  name : String)

(defstruct course
  id      : String
  name    : String
  credits : Float)

(defstruct assignment
  id         : String
  course-id  : String
  name       : String
  max-points : Float
  weight     : Float)

(defstruct grade
  student-id    : String
  assignment-id : String
  points        : Float
  note          : String)

(defstruct grade-book
  students    : {String -> Student}
  courses     : {String -> Course}
  assignments : {String -> Assignment}
  grades      : {(String, String) -> Grade})
```

## Resolver Natives

```innate
@grade-book{}                                                     -> GradeBook
@gb/add-student{student}                                          -> GradeBook
@gb/add-course{course}                                            -> GradeBook
@gb/add-assignment{assignment}                                    -> GradeBook
@gb/record-grade{student-id, assignment-id, points}               -> Grade
@gb/grade-of{student-id, assignment-id}                           -> Grade?
@gb/course-average{student-id, course-id, missing-as-zero?}       -> Float?
@gb/gpa{student-id, missing-as-zero?}                             -> Float?
@gb/class-average{assignment-id}                                  -> Float?
@gb/missing{student-id, course-id?}                               -> [Assignment]
@gb/distribution{course-id}                                       -> {String -> Nat}
```

## Demo

```innate
(@demo){
  @gb <- @grade-book{}
    .add-student{id: "S001", name: "Alice"}
    .add-course{id: "CS101", name: "Intro", credits: 4}
    .add-assignment{id: "HW1", course-id: "CS101", max-points: 100, weight: 1}
    .add-assignment{id: "FIN", course-id: "CS101", max-points: 100, weight: 3}
  @gb <- @gb/record-grade{student-id: "S001", assignment-id: "HW1", points: 80}
  @gb <- @gb/record-grade{student-id: "S001", assignment-id: "FIN", points: 100}
  ;; course-average = (80*1 + 100*3) / 4 = 95
  @avg <- @gb/course-average{student-id: "S001", course-id: "CS101"}
  ;; => 95.0
}
```

## Where

Aggregations MUST be computed from the grade table, never stored. `course_average` MUST treat missing distinctly from zero by default. GPA MUST use the credit-weighted mean of course-level GPA points, not a mean of percentages. Those three rules are what makes a grade book a grade book, rather than a spreadsheet that happens to hold grades.
