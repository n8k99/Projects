;;;; Movie Store — two entities joined by a rental, with due dates and temporal obligations.

(defpackage #:movie-store
  (:use #:cl)
  (:export #:make-movie
           #:movie-id #:movie-title #:movie-genre #:movie-copies-total
           #:make-customer
           #:customer-id #:customer-name
           #:make-rental
           #:rental-id #:rental-movie-id #:rental-customer-id
           #:rental-rented-at #:rental-due-at #:rental-returned-at
           #:rental-active-p
           #:rental-overdue-p
           #:make-movie-store
           #:add-movie
           #:add-customer
           #:rent
           #:return-movie
           #:copies-available
           #:active-rentals
           #:overdue
           #:movie-history
           #:customer-history
           #:by-genre))

(in-package #:movie-store)

(defstruct movie
  (id "" :type string)
  (title "" :type string)
  (director "" :type string)
  (year 0 :type integer)
  (genre "" :type string)
  (copies-total 1 :type integer))

(defstruct customer
  (id "" :type string)
  (name "" :type string))

(defstruct rental
  (id 0 :type integer)
  (movie-id "" :type string)
  (customer-id "" :type string)
  (rented-at 0 :type integer)        ; universal time
  (due-at 0 :type integer)
  (returned-at nil :type (or null integer)))

(defun rental-active-p (r)
  (null (rental-returned-at r)))

(defun rental-overdue-p (r now)
  (and (rental-active-p r) (> now (rental-due-at r))))

(defstruct movie-store
  (movies (make-hash-table :test 'equal))
  (customers (make-hash-table :test 'equal))
  (rentals (make-hash-table :test 'eql))
  (next-id 1 :type integer))

(defparameter *default-rental-days* 7)
(defparameter *seconds-per-day* 86400)

(defun add-movie (store movie)
  (when (gethash (movie-id movie) (movie-store-movies store))
    (error "duplicate movie id: ~A" (movie-id movie)))
  (setf (gethash (movie-id movie) (movie-store-movies store)) movie)
  movie)

(defun add-customer (store customer)
  (when (gethash (customer-id customer) (movie-store-customers store))
    (error "duplicate customer id: ~A" (customer-id customer)))
  (setf (gethash (customer-id customer) (movie-store-customers store)) customer)
  customer)

(defun lookup-movie (store id)
  (or (gethash id (movie-store-movies store))
      (error "unknown movie: ~A" id)))

(defun lookup-customer (store id)
  (or (gethash id (movie-store-customers store))
      (error "unknown customer: ~A" id)))

(defun rent (store movie-id customer-id &key (duration-days *default-rental-days*)
                                              (now (get-universal-time)))
  (lookup-movie store movie-id)
  (lookup-customer store customer-id)
  (when (zerop (copies-available store movie-id))
    (error "no copies available: ~A" movie-id))
  (let* ((id (movie-store-next-id store))
         (r (make-rental :id id
                         :movie-id movie-id
                         :customer-id customer-id
                         :rented-at now
                         :due-at (+ now (* duration-days *seconds-per-day*)))))
    (setf (gethash id (movie-store-rentals store)) r)
    (incf (movie-store-next-id store))
    r))

(defun return-movie (store rental-id &optional (now (get-universal-time)))
  (let ((r (gethash rental-id (movie-store-rentals store))))
    (unless r (error "unknown rental: ~A" rental-id))
    (when (rental-returned-at r)
      (error "rental ~A already returned" rental-id))
    (setf (rental-returned-at r) now)
    r))

(defun copies-available (store movie-id)
  (let ((movie (lookup-movie store movie-id))
        (active 0))
    (maphash (lambda (k r)
               (declare (ignore k))
               (when (and (string= (rental-movie-id r) movie-id)
                          (rental-active-p r))
                 (incf active)))
             (movie-store-rentals store))
    (- (movie-copies-total movie) active)))

(defun collect-rentals (store pred)
  (let ((result nil))
    (maphash (lambda (k r)
               (declare (ignore k))
               (when (funcall pred r)
                 (push r result)))
             (movie-store-rentals store))
    result))

(defun active-rentals (store customer-id)
  (lookup-customer store customer-id)
  (collect-rentals store
                   (lambda (r) (and (string= (rental-customer-id r) customer-id)
                                    (rental-active-p r)))))

(defun overdue (store &optional (now (get-universal-time)))
  (collect-rentals store (lambda (r) (rental-overdue-p r now))))

(defun movie-history (store movie-id)
  (lookup-movie store movie-id)
  (collect-rentals store (lambda (r) (string= (rental-movie-id r) movie-id))))

(defun customer-history (store customer-id)
  (lookup-customer store customer-id)
  (collect-rentals store (lambda (r) (string= (rental-customer-id r) customer-id))))

(defun by-genre (store genre)
  (let ((result nil))
    (maphash (lambda (id m)
               (declare (ignore id))
               (when (string= (movie-genre m) genre)
                 (push m result)))
             (movie-store-movies store))
    result))
