;;;; ERD Creator — entity-relationship diagram with cardinality + topo sort.

(defpackage #:erd-creator
  (:use #:cl)
  (:export #:make-attribute #:make-entity #:make-relationship #:make-erd
           #:erd-add-entity #:erd-add-relationship
           #:erd-dependencies-of #:erd-topo-sort #:erd-find-cycles
           #:erd-validate #:erd-to-dot))

(in-package #:erd-creator)

(defstruct attribute
  (name "" :type string)
  (ty "" :type string)
  (primary-key nil :type boolean)
  (nullable nil :type boolean))

(defstruct entity
  (name "" :type string)
  (attributes nil :type list))

(defstruct relationship
  (from-entity "" :type string)
  (from-attr "" :type string)
  (to-entity "" :type string)
  (to-attr "" :type string)
  (cardinality :one-to-one :type keyword))

(defstruct erd
  (entities (make-hash-table :test 'equal) :type hash-table)
  (relationships nil :type list))

(defun erd-add-entity (erd e)
  (setf (gethash (entity-name e) (erd-entities erd)) e))

(defun erd-add-relationship (erd r)
  (setf (erd-relationships erd) (append (erd-relationships erd) (list r))))

(defun erd-dependencies-of (erd name)
  (let ((deps (make-hash-table :test 'equal)))
    (dolist (r (erd-relationships erd))
      (when (and (string= (relationship-from-entity r) name)
                  (not (string= (relationship-to-entity r) name)))
        (setf (gethash (relationship-to-entity r) deps) t)))
    (sort (loop for k being the hash-keys of deps collect k) #'string<)))

(defun erd-topo-sort (erd)
  (let ((in-degree (make-hash-table :test 'equal)))
    (maphash (lambda (k v) (declare (ignore v))
               (setf (gethash k in-degree) 0))
             (erd-entities erd))
    (dolist (r (erd-relationships erd))
      (when (and (not (string= (relationship-from-entity r) (relationship-to-entity r)))
                  (gethash (relationship-from-entity r) (erd-entities erd))
                  (gethash (relationship-to-entity r) (erd-entities erd)))
        (incf (gethash (relationship-from-entity r) in-degree 0))))
    (let ((queue (sort (loop for k being the hash-keys of in-degree
                              when (zerop (gethash k in-degree)) collect k)
                        #'string<))
          (out nil))
      (loop while queue do
            (let ((n (pop queue)))
              (push n out)
              (dolist (r (erd-relationships erd))
                (when (and (string= (relationship-to-entity r) n)
                            (not (string= (relationship-from-entity r)
                                           (relationship-to-entity r))))
                  (let ((from (relationship-from-entity r)))
                    (when (gethash from in-degree)
                      (when (> (gethash from in-degree) 0)
                        (decf (gethash from in-degree)))
                      (when (and (zerop (gethash from in-degree))
                                  (not (member from out :test #'string=))
                                  (not (member from queue :test #'string=)))
                        (push from queue)
                        (setf queue (sort queue #'string<)))))))))
      (let ((result (nreverse out)))
        (if (= (length result) (hash-table-count (erd-entities erd)))
            result
            nil)))))

(defun erd-find-cycles (erd)
  (let ((adj (make-hash-table :test 'equal))
        (cycles nil)
        (visited (make-hash-table :test 'equal)))
    (maphash (lambda (k v) (declare (ignore v))
               (setf (gethash k adj) nil))
             (erd-entities erd))
    (dolist (r (erd-relationships erd))
      (when (and (not (string= (relationship-from-entity r) (relationship-to-entity r)))
                  (gethash (relationship-from-entity r) (erd-entities erd))
                  (gethash (relationship-to-entity r) (erd-entities erd)))
        (push (relationship-to-entity r)
              (gethash (relationship-from-entity r) adj))))
    (labels ((dfs (node path on-path)
               (dolist (child (gethash node adj))
                 (cond
                   ((gethash child on-path)
                    (let ((start (position child path :test #'string=)))
                      (push (append (subseq path start) (list child)) cycles)))
                   ((not (gethash child visited))
                    (dfs child (append path (list child))
                          (let ((new (make-hash-table :test 'equal)))
                            (maphash (lambda (k v) (setf (gethash k new) v)) on-path)
                            (setf (gethash child new) t)
                            new))))
                 (setf (gethash node visited) t))))
      (dolist (start (sort (loop for k being the hash-keys of (erd-entities erd)
                                 collect k)
                           #'string<))
        (unless (gethash start visited)
          (let ((on-path (make-hash-table :test 'equal)))
            (setf (gethash start on-path) t)
            (dfs start (list start) on-path)))))
    cycles))

(defun erd-validate (erd)
  (let ((issues nil)
        (i 0))
    (dolist (r (erd-relationships erd))
      (let ((from-e (gethash (relationship-from-entity r) (erd-entities erd)))
            (to-e (gethash (relationship-to-entity r) (erd-entities erd))))
        (if from-e
            (unless (find (relationship-from-attr r)
                          (entity-attributes from-e)
                          :key #'attribute-name :test #'string=)
              (push (format nil "relationship ~D: ~A.~A — attribute missing"
                              i (relationship-from-entity r)
                              (relationship-from-attr r))
                    issues))
            (push (format nil "relationship ~D: unknown from_entity ~A"
                            i (relationship-from-entity r))
                  issues))
        (if to-e
            (unless (find (relationship-to-attr r)
                          (entity-attributes to-e)
                          :key #'attribute-name :test #'string=)
              (push (format nil "relationship ~D: ~A.~A — attribute missing"
                              i (relationship-to-entity r)
                              (relationship-to-attr r))
                    issues))
            (push (format nil "relationship ~D: unknown to_entity ~A"
                            i (relationship-to-entity r))
                  issues))
        (incf i)))
    (nreverse issues)))

(defun cardinality-arrow (c)
  (case c
    (:one-to-one "none")
    (:one-to-many "crow")
    (:many-to-many "crowodot")
    (t "none")))

(defun erd-to-dot (erd)
  (with-output-to-string (out)
    (format out "digraph ERD {~%  node [shape=record];~%")
    (let ((names (sort (loop for k being the hash-keys of (erd-entities erd) collect k)
                        #'string<)))
      (dolist (name names)
        (let* ((e (gethash name (erd-entities erd)))
               (attrs-str (format nil "~{~A~^\\l~}"
                                    (mapcar (lambda (a)
                                              (format nil "~A~A: ~A"
                                                      (if (attribute-primary-key a) "🔑 " "")
                                                      (attribute-name a)
                                                      (attribute-ty a)))
                                            (entity-attributes e)))))
          (format out "  ~A [label=\"{~A|~A\\l}\"];~%" name name attrs-str))))
    (dolist (r (erd-relationships erd))
      (format out "  ~A -> ~A [label=\"~A.~A -> ~A.~A\", arrowhead=\"~A\"];~%"
              (relationship-from-entity r) (relationship-to-entity r)
              (relationship-from-entity r) (relationship-from-attr r)
              (relationship-to-entity r) (relationship-to-attr r)
              (cardinality-arrow (relationship-cardinality r))))
    (format out "}~%")))

(defun demo ()
  (let ((erd (make-erd)))
    (erd-add-entity erd (make-entity :name "User"
                                      :attributes (list (make-attribute :name "id" :ty "INT" :primary-key t)
                                                         (make-attribute :name "email" :ty "TEXT")
                                                         (make-attribute :name "name" :ty "TEXT"))))
    (erd-add-entity erd (make-entity :name "Order"
                                      :attributes (list (make-attribute :name "id" :ty "INT" :primary-key t)
                                                         (make-attribute :name "user_id" :ty "INT")
                                                         (make-attribute :name "total" :ty "INT"))))
    (erd-add-entity erd (make-entity :name "OrderItem"
                                      :attributes (list (make-attribute :name "id" :ty "INT" :primary-key t)
                                                         (make-attribute :name "order_id" :ty "INT")
                                                         (make-attribute :name "product_id" :ty "INT"))))
    (erd-add-entity erd (make-entity :name "Product"
                                      :attributes (list (make-attribute :name "id" :ty "INT" :primary-key t)
                                                         (make-attribute :name "name" :ty "TEXT")
                                                         (make-attribute :name "price" :ty "INT"))))
    (erd-add-relationship erd (make-relationship :from-entity "Order" :from-attr "user_id"
                                                  :to-entity "User" :to-attr "id"
                                                  :cardinality :one-to-many))
    (erd-add-relationship erd (make-relationship :from-entity "OrderItem" :from-attr "order_id"
                                                  :to-entity "Order" :to-attr "id"
                                                  :cardinality :one-to-many))
    (erd-add-relationship erd (make-relationship :from-entity "OrderItem" :from-attr "product_id"
                                                  :to-entity "Product" :to-attr "id"
                                                  :cardinality :many-to-many))

    (format t "=== topological sort ===~%  ~A~%" (erd-topo-sort erd))
    (format t "~%=== cycles ===~%  ~A~%" (erd-find-cycles erd))
    (format t "~%=== validation ===~%")
    (dolist (i (erd-validate erd)) (format t "  ~A~%" i))
    (format t "~%=== DOT export (first 200 chars) ===~%")
    (let ((dot (erd-to-dot erd)))
      (format t "~A" (subseq dot 0 (min 200 (length dot)))))))
