;;;; Bulk Picture Manipulator — transformation pipeline applied over a batch.
;;;;
;;;; Pipeline is data (list of op plists). Pure fold via apply-pipeline.
;;;; Dry-run predicts dims without touching pixels. Batch isolates errors.

(defpackage #:bulk-picture
  (:use #:cl)
  (:export #:op-resize #:op-grayscale #:op-rotate-90 #:op-rotate-180
           #:op-rotate-270 #:op-flip-h #:op-flip-v #:op-crop
           #:op-brightness #:op-contrast
           #:apply-op #:apply-pipeline #:run-batch #:dry-run-dims))

(in-package #:bulk-picture)

;; Each op is a plist: (:kind :resize :width W :height H ...).

(defun op-resize (w h) (list :kind :resize :width w :height h))
(defun op-grayscale (algo) (list :kind :grayscale :algo algo))
(defun op-rotate-90 () (list :kind :rotate-90))
(defun op-rotate-180 () (list :kind :rotate-180))
(defun op-rotate-270 () (list :kind :rotate-270))
(defun op-flip-h () (list :kind :flip-h))
(defun op-flip-v () (list :kind :flip-v))
(defun op-crop (x y w h) (list :kind :crop :x x :y y :width w :height h))
(defun op-brightness (delta) (list :kind :brightness :delta delta))
(defun op-contrast (factor) (list :kind :contrast :factor factor))

(define-condition op-error (error) ((reason :initarg :reason :reader op-error-reason)))

(defun signal-op-error (reason)
  (error 'op-error :reason reason))

(defun get-pixel (img x y)
  (grayscale::image-get img x y))

(defun clamp-u8 (v) (max 0 (min 255 v)))

(defun adj-brightness (p delta)
  (grayscale::make-rgb
   :r (clamp-u8 (+ (grayscale::rgb-r p) delta))
   :g (clamp-u8 (+ (grayscale::rgb-g p) delta))
   :b (clamp-u8 (+ (grayscale::rgb-b p) delta))))

(defun adj-contrast (p factor)
  (flet ((adj (v) (clamp-u8 (round (+ (* (- v 128) factor) 128)))))
    (grayscale::make-rgb
     :r (adj (grayscale::rgb-r p))
     :g (adj (grayscale::rgb-g p))
     :b (adj (grayscale::rgb-b p)))))

(defun map-pixels (img f)
  (grayscale::make-image
   :width (grayscale::image-width img)
   :height (grayscale::image-height img)
   :pixels (mapcar f (grayscale::image-pixels img))))

(defun grayscale-rgb (img algo)
  (map-pixels img (lambda (p)
                    (let ((v (grayscale::pixel-to-gray p algo)))
                      (grayscale::make-rgb :r v :g v :b v)))))

(defun resize-nearest (img new-w new-h)
  (let ((w (grayscale::image-width img))
        (h (grayscale::image-height img))
        (pixels nil))
    (loop for y from 0 below new-h do
          (loop for x from 0 below new-w do
                (let ((sx (min (1- w) (floor (* x w) new-w)))
                      (sy (min (1- h) (floor (* y h) new-h))))
                  (push (get-pixel img sx sy) pixels))))
    (grayscale::make-image :width new-w :height new-h
                            :pixels (nreverse pixels))))

(defun build-image (w h cell-fn)
  (let ((pixels nil))
    (loop for y from 0 below h do
          (loop for x from 0 below w do
                (push (funcall cell-fn x y) pixels)))
    (grayscale::make-image :width w :height h :pixels (nreverse pixels))))

(defun rotate-90 (img)
  (let ((w (grayscale::image-width img))
        (h (grayscale::image-height img)))
    (build-image h w
                  (lambda (nx ny)
                    ;; new (nx, ny) maps from old (ny, h - 1 - nx)
                    (get-pixel img ny (- h 1 nx))))))

(defun rotate-180 (img)
  (let ((w (grayscale::image-width img))
        (h (grayscale::image-height img)))
    (build-image w h
                  (lambda (nx ny)
                    (get-pixel img (- w 1 nx) (- h 1 ny))))))

(defun rotate-270 (img)
  (let ((w (grayscale::image-width img))
        (h (grayscale::image-height img)))
    (build-image h w
                  (lambda (nx ny)
                    (get-pixel img (- w 1 ny) nx)))))

(defun flip-h (img)
  (let ((w (grayscale::image-width img))
        (h (grayscale::image-height img)))
    (build-image w h
                  (lambda (nx ny) (get-pixel img (- w 1 nx) ny)))))

(defun flip-v (img)
  (let ((w (grayscale::image-width img))
        (h (grayscale::image-height img)))
    (build-image w h
                  (lambda (nx ny) (get-pixel img nx (- h 1 ny))))))

(defun crop-img (img x0 y0 w h)
  (build-image w h
                (lambda (nx ny) (get-pixel img (+ x0 nx) (+ y0 ny)))))

(defun apply-op (img op)
  (case (getf op :kind)
    (:resize
     (let ((w (getf op :width)) (h (getf op :height)))
       (when (or (zerop w) (zerop h)) (signal-op-error :zero-dimension))
       (resize-nearest img w h)))
    (:grayscale (grayscale-rgb img (getf op :algo)))
    (:rotate-90 (rotate-90 img))
    (:rotate-180 (rotate-180 img))
    (:rotate-270 (rotate-270 img))
    (:flip-h (flip-h img))
    (:flip-v (flip-v img))
    (:crop
     (let ((x (getf op :x)) (y (getf op :y))
           (w (getf op :width)) (h (getf op :height)))
       (when (or (zerop w) (zerop h)) (signal-op-error :zero-dimension))
       (when (or (> (+ x w) (grayscale::image-width img))
                  (> (+ y h) (grayscale::image-height img)))
         (signal-op-error :crop-out-of-bounds))
       (crop-img img x y w h)))
    (:brightness (map-pixels img (lambda (p) (adj-brightness p (getf op :delta)))))
    (:contrast (map-pixels img (lambda (p) (adj-contrast p (getf op :factor)))))))

(defun apply-pipeline (img pipeline)
  (let ((cur img))
    (dolist (op pipeline) (setf cur (apply-op cur op)))
    cur))

(defun run-batch (images pipeline)
  (let ((results nil)
        (i 0))
    (dolist (img images)
      (handler-case
          (let ((out (apply-pipeline img pipeline)))
            (push (list :index i :output out) results))
        (op-error (e)
          (push (list :index i :error (op-error-reason e)) results)))
      (incf i))
    (nreverse results)))

(defun dry-run-dims (start pipeline)
  (let ((out (list start))
        (cur start))
    (dolist (op pipeline)
      (case (getf op :kind)
        (:resize
         (let ((w (getf op :width)) (h (getf op :height)))
           (when (or (zerop w) (zerop h)) (signal-op-error :zero-dimension))
           (setf cur (cons w h))))
        ((:grayscale :flip-h :flip-v :rotate-180 :brightness :contrast)) ; noop
        ((:rotate-90 :rotate-270)
         (setf cur (cons (cdr cur) (car cur))))
        (:crop
         (let ((x (getf op :x)) (y (getf op :y))
               (w (getf op :width)) (h (getf op :height)))
           (when (or (zerop w) (zerop h)) (signal-op-error :zero-dimension))
           (when (or (> (+ x w) (car cur)) (> (+ y h) (cdr cur)))
             (signal-op-error :crop-out-of-bounds))
           (setf cur (cons w h)))))
      (push cur out))
    (nreverse out)))

(defun demo ()
  (let* ((pixels (loop for y from 0 below 4 append
                          (loop for x from 0 below 4 collect
                                (if (zerop (mod (+ x y) 2))
                                    (grayscale::make-rgb :r 200 :g 100 :b 50)
                                    (grayscale::make-rgb :r 50 :g 100 :b 200)))))
         (img (grayscale::make-image :width 4 :height 4 :pixels pixels))
         (pipeline (list (op-rotate-90)
                          (op-brightness 10)
                          (op-grayscale :luminosity))))
    (let ((out (apply-pipeline img pipeline))
          (dims (dry-run-dims (cons 4 4) pipeline)))
      (format t "pipeline steps: ~D~%" (length pipeline))
      (format t "dry-run dims: ~A~%" dims)
      (let ((p (first (grayscale::image-pixels out))))
        (format t "output: ~Dx~D pixels[0]=(~D,~D,~D)~%"
                (grayscale::image-width out)
                (grayscale::image-height out)
                (grayscale::rgb-r p) (grayscale::rgb-g p) (grayscale::rgb-b p))))
    (let* ((batch (list (grayscale::make-image :width 6 :height 6
                                                  :pixels (loop repeat 36 collect
                                                                 (grayscale::make-rgb :r 128 :g 128 :b 128)))
                          (grayscale::make-image :width 2 :height 2
                                                  :pixels (loop repeat 4 collect
                                                                 (grayscale::make-rgb :r 128 :g 128 :b 128)))))
            (crop-pipe (list (op-crop 0 0 3 3)))
            (results (run-batch batch crop-pipe)))
      (dolist (r results)
        (if (getf r :output)
            (format t "  [~D] ok: ~Dx~D~%"
                    (getf r :index)
                    (grayscale::image-width (getf r :output))
                    (grayscale::image-height (getf r :output)))
            (format t "  [~D] err: ~A~%" (getf r :index) (getf r :error)))))))
