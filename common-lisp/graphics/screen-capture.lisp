;;;; Screen Capture Program — region + delay FSM + annotations + ring history.

(defpackage #:screen-capture
  (:use #:cl)
  (:export #:make-region-full #:make-region-rect #:make-region-window
           #:make-annotation-rect #:make-annotation-highlight
           #:make-annotation-text #:make-annotation-arrow
           #:make-capture-spec #:make-recorder
           #:recorder-arm #:recorder-tick #:recorder-capture #:recorder-reset
           #:recorder-state #:recorder-countdown-remaining-ms
           #:recorder-history #:recorder-events #:recorder-elapsed-ms
           #:apply-annotations
           #:capture-id #:capture-image))

(in-package #:screen-capture)

;; Regions and annotations as plists for flexibility.
(defun make-region-full () (list :kind :full-screen))
(defun make-region-rect (x y w h) (list :kind :rect :x x :y y :width w :height h))
(defun make-region-window (id) (list :kind :window :window-id id))

(defun make-annotation-rect (x y w h color stroke)
  (list :kind :rectangle :x x :y y :width w :height h
         :color color :stroke stroke))
(defun make-annotation-highlight (x y w h color opacity)
  (list :kind :highlight :x x :y y :width w :height h
         :color color :opacity opacity))
(defun make-annotation-text (x y text color)
  (list :kind :text :x x :y y :text text :color color))
(defun make-annotation-arrow (x1 y1 x2 y2 color)
  (list :kind :arrow :x1 x1 :y1 y1 :x2 x2 :y2 y2 :color color))

(defstruct capture-spec
  (region nil :type list)
  (delay-ms 0 :type integer)
  (include-cursor nil :type boolean)
  (annotations nil :type list))

(defstruct capture
  (id 0 :type integer)
  (spec nil :type (or null capture-spec))
  (image nil :type (or null grayscale::image))
  (captured-at-ms 0 :type integer))

(defstruct recorder-event
  (kind :armed :type keyword)
  (id 0 :type integer)
  (detail "" :type string))

(defstruct recorder
  (state :idle :type keyword)
  (pending-spec nil :type (or null capture-spec))
  (pending-id 0 :type integer)
  (countdown-remaining-ms 0 :type integer)
  (history nil :type list)
  (history-max 5 :type integer)
  (elapsed-ms 0 :type integer)
  (events nil :type list)
  (next-id 1 :type integer))

(defun recorder-arm (rec spec)
  (let ((id (recorder-next-id rec)))
    (incf (recorder-next-id rec))
    (setf (recorder-pending-id rec) id)
    (setf (recorder-countdown-remaining-ms rec) (capture-spec-delay-ms spec))
    (setf (recorder-pending-spec rec) spec)
    (setf (recorder-events rec)
          (append (recorder-events rec)
                  (list (make-recorder-event
                         :kind :armed :id id
                         :detail (format nil "delay=~Dms"
                                          (capture-spec-delay-ms spec))))))
    (setf (recorder-state rec)
          (if (zerop (capture-spec-delay-ms spec)) :capturing :counting))
    id))

(defun recorder-tick (rec ms)
  (incf (recorder-elapsed-ms rec) ms)
  (when (eq (recorder-state rec) :counting)
    (if (> (recorder-countdown-remaining-ms rec) ms)
        (decf (recorder-countdown-remaining-ms rec) ms)
        (progn
          (setf (recorder-countdown-remaining-ms rec) 0)
          (setf (recorder-state rec) :capturing)))
    (setf (recorder-events rec)
          (append (recorder-events rec)
                  (list (make-recorder-event
                         :kind :countdown-tick
                         :id (recorder-pending-id rec)
                         :detail (format nil "remaining=~Dms"
                                          (recorder-countdown-remaining-ms rec))))))))

(defun crop-image (img x0 y0 w h)
  (let ((pixels nil))
    (loop for y from 0 below h do
          (loop for x from 0 below w do
                (push (grayscale::image-get img (+ x0 x) (+ y0 y)) pixels)))
    (grayscale::make-image :width w :height h :pixels (nreverse pixels))))

(defun clamp-u8 (v) (max 0 (min 255 v)))

(defun blend-channel (a b alpha)
  (floor (+ (* a (- 255 alpha)) (* b alpha)) 255))

(defun image-set (img x y color)
  (when (and (>= x 0) (< x (grayscale::image-width img))
             (>= y 0) (< y (grayscale::image-height img)))
    (setf (nth (+ (* y (grayscale::image-width img)) x)
                (grayscale::image-pixels img))
          color)))

(defun image-get (img x y)
  (when (and (>= x 0) (< x (grayscale::image-width img))
             (>= y 0) (< y (grayscale::image-height img)))
    (nth (+ (* y (grayscale::image-width img)) x)
          (grayscale::image-pixels img))))

(defun draw-rect-outline (img x y w h color stroke)
  (let ((stroke (max 1 stroke)))
    (loop for s from 0 below stroke do
          (loop for dx from 0 below w do
                (image-set img (+ x dx) (+ y s) color)
                (when (> h s) (image-set img (+ x dx) (+ y h (- -1 s)) color)))
          (loop for dy from 0 below h do
                (image-set img (+ x s) (+ y dy) color)
                (when (> w s) (image-set img (+ x w (- -1 s)) (+ y dy) color))))))

(defun blend-rect (img x y w h color opacity)
  (loop for dy from 0 below h do
        (loop for dx from 0 below w do
              (let ((px (+ x dx)) (py (+ y dy)))
                (let ((cur (image-get img px py)))
                  (when cur
                    (image-set img px py
                                (grayscale::make-rgb
                                 :r (blend-channel (grayscale::rgb-r cur)
                                                    (grayscale::rgb-r color) opacity)
                                 :g (blend-channel (grayscale::rgb-g cur)
                                                    (grayscale::rgb-g color) opacity)
                                 :b (blend-channel (grayscale::rgb-b cur)
                                                    (grayscale::rgb-b color) opacity)))))))))

(defun draw-line (img x1 y1 x2 y2 color)
  (let* ((x x1) (y y1)
         (dx (abs (- x2 x)))
         (dy (- (abs (- y2 y))))
         (sx (if (< x x2) 1 -1))
         (sy (if (< y y2) 1 -1))
         (err (+ dx dy)))
    (loop
       do (when (and (>= x 0) (>= y 0)) (image-set img x y color))
       until (and (= x x2) (= y y2))
       do (let ((e2 (* 2 err)))
            (when (>= e2 dy) (incf err dy) (incf x sx))
            (when (<= e2 dx) (incf err dx) (incf y sy))))))

(defun apply-one-annotation (img ann)
  (case (getf ann :kind)
    (:rectangle
     (draw-rect-outline img (getf ann :x) (getf ann :y)
                         (getf ann :width) (getf ann :height)
                         (getf ann :color) (getf ann :stroke)))
    (:highlight
     (blend-rect img (getf ann :x) (getf ann :y)
                  (getf ann :width) (getf ann :height)
                  (getf ann :color) (getf ann :opacity)))
    (:text
     (image-set img (getf ann :x) (getf ann :y) (getf ann :color)))
    (:arrow
     (draw-line img (getf ann :x1) (getf ann :y1)
                 (getf ann :x2) (getf ann :y2) (getf ann :color)))))

(defun apply-annotations (base anns)
  (let ((img (grayscale::make-image
              :width (grayscale::image-width base)
              :height (grayscale::image-height base)
              :pixels (copy-list (grayscale::image-pixels base)))))
    (dolist (ann anns) (apply-one-annotation img ann))
    img))

(defun recorder-capture (rec screen window-lookup)
  (unless (eq (recorder-state rec) :capturing) (return-from recorder-capture nil))
  (let* ((spec (recorder-pending-spec rec))
         (id (recorder-pending-id rec)))
    (unless spec (return-from recorder-capture nil))
    (setf (recorder-pending-spec rec) nil)
    (let ((base
            (case (getf (capture-spec-region spec) :kind)
              (:full-screen
               (grayscale::make-image
                :width (grayscale::image-width screen)
                :height (grayscale::image-height screen)
                :pixels (copy-list (grayscale::image-pixels screen))))
              (:rect
               (let ((region (capture-spec-region spec)))
                 (let ((x (getf region :x)) (y (getf region :y))
                       (w (getf region :width)) (h (getf region :height)))
                   (if (or (> (+ x w) (grayscale::image-width screen))
                            (> (+ y h) (grayscale::image-height screen)))
                       nil
                       (crop-image screen x y w h)))))
              (:window
               (let ((id2 (getf (capture-spec-region spec) :window-id)))
                 (funcall window-lookup id2))))))
      (unless base
        (setf (recorder-events rec)
              (append (recorder-events rec)
                      (list (make-recorder-event
                             :kind :capture-failed :id id
                             :detail "region invalid"))))
        (setf (recorder-state rec) :idle)
        (return-from recorder-capture nil))
      (let* ((annotated (apply-annotations base (capture-spec-annotations spec)))
             (cap (make-capture :id id :spec spec :image annotated
                                 :captured-at-ms (recorder-elapsed-ms rec))))
        (setf (recorder-history rec) (append (recorder-history rec) (list cap)))
        (when (> (length (recorder-history rec)) (recorder-history-max rec))
          (let ((evicted (first (recorder-history rec))))
            (setf (recorder-history rec) (rest (recorder-history rec)))
            (setf (recorder-events rec)
                  (append (recorder-events rec)
                          (list (make-recorder-event
                                 :kind :evicted :id (capture-id evicted)))))))
        (setf (recorder-events rec)
              (append (recorder-events rec)
                      (list (make-recorder-event
                             :kind :captured :id id
                             :detail (format nil "at=~Dms"
                                              (recorder-elapsed-ms rec))))))
        (setf (recorder-state rec) :captured)
        cap))))

(defun recorder-reset (rec)
  (setf (recorder-state rec) :idle)
  (setf (recorder-pending-spec rec) nil)
  (setf (recorder-pending-id rec) 0)
  (setf (recorder-countdown-remaining-ms rec) 0))

(defun demo ()
  (let ((rec (make-recorder :history-max 3)))
    (recorder-arm rec (make-capture-spec :region (make-region-full) :delay-ms 1000))
    (format t "state: ~A remaining: ~Dms~%" (recorder-state rec)
            (recorder-countdown-remaining-ms rec))
    (recorder-tick rec 400)
    (format t "after 400ms tick: state: ~A remaining: ~Dms~%"
            (recorder-state rec) (recorder-countdown-remaining-ms rec))
    (recorder-tick rec 700)
    (format t "after 700ms tick: state: ~A remaining: ~Dms~%"
            (recorder-state rec) (recorder-countdown-remaining-ms rec))

    (let ((screen (grayscale::make-image
                    :width 10 :height 8
                    :pixels (loop repeat 80 collect
                                   (grayscale::make-rgb :r 100 :g 100 :b 100)))))
      (let ((cap (recorder-capture rec screen (lambda (_) (declare (ignore _)) nil))))
        (format t "captured: id=~D image=~Dx~D~%"
                (capture-id cap)
                (grayscale::image-width (capture-image cap))
                (grayscale::image-height (capture-image cap))))

      (recorder-reset rec)
      (let ((spec (make-capture-spec
                    :region (make-region-rect 2 1 6 5)
                    :delay-ms 0
                    :annotations (list (make-annotation-highlight
                                         0 0 6 5
                                         (grayscale::make-rgb :r 255 :g 255 :b 0) 128)
                                        (make-annotation-rect
                                         1 1 4 3
                                         (grayscale::make-rgb :r 255 :g 0 :b 0) 1)
                                        (make-annotation-arrow
                                         0 0 5 4
                                         (grayscale::make-rgb :r 0 :g 255 :b 0))))))
        (recorder-arm rec spec)
        (let ((cap2 (recorder-capture rec screen (lambda (_) (declare (ignore _)) nil))))
          (format t "annotated: ~Dx~D~%"
                  (grayscale::image-width (capture-image cap2))
                  (grayscale::image-height (capture-image cap2)))
          (let ((p (first (grayscale::image-pixels (capture-image cap2)))))
            (format t "pixels[0]=(~D,~D,~D)~%"
                    (grayscale::rgb-r p) (grayscale::rgb-g p) (grayscale::rgb-b p)))))

      (loop repeat 5 do
            (recorder-reset rec)
            (recorder-arm rec (make-capture-spec :region (make-region-full) :delay-ms 0))
            (recorder-capture rec screen (lambda (_) (declare (ignore _)) nil)))
      (format t "history size: ~D~%" (length (recorder-history rec)))
      (format t "history ids: ~A~%"
              (mapcar #'capture-id (recorder-history rec))))))
