;;;; File Downloader — resumable transfer with atomic file creation.
;;;;
;;;; Bytes flow to `<dest>.partial` during transfer. On success, RENAME
;;;; promotes the file in one operation. On interruption, `.partial` persists;
;;;; the next call resumes from its current size. Optional hash verification
;;;; refuses the rename on mismatch and keeps `.partial` for inspection.

(defpackage #:file-downloader
  (:use #:cl)
  (:export #:make-opts #:download
           #:outcome-bytes-written #:outcome-final-path #:outcome-verified
           #:byte-sum-hash
           #:hash-mismatch))

(in-package #:file-downloader)

(define-condition hash-mismatch (error)
  ((expected :initarg :expected :reader hash-mismatch-expected)
   (actual :initarg :actual :reader hash-mismatch-actual))
  (:report (lambda (c s)
             (format s "hash mismatch: expected ~X, got ~X"
                     (hash-mismatch-expected c) (hash-mismatch-actual c)))))

(defstruct opts
  (resume t :type boolean)
  (chunk-size 8192 :type integer)
  (expected-hash nil :type (or null integer)))

(defstruct outcome
  (bytes-written 0 :type integer)
  (final-path "" :type string)
  (verified nil :type boolean))

(defun byte-sum-hash (bytes)
  "Byte-sum hash matching the streamable version's finalize."
  (let ((h 0))
    (loop for b across bytes do
      (setf h (logand (+ (* h 31) b) #xFFFFFFFFFFFFFFFF)))
    (logand (+ h (length bytes)) #xFFFFFFFFFFFFFFFF)))

(defun %partial-path (dest) (concatenate 'string dest ".partial"))

(defun %file-bytes (path)
  (with-open-file (s path :direction :input :element-type '(unsigned-byte 8))
    (let ((buf (make-array (file-length s) :element-type '(unsigned-byte 8))))
      (read-sequence buf s)
      buf)))

(defun download (read-from dest opts &optional (on-progress (lambda (n) (declare (ignore n)))))
  "READ-FROM is a function of (offset size) returning a byte vector (empty = EOF).
   Errors from READ-FROM leave .partial intact."
  (let* ((partial (%partial-path dest))
         (start-offset 0)
         (hash-state 0)
         (bytes-hashed 0))
    (when (and (opts-resume opts) (probe-file partial))
      (let ((existing (%file-bytes partial)))
        (setf start-offset (length existing))
        (setf bytes-hashed start-offset)
        (loop for b across existing do
          (setf hash-state (logand (+ (* hash-state 31) b) #xFFFFFFFFFFFFFFFF)))))
    (let ((direction (if (and (opts-resume opts) (probe-file partial)) :append :output)))
      (with-open-file (out partial :direction :output
                                   :element-type '(unsigned-byte 8)
                                   :if-exists direction
                                   :if-does-not-exist :create)
        (let ((total start-offset))
          (loop
            (let ((chunk (funcall read-from total (opts-chunk-size opts))))
              (when (zerop (length chunk)) (return))
              (write-sequence chunk out)
              (incf bytes-hashed (length chunk))
              (loop for b across chunk do
                (setf hash-state (logand (+ (* hash-state 31) b) #xFFFFFFFFFFFFFFFF)))
              (incf total (length chunk))
              (funcall on-progress total)))
          (let ((final-hash (logand (+ hash-state bytes-hashed) #xFFFFFFFFFFFFFFFF)))
            (finish-output out)
            (when (and (opts-expected-hash opts)
                       (/= (opts-expected-hash opts) final-hash))
              (error 'hash-mismatch
                     :expected (opts-expected-hash opts)
                     :actual final-hash))))))
    (rename-file partial dest)
    (make-outcome :bytes-written (file-length (open dest))
                  :final-path dest
                  :verified (not (null (opts-expected-hash opts))))))
