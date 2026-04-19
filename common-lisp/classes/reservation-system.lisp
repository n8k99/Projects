;;;; Reservation System — bookable resources with time intervals and conflict detection.
;;;;
;;;; Models airline seats and hotel rooms uniformly. Overlap is the core primitive:
;;;; two reservations on the same resource collide iff their half-open intervals
;;;; [start, end) overlap.

(defpackage #:reservation-system
  (:use #:cl)
  (:export #:make-resource
           #:resource-id #:resource-kind #:resource-label
           #:make-customer
           #:customer-id #:customer-name
           #:make-reservation
           #:reservation-id #:reservation-resource-id #:reservation-customer-id
           #:reservation-start #:reservation-end #:reservation-status
           #:reservation-active-p
           #:reservation-overlaps-p
           #:make-reservation-system
           #:add-resource
           #:add-customer
           #:book
           #:cancel-reservation
           #:conflicts
           #:available
           #:occupancy
           #:customer-bookings))

(in-package #:reservation-system)

(defstruct resource
  (id "" :type string)
  (kind "" :type string)
  (label "" :type string))

(defstruct customer
  (id "" :type string)
  (name "" :type string))

(defstruct reservation
  (id 0 :type integer)
  (resource-id "" :type string)
  (customer-id "" :type string)
  (start 0 :type integer)                    ; universal time
  (end 0 :type integer)
  (status :booked :type keyword))            ; :booked :cancelled :completed

(defun reservation-active-p (r)
  (eq (reservation-status r) :booked))

(defun reservation-overlaps-p (r start end)
  "Half-open overlap: touching boundaries do not conflict."
  (and (< (reservation-start r) end)
       (< start (reservation-end r))))

(defstruct reservation-system
  (resources (make-hash-table :test 'equal))
  (customers (make-hash-table :test 'equal))
  (reservations (make-hash-table :test 'eql))
  (next-id 1 :type integer))

(defun add-resource (system res)
  (when (gethash (resource-id res) (reservation-system-resources system))
    (error "duplicate resource id: ~A" (resource-id res)))
  (setf (gethash (resource-id res) (reservation-system-resources system)) res)
  res)

(defun add-customer (system cust)
  (when (gethash (customer-id cust) (reservation-system-customers system))
    (error "duplicate customer id: ~A" (customer-id cust)))
  (setf (gethash (customer-id cust) (reservation-system-customers system)) cust)
  cust)

(defun %lookup-resource (system id)
  (or (gethash id (reservation-system-resources system))
      (error "unknown resource: ~A" id)))

(defun %lookup-customer (system id)
  (or (gethash id (reservation-system-customers system))
      (error "unknown customer: ~A" id)))

(defun %validate-interval (start end)
  (unless (< start end)
    (error "start must be before end (got [~A, ~A))" start end)))

(defun conflicts (system resource-id start end)
  "Active reservations on resource-id whose interval overlaps [start, end)."
  (%lookup-resource system resource-id)
  (%validate-interval start end)
  (let ((result nil))
    (maphash (lambda (k r)
               (declare (ignore k))
               (when (and (string= (reservation-resource-id r) resource-id)
                          (reservation-active-p r)
                          (reservation-overlaps-p r start end))
                 (push r result)))
             (reservation-system-reservations system))
    result))

(defun book (system resource-id customer-id start end)
  (%lookup-resource system resource-id)
  (%lookup-customer system customer-id)
  (%validate-interval start end)
  (let ((colliding (conflicts system resource-id start end)))
    (when colliding
      (error "conflict: ~A overlaps ~D existing reservation(s)"
             resource-id (length colliding))))
  (let* ((id (reservation-system-next-id system))
         (r (make-reservation :id id
                              :resource-id resource-id
                              :customer-id customer-id
                              :start start
                              :end end
                              :status :booked)))
    (setf (gethash id (reservation-system-reservations system)) r)
    (incf (reservation-system-next-id system))
    r))

(defun cancel-reservation (system reservation-id)
  (let ((r (gethash reservation-id (reservation-system-reservations system))))
    (unless r (error "unknown reservation: ~A" reservation-id))
    (when (eq (reservation-status r) :cancelled)
      (error "reservation ~A already cancelled" reservation-id))
    (setf (reservation-status r) :cancelled)
    r))

(defun available (system start end &optional kind)
  "Resources (optionally filtered by kind) with no overlap on [start, end)."
  (%validate-interval start end)
  (let ((result nil))
    (maphash (lambda (id res)
               (declare (ignore id))
               (when (or (null kind) (string= (resource-kind res) kind))
                 (unless (conflicts system (resource-id res) start end)
                   (push res result))))
             (reservation-system-resources system))
    result))

(defun occupancy (system resource-id)
  (%lookup-resource system resource-id)
  (let ((active nil))
    (maphash (lambda (k r)
               (declare (ignore k))
               (when (and (string= (reservation-resource-id r) resource-id)
                          (reservation-active-p r))
                 (push r active)))
             (reservation-system-reservations system))
    (sort active #'< :key #'reservation-start)))

(defun customer-bookings (system customer-id)
  (%lookup-customer system customer-id)
  (let ((result nil))
    (maphash (lambda (k r)
               (declare (ignore k))
               (when (string= (reservation-customer-id r) customer-id)
                 (push r result)))
             (reservation-system-reservations system))
    result))
