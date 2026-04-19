;;;; Matrix Class — a 2D value class whose shape constrains its operations.

(defpackage #:matrix-class
  (:use #:cl)
  (:export #:make-matrix
           #:matrix-rows #:matrix-ncols #:matrix-nrows #:matrix-shape
           #:matrix-square-p
           #:matrix-zeros
           #:matrix-identity
           #:matrix-get #:matrix-set
           #:matrix-add
           #:matrix-sub
           #:matrix-scalar-mul
           #:matrix-matmul
           #:matrix-transpose
           #:matrix-determinant
           #:matrix-equal))

(in-package #:matrix-class)

(defstruct matrix
  (rows nil :type list))

(defun matrix-nrows (m) (length (matrix-rows m)))

(defun matrix-ncols (m)
  (if (matrix-rows m) (length (car (matrix-rows m))) 0))

(defun matrix-shape (m) (values (matrix-nrows m) (matrix-ncols m)))

(defun matrix-square-p (m) (= (matrix-nrows m) (matrix-ncols m)))

(defun %check-rectangular (rows)
  (when (null rows) (error "matrix must have at least one row"))
  (let ((cols (length (car rows))))
    (when (zerop cols) (error "matrix must have at least one column"))
    (dolist (r rows)
      (unless (= (length r) cols)
        (error "row length mismatch: ~A vs ~A" (length r) cols)))))

(defun build-matrix (rows)
  (%check-rectangular rows)
  ;; defensive copy so downstream mutation doesn't leak in
  (make-matrix :rows (mapcar #'copy-list rows)))

(defun matrix-zeros (nrows ncols)
  (when (or (<= nrows 0) (<= ncols 0))
    (error "non-positive dimension: ~Ax~A" nrows ncols))
  (make-matrix :rows (loop repeat nrows
                           collect (make-list ncols :initial-element 0))))

(defun matrix-identity (n)
  (let ((m (matrix-zeros n n)))
    (dotimes (i n)
      (setf (nth i (nth i (matrix-rows m))) 1))
    m))

(defun matrix-get (m i j) (nth j (nth i (matrix-rows m))))

(defun matrix-set (m i j v)
  (setf (nth j (nth i (matrix-rows m))) v))

(defun matrix-equal (a b &key (eps 1d-12))
  (and (= (matrix-nrows a) (matrix-nrows b))
       (= (matrix-ncols a) (matrix-ncols b))
       (loop for ra in (matrix-rows a)
             for rb in (matrix-rows b)
             always (every (lambda (x y) (<= (abs (- x y)) eps)) ra rb))))

(defun %elementwise (a b op)
  (unless (and (= (matrix-nrows a) (matrix-nrows b))
               (= (matrix-ncols a) (matrix-ncols b)))
    (error "shape mismatch: ~Ax~A vs ~Ax~A"
           (matrix-nrows a) (matrix-ncols a)
           (matrix-nrows b) (matrix-ncols b)))
  (make-matrix
    :rows (loop for ra in (matrix-rows a)
                for rb in (matrix-rows b)
                collect (mapcar op ra rb))))

(defun matrix-add (a b) (%elementwise a b #'+))
(defun matrix-sub (a b) (%elementwise a b #'-))

(defun matrix-scalar-mul (m k)
  (make-matrix
    :rows (mapcar (lambda (row) (mapcar (lambda (v) (* k v)) row))
                  (matrix-rows m))))

(defun matrix-matmul (a b)
  (unless (= (matrix-ncols a) (matrix-nrows b))
    (error "inner dimensions differ: ~Ax~A * ~Ax~A"
           (matrix-nrows a) (matrix-ncols a)
           (matrix-nrows b) (matrix-ncols b)))
  (let* ((m (matrix-nrows a))
         (n (matrix-ncols a))
         (p (matrix-ncols b))
         (out (matrix-zeros m p))
         (a-rows (matrix-rows a))
         (b-rows (matrix-rows b)))
    (dotimes (i m)
      (dotimes (j p)
        (let ((s 0))
          (dotimes (k n)
            (incf s (* (nth k (nth i a-rows))
                       (nth j (nth k b-rows)))))
          (setf (nth j (nth i (matrix-rows out))) s))))
    out))

(defun matrix-transpose (m)
  (let* ((r (matrix-nrows m))
         (c (matrix-ncols m))
         (out (matrix-zeros c r)))
    (dotimes (i r)
      (dotimes (j c)
        (setf (nth i (nth j (matrix-rows out)))
              (nth j (nth i (matrix-rows m))))))
    out))

(defun %det (rows)
  "Cofactor expansion on the first row."
  (let ((n (length rows)))
    (cond
      ((= n 1) (nth 0 (nth 0 rows)))
      ((= n 2)
       (- (* (nth 0 (nth 0 rows)) (nth 1 (nth 1 rows)))
          (* (nth 1 (nth 0 rows)) (nth 0 (nth 1 rows)))))
      (t
       (let ((total 0))
         (dotimes (c n)
           (let* ((minor (loop for i from 1 below n
                               collect (loop for k from 0 below n
                                             when (/= k c)
                                               collect (nth k (nth i rows)))))
                  (sign (if (evenp c) 1 -1)))
             (incf total (* sign (nth c (nth 0 rows)) (%det minor)))))
         total)))))

(defun matrix-determinant (m)
  (unless (matrix-square-p m)
    (error "determinant needs square matrix, got ~Ax~A"
           (matrix-nrows m) (matrix-ncols m)))
  (%det (matrix-rows m)))
