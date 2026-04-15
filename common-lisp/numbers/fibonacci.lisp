;;;; Fibonacci sequence — generate to a count or up to a value.
;;;; Common Lisp native bignums handle arbitrarily large Fibonacci numbers.

(defpackage :numbers/fibonacci
  (:use :cl)
  (:export :fibonacci :fibonacci-up-to :fib))

(in-package :numbers/fibonacci)

(defun fibonacci (n)
  "Return the first N Fibonacci numbers as a list."
  (when (< n 0) (error "n must be non-negative"))
  (cond
    ((= n 0) nil)
    ((= n 1) '(0))
    (t (let ((seq (list 0 1)))
         (loop for i from 2 below n
               for next = (+ (car (last seq)) (car (last (butlast seq))))
               do (setf seq (append seq (list next))))
         seq))))

(defun fibonacci-up-to (limit)
  "Return all Fibonacci numbers up to and including LIMIT."
  (when (< limit 0) (return-from fibonacci-up-to nil))
  (let ((seq (list 0))
        (a 0)
        (b 1))
    (loop while (<= b limit)
          do (push b seq)
             (psetf a b b (+ a b)))
    (nreverse seq)))

(defun fib (n)
  "Return the Nth Fibonacci number (0-indexed)."
  (when (< n 0) (error "n must be non-negative"))
  (let ((a 0) (b 1))
    (loop for i from 0 below n
          do (psetf a b b (+ a b)))
    a))
