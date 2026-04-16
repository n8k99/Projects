;;;; Binary to Decimal and Back — convert between number systems.
;;;; CL has FORMAT ~B and PARSE-INTEGER :radix built in,
;;;; but we implement the algorithms explicitly.

(defpackage :numbers/binary
  (:use :cl)
  (:export :to-binary :to-decimal :to-base :from-base))

(in-package :numbers/binary)

(defun to-binary (n)
  "Convert a non-negative integer to its binary string."
  (assert (>= n 0) (n) "n must be non-negative")
  (if (zerop n)
      "0"
      (let ((bits '()))
        (loop while (plusp n)
              do (push (if (oddp n) #\1 #\0) bits)
                 (setf n (ash n -1)))
        (coerce bits 'string))))

(defun to-decimal (binary-string)
  "Convert a binary string to its decimal integer."
  (assert (plusp (length binary-string)) () "empty string")
  (loop for c across binary-string
        do (assert (or (char= c #\0) (char= c #\1)) ()
             "invalid binary digit: ~a" c))
  (loop for c across binary-string
        for result = (digit-char-p c) then (+ (ash result 1) (digit-char-p c))
        finally (return result)))

(defun to-base (n base)
  "Convert a non-negative integer to a string in the given base (2-36)."
  (assert (and (>= base 2) (<= base 36)) (base) "base must be 2-36")
  (assert (>= n 0) (n) "n must be non-negative")
  (if (zerop n)
      "0"
      (let ((digits "0123456789abcdefghijklmnopqrstuvwxyz")
            (chars '()))
        (loop while (plusp n)
              do (push (char digits (mod n base)) chars)
                 (setf n (floor n base)))
        (coerce chars 'string))))

(defun from-base (string base)
  "Convert a string in the given base (2-36) to a decimal integer."
  (assert (and (>= base 2) (<= base 36)) (base) "base must be 2-36")
  (assert (plusp (length string)) () "empty string")
  (loop for c across (string-downcase string)
        for val = (digit-char-p c 36)
        do (assert (and val (< val base)) ()
             "digit '~a' invalid for base ~a" c base)
        for result = val then (+ (* result base) val)
        finally (return result)))
