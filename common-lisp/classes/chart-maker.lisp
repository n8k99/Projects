;;;; Chart Making — a visualization pipeline as composed functions.
;;;;
;;;; data → scale → encode → layout → render. Scales are callable; specs are
;;;; declarative; rendering is a pluggable ASCII backend.

(defpackage #:chart-maker
  (:use #:cl)
  (:export #:make-data-point
           #:data-point-label #:data-point-value #:data-point-series
           #:make-linear-scale
           #:linear-scale-domain-min #:linear-scale-domain-max
           #:linear-scale-range-min #:linear-scale-range-max
           #:scale-at
           #:fit-linear-scale
           #:make-bar-chart
           #:bar-chart-title #:bar-chart-points #:bar-chart-width
           #:render-bar-chart
           #:make-line-chart
           #:line-chart-title #:line-chart-points
           #:line-chart-width #:line-chart-height
           #:render-line-chart))

(in-package #:chart-maker)

(defstruct data-point
  (label "" :type string)
  (value 0.0 :type number)
  (series "" :type string))

(defstruct linear-scale
  (domain-min 0.0 :type number)
  (domain-max 1.0 :type number)
  (range-min 0.0 :type number)
  (range-max 1.0 :type number))

(defun scale-at (scale x)
  "Evaluate the scale at X. Degenerate domain maps to the range midpoint."
  (if (= (linear-scale-domain-max scale) (linear-scale-domain-min scale))
      (/ (+ (linear-scale-range-min scale) (linear-scale-range-max scale)) 2.0)
      (let ((t0 (/ (- x (linear-scale-domain-min scale))
                   (- (linear-scale-domain-max scale) (linear-scale-domain-min scale)))))
        (+ (linear-scale-range-min scale)
           (* t0 (- (linear-scale-range-max scale) (linear-scale-range-min scale)))))))

(defun fit-linear-scale (values range-min range-max &key (include-zero t))
  (cond
    ((null values)
     (make-linear-scale :domain-min 0.0 :domain-max 1.0
                        :range-min range-min :range-max range-max))
    (t
     (let ((lo (reduce #'min values))
           (hi (reduce #'max values)))
       (when include-zero
         (setf lo (min lo 0))
         (setf hi (max hi 0)))
       (when (= lo hi)
         (decf lo)
         (incf hi))
       (make-linear-scale :domain-min lo :domain-max hi
                          :range-min range-min :range-max range-max)))))

(defstruct bar-chart
  (title "" :type string)
  (points nil :type list)
  (width 40 :type integer))

(defun repeat-char (ch n)
  (make-string (max 0 n) :initial-element ch))

(defun %max-label-width (points)
  (reduce #'max points :key (lambda (p) (length (data-point-label p))) :initial-value 0))

(defun render-bar-chart (chart)
  (cond
    ((null (bar-chart-points chart))
     (format nil "~A~%(no data)" (bar-chart-title chart)))
    (t
     (let* ((pts (bar-chart-points chart))
            (w (bar-chart-width chart))
            (values (mapcar #'data-point-value pts))
            (scale (fit-linear-scale values 0 w :include-zero t))
            (label-w (%max-label-width pts))
            (sep-len (+ label-w 4 w 10))
            (lines (list (bar-chart-title chart) (repeat-char #\= sep-len))))
       (dolist (p pts)
         (let* ((bar-len (max 0 (round (scale-at scale (data-point-value p)))))
                (bar (coerce (loop repeat bar-len collect #\█) 'string))
                (label-padded (format nil "~VA" label-w (data-point-label p)))
                (bar-padded (format nil "~VA" w bar)))
           (push (format nil "  ~A  ~A  ~,2F"
                         label-padded bar-padded (data-point-value p))
                 lines)))
       (format nil "~{~A~^~%~}" (nreverse lines))))))

(defstruct line-chart
  (title "" :type string)
  (points nil :type list)
  (width 40 :type integer)
  (height 12 :type integer))

(defun %make-grid (rows cols)
  (let ((g (make-array (list rows cols))))
    (dotimes (r rows)
      (dotimes (c cols)
        (setf (aref g r c) #\Space)))
    g))

(defun %plot (grid col row ch rows cols)
  (when (and (>= col 0) (< col cols) (>= row 0) (< row rows))
    (setf (aref grid row col) ch)))

(defun render-line-chart (chart)
  (cond
    ((null (line-chart-points chart))
     (format nil "~A~%(no data)" (line-chart-title chart)))
    (t
     (let* ((pts (line-chart-points chart))
            (n (length pts))
            (w (line-chart-width chart))
            (h (line-chart-height chart))
            (values (mapcar #'data-point-value pts))
            (x-scale (make-linear-scale :domain-min 0
                                         :domain-max (max 1 (1- n))
                                         :range-min 0
                                         :range-max (1- w)))
            (y-scale (fit-linear-scale values 0 (1- h) :include-zero nil))
            (grid (%make-grid h w))
            (prev-col nil)
            (prev-row nil))
       (loop for p in pts
             for i from 0
             do
                (let* ((col (round (scale-at x-scale i)))
                       (row (- (1- h) (round (scale-at y-scale (data-point-value p))))))
                  (%plot grid col row #\● h w)
                  (when prev-col
                    (let* ((dx (- col prev-col))
                           (dy (- row prev-row))
                           (steps (max (abs dx) (abs dy))))
                      (when (> steps 1)
                        (loop for s from 1 below steps do
                          (let* ((tt (/ s (float steps)))
                                 (ic (round (+ prev-col (* dx tt))))
                                 (ir (round (+ prev-row (* dy tt)))))
                            (when (and (>= ir 0) (< ir h) (>= ic 0) (< ic w)
                                       (char= (aref grid ir ic) #\Space))
                              (%plot grid ic ir #\· h w)))))))
                  (setf prev-col col prev-row row)))
       (with-output-to-string (s)
         (format s "~A~%" (line-chart-title chart))
         (loop for r from 0 below h do
           (let ((left (cond
                         ((= r 0)
                          (format nil "~7,2F ┤" (linear-scale-domain-max y-scale)))
                         ((= r (1- h))
                          (format nil "~7,2F ┤" (linear-scale-domain-min y-scale)))
                         (t "        │"))))
             (format s "~A" left)
             (loop for c from 0 below w do (format s "~C" (aref grid r c)))
             (format s "~%")))
         (format s "        └~A" (repeat-char #\─ w)))))))
