;;;; Tax Calculator — progressive income tax computation.

(defpackage :numbers/tax-calculator
  (:use :cl)
  (:export :calculate-tax))

(in-package :numbers/tax-calculator)

;; 2024 US federal single-filer brackets: (upper-limit rate).
(defparameter *brackets*
  '((11600 1/10)
    (47150 3/25)
    (100525 11/50)
    (191950 6/25)
    (243725 8/25)
    (609350 7/20)
    (nil 37/100)))

(defun calculate-tax (income)
  "Calculate progressive income tax. Returns plist with :total-tax, :effective-rate,
:marginal-rate, and :breakdown (list of plists with :floor, :rate, :taxable, :tax)."
  (if (<= income 0)
      (list :total-tax 0 :effective-rate 0 :marginal-rate 0 :breakdown nil)
      (let ((total-tax 0)
            (prev-limit 0)
            (marginal-rate 0)
            (breakdown nil))
        (dolist (bracket *brackets*)
          (destructuring-bind (limit rate) bracket
            (when (> income prev-limit)
              (let* ((cap (if limit (min income limit) income))
                     (taxable (- cap prev-limit))
                     (tax (* taxable rate)))
                (incf total-tax tax)
                (setf marginal-rate rate)
                (push (list :floor prev-limit :rate rate :taxable taxable :tax tax)
                      breakdown)
                (setf prev-limit (or limit income))))))
        (list :total-tax (coerce total-tax 'double-float)
              :effective-rate (coerce (/ total-tax income) 'double-float)
              :marginal-rate (coerce marginal-rate 'double-float)
              :breakdown (nreverse breakdown)))))
