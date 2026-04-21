;;;; Signature Maker — stroke model + Catmull-Rom + variable-width + normalize.

(defpackage #:signature-maker
  (:use #:cl)
  (:export #:make-point #:point-x #:point-y #:point-pressure #:point-t-ms
           #:make-stroke #:stroke-points
           #:make-signature #:signature-add-stroke #:signature-strokes
           #:signature-point-count #:signature-bounding-box
           #:catmull-rom #:smooth-stroke #:normalize #:similarity-distance))

(in-package #:signature-maker)

(defstruct point
  (x 0 :type integer)
  (y 0 :type integer)
  (pressure 0 :type integer)
  (t-ms 0 :type integer))

(defstruct stroke
  (points nil :type list))

(defstruct signature
  (strokes nil :type list))

(defun signature-add-stroke (sig s)
  (setf (signature-strokes sig) (append (signature-strokes sig) (list s))))

(defun signature-point-count (sig)
  (reduce #'+ (signature-strokes sig) :key (lambda (s) (length (stroke-points s)))))

(defun signature-bounding-box (sig)
  (let ((min-x nil) (min-y nil) (max-x nil) (max-y nil))
    (dolist (stroke (signature-strokes sig))
      (dolist (p (stroke-points stroke))
        (when (or (null min-x) (< (point-x p) min-x)) (setf min-x (point-x p)))
        (when (or (null min-y) (< (point-y p) min-y)) (setf min-y (point-y p)))
        (when (or (null max-x) (> (point-x p) max-x)) (setf max-x (point-x p)))
        (when (or (null max-y) (> (point-y p) max-y)) (setf max-y (point-y p)))))
    (when min-x (list min-x min-y max-x max-y))))

(defun cr-compute (a b c d tt t2 t3)
  (let* ((term-a (* 2 b))
         (term-b (* (+ (- a) c) tt))
         (term-c (* (+ (- (* 2 a) (* 5 b)) (* 4 c) (- d)) t2))
         (term-d (* (+ (- a) (* 3 b) (- (* 3 c)) d) t3))
         (sum (+ (* term-a 1000000)
                  (* term-b 1000)
                  term-c
                  (floor term-d 1000))))
    (floor sum 2000000)))

(defun catmull-rom (p0 p1 p2 p3 tt)
  "p0 p1 p2 p3 are (x . y) conses. tt in 0..1000. Returns (x . y)."
  (let* ((t2 (* tt tt))
         (t3 (floor (* t2 tt) 1000))
         (x (cr-compute (car p0) (car p1) (car p2) (car p3) tt t2 t3))
         (y (cr-compute (cdr p0) (cdr p1) (cdr p2) (cdr p3) tt t2 t3)))
    (cons x y)))

(defun smooth-stroke (stroke steps)
  (let ((pts (stroke-points stroke)))
    (cond
      ((or (< (length pts) 4) (zerop steps))
       (make-stroke :points (copy-list pts)))
      (t
       (let ((out (list (first pts))))
         (loop for i from 0 below (- (length pts) 3) do
               (let ((p0 (cons (point-x (nth i pts)) (point-y (nth i pts))))
                     (p1 (cons (point-x (nth (+ i 1) pts))
                                (point-y (nth (+ i 1) pts))))
                     (p2 (cons (point-x (nth (+ i 2) pts))
                                (point-y (nth (+ i 2) pts))))
                     (p3 (cons (point-x (nth (+ i 3) pts))
                                (point-y (nth (+ i 3) pts)))))
                 (loop for s from 1 to steps do
                       (let* ((tt (floor (* s 1000) steps))
                              (interp (catmull-rom p0 p1 p2 p3 tt))
                              (p-prev (nth (+ i 1) pts))
                              (p-next (nth (+ i 2) pts))
                              (pressure (max 0 (min 1024
                                                     (+ (point-pressure p-prev)
                                                         (floor (* (- (point-pressure p-next)
                                                                        (point-pressure p-prev))
                                                                    tt)
                                                                 1000)))))
                              (t-ms (max 0 (+ (point-t-ms p-prev)
                                               (floor (* (- (point-t-ms p-next)
                                                              (point-t-ms p-prev))
                                                          tt)
                                                       1000)))))
                         (push (make-point :x (car interp) :y (cdr interp)
                                             :pressure pressure :t-ms t-ms)
                                out)))))
         (push (car (last pts)) out)
         (make-stroke :points (nreverse out)))))))

(defun normalize (sig target-w target-h)
  (let ((bbox (signature-bounding-box sig)))
    (cond
      ((null bbox)
       (make-signature
        :strokes (mapcar (lambda (s)
                             (make-stroke :points (copy-list (stroke-points s))))
                           (signature-strokes sig))))
      (t
       (destructuring-bind (min-x min-y max-x max-y) bbox
         (let* ((w (max 1 (- max-x min-x)))
                (h (max 1 (- max-y min-y)))
                (scale-x (floor (* target-w 1000) w))
                (scale-y (floor (* target-h 1000) h)))
           (make-signature
            :strokes
            (mapcar (lambda (s)
                       (make-stroke
                        :points
                        (mapcar (lambda (p)
                                   (make-point
                                    :x (floor (* (- (point-x p) min-x) scale-x) 1000)
                                    :y (floor (* (- (point-y p) min-y) scale-y) 1000)
                                    :pressure (point-pressure p)
                                    :t-ms (point-t-ms p)))
                                 (stroke-points s))))
                     (signature-strokes sig)))))))))

(defun similarity-distance (a b)
  (let ((total 0))
    (dolist (stroke-a (signature-strokes a))
      (dolist (p (stroke-points stroke-a))
        (let ((min-d2 most-positive-fixnum))
          (dolist (stroke-b (signature-strokes b))
            (dolist (q (stroke-points stroke-b))
              (let* ((dx (- (point-x p) (point-x q)))
                     (dy (- (point-y p) (point-y q)))
                     (d2 (+ (* dx dx) (* dy dy))))
                (when (< d2 min-d2) (setf min-d2 d2)))))
          (incf total min-d2))))
    total))

(defun demo ()
  (let ((sig (make-signature)))
    (signature-add-stroke sig
      (make-stroke :points (list (make-point :x 10 :y 20 :pressure 512 :t-ms 0)
                                   (make-point :x 30 :y 40 :pressure 768 :t-ms 100)
                                   (make-point :x 50 :y 30 :pressure 1024 :t-ms 200)
                                   (make-point :x 70 :y 50 :pressure 512 :t-ms 300))))
    (signature-add-stroke sig
      (make-stroke :points (list (make-point :x 80 :y 30 :pressure 600 :t-ms 400)
                                   (make-point :x 100 :y 50 :pressure 700 :t-ms 500))))

    (format t "point count: ~D~%" (signature-point-count sig))
    (format t "bbox: ~A~%" (signature-bounding-box sig))

    (let ((smoothed (smooth-stroke (first (signature-strokes sig)) 5)))
      (format t "smoothed points: ~D~%" (length (stroke-points smoothed))))

    (let ((norm (normalize sig 1000 1000)))
      (format t "normalized bbox: ~A~%" (signature-bounding-box norm)))

    (let ((r0 (catmull-rom '(0 . 0) '(10 . 10) '(20 . 10) '(30 . 0) 0))
          (r1 (catmull-rom '(0 . 0) '(10 . 10) '(20 . 10) '(30 . 0) 1000)))
      (format t "cr t=0: (~D, ~D)~%" (car r0) (cdr r0))
      (format t "cr t=1000: (~D, ~D)~%" (car r1) (cdr r1)))

    (format t "self-similarity: ~D~%" (similarity-distance sig sig))

    (let ((small (make-signature))
          (big (make-signature)))
      (signature-add-stroke small
        (make-stroke :points (list (make-point :x 0 :y 0 :pressure 500 :t-ms 0)
                                     (make-point :x 10 :y 0 :pressure 500 :t-ms 100)
                                     (make-point :x 10 :y 10 :pressure 500 :t-ms 200)
                                     (make-point :x 0 :y 10 :pressure 500 :t-ms 300))))
      (signature-add-stroke big
        (make-stroke :points (list (make-point :x 0 :y 0 :pressure 500 :t-ms 0)
                                     (make-point :x 100 :y 0 :pressure 500 :t-ms 100)
                                     (make-point :x 100 :y 100 :pressure 500 :t-ms 200)
                                     (make-point :x 0 :y 100 :pressure 500 :t-ms 300))))
      (let ((n-s (normalize small 1000 1000))
            (n-b (normalize big 1000 1000)))
        (format t "normalized same-shape similarity: ~D~%"
                (similarity-distance n-s n-b))))))
