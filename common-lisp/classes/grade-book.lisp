;;;; Student Grade Book — the join record carries a measurement. Aggregation lives on the edge.

(defpackage #:grade-book
  (:use #:cl)
  (:export #:make-student
           #:student-id #:student-name
           #:make-course
           #:course-id #:course-name #:course-credits
           #:make-assignment
           #:assignment-id #:assignment-course-id #:assignment-name
           #:assignment-max-points #:assignment-weight
           #:make-grade
           #:grade-student-id #:grade-assignment-id #:grade-points
           #:make-grade-book
           #:add-student #:add-course #:add-assignment
           #:record-grade
           #:grade-of
           #:course-assignments
           #:missing
           #:course-average
           #:gpa
           #:class-average
           #:distribution
           #:percent-to-letter))

(in-package #:grade-book)

(defstruct student
  (id "" :type string)
  (name "" :type string))

(defstruct course
  (id "" :type string)
  (name "" :type string)
  (credits 1.0 :type number))

(defstruct assignment
  (id "" :type string)
  (course-id "" :type string)
  (name "" :type string)
  (max-points 100.0 :type number)
  (weight 1.0 :type number))

(defstruct grade
  (student-id "" :type string)
  (assignment-id "" :type string)
  (points 0.0 :type number)
  (note "" :type string))

(defstruct grade-book
  (students (make-hash-table :test 'equal))
  (courses (make-hash-table :test 'equal))
  (assignments (make-hash-table :test 'equal))
  ;; grades keyed by cons (student-id . assignment-id) for idempotent record
  (grades (make-hash-table :test 'equal)))

(defparameter *letter-scale*
  '((93 "A"  4.0)
    (90 "A-" 3.7)
    (87 "B+" 3.3)
    (83 "B"  3.0)
    (80 "B-" 2.7)
    (77 "C+" 2.3)
    (73 "C"  2.0)
    (70 "C-" 1.7)
    (67 "D+" 1.3)
    (63 "D"  1.0)
    (60 "D-" 0.7)))

(defun percent-to-letter (percent)
  "Return (values letter gpa-points) for a percentage score."
  (dolist (band *letter-scale*)
    (destructuring-bind (cutoff letter gpa) band
      (when (>= percent cutoff)
        (return-from percent-to-letter (values letter gpa)))))
  (values "F" 0.0))

(defun add-student (gb s)
  (when (gethash (student-id s) (grade-book-students gb))
    (error "duplicate student: ~A" (student-id s)))
  (setf (gethash (student-id s) (grade-book-students gb)) s)
  s)

(defun add-course (gb c)
  (when (gethash (course-id c) (grade-book-courses gb))
    (error "duplicate course: ~A" (course-id c)))
  (setf (gethash (course-id c) (grade-book-courses gb)) c)
  c)

(defun add-assignment (gb a)
  (unless (gethash (assignment-course-id a) (grade-book-courses gb))
    (error "unknown course: ~A" (assignment-course-id a)))
  (when (gethash (assignment-id a) (grade-book-assignments gb))
    (error "duplicate assignment: ~A" (assignment-id a)))
  (setf (gethash (assignment-id a) (grade-book-assignments gb)) a)
  a)

(defun record-grade (gb student-id assignment-id points &optional (note ""))
  (unless (gethash student-id (grade-book-students gb))
    (error "unknown student: ~A" student-id))
  (unless (gethash assignment-id (grade-book-assignments gb))
    (error "unknown assignment: ~A" assignment-id))
  (when (minusp points)
    (error "points must be non-negative"))
  (let ((g (make-grade :student-id student-id
                       :assignment-id assignment-id
                       :points points
                       :note note)))
    (setf (gethash (cons student-id assignment-id) (grade-book-grades gb)) g)
    g))

(defun grade-of (gb student-id assignment-id)
  (gethash (cons student-id assignment-id) (grade-book-grades gb)))

(defun course-assignments (gb course-id)
  (let ((result nil))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (string= (assignment-course-id a) course-id)
                 (push a result)))
             (grade-book-assignments gb))
    result))

(defun missing (gb student-id &optional course-id)
  (let ((result nil))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (or (null course-id)
                         (string= (assignment-course-id a) course-id))
                 (unless (grade-of gb student-id (assignment-id a))
                   (push a result))))
             (grade-book-assignments gb))
    result))

(defun course-average (gb student-id course-id &key (missing-as-zero nil))
  "Weighted percentage across course assignments, or NIL if no data."
  (let ((assigns (course-assignments gb course-id))
        (weighted-sum 0.0)
        (weight-total 0.0))
    (unless assigns (return-from course-average nil))
    (dolist (a assigns)
      (let* ((g (grade-of gb student-id (assignment-id a)))
             (percent (cond
                        ((and g (plusp (assignment-max-points a)))
                         (* (/ (grade-points g) (assignment-max-points a)) 100.0))
                        (g 0.0)
                        ((and (null g) missing-as-zero) 0.0)
                        (t nil))))
        (when percent
          (incf weighted-sum (* percent (assignment-weight a)))
          (incf weight-total (assignment-weight a)))))
    (if (zerop weight-total)
        nil
        (/ weighted-sum weight-total))))

(defun gpa (gb student-id &key (missing-as-zero nil))
  (let ((total-credits 0.0)
        (weighted 0.0))
    (maphash (lambda (id c)
               (declare (ignore id))
               (let ((avg (course-average gb student-id (course-id c)
                                          :missing-as-zero missing-as-zero)))
                 (when avg
                   (multiple-value-bind (letter points) (percent-to-letter avg)
                     (declare (ignore letter))
                     (incf weighted (* points (course-credits c)))
                     (incf total-credits (course-credits c))))))
             (grade-book-courses gb))
    (if (zerop total-credits) nil (/ weighted total-credits))))

(defun class-average (gb assignment-id)
  (let ((a (gethash assignment-id (grade-book-assignments gb))))
    (unless a (error "unknown assignment: ~A" assignment-id))
    (when (<= (assignment-max-points a) 0)
      (return-from class-average nil))
    (let ((sum 0.0) (count 0))
      (maphash (lambda (k g)
                 (declare (ignore k))
                 (when (string= (grade-assignment-id g) assignment-id)
                   (incf sum (grade-points g))
                   (incf count)))
               (grade-book-grades gb))
      (if (zerop count) nil
          (* (/ (/ sum count) (assignment-max-points a)) 100.0)))))

(defun distribution (gb course-id)
  (let ((hist (make-hash-table :test 'equal)))
    (maphash (lambda (id s)
               (declare (ignore id))
               (let ((avg (course-average gb (student-id s) course-id)))
                 (when avg
                   (multiple-value-bind (letter points) (percent-to-letter avg)
                     (declare (ignore points))
                     (incf (gethash letter hist 0))))))
             (grade-book-students gb))
    hist))
