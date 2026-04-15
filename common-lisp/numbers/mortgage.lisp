;;;; Mortgage calculator — monthly payments, amortization, pace tracking.

(defpackage :numbers/mortgage
  (:use :cl)
  (:export :monthly-payment :total-cost :amortization-schedule :pace-check))

(in-package :numbers/mortgage)

(defun monthly-payment (principal annual-rate years)
  "Calculate fixed monthly payment for a mortgage."
  (if (zerop annual-rate)
      (/ principal (* years 12))
      (let* ((r (/ annual-rate 100 12))
             (n (* years 12))
             (factor (expt (1+ r) n)))
        (/ (* principal r factor) (1- factor)))))

(defun total-cost (principal annual-rate years)
  "Return (monthly-payment total-payments total-paid total-interest)."
  (let* ((pmt (monthly-payment principal annual-rate years))
         (n (* years 12))
         (total (* pmt n))
         (interest (- total principal)))
    (values pmt n total interest)))

(defun amortization-schedule (principal annual-rate years)
  "Generate amortization schedule as list of (period payment principal interest balance)."
  (let* ((pmt (monthly-payment principal annual-rate years))
         (r (/ annual-rate 100 12))
         (balance principal)
         (schedule nil))
    (loop for period from 1 to (* years 12)
          for interest = (* balance r)
          for princ = (- pmt interest)
          do (decf balance princ)
             (push (list period pmt princ interest balance) schedule))
    (nreverse schedule)))

(defun pace-check (target periods current-period cumulative)
  "Check if cumulative payments are on pace. Returns (expected delta pace-pct on-track-p)."
  (let* ((expected (* target (/ current-period periods)))
         (delta (- cumulative expected))
         (pace (if (plusp expected) (* (/ cumulative expected) 100) 0)))
    (values expected delta pace (>= delta 0))))
