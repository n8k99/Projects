;;;; Josephus Problem — circular elimination and the survivor.
;;;;
;;;; N people in a ring (0..n). Starting at 0, count K and remove that
;;;; person; continue from the next remaining position. The last one
;;;; standing is the survivor.
;;;;
;;;; Two different queries on the same process:
;;;;   JOSEPHUS            — O(n) recurrence, survivor only.
;;;;   JOSEPHUS-SIMULATE   — O(n²) simulation, survivor + elimination order.
;;;;
;;;; The recurrence is the first closed-form shortcut in Rosetta Stone
;;;; Classes: a problem whose answer can be computed without materialising
;;;; the process.

(defpackage #:josephus
  (:use #:cl)
  (:export #:josephus
           #:josephus-simulate))

(in-package #:josephus)

(defun josephus (n k)
  "Return the 0-indexed survivor via the O(n) recurrence
   J(1,k)=0 ;  J(n,k) = (J(n-1,k) + k) mod n."
  (check-type n (integer 1 *))
  (check-type k (integer 1 *))
  (let ((survivor 0))
    (loop for m from 2 to n do
      (setf survivor (mod (+ survivor k) m)))
    survivor))

(defun josephus-simulate (n k)
  "Simulate elimination directly. Returns (values SURVIVOR ELIM-ORDER)."
  (check-type n (integer 1 *))
  (check-type k (integer 1 *))
  (let ((ring (loop for i from 0 below n collect i))
        (order nil)
        (pos 0))
    (loop while (> (length ring) 1) do
      (setf pos (mod (+ pos (1- k)) (length ring)))
      (let ((victim (nth pos ring)))
        (push victim order)
        ;; Remove the pos-th element, leaving pos pointing at the next one.
        (setf ring (append (subseq ring 0 pos) (subseq ring (1+ pos))))))
    (values (first ring) (nreverse order))))
