;;;; Turtle Graphics — the Rosetta Stone finale.
;;;;
;;;; Integer-degree heading, integer-scaled sin/cos table (×10000),
;;;; vector-line canvas, state stack for nested procedures.

(defpackage #:turtle
  (:use #:cl)
  (:export #:sin-table #:cos-table
           #:make-t-turtle #:t-turtle-x #:t-turtle-y #:t-turtle-heading
           #:t-turtle-pen-down #:t-turtle-pen-color #:t-turtle-pen-width
           #:make-t-line
           #:cmd-forward #:cmd-backward #:cmd-turn #:cmd-set-heading
           #:cmd-pen-up #:cmd-pen-down #:cmd-set-color #:cmd-set-pen-width
           #:cmd-push-state #:cmd-pop-state #:cmd-goto
           #:make-canvas #:canvas-lines #:canvas-turtle
           #:canvas-execute #:canvas-step))

(in-package #:turtle)

(defparameter *sin-0-90*
  #(0 175 349 523 698 872 1045 1219 1392 1564
    1736 1908 2079 2250 2419 2588 2756 2924 3090 3256
    3420 3584 3746 3907 4067 4226 4384 4540 4695 4848
    5000 5150 5299 5446 5592 5736 5878 6018 6157 6293
    6428 6561 6691 6820 6947 7071 7193 7314 7431 7547
    7660 7771 7880 7986 8090 8192 8290 8387 8480 8572
    8660 8746 8829 8910 8988 9063 9135 9205 9272 9336
    9397 9455 9511 9563 9613 9659 9703 9744 9781 9816
    9848 9877 9903 9925 9945 9962 9976 9986 9994 9998
    10000))

(defun sin-table (angle)
  (let ((a (mod angle 360)))
    (cond
      ((<= a 90) (aref *sin-0-90* a))
      ((<= a 180) (aref *sin-0-90* (- 180 a)))
      ((<= a 270) (- (aref *sin-0-90* (- a 180))))
      (t (- (aref *sin-0-90* (- 360 a)))))))

(defun cos-table (angle)
  (sin-table (mod (+ angle 90) 360)))

(defstruct t-turtle
  (x 0 :type integer)
  (y 0 :type integer)
  (heading 0 :type integer)
  (pen-down t :type boolean)
  (pen-color nil :type (or null grayscale::rgb))
  (pen-width 1 :type integer))

(defun make-initial-turtle ()
  (make-t-turtle :pen-color (grayscale::make-rgb :r 0 :g 0 :b 0)))

(defstruct t-line
  (x1 0 :type integer)
  (y1 0 :type integer)
  (x2 0 :type integer)
  (y2 0 :type integer)
  (color nil :type (or null grayscale::rgb))
  (width 1 :type integer))

;; Commands as plists for flexibility.
(defun cmd-forward (n) (list :kind :forward :n n))
(defun cmd-backward (n) (list :kind :backward :n n))
(defun cmd-turn (deg) (list :kind :turn :deg deg))
(defun cmd-set-heading (h) (list :kind :set-heading :heading h))
(defun cmd-pen-up () (list :kind :pen-up))
(defun cmd-pen-down () (list :kind :pen-down))
(defun cmd-set-color (c) (list :kind :set-color :color c))
(defun cmd-set-pen-width (w) (list :kind :set-pen-width :width w))
(defun cmd-push-state () (list :kind :push-state))
(defun cmd-pop-state () (list :kind :pop-state))
(defun cmd-goto (x y) (list :kind :goto :x x :y y))

(defstruct canvas
  (lines nil :type list)
  (turtle nil :type (or null t-turtle))
  (state-stack nil :type list))

(defun make-new-canvas ()
  (make-canvas :turtle (make-initial-turtle)))

(defun copy-turtle (tt)
  (make-t-turtle :x (t-turtle-x tt)
                  :y (t-turtle-y tt)
                  :heading (t-turtle-heading tt)
                  :pen-down (t-turtle-pen-down tt)
                  :pen-color (t-turtle-pen-color tt)
                  :pen-width (t-turtle-pen-width tt)))

(defun move-by (canvas distance)
  (let* ((tt (canvas-turtle canvas))
         (heading (t-turtle-heading tt))
         (cos-v (cos-table heading))
         (sin-v (sin-table heading))
         (new-x (+ (t-turtle-x tt) (floor (* distance cos-v) 10000)))
         (new-y (+ (t-turtle-y tt) (floor (* distance sin-v) 10000))))
    (when (t-turtle-pen-down tt)
      (setf (canvas-lines canvas)
            (append (canvas-lines canvas)
                    (list (make-t-line
                           :x1 (t-turtle-x tt) :y1 (t-turtle-y tt)
                           :x2 new-x :y2 new-y
                           :color (t-turtle-pen-color tt)
                           :width (t-turtle-pen-width tt))))))
    (setf (t-turtle-x tt) new-x)
    (setf (t-turtle-y tt) new-y)))

(defun canvas-step (canvas cmd)
  (let ((tt (canvas-turtle canvas)))
    (case (getf cmd :kind)
      (:forward (move-by canvas (getf cmd :n)))
      (:backward (move-by canvas (- (getf cmd :n))))
      (:turn (setf (t-turtle-heading tt)
                    (mod (+ (t-turtle-heading tt) (getf cmd :deg)) 360)))
      (:set-heading (setf (t-turtle-heading tt) (mod (getf cmd :heading) 360)))
      (:pen-up (setf (t-turtle-pen-down tt) nil))
      (:pen-down (setf (t-turtle-pen-down tt) t))
      (:set-color (setf (t-turtle-pen-color tt) (getf cmd :color)))
      (:set-pen-width (setf (t-turtle-pen-width tt) (getf cmd :width)))
      (:push-state
       (setf (canvas-state-stack canvas)
             (cons (copy-turtle tt) (canvas-state-stack canvas))))
      (:pop-state
       (when (canvas-state-stack canvas)
         (setf (canvas-turtle canvas) (car (canvas-state-stack canvas)))
         (setf (canvas-state-stack canvas) (cdr (canvas-state-stack canvas)))))
      (:goto
       (let ((old-x (t-turtle-x tt))
             (old-y (t-turtle-y tt))
             (new-x (getf cmd :x))
             (new-y (getf cmd :y)))
         (setf (t-turtle-x tt) new-x)
         (setf (t-turtle-y tt) new-y)
         (when (t-turtle-pen-down tt)
           (setf (canvas-lines canvas)
                 (append (canvas-lines canvas)
                         (list (make-t-line
                                :x1 old-x :y1 old-y
                                :x2 new-x :y2 new-y
                                :color (t-turtle-pen-color tt)
                                :width (t-turtle-pen-width tt)))))))))))

(defun canvas-execute (canvas program)
  (dolist (cmd program) (canvas-step canvas cmd)))

(defun demo ()
  (format t "sin(0)=~D cos(0)=~D~%" (sin-table 0) (cos-table 0))
  (format t "sin(90)=~D cos(90)=~D~%" (sin-table 90) (cos-table 90))
  (format t "sin(180)=~D cos(180)=~D~%" (sin-table 180) (cos-table 180))
  (format t "sin(270)=~D cos(270)=~D~%" (sin-table 270) (cos-table 270))
  (format t "sin(45)=~D cos(45)=~D~%" (sin-table 45) (cos-table 45))

  (let ((c (make-new-canvas))
        (program (list (cmd-forward 100) (cmd-turn 90)
                        (cmd-forward 100) (cmd-turn 90)
                        (cmd-forward 100) (cmd-turn 90)
                        (cmd-forward 100) (cmd-turn 90))))
    (canvas-execute c program)
    (format t "square: turtle=(~D, ~D) heading=~D lines=~D~%"
            (t-turtle-x (canvas-turtle c))
            (t-turtle-y (canvas-turtle c))
            (t-turtle-heading (canvas-turtle c))
            (length (canvas-lines c))))

  (let ((c (make-new-canvas)))
    (canvas-execute c (list (cmd-forward 50)
                              (cmd-push-state)
                              (cmd-turn 90) (cmd-forward 30)
                              (cmd-pop-state)))
    (format t "after push/pop: turtle=(~D, ~D) heading=~D~%"
            (t-turtle-x (canvas-turtle c))
            (t-turtle-y (canvas-turtle c))
            (t-turtle-heading (canvas-turtle c)))
    (format t "lines drawn: ~D~%" (length (canvas-lines c))))

  (let ((c (make-new-canvas)))
    (canvas-execute c (cons (cmd-turn 45)
                              (loop repeat 4 append (list (cmd-forward 100) (cmd-turn 90)))))
    (format t "45-degree square: turtle=(~D, ~D)~%"
            (t-turtle-x (canvas-turtle c))
            (t-turtle-y (canvas-turtle c))))

  (let ((c (make-new-canvas)))
    (loop repeat 5 do
          (canvas-execute c (list (cmd-forward 100) (cmd-turn 144))))
    (format t "five-pointed star: lines=~D turtle=(~D, ~D)~%"
            (length (canvas-lines c))
            (t-turtle-x (canvas-turtle c))
            (t-turtle-y (canvas-turtle c)))))
