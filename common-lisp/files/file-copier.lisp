;;;; File Copy Utility — recursive tree copy with policy + filter + progress events.

(defpackage #:file-copier
  (:use #:cl)
  (:export #:make-fs #:fs-add #:fs-paths #:fc-copy-tree
           #:make-filter #:filter-allow-all
           #:make-item-file #:make-item-dir
           #:item-kind #:item-size
           #:copy-event-kind #:copy-event-source #:copy-event-dest #:copy-event-bytes
           #:total-size-copied))

(in-package #:file-copier)

(defstruct item
  (kind :file :type keyword)
  (size 0 :type integer))

(defun make-item-file (size) (make-item :kind :file :size size))
(defun make-item-dir () (make-item :kind :dir))

(defstruct fs
  (items (make-hash-table :test 'equal) :type hash-table))

(defun fs-add (fs path item)
  (setf (gethash path (fs-items fs)) item))

(defun fs-paths (fs)
  (sort (loop for k being the hash-keys of (fs-items fs) collect k)
        #'string<))

(defstruct filter
  (include nil :type list)
  (exclude nil :type list))

(defun filter-allow-all () (make-filter))

(defun filter-accepts (f path)
  (and (or (null (filter-include f))
           (some (lambda (pat) (search pat path)) (filter-include f)))
       (not (some (lambda (pat) (search pat path)) (filter-exclude f)))))

(defstruct copy-event
  (kind :copied :type keyword)
  (source "" :type string)
  (dest "" :type string)
  (bytes 0 :type integer))

(defun next-available-name (items base)
  (let ((n 1))
    (loop
      (let ((candidate (format nil "~A.~D" base n)))
        (unless (gethash candidate items)
          (return candidate))
        (incf n)))))

(defun starts-with (str prefix)
  (and (>= (length str) (length prefix))
       (string= str prefix :end1 (length prefix))))

(defun fc-copy-tree (fs src-root dest-root policy filter)
  (let ((src-paths nil))
    (maphash (lambda (p v) (declare (ignore v))
               (when (or (string= p src-root)
                          (starts-with p (concatenate 'string src-root "/")))
                 (push p src-paths)))
             (fs-items fs))
    (when (null src-paths) (error "source not found: ~A" src-root))
    (setf src-paths (sort src-paths #'string<))
    (let ((events nil))
      (dolist (src-path src-paths)
        (let ((item (gethash src-path (fs-items fs))))
          (when (filter-accepts filter src-path)
            (block process-one
              (let* ((rel (if (string= src-path src-root)
                              ""
                              (subseq src-path (1+ (length src-root)))))
                     (dest-path (if (string= rel "")
                                    dest-root
                                    (concatenate 'string dest-root "/" rel)))
                     (final-dest dest-path))
                (when (gethash dest-path (fs-items fs))
                  (cond
                    ((eq policy :skip)
                     (push (make-copy-event :kind :skipped :source src-path :dest dest-path) events)
                     (return-from process-one))
                    ((eq policy :rename-with-suffix)
                     (let ((renamed (next-available-name (fs-items fs) dest-path)))
                       (push (make-copy-event :kind :renamed :source src-path :dest renamed) events)
                       (setf final-dest renamed)))))
                (if (eq (item-kind item) :dir)
                    (progn
                      (setf (gethash final-dest (fs-items fs)) (make-item-dir))
                      (push (make-copy-event :kind :created-dir :source src-path
                                              :dest final-dest) events))
                    (progn
                      (setf (gethash final-dest (fs-items fs))
                            (make-item-file (item-size item)))
                      (push (make-copy-event :kind :copied :source src-path
                                              :dest final-dest
                                              :bytes (item-size item)) events))))))))
      (nreverse events))))

(defun total-size-copied (events)
  (loop for e in events
        when (eq (copy-event-kind e) :copied)
        sum (copy-event-bytes e)))

(defun demo ()
  (let ((fs (make-fs)))
    (fs-add fs "src" (make-item-dir))
    (fs-add fs "src/a.txt" (make-item-file 100))
    (fs-add fs "src/b.txt" (make-item-file 200))
    (fs-add fs "src/sub" (make-item-dir))
    (fs-add fs "src/sub/c.txt" (make-item-file 50))
    (let ((events (fc-copy-tree fs "src" "dest" :overwrite (filter-allow-all))))
      (format t "=== copy: src -> dest ===~%")
      (dolist (e events)
        (format t "  ~12A ~20A -> ~20A (~DB)~%"
                (copy-event-kind e)
                (copy-event-source e)
                (copy-event-dest e)
                (copy-event-bytes e)))
      (format t "~%total bytes copied: ~D~%" (total-size-copied events))
      (format t "all paths: ~A~%" (fs-paths fs)))))
