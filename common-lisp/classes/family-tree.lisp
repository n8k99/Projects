;;;; Family Tree Creator — the first self-referential graph with cycle prevention.
;;;;
;;;; A PERSON refers to other persons (as parents). The structure is a DAG, not
;;;; a tree — "family tree" is a misnomer. A person has at most two parents, but
;;;; joining unrelated branches yields multiple paths to shared ancestors, so the
;;;; code treats it as a graph throughout.
;;;;
;;;; SET-PARENTS runs a transitive-closure check: a candidate parent must not
;;;; already be a descendant of the child. First Classes project where an
;;;; invariant spans the entire reachable structure.

(defpackage #:family-tree
  (:use #:cl)
  (:export #:make-ftree
           #:add-person #:get-person
           #:set-parents
           #:ancestors #:descendants
           #:common-ancestors #:siblings
           #:person-name))

(in-package #:family-tree)

(defstruct person
  (id 0 :type integer)
  (name "" :type string)
  (birth nil :type (or null integer))
  (death nil :type (or null integer))
  (parents nil :type list))

(defstruct ftree
  (persons (make-hash-table :test 'eql))        ; id -> person
  (children (make-hash-table :test 'eql))       ; id -> list of child ids
  (next-id 1 :type integer))

(defun add-person (ftree name &key birth death)
  (let ((id (ftree-next-id ftree)))
    (setf (gethash id (ftree-persons ftree))
          (make-person :id id :name name :birth birth :death death))
    (setf (gethash id (ftree-children ftree)) nil)
    (incf (ftree-next-id ftree))
    id))

(defun get-person (ftree id)
  (or (gethash id (ftree-persons ftree))
      (error "unknown person: ~A" id)))

(defun %assert-known (ftree id)
  (unless (gethash id (ftree-persons ftree))
    (error "unknown person: ~A" id)))

(defun set-parents (ftree child-id new-parents)
  (%assert-known ftree child-id)
  (when (> (length new-parents) 2)
    (error "at most 2 parents, got ~A" (length new-parents)))
  (when (/= (length new-parents) (length (remove-duplicates new-parents)))
    (error "duplicate parent"))
  (dolist (p new-parents)
    (when (= p child-id) (error "a person cannot be their own parent"))
    (%assert-known ftree p))
  (let ((descs (descendants ftree child-id)))
    (dolist (p new-parents)
      (when (member p descs)
        (error "~A is already a descendant of ~A; would create cycle" p child-id))))
  ;; Detach from old parents' children lists.
  (let ((child (get-person ftree child-id)))
    (dolist (op (person-parents child))
      (setf (gethash op (ftree-children ftree))
            (remove child-id (gethash op (ftree-children ftree)))))
    (setf (person-parents child) (copy-list new-parents))
    (dolist (p new-parents)
      (pushnew child-id (gethash p (ftree-children ftree))))))

(defun %bfs (ftree start successor-fn max-depth)
  "Return a list of ids reachable from START via SUCCESSOR-FN, excluding START.
   MAX-DEPTH may be NIL for unbounded."
  (declare (ignore ftree))
  (let ((seen (make-hash-table :test 'eql))
        (queue (list (cons start 0)))
        (out nil))
    (loop while queue do
      (let* ((pair (pop queue))
             (cur (car pair))
             (depth (cdr pair)))
        (unless (and max-depth (>= depth max-depth))
          (dolist (next (funcall successor-fn cur))
            (unless (gethash next seen)
              (setf (gethash next seen) t)
              (push next out)
              (setf queue (append queue (list (cons next (1+ depth))))))))))
    (nreverse out)))

(defun ancestors (ftree id &optional max-depth)
  (%bfs ftree id
        (lambda (x)
          (let ((p (gethash x (ftree-persons ftree))))
            (if p (person-parents p) nil)))
        max-depth))

(defun descendants (ftree id &optional max-depth)
  (%bfs ftree id
        (lambda (x) (gethash x (ftree-children ftree)))
        max-depth))

(defun common-ancestors (ftree a b)
  (intersection (ancestors ftree a) (ancestors ftree b)))

(defun siblings (ftree id)
  (let ((p (gethash id (ftree-persons ftree)))
        (out nil))
    (when p
      (dolist (par (person-parents p))
        (dolist (k (gethash par (ftree-children ftree)))
          (when (and (/= k id) (not (member k out)))
            (push k out)))))
    (nreverse out)))
