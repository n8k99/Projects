;;;; Tile cost calculator — total cost to cover a W x H floor.

(defpackage :numbers/tile-cost
  (:use :cl)
  (:export :tile-cost))

(in-package :numbers/tile-cost)

(defun tile-cost (floor-width floor-height tile-width tile-height cost-per-tile)
  "Calculate tiles needed and total cost. Returns (tiles-wide tiles-high total-tiles total-cost)."
  (let* ((tiles-wide (ceiling floor-width tile-width))
         (tiles-high (ceiling floor-height tile-height))
         (total-tiles (* tiles-wide tiles-high))
         (total-cost (* total-tiles cost-per-tile)))
    (values tiles-wide tiles-high total-tiles total-cost)))
