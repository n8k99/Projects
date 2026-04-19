;;;; Online White Board — shared canvas of strokes with spatial queries.

(defpackage #:whiteboard
  (:use #:cl)
  (:export #:make-point #:point-x #:point-y #:distance
           #:make-board
           #:add-stroke #:remove-stroke #:clear-board
           #:stroke-count #:all-strokes
           #:strokes-by-author #:strokes-near #:bounding-box
           #:to-svg
           #:stroke-id #:stroke-author #:stroke-color #:stroke-thickness #:stroke-points))

(in-package #:whiteboard)

(defstruct point (x 0.0 :type real) (y 0.0 :type real))

(defun distance (a b)
  (sqrt (+ (expt (- (point-x a) (point-x b)) 2)
           (expt (- (point-y a) (point-y b)) 2))))

(defstruct stroke
  (id 0 :type integer)
  (author "" :type string)
  (color "" :type string)
  (thickness 1.0 :type real)
  (points nil :type list))

(defstruct board
  (strokes nil :type list)
  (next-id 1 :type integer))

(defun add-stroke (board author color thickness points)
  (let ((id (board-next-id board)))
    (incf (board-next-id board))
    (setf (board-strokes board)
          (append (board-strokes board)
                  (list (make-stroke :id id :author author :color color
                                     :thickness thickness :points points))))
    id))

(defun remove-stroke (board id)
  (let ((pos (position id (board-strokes board) :key #'stroke-id)))
    (when pos
      (setf (board-strokes board)
            (append (subseq (board-strokes board) 0 pos)
                    (subseq (board-strokes board) (1+ pos))))
      t)))

(defun clear-board (board) (setf (board-strokes board) nil))
(defun stroke-count (board) (length (board-strokes board)))
(defun all-strokes (board) (board-strokes board))

(defun strokes-by-author (board author)
  (remove-if-not (lambda (s) (string= (stroke-author s) author))
                 (board-strokes board)))

(defun %stroke-distance-to (s p)
  (if (null (stroke-points s))
      most-positive-double-float
      (reduce #'min (mapcar (lambda (q) (distance q p)) (stroke-points s)))))

(defun strokes-near (board p radius)
  (remove-if-not (lambda (s) (<= (%stroke-distance-to s p) radius))
                 (board-strokes board)))

(defun %stroke-bbox (s)
  (when (stroke-points s)
    (let ((xs (mapcar #'point-x (stroke-points s)))
          (ys (mapcar #'point-y (stroke-points s))))
      (list (make-point :x (reduce #'min xs) :y (reduce #'min ys))
            (make-point :x (reduce #'max xs) :y (reduce #'max ys))))))

(defun bounding-box (board)
  (let ((boxes (remove-if #'null (mapcar #'%stroke-bbox (board-strokes board)))))
    (when boxes
      (list (make-point :x (reduce #'min (mapcar (lambda (b) (point-x (first b))) boxes))
                        :y (reduce #'min (mapcar (lambda (b) (point-y (first b))) boxes)))
            (make-point :x (reduce #'max (mapcar (lambda (b) (point-x (second b))) boxes))
                        :y (reduce #'max (mapcar (lambda (b) (point-y (second b))) boxes)))))))

(defun to-svg (board width height)
  (with-output-to-string (out)
    (format out "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"~A\" height=\"~A\">~%"
            width height)
    (dolist (s (board-strokes board))
      (when (stroke-points s)
        (format out "  <polyline fill=\"none\" stroke=\"~A\" stroke-width=\"~A\" points=\"~{~A~^ ~}\"/>~%"
                (stroke-color s) (stroke-thickness s)
                (mapcar (lambda (p) (format nil "~A,~A" (point-x p) (point-y p)))
                        (stroke-points s)))))
    (format out "</svg>")))
