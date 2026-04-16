;;;; Unit Converter — convert between measurement units.

(defpackage :numbers/unit-converter
  (:use :cl)
  (:export :convert-unit))

(in-package :numbers/unit-converter)

;;; Conversion factors to base unit per category.
;;; Base units: meters (distance), grams (weight), liters (volume).

(defparameter *unit-table*
  '(;; distance -> meters
    ("km"  :distance 1000.0d0)
    ("m"   :distance 1.0d0)
    ("mi"  :distance 1609.344d0)
    ("ft"  :distance 0.3048d0)
    ;; weight -> grams
    ("kg"  :weight 1000.0d0)
    ("g"   :weight 1.0d0)
    ("lb"  :weight 453.592d0)
    ("oz"  :weight 28.3495d0)
    ;; volume -> liters
    ("L"   :volume 1.0d0)
    ("gal" :volume 3.78541d0)
    ("mL"  :volume 0.001d0)
    ("cup" :volume 0.236588d0)))

(defun find-unit (name)
  "Return (category factor) for a ratio-based unit, or NIL."
  (let ((entry (find name *unit-table* :key #'first :test #'string=)))
    (when entry
      (values (second entry) (third entry)))))

(defun temperature-p (unit)
  (member unit '("celsius" "fahrenheit" "kelvin") :test #'string=))

(defun to-celsius (value unit)
  (cond
    ((string= unit "celsius")    value)
    ((string= unit "fahrenheit") (* (- value 32.0d0) (/ 5.0d0 9.0d0)))
    ((string= unit "kelvin")     (- value 273.15d0))
    (t (error "Unknown temperature unit: ~a" unit))))

(defun from-celsius (value unit)
  (cond
    ((string= unit "celsius")    value)
    ((string= unit "fahrenheit") (+ (* value (/ 9.0d0 5.0d0)) 32.0d0))
    ((string= unit "kelvin")     (+ value 273.15d0))
    (t (error "Unknown temperature unit: ~a" unit))))

(defun convert-unit (value from-unit to-unit)
  "Convert VALUE from FROM-UNIT to TO-UNIT.
Supports temperature (celsius, fahrenheit, kelvin),
distance (km, mi, m, ft), weight (kg, lb, oz, g),
and volume (L, gal, mL, cup)."
  (when (string= from-unit to-unit)
    (return-from convert-unit (coerce value 'double-float)))
  ;; Temperature: formula-based.
  (when (or (temperature-p from-unit) (temperature-p to-unit))
    (unless (and (temperature-p from-unit) (temperature-p to-unit))
      (error "Cannot convert ~a to ~a" from-unit to-unit))
    (return-from convert-unit
      (from-celsius (to-celsius (coerce value 'double-float) from-unit) to-unit)))
  ;; Ratio-based: normalize to base then to target.
  (multiple-value-bind (from-cat from-factor) (find-unit from-unit)
    (unless from-cat
      (error "Unknown unit: ~a" from-unit))
    (multiple-value-bind (to-cat to-factor) (find-unit to-unit)
      (unless to-cat
        (error "Unknown unit: ~a" to-unit))
      (unless (eq from-cat to-cat)
        (error "Cannot convert ~a (~a) to ~a (~a)"
               from-unit from-cat to-unit to-cat))
      (/ (* (coerce value 'double-float) from-factor) to-factor))))
