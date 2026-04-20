;;;; Versioning Manager — content-addressed version store, minimal Git-like.

(defpackage #:versioning-manager
  (:use #:cl)
  (:export #:fnv1a-hash #:make-repo #:repo-commit #:repo-checkout
           #:repo-diff #:repo-log #:repo-head #:repo-blob-count #:repo-commit-count
           #:commit-vm-tree-hash #:commit-parent #:commit-message #:commit-timestamp
           #:diff-op-kind #:diff-op-name))

(in-package #:versioning-manager)

(defun fnv1a-hash (bytes)
  (let ((h #xcbf29ce484222325))
    (loop for b across bytes do
          (setf h (logxor h b))
          (setf h (logand (* h #x100000001b3) #xFFFFFFFFFFFFFFFF)))
    h))

(defun string-to-bytes (s)
  (map '(simple-array (unsigned-byte 8) (*)) #'char-code s))

(defstruct blob
  (data #() :type (simple-array (unsigned-byte 8) (*))))

(defstruct vm-tree
  (entries nil :type list))  ; alist of (name . hash), sorted by name

(defstruct commit
  (vm-tree-hash 0 :type integer)
  (parent nil :type (or null integer))
  (message "" :type string)
  (timestamp 0 :type integer))

(defstruct diff-op
  (kind :added :type keyword)
  (name "" :type string)
  (from-hash 0 :type integer)
  (to-hash 0 :type integer))

(defstruct repo
  (blobs (make-hash-table) :type hash-table)
  (vm-trees (make-hash-table) :type hash-table)
  (commits (make-hash-table) :type hash-table)
  (head nil :type (or null integer)))

(defun put-blob (r data)
  (let* ((bytes (if (stringp data) (string-to-bytes data) data))
         (h (fnv1a-hash bytes)))
    (unless (gethash h (repo-blobs r))
      (setf (gethash h (repo-blobs r)) (make-blob :data bytes)))
    h))

(defun canonical-vm-tree-bytes (vm-tree)
  (with-output-to-string (out)
    (format out "tree~%")
    (dolist (entry (vm-tree-entries vm-tree))
      (format out "~A	~(~16,'0X~)~%" (car entry) (cdr entry)))))

(defun put-vm-tree (r entries)
  (let* ((sorted-entries (sort (copy-list entries)
                                (lambda (a b) (string< (car a) (car b)))))
         (vm-tree (make-vm-tree :entries sorted-entries))
         (canonical (string-to-bytes (canonical-vm-tree-bytes vm-tree)))
         (h (fnv1a-hash canonical)))
    (unless (gethash h (repo-vm-trees r))
      (setf (gethash h (repo-vm-trees r)) vm-tree))
    h))

(defun canonical-commit-bytes (c)
  (with-output-to-string (out)
    (format out "commit~%")
    (format out "tree ~(~16,'0X~)~%" (commit-vm-tree-hash c))
    (when (commit-parent c)
      (format out "parent ~(~16,'0X~)~%" (commit-parent c)))
    (format out "ts ~D~%" (commit-timestamp c))
    (format out "msg ~A~%" (commit-message c))))

(defun repo-commit (r files message timestamp)
  (let* ((entries (mapcar (lambda (pair)
                             (cons (first pair) (put-blob r (second pair))))
                           files))
         (vm-tree-hash (put-vm-tree r entries))
         (c (make-commit :vm-tree-hash vm-tree-hash :parent (repo-head r)
                          :message message :timestamp timestamp))
         (canonical (string-to-bytes (canonical-commit-bytes c)))
         (h (fnv1a-hash canonical)))
    (setf (gethash h (repo-commits r)) c)
    (setf (repo-head r) h)
    h))

(defun repo-checkout (r commit-hash)
  (let ((c (gethash commit-hash (repo-commits r))))
    (unless c (return-from repo-checkout nil))
    (let ((vm-tree (gethash (commit-vm-tree-hash c) (repo-vm-trees r)))
          (result nil))
      (unless vm-tree (return-from repo-checkout nil))
      (dolist (entry (vm-tree-entries vm-tree))
        (let ((blob (gethash (cdr entry) (repo-blobs r))))
          (unless blob (return-from repo-checkout nil))
          (push (cons (car entry) (blob-data blob)) result)))
      (nreverse result))))

(defun op-key (op)
  (concatenate 'string
               (case (diff-op-kind op)
                 (:added "A")
                 (:removed "R")
                 (:changed "C"))
               (diff-op-name op)))

(defun repo-diff (r from-hash to-hash)
  (let* ((from-c (gethash from-hash (repo-commits r)))
         (to-c (gethash to-hash (repo-commits r))))
    (unless (and from-c to-c) (return-from repo-diff nil))
    (let* ((from-vm-tree (gethash (commit-vm-tree-hash from-c) (repo-vm-trees r)))
           (to-vm-tree (gethash (commit-vm-tree-hash to-c) (repo-vm-trees r)))
           (ops nil))
      (unless (and from-vm-tree to-vm-tree) (return-from repo-diff nil))
      ;; added or changed
      (dolist (entry (vm-tree-entries to-vm-tree))
        (let ((from-entry (assoc (car entry) (vm-tree-entries from-vm-tree) :test #'string=)))
          (cond
            ((null from-entry)
             (push (make-diff-op :kind :added :name (car entry) :to-hash (cdr entry)) ops))
            ((/= (cdr from-entry) (cdr entry))
             (push (make-diff-op :kind :changed :name (car entry)
                                  :from-hash (cdr from-entry) :to-hash (cdr entry)) ops)))))
      ;; removed
      (dolist (entry (vm-tree-entries from-vm-tree))
        (unless (assoc (car entry) (vm-tree-entries to-vm-tree) :test #'string=)
          (push (make-diff-op :kind :removed :name (car entry)
                               :from-hash (cdr entry)) ops)))
      (sort ops (lambda (a b) (string< (op-key a) (op-key b)))))))

(defun repo-log (r)
  (let ((out nil)
        (cursor (repo-head r)))
    (loop while cursor do
          (let ((c (gethash cursor (repo-commits r))))
            (unless c (return))
            (push (cons cursor c) out)
            (setf cursor (commit-parent c))))
    (nreverse out)))

(defun repo-blob-count (r) (hash-table-count (repo-blobs r)))
(defun repo-commit-count (r) (hash-table-count (repo-commits r)))

(defun demo ()
  (let* ((r (make-repo))
         (c1 (repo-commit r '(("a.txt" "hello") ("b.txt" "world")) "initial" 1000))
         (c2 (repo-commit r '(("a.txt" "hello!") ("b.txt" "world") ("c.txt" "new"))
                           "update a, add c" 2000))
         (c3 (repo-commit r '(("a.txt" "hello!") ("c.txt" "new")) "remove b" 3000)))
    (declare (ignore c3))
    (format t "=== log ===~%")
    (dolist (entry (repo-log r))
      (let ((h (car entry)) (c (cdr entry)))
        (format t "  ~16,'0X vm-tree=~16,'0X"
                h (commit-vm-tree-hash c))
        (when (commit-parent c)
          (format t " <- ~16,'0X" (commit-parent c)))
        (format t "  ~A~%" (commit-message c))))
    (format t "~%=== diff c1 -> c2 ===~%")
    (dolist (op (repo-diff r c1 c2))
      (format t "  ~7A ~A~%" (diff-op-kind op) (diff-op-name op)))
    (format t "~%=== storage ===~%")
    (format t "  blobs: ~D  commits: ~D~%"
            (repo-blob-count r) (repo-commit-count r))
    (format t "  HEAD: ~16,'0X~%" (repo-head r))))
