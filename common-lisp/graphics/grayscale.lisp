;;;; Grayscale Converter — pixel buffers + named conversion algorithms + histogram.

(defpackage #:grayscale
  (:use #:cl)
  (:export #:make-rgb #:rgb-r #:rgb-g #:rgb-b
           #:make-image #:image-from-pixels #:image-get #:image-set
           #:image-width #:image-height #:image-pixels
           #:make-gray-image #:gray-image-get
           #:gray-image-width #:gray-image-height #:gray-image-pixels
           #:pixel-to-gray #:convert #:histogram
           #:mean-brightness #:contrast-stddev #:dynamic-range))

(in-package #:grayscale)

(defstruct rgb
  (r 0 :type integer)
  (g 0 :type integer)
  (b 0 :type integer))

(defstruct image
  (width 0 :type integer)
  (height 0 :type integer)
  (pixels nil :type list))

(defstruct gray-image
  (width 0 :type integer)
  (height 0 :type integer)
  (pixels nil :type list))

(defun image-from-pixels (width height pixels)
  (when (= (length pixels) (* width height))
    (make-image :width width :height height :pixels pixels)))

(defun image-filled (width height color)
  (make-image :width width :height height
              :pixels (loop repeat (* width height) collect color)))

(defun image-get (img x y)
  (when (and (>= x 0) (< x (image-width img))
             (>= y 0) (< y (image-height img)))
    (nth (+ (* y (image-width img)) x) (image-pixels img))))

(defun gray-image-get (img x y)
  (when (and (>= x 0) (< x (gray-image-width img))
             (>= y 0) (< y (gray-image-height img)))
    (nth (+ (* y (gray-image-width img)) x) (gray-image-pixels img))))

(defun clamp-u8 (v)
  (max 0 (min 255 v)))

(defun pixel-to-gray (p algo)
  (case algo
    (:luminosity
     (clamp-u8 (round (+ (* 0.2126 (rgb-r p))
                          (* 0.7152 (rgb-g p))
                          (* 0.0722 (rgb-b p))))))
    (:average (floor (+ (rgb-r p) (rgb-g p) (rgb-b p)) 3))
    (:lightness
     (let ((mx (max (rgb-r p) (rgb-g p) (rgb-b p)))
           (mn (min (rgb-r p) (rgb-g p) (rgb-b p))))
       (floor (+ mx mn) 2)))
    (:red (rgb-r p))
    (:green (rgb-g p))
    (:blue (rgb-b p))))

(defun convert (img algo)
  (make-gray-image
   :width (image-width img)
   :height (image-height img)
   :pixels (mapcar (lambda (p) (pixel-to-gray p algo)) (image-pixels img))))

(defun histogram (gray)
  (let ((bins (make-array 256 :initial-element 0 :element-type 'integer)))
    (dolist (v (gray-image-pixels gray))
      (incf (aref bins v)))
    (coerce bins 'list)))

(defun mean-brightness (gray)
  (if (null (gray-image-pixels gray)) 0.0
      (/ (reduce #'+ (gray-image-pixels gray) :initial-value 0)
         (float (length (gray-image-pixels gray))))))

(defun contrast-stddev (gray)
  (if (null (gray-image-pixels gray)) 0.0
      (let* ((m (mean-brightness gray))
             (n (length (gray-image-pixels gray)))
             (var (/ (reduce #'+ (gray-image-pixels gray)
                              :key (lambda (v) (expt (- v m) 2))
                              :initial-value 0.0)
                      n)))
        (sqrt var))))

(defun dynamic-range (gray)
  (when (gray-image-pixels gray)
    (list (reduce #'min (gray-image-pixels gray))
          (reduce #'max (gray-image-pixels gray)))))

(defun demo ()
  (let ((img (image-from-pixels 2 2
               (list (make-rgb :r 0 :g 0 :b 0)
                     (make-rgb :r 255 :g 0 :b 0)
                     (make-rgb :r 0 :g 255 :b 0)
                     (make-rgb :r 0 :g 0 :b 255)))))
    (dolist (algo '(:luminosity :average :lightness))
      (let ((g (convert img algo)))
        (format t "~10A: pixels=~A mean=~,2F stddev=~,2F range=~A~%"
                algo (gray-image-pixels g)
                (mean-brightness g) (contrast-stddev g)
                (dynamic-range g))))))
