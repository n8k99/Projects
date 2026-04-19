;;;; Shape Area and Perimeter — one interface, many implementations.
;;;;
;;;; Common Lisp dispatches on CLOS classes via generic functions. A defgeneric
;;;; declares the function; defmethod binds an implementation to a specific
;;;; argument class. Adding a new shape means adding a class + three defmethods
;;;; in ANY file — no base type needs modification. This is classic **open
;;;; polymorphism via multimethod dispatch**.

(defpackage #:shapes
  (:use #:cl)
  (:shadow #:area)
  (:export #:area
           #:perimeter
           #:shape-name
           #:circle #:make-circle #:circle-radius
           #:rectangle #:make-rectangle #:rectangle-width #:rectangle-height
           #:triangle #:make-triangle #:triangle-a #:triangle-b #:triangle-c
           #:regular-polygon #:make-regular-polygon
           #:regular-polygon-sides #:regular-polygon-side-length
           #:total-area
           #:total-perimeter
           #:largest-by-area
           #:sort-by-perimeter))

(in-package #:shapes)

;; --- generic functions ---

(defgeneric area (shape)
  (:documentation "Area of the shape."))
(defgeneric perimeter (shape)
  (:documentation "Perimeter of the shape."))
(defgeneric shape-name (shape)
  (:documentation "Human-readable shape name."))

;; --- Circle ---

(defclass circle ()
  ((radius :initarg :radius :reader circle-radius :type number)))

(defun make-circle (radius)
  (when (minusp radius)
    (error "negative radius: ~A" radius))
  (make-instance 'circle :radius radius))

(defmethod area ((c circle))
  (* pi (circle-radius c) (circle-radius c)))

(defmethod perimeter ((c circle))
  (* 2 pi (circle-radius c)))

(defmethod shape-name ((c circle)) "Circle")

;; --- Rectangle ---

(defclass rectangle ()
  ((width  :initarg :width  :reader rectangle-width  :type number)
   (height :initarg :height :reader rectangle-height :type number)))

(defun make-rectangle (width height)
  (when (or (minusp width) (minusp height))
    (error "negative dimension: ~A x ~A" width height))
  (make-instance 'rectangle :width width :height height))

(defmethod area ((r rectangle))
  (* (rectangle-width r) (rectangle-height r)))

(defmethod perimeter ((r rectangle))
  (* 2 (+ (rectangle-width r) (rectangle-height r))))

(defmethod shape-name ((r rectangle))
  (if (= (rectangle-width r) (rectangle-height r)) "Square" "Rectangle"))

;; --- Triangle ---

(defclass triangle ()
  ((a :initarg :a :reader triangle-a :type number)
   (b :initarg :b :reader triangle-b :type number)
   (c :initarg :c :reader triangle-c :type number)))

(defun make-triangle (a b c)
  (when (or (not (plusp a)) (not (plusp b)) (not (plusp c)))
    (error "non-positive side: ~A ~A ~A" a b c))
  (unless (and (> (+ a b) c) (> (+ a c) b) (> (+ b c) a))
    (error "triangle inequality violated: ~A ~A ~A" a b c))
  (make-instance 'triangle :a a :b b :c c))

(defmethod area ((tr triangle))
  (let* ((a (triangle-a tr))
         (b (triangle-b tr))
         (c (triangle-c tr))
         (s (/ (+ a b c) 2)))
    (sqrt (* s (- s a) (- s b) (- s c)))))

(defmethod perimeter ((tr triangle))
  (+ (triangle-a tr) (triangle-b tr) (triangle-c tr)))

(defmethod shape-name ((tr triangle)) "Triangle")

;; --- Regular Polygon ---

(defclass regular-polygon ()
  ((sides       :initarg :sides       :reader regular-polygon-sides       :type integer)
   (side-length :initarg :side-length :reader regular-polygon-side-length :type number)))

(defun make-regular-polygon (sides side-length)
  (when (< sides 3)
    (error "polygon needs >= 3 sides: got ~A" sides))
  (when (minusp side-length)
    (error "negative side: ~A" side-length))
  (make-instance 'regular-polygon :sides sides :side-length side-length))

(defmethod area ((p regular-polygon))
  (let ((n (regular-polygon-sides p))
        (s (regular-polygon-side-length p)))
    (/ (* n s s) (* 4 (tan (/ pi n))))))

(defmethod perimeter ((p regular-polygon))
  (* (regular-polygon-sides p) (regular-polygon-side-length p)))

(defmethod shape-name ((p regular-polygon))
  (format nil "Regular-~D-gon" (regular-polygon-sides p)))

;; --- heterogeneous collection operations ---

(defun total-area (shapes)
  (reduce #'+ shapes :key #'area :initial-value 0))

(defun total-perimeter (shapes)
  (reduce #'+ shapes :key #'perimeter :initial-value 0))

(defun largest-by-area (shapes)
  (unless shapes (error "empty shape list"))
  (reduce (lambda (a b) (if (> (area a) (area b)) a b)) shapes))

(defun sort-by-perimeter (shapes)
  (sort (copy-list shapes) #'< :key #'perimeter))
