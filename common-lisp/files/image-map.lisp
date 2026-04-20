;;;; Image Map Generator — clickable regions on an image with hit testing.

(defpackage #:image-map
  (:use #:cl)
  (:export #:make-rect #:make-circle #:make-polygon
           #:shape-contains #:shape-bbox
           #:make-image-map #:add-region #:hit-test #:hit-test-all #:to-html
           #:make-region #:region-id #:region-action #:region-alt #:region-z))

(in-package #:image-map)

(defstruct shape
  (kind :rect :type keyword)
  (x 0 :type integer)
  (y 0 :type integer)
  (w 0 :type integer)
  (h 0 :type integer)
  (cx 0 :type integer)
  (cy 0 :type integer)
  (r 0 :type integer)
  (points nil :type list))

(defun make-rect (x y w h)
  (make-shape :kind :rect :x x :y y :w w :h h))
(defun make-circle (cx cy r)
  (make-shape :kind :circle :cx cx :cy cy :r r))
(defun make-polygon (points)
  (make-shape :kind :polygon :points points))

(defstruct region
  (id 0 :type integer)
  (shape nil :type (or null shape))
  (action "" :type string)
  (alt "" :type string)
  (z 0 :type integer))

(defstruct image-map (regions nil :type list))

(defun point-in-polygon (x y points)
  (if (< (length points) 3)
      nil
      (let ((inside nil)
            (n (length points)))
        (loop for i from 0 below n
              for j = (mod (1- i) n)
              for (xi yi) = (nth i points)
              for (xj yj) = (nth j points)
              do (when (not (eq (> yi y) (> yj y)))
                   (let ((x-int (+ (truncate (* (- xj xi) (- y yi)) (- yj yi)) xi)))
                     (when (< x x-int)
                       (setf inside (not inside))))))
        inside)))

(defun shape-contains (shape x y)
  (case (shape-kind shape)
    (:rect
     (and (>= x (shape-x shape))
          (< x (+ (shape-x shape) (shape-w shape)))
          (>= y (shape-y shape))
          (< y (+ (shape-y shape) (shape-h shape)))))
    (:circle
     (let ((dx (- x (shape-cx shape)))
           (dy (- y (shape-cy shape))))
       (<= (+ (* dx dx) (* dy dy)) (* (shape-r shape) (shape-r shape)))))
    (:polygon (point-in-polygon x y (shape-points shape)))))

(defun shape-bbox (shape)
  (case (shape-kind shape)
    (:rect (list (shape-x shape) (shape-y shape) (shape-w shape) (shape-h shape)))
    (:circle (list (- (shape-cx shape) (shape-r shape))
                    (- (shape-cy shape) (shape-r shape))
                    (* 2 (shape-r shape))
                    (* 2 (shape-r shape))))
    (:polygon
     (let ((xs (mapcar #'first (shape-points shape)))
           (ys (mapcar #'second (shape-points shape))))
       (list (reduce #'min xs) (reduce #'min ys)
             (- (reduce #'max xs) (reduce #'min xs))
             (- (reduce #'max ys) (reduce #'min ys)))))))

(defun add-region (m region)
  (setf (image-map-regions m)
        (append (image-map-regions m) (list region)))
  m)

(defun sorted-by-z (regions)
  (sort (copy-list regions) (lambda (a b) (> (region-z a) (region-z b)))))

(defun hit-test (m x y)
  (find-if (lambda (r) (shape-contains (region-shape r) x y))
           (sorted-by-z (image-map-regions m))))

(defun hit-test-all (m x y)
  (remove-if-not (lambda (r) (shape-contains (region-shape r) x y))
                 (sorted-by-z (image-map-regions m))))

(defun shape-to-html (shape)
  (case (shape-kind shape)
    (:rect
     (values "rect" (format nil "~D,~D,~D,~D"
                             (shape-x shape) (shape-y shape)
                             (+ (shape-x shape) (shape-w shape))
                             (+ (shape-y shape) (shape-h shape)))))
    (:circle
     (values "circle" (format nil "~D,~D,~D"
                               (shape-cx shape) (shape-cy shape) (shape-r shape))))
    (:polygon
     (values "poly"
              (format nil "~{~D,~D~^,~}"
                      (loop for (x y) in (shape-points shape)
                            collect x collect y))))))

(defun to-html (m name)
  (with-output-to-string (out)
    (format out "<map name=\"~A\">~%" name)
    (dolist (r (image-map-regions m))
      (multiple-value-bind (shape-name coords) (shape-to-html (region-shape r))
        (format out "  <area shape=\"~A\" coords=\"~A\" href=\"~A\" alt=\"~A\">~%"
                shape-name coords (region-action r) (region-alt r))))
    (format out "</map>~%")))

(defun demo ()
  (let ((m (make-image-map)))
    (add-region m (make-region :id 1 :shape (make-rect 0 0 100 100)
                                :action "/home" :alt "Home" :z 0))
    (add-region m (make-region :id 2 :shape (make-circle 50 50 20)
                                :action "/center" :alt "Centre" :z 1))
    (add-region m (make-region :id 3 :shape (make-polygon '((200 0) (250 0) (225 50)))
                                :action "/tri" :alt "Tri" :z 0))
    (format t "=== hit tests ===~%")
    (dolist (pt '((10 10) (50 50) (225 25) (500 500)))
      (let ((hit (hit-test m (first pt) (second pt))))
        (format t "  (~3D,~3D) -> ~A~%"
                (first pt) (second pt)
                (if hit (region-action hit) "miss"))))
    (format t "~%=== HTML ===~%~A" (to-html m "nav"))))
