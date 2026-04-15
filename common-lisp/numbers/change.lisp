;;;; Change return program — break an amount into denominations.

(defpackage :numbers/change
  (:use :cl)
  (:export :make-change :cover-obligations))

(in-package :numbers/change)

(defun make-change (amount &optional (denominations '(25 10 5 1)))
  "Break AMOUNT (in cents) into fewest coins. Returns alist of (denomination . count)."
  (let ((remaining amount)
        (result nil))
    (dolist (denom (sort (copy-list denominations) #'>))
      (let ((count (floor remaining denom)))
        (when (plusp count)
          (push (cons denom count) result)
          (decf remaining (* count denom)))))
    (nreverse result)))

(defun cover-obligations (available obligations)
  "Cover obligations in order of due-day. OBLIGATIONS: list of (name amount due-day).
   Returns (covered uncovered remaining)."
  (let ((sorted (sort (copy-list obligations) #'< :key #'third))
        (remaining available)
        (covered nil)
        (uncovered nil))
    (dolist (ob sorted)
      (destructuring-bind (name amount due-day) ob
        (declare (ignore due-day))
        (if (>= remaining amount)
            (progn
              (decf remaining amount)
              (push ob covered))
            (push ob uncovered))))
    (values (nreverse covered) (nreverse uncovered) remaining)))
