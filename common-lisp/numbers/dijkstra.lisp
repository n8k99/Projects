;;;; Dijkstra's Algorithm — shortest paths in weighted graphs.

(defpackage :numbers/dijkstra
  (:use :cl)
  (:export :make-graph :add-edge :shortest-path :shortest-distances))

(in-package :numbers/dijkstra)

;;; Graph representation: hash-table mapping node -> list of (neighbor . weight).

(defstruct graph
  (adj (make-hash-table :test 'equal) :type hash-table))

(defun ensure-node (g node)
  "Ensure NODE exists in graph G's adjacency table."
  (unless (gethash node (graph-adj g))
    (setf (gethash node (graph-adj g)) nil)))

(defun add-edge (g u v weight &optional undirected)
  "Add a weighted edge from U to V. If UNDIRECTED, add both directions."
  (push (cons v weight) (gethash u (graph-adj g)))
  (ensure-node g v)
  (when undirected
    (push (cons u weight) (gethash v (graph-adj g)))))

;;; Priority queue via sorted insertion (adequate for small graphs).

(defun pq-insert (pq cost node)
  "Insert (COST . NODE) into priority queue PQ, maintaining sort order."
  (merge 'list (list (cons cost node)) pq #'< :key #'car))

(defun shortest-distances (g start)
  "Return hash-table mapping each node to its shortest distance from START."
  (let ((dist (make-hash-table :test 'equal))
        (adj (graph-adj g)))
    ;; Initialise all distances to infinity.
    (maphash (lambda (node _)
               (declare (ignore _))
               (setf (gethash node dist) most-positive-double-float))
             adj)
    (setf (gethash start dist) 0.0d0)
    (let ((pq (list (cons 0.0d0 start))))
      (loop while pq do
        (let* ((top (pop pq))
               (d (car top))
               (u (cdr top)))
          (when (<= d (gethash u dist most-positive-double-float))
            (dolist (edge (gethash u adj))
              (let* ((v (car edge))
                     (w (cdr edge))
                     (nd (+ d w)))
                (when (< nd (gethash v dist most-positive-double-float))
                  (setf (gethash v dist) nd)
                  (setf pq (pq-insert pq nd v)))))))))
    dist))

(defun shortest-path (g start end)
  "Return (VALUES distance path) for the shortest path from START to END.
   Returns (VALUES NIL NIL) if no path exists."
  (let ((dist (make-hash-table :test 'equal))
        (prev (make-hash-table :test 'equal))
        (adj (graph-adj g)))
    ;; Initialise.
    (maphash (lambda (node _)
               (declare (ignore _))
               (setf (gethash node dist) most-positive-double-float))
             adj)
    (setf (gethash start dist) 0.0d0)
    (let ((pq (list (cons 0.0d0 start))))
      (loop while pq do
        (let* ((top (pop pq))
               (d (car top))
               (u (cdr top)))
          (when (<= d (gethash u dist most-positive-double-float))
            (when (equal u end)
              (return))
            (dolist (edge (gethash u adj))
              (let* ((v (car edge))
                     (w (cdr edge))
                     (nd (+ d w)))
                (when (< nd (gethash v dist most-positive-double-float))
                  (setf (gethash v dist) nd)
                  (setf (gethash v prev) u)
                  (setf pq (pq-insert pq nd v)))))))))
    ;; Check reachability.
    (let ((d (gethash end dist most-positive-double-float)))
      (if (= d most-positive-double-float)
          (values nil nil)
          ;; Reconstruct path.
          (let ((path nil)
                (node end))
            (loop while node do
              (push node path)
              (setf node (gethash node prev)))
            (values d path))))))
