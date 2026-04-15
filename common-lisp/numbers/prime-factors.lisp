;;;; Prime factorization — find all prime factors of a number.

(defpackage :numbers/prime-factors
  (:use :cl)
  (:export :prime-factors :prime-factorization))

(in-package :numbers/prime-factors)

(defun prime-factors (n)
  "Return the prime factors of N in ascending order, with repetition."
  (when (< n 2) (return-from prime-factors nil))
  (let ((factors nil)
        (d 2))
    (loop while (<= (* d d) n)
          do (loop while (zerop (mod n d))
                   do (push d factors)
                      (setf n (floor n d)))
             (incf d))
    (when (> n 1)
      (push n factors))
    (nreverse factors)))

(defun prime-factorization (n)
  "Return prime factorization as an alist of (prime . exponent)."
  (let ((factors (prime-factors n))
        (result nil))
    (dolist (f factors)
      (let ((entry (assoc f result)))
        (if entry
            (incf (cdr entry))
            (push (cons f 1) result))))
    (nreverse result)))
