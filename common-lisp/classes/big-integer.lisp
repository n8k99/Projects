;;;; Handle Large Numbers — an arbitrary-precision signed integer as a class.
;;;;
;;;; Note: Common Lisp has native arbitrary-precision integers (bignums) built
;;;; into the language. We implement our own BigInt here for the Rosetta Stone
;;;; exercise, demonstrating the schoolbook algorithms that CL's runtime provides
;;;; for free. Digits are base-10, little-endian; canonical form has no trailing
;;;; zeros; zero is represented with an empty digit list.

(defpackage #:big-integer
  (:use #:cl)
  (:shadow #:zerop)
  (:export #:make-big
           #:big-negative
           #:big-digits
           #:big-from-int
           #:big-from-string
           #:big-to-string
           #:big-zerop
           #:big-neg
           #:big-abs
           #:big-cmp
           #:big-eq
           #:big-add
           #:big-sub
           #:big-mul))

(in-package #:big-integer)

(defstruct big
  (negative nil :type boolean)
  (digits nil :type list))        ; little-endian base-10; NIL = zero

(defun big-zerop (x) (null (big-digits x)))

(defun big-from-int (n)
  (cond
    ((zerop n) (make-big :negative nil :digits nil))
    (t (let ((neg (minusp n))
             (x (abs n))
             (digs nil))
         (loop while (plusp x) do
           (push (mod x 10) digs)
           (setf x (floor x 10)))
         (make-big :negative neg :digits (nreverse digs))))))

(defun big-from-string (s)
  (let* ((s (string-trim '(#\Space #\Tab #\Newline) s)))
    (when (zerop (length s))
      (error "empty string"))
    (let* ((neg (char= (char s 0) #\-))
           (body (if (or neg (char= (char s 0) #\+)) (subseq s 1) s)))
      (when (zerop (length body))
        (error "not a decimal integer: ~S" s))
      (loop for c across body
            unless (digit-char-p c)
              do (error "not a decimal integer: ~S" s))
      (let ((digs (loop for i from (1- (length body)) downto 0
                        collect (digit-char-p (char body i)))))
        ;; strip leading zeros (at high end — tail of the list)
        (setf digs (nreverse (loop with saw-nonzero = nil
                                   for d in (reverse digs)
                                   when (or (not (zerop d)) saw-nonzero)
                                     do (setf saw-nonzero t)
                                     and collect d)))
        (if (null digs)
            (make-big :negative nil :digits nil)
            (make-big :negative neg :digits digs))))))

(defun big-to-string (x)
  (if (big-zerop x)
      "0"
      (let* ((sign (if (big-negative x) "-" ""))
             (body (coerce (loop for d in (reverse (big-digits x))
                                 collect (digit-char d))
                           'string)))
        (concatenate 'string sign body))))

(defun big-neg (x)
  (if (big-zerop x) x
      (make-big :negative (not (big-negative x))
                :digits (copy-list (big-digits x)))))

(defun big-abs (x) (make-big :negative nil :digits (copy-list (big-digits x))))

(defun %cmp-magnitude (a b)
  "Returns -1 / 0 / 1 comparing |a| and |b| via their digit lists."
  (let ((la (length a)) (lb (length b)))
    (cond ((< la lb) -1)
          ((> la lb) 1)
          (t
           ;; Compare from most significant digit (end of little-endian list).
           (loop for x in (reverse a)
                 for y in (reverse b)
                 do (cond ((< x y) (return -1))
                          ((> x y) (return 1)))
                 finally (return 0))))))

(defun big-cmp (x y)
  "Total order: -big < -small < 0 < +small < +big. Returns -1 / 0 / 1."
  (cond
    ((and (big-zerop x) (big-zerop y)) 0)
    ((and (big-negative x) (not (big-negative y))) -1)
    ((and (not (big-negative x)) (big-negative y)) 1)
    ((not (big-negative x))
     (%cmp-magnitude (big-digits x) (big-digits y)))
    (t
     (%cmp-magnitude (big-digits y) (big-digits x)))))

(defun big-eq (x y) (= (big-cmp x y) 0))

(defun %add-digits (a b)
  "Schoolbook add with carry. a,b are little-endian base-10."
  (cond
    ((null a) (copy-list b))
    ((null b) (copy-list a))
    (t
     (let ((out nil) (carry 0) (la (length a)) (lb (length b)))
       (dotimes (i (max la lb))
         (let* ((da (if (< i la) (nth i a) 0))
                (db (if (< i lb) (nth i b) 0))
                (s  (+ da db carry)))
           (push (mod s 10) out)
           (setf carry (floor s 10))))
       (when (plusp carry)
         (push carry out))
       (nreverse out)))))

(defun %sub-digits (a b)
  "Schoolbook subtract assuming |a| >= |b|. Returns canonical (stripped) list."
  (let ((out nil) (borrow 0) (la (length a)))
    (dotimes (i la)
      (let* ((da (nth i a))
             (db (if (< i (length b)) (nth i b) 0))
             (d  (- da db borrow)))
        (if (minusp d)
            (progn (setf d (+ d 10) borrow 1))
            (setf borrow 0))
        (push d out)))
    (unless (zerop borrow)
      (error "sub-digits called with |a| < |b|"))
    (setf out (nreverse out))
    ;; Strip trailing zeros (high end = tail of little-endian list).
    (loop while (and (> (length out) 1)
                     (zerop (car (last out))))
          do (setf out (butlast out)))
    (if (equal out '(0)) nil out)))

(defun %mul-digits (a b)
  "Schoolbook multiply. Partial products shifted and accumulated."
  (when (or (null a) (null b))
    (return-from %mul-digits nil))
  (let* ((la (length a))
         (lb (length b))
         (acc (make-array (+ la lb) :initial-element 0)))
    (dotimes (i la)
      (let ((carry 0)
            (da (nth i a)))
        (dotimes (j lb)
          (let* ((s (+ (aref acc (+ i j)) (* da (nth j b)) carry)))
            (setf (aref acc (+ i j)) (mod s 10))
            (setf carry (floor s 10))))
        (incf (aref acc (+ i lb)) carry)))
    (let ((out (coerce acc 'list)))
      (loop while (and (> (length out) 1)
                       (zerop (car (last out))))
            do (setf out (butlast out)))
      (if (equal out '(0)) nil out))))

(defun big-add (x y)
  (cond
    ((eq (big-negative x) (big-negative y))
     (if (and (big-zerop x) (big-zerop y))
         (make-big :negative nil :digits nil)
         (make-big :negative (big-negative x)
                   :digits (%add-digits (big-digits x) (big-digits y)))))
    (t
     (let ((c (%cmp-magnitude (big-digits x) (big-digits y))))
       (cond
         ((zerop c) (make-big :negative nil :digits nil))
         ((plusp c) (make-big :negative (big-negative x)
                              :digits (%sub-digits (big-digits x) (big-digits y))))
         (t (make-big :negative (big-negative y)
                      :digits (%sub-digits (big-digits y) (big-digits x)))))))))

(defun big-sub (x y) (big-add x (big-neg y)))

(defun big-mul (x y)
  (if (or (big-zerop x) (big-zerop y))
      (make-big :negative nil :digits nil)
      (make-big :negative (not (eq (big-negative x) (big-negative y)))
                :digits (%mul-digits (big-digits x) (big-digits y)))))
