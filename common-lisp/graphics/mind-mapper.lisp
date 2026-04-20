;;;; Mind Mapper — radial layout + SVG emission.

(defpackage #:mind-mapper
  (:use #:cl)
  (:export #:make-mind-node #:mind-node-with-child #:mind-node-with-color
           #:mind-node-find #:mind-node-toggle-collapse
           #:mind-node-visible-count #:mind-node-total-count #:mind-node-max-depth
           #:radial-layout #:traverse-dfs #:traverse-bfs #:to-svg
           #:laid-out-node-id #:laid-out-node-label #:laid-out-node-depth
           #:laid-out-node-x #:laid-out-node-y #:laid-out-node-parent-id
           #:laid-out-node-color #:laid-out-node-collapsed
           #:mind-node-id #:mind-node-label #:mind-node-color
           #:mind-node-collapsed #:mind-node-children))

(in-package #:mind-mapper)

(defstruct mind-node
  (id 0 :type integer)
  (label "" :type string)
  (color "#3b82f6" :type string)
  (collapsed nil :type boolean)
  (children nil :type list))

(defun mind-node-with-child (n child)
  (setf (mind-node-children n) (append (mind-node-children n) (list child)))
  n)

(defun mind-node-with-color (n color)
  (setf (mind-node-color n) color)
  n)

(defun mind-node-find (n id)
  (if (= (mind-node-id n) id)
      n
      (loop for c in (mind-node-children n)
            for found = (mind-node-find c id)
            when found return found)))

(defun mind-node-toggle-collapse (n id)
  (let ((target (mind-node-find n id)))
    (when target
      (setf (mind-node-collapsed target) (not (mind-node-collapsed target)))
      (mind-node-collapsed target))))

(defun mind-node-visible-count (n)
  (if (mind-node-collapsed n) 1
      (+ 1 (reduce #'+ (mind-node-children n)
                    :key #'mind-node-visible-count :initial-value 0))))

(defun mind-node-total-count (n)
  (+ 1 (reduce #'+ (mind-node-children n)
                :key #'mind-node-total-count :initial-value 0)))

(defun mind-node-max-depth (n)
  (if (or (null (mind-node-children n)) (mind-node-collapsed n))
      0
      (1+ (reduce #'max (mind-node-children n) :key #'mind-node-max-depth
                   :initial-value 0))))

(defstruct laid-out-node
  (id 0 :type integer)
  (label "" :type string)
  (color "" :type string)
  (depth 0 :type integer)
  (x 0.0 :type double-float)
  (y 0.0 :type double-float)
  (parent-id nil :type (or null integer))
  (collapsed nil :type boolean))

(defun layout-children (parent arc-start arc-end depth radius-step cx cy out)
  (let ((n (length (mind-node-children parent))))
    (when (> n 0)
      (let ((slice-width (/ (- arc-end arc-start) n)))
        (loop for child in (mind-node-children parent)
              for i from 0
              for a0 = (+ arc-start (* slice-width i))
              for a1 = (+ arc-start (* slice-width (1+ i)))
              for angle = (/ (+ a0 a1) 2)
              for r = (* depth radius-step)
              for x = (+ cx (* r (cos angle)))
              for y = (+ cy (* r (sin angle)))
              do (vector-push-extend
                   (make-laid-out-node :id (mind-node-id child)
                                        :label (mind-node-label child)
                                        :color (mind-node-color child)
                                        :depth depth :x (coerce x 'double-float)
                                        :y (coerce y 'double-float)
                                        :parent-id (mind-node-id parent)
                                        :collapsed (mind-node-collapsed child))
                   out)
                 (unless (mind-node-collapsed child)
                   (layout-children child a0 a1 (1+ depth) radius-step cx cy out)))))))

(defun radial-layout (root cx cy radius-step)
  (let ((out (make-array 16 :fill-pointer 0 :adjustable t)))
    (vector-push-extend (make-laid-out-node :id (mind-node-id root)
                                              :label (mind-node-label root)
                                              :color (mind-node-color root)
                                              :depth 0
                                              :x (coerce cx 'double-float)
                                              :y (coerce cy 'double-float)
                                              :collapsed (mind-node-collapsed root))
                         out)
    (unless (mind-node-collapsed root)
      (layout-children root 0.0 (* 2 pi) 1 radius-step cx cy out))
    (coerce out 'list)))

(defun dfs-visit (node out)
  (vector-push-extend (mind-node-id node) out)
  (unless (mind-node-collapsed node)
    (dolist (c (mind-node-children node))
      (dfs-visit c out))))

(defun traverse-dfs (root)
  (let ((out (make-array 8 :fill-pointer 0 :adjustable t)))
    (dfs-visit root out)
    (coerce out 'list)))

(defun traverse-bfs (root)
  (let ((out nil)
        (queue (list root)))
    (loop while queue do
          (let ((n (pop queue)))
            (push (mind-node-id n) out)
            (unless (mind-node-collapsed n)
              (dolist (c (mind-node-children n))
                (setf queue (append queue (list c)))))))
    (nreverse out)))

(defun escape-xml (s)
  (with-output-to-string (out)
    (loop for c across s do
          (case c
            (#\& (write-string "&amp;" out))
            (#\< (write-string "&lt;" out))
            (#\> (write-string "&gt;" out))
            (#\" (write-string "&quot;" out))
            (t (write-char c out))))))

(defun to-svg (nodes width height radius)
  (with-output-to-string (out)
    (format out "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"~A\" height=\"~A\" viewBox=\"0 0 ~A ~A\">~%"
            width height width height)
    (dolist (n nodes)
      (when (laid-out-node-parent-id n)
        (let ((parent (find (laid-out-node-parent-id n) nodes
                             :key #'laid-out-node-id)))
          (when parent
            (format out "  <line x1=\"~,2F\" y1=\"~,2F\" x2=\"~,2F\" y2=\"~,2F\" stroke=\"#cbd5e1\" stroke-width=\"2\"/>~%"
                    (laid-out-node-x parent) (laid-out-node-y parent)
                    (laid-out-node-x n) (laid-out-node-y n))))))
    (dolist (n nodes)
      (format out "  <circle cx=\"~,2F\" cy=\"~,2F\" r=\"~A\" fill=\"~A\"/>~%"
              (laid-out-node-x n) (laid-out-node-y n) radius (laid-out-node-color n))
      (format out "  <text x=\"~,2F\" y=\"~,2F\" text-anchor=\"middle\" dy=\"0.35em\" fill=\"white\" font-size=\"12\">~A</text>~%"
              (laid-out-node-x n) (laid-out-node-y n) (escape-xml (laid-out-node-label n))))
    (format out "</svg>~%")))

(defun demo ()
  (let ((tree (mind-node-with-child
               (mind-node-with-child
                (make-mind-node :id 1 :label "Rosetta Stone")
                (mind-node-with-child
                 (mind-node-with-child
                  (make-mind-node :id 2 :label "Languages")
                  (make-mind-node :id 5 :label "Rust"))
                 (make-mind-node :id 6 :label "Python")))
               (mind-node-with-child
                (make-mind-node :id 3 :label "Categories")
                (make-mind-node :id 8 :label "Graphics")))))
    (let ((nodes (radial-layout tree 500.0 500.0 150.0)))
      (format t "total layout nodes: ~D~%" (length nodes))
      (dolist (n nodes)
        (format t "  #~2D ~20A depth=~D at (~6,1F, ~6,1F)~%"
                (laid-out-node-id n) (laid-out-node-label n)
                (laid-out-node-depth n)
                (laid-out-node-x n) (laid-out-node-y n))))
    (format t "~%DFS: ~A~%" (traverse-dfs tree))
    (format t "BFS: ~A~%" (traverse-bfs tree))
    (mind-node-toggle-collapse tree 2)
    (format t "~%After collapse(2), visible count: ~D~%"
            (mind-node-visible-count tree))))
