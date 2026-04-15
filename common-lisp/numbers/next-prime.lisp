;;;; Next prime number — continuously find primes on demand.

(defpackage :numbers/next-prime
  (:use :cl)
  (:export :is-prime :next-prime :nth-prime))

(in-package :numbers/next-prime)

(defun is-prime (n)
  "Test whether N is prime."
  (cond
    ((< n 2) nil)
    ((< n 4) t)
    ((or (zerop (mod n 2)) (zerop (mod n 3))) nil)
    (t (loop for d from 5 by 6
             while (<= (* d d) n)
             never (or (zerop (mod n d))
                       (zerop (mod n (+ d 2))))))))

(defun next-prime (n)
  "Return the smallest prime greater than N."
  (let ((candidate (if (>= n 2) (1+ n) 2)))
    (when (= candidate 2) (return-from next-prime 2))
    (when (evenp candidate) (incf candidate))
    (loop until (is-prime candidate)
          do (incf candidate 2))
    candidate))

(defun nth-prime (n)
  "Return the Nth prime (1-indexed: nth-prime(1) = 2)."
  (when (< n 1) (error "n must be positive"))
  (let ((count 0)
        (candidate 2))
    (loop
      (when (is-prime candidate)
        (incf count)
        (when (= count n)
          (return candidate)))
      (incf candidate (if (= candidate 2) 1 2)))))
