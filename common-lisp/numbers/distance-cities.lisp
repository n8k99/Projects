;;;; Distance Between Two Cities — great-circle distance via Haversine.

(defpackage :numbers/distance-cities
  (:use :cl)
  (:export :haversine :city-distance))

(in-package :numbers/distance-cities)

;;; Earth's mean radius in kilometres.
(defconstant +earth-radius-km+ 6371.0d0)

;;; City coordinates: (name latitude longitude) in degrees.
(defparameter *cities*
  '(("New York"   40.7128d0  -74.0060d0)
    ("London"     51.5074d0   -0.1278d0)
    ("Tokyo"      35.6762d0  139.6503d0)
    ("Sydney"    -33.8688d0  151.2093d0)
    ("São Paulo" -23.5505d0  -46.6333d0)
    ("Cairo"      30.0444d0   31.2357d0)))

(defun deg->rad (degrees)
  "Convert degrees to radians."
  (* degrees (/ pi 180.0d0)))

(defun haversine (lat1 lon1 lat2 lon2)
  "Return great-circle distance in km between two points given in degrees."
  (let* ((lat1 (deg->rad lat1))
         (lon1 (deg->rad lon1))
         (lat2 (deg->rad lat2))
         (lon2 (deg->rad lon2))
         (dlat (- lat2 lat1))
         (dlon (- lon2 lon1))
         (a (+ (expt (sin (/ dlat 2.0d0)) 2)
               (* (cos lat1) (cos lat2)
                  (expt (sin (/ dlon 2.0d0)) 2))))
         (c (* 2.0d0 (atan (sqrt a) (sqrt (- 1.0d0 a))))))
    (* +earth-radius-km+ c)))

(defun find-city (name)
  "Return (latitude longitude) for a named city, or NIL."
  (let ((entry (find name *cities* :key #'first :test #'string=)))
    (when entry
      (values (second entry) (third entry)))))

(defun city-distance (city1 city2)
  "Return great-circle distance in km between two named cities."
  (multiple-value-bind (lat1 lon1) (find-city city1)
    (unless lat1
      (error "Unknown city: ~a" city1))
    (multiple-value-bind (lat2 lon2) (find-city city2)
      (unless lat2
        (error "Unknown city: ~a" city2))
      (haversine lat1 lon1 lat2 lon2))))
