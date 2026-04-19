;;;; Bulk Thumbnail Creator — CPU-bound fan-out with content-addressed dedup.
;;;;
;;;; G066's pattern was fan-out with isolated per-job results. G068 is fan-out
;;;; with a shared INDEX as the output: the result IS the aggregation. Workers
;;;; compute a content hash and check the index before running the expensive
;;;; thumbnailer — so identical content processed twice produces one thumbnail
;;;; and the second request simply records the existing result.
;;;;
;;;; Reservation pattern: under the index lock, check if the hash is present;
;;;; if not, install a placeholder and release the lock BEFORE running the
;;;; thumbnailer. Concurrent workers with the same hash see the placeholder
;;;; and skip generation. This keeps the critical section tiny while the
;;;; expensive work runs unlocked and in parallel.

(defpackage #:bulk-thumbnail
  (:use #:cl)
  (:export #:make-creator
           #:enqueue
           #:run-with
           #:snapshot-thumbnails
           #:snapshot-mappings
           #:snapshot-generated
           #:snapshot-deduplicated))

(in-package #:bulk-thumbnail)

(defstruct request
  (source-id "" :type string)
  (content #() :type vector))

(defstruct snapshot
  (thumbnails (make-hash-table :test 'eql) :type hash-table)
  (mappings (make-hash-table :test 'equal) :type hash-table)
  (generated 0 :type integer)
  (deduplicated 0 :type integer))

(defstruct creator
  (worker-count 1 :type integer)
  (queue nil :type list)
  (queue-lock (sb-thread:make-mutex))
  (thumbnails (make-hash-table :test 'eql))   ; hash -> thumbnail
  (mappings (make-hash-table :test 'equal))   ; source-id -> hash
  (generated 0 :type integer)
  (deduplicated 0 :type integer)
  (index-lock (sb-thread:make-mutex)))

(defun make-creator-with (worker-count)
  (when (< worker-count 1) (error "worker-count must be >= 1"))
  (make-creator :worker-count worker-count))

(defun enqueue (cr req)
  (sb-thread:with-mutex ((creator-queue-lock cr))
    (setf (creator-queue cr)
          (append (creator-queue cr) (list req)))))

(defun %pop-request (cr)
  (sb-thread:with-mutex ((creator-queue-lock cr))
    (let ((r (first (creator-queue cr))))
      (when r (setf (creator-queue cr) (rest (creator-queue cr))))
      r)))

(defun %claim-slot (cr hash)
  "Under lock, atomically check+reserve. Return T if this worker must generate."
  (sb-thread:with-mutex ((creator-index-lock cr))
    (multiple-value-bind (val present) (gethash hash (creator-thumbnails cr))
      (declare (ignore val))
      (if present
          nil
          (progn
            (setf (gethash hash (creator-thumbnails cr)) "")  ; placeholder
            t)))))

(defun %install-thumbnail (cr hash thumb source-id)
  (sb-thread:with-mutex ((creator-index-lock cr))
    (setf (gethash hash (creator-thumbnails cr)) thumb)
    (incf (creator-generated cr))
    (setf (gethash source-id (creator-mappings cr)) hash)))

(defun %mark-dedup (cr hash source-id)
  (sb-thread:with-mutex ((creator-index-lock cr))
    (incf (creator-deduplicated cr))
    (setf (gethash source-id (creator-mappings cr)) hash)))

(defun %worker (cr hasher thumbnailer)
  (loop
    (let ((req (%pop-request cr)))
      (unless req (return))
      (let ((hash (funcall hasher (request-content req))))
        (if (%claim-slot cr hash)
            (let ((thumb (funcall thumbnailer hash (request-content req))))
              (%install-thumbnail cr hash thumb (request-source-id req)))
            (%mark-dedup cr hash (request-source-id req)))))))

(defun run-with (cr hasher thumbnailer)
  "Spawn WORKER-COUNT threads to drain the queue. HASHER takes content bytes
   and returns an integer key. THUMBNAILER takes (hash content) and returns
   the thumbnail string. Returns a snapshot after all workers finish."
  (let ((threads (loop repeat (creator-worker-count cr)
                       collect (sb-thread:make-thread
                                (lambda () (%worker cr hasher thumbnailer))))))
    (dolist (th threads) (sb-thread:join-thread th))
    (sb-thread:with-mutex ((creator-index-lock cr))
      (make-snapshot
       :thumbnails (copy-hash-table (creator-thumbnails cr))
       :mappings (copy-hash-table (creator-mappings cr))
       :generated (creator-generated cr)
       :deduplicated (creator-deduplicated cr)))))

(defun copy-hash-table (ht)
  (let ((new (make-hash-table :test (hash-table-test ht)
                              :size (hash-table-size ht))))
    (maphash (lambda (k v) (setf (gethash k new) v)) ht)
    new))
