;;;; Find PI to the Nth digit using the Machin formula with arbitrary precision.
;;;; Common Lisp has native bignum — no external libraries needed.

(defpackage :numbers/pi
  (:use :cl)
  (:export :pi-digits))

(in-package :numbers/pi)

(defun arccot (x n-terms precision)
  "Compute arccot(x) using Taylor series to n-terms with given decimal precision.
   Returns a rational number scaled by 10^precision."
  (let ((power (floor (expt 10 precision) x))  ; x^(-1) * 10^precision
        (sum 0))
    (loop for i from 0 below n-terms
          for sign = 1 then (- sign)
          for k = 1 then (+ k 2)
          do (incf sum (* sign (floor power k)))
             (setf power (floor power (* x x))))
    sum))

(defun pi-digits (n)
  "Return PI to N decimal places as a string.
   Uses Machin's formula: pi/4 = 4*arccot(5) - arccot(239)"
  (when (< n 0)
    (error "n must be non-negative"))
  (when (= n 0)
    (return-from pi-digits "3"))
  (let* ((precision (+ n 10))
         (n-terms (+ precision 10))
         (result (* 4 (- (* 4 (arccot 5 n-terms precision))
                         (arccot 239 n-terms precision)))))
    (let* ((s (format nil "~a" result))
           (len (length s)))
      ;; result is pi * 10^precision — insert decimal point
      (let ((int-part (subseq s 0 1))
            (dec-part (subseq s 1 (min (1+ n) len))))
        (if (> n 0)
            (format nil "~a.~a" int-part dec-part)
            int-part)))))
