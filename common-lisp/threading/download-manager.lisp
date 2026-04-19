;;;; Download Manager — N concurrent workers sharing one job queue.
;;;;
;;;; Fan-out: one producer adds jobs to a shared queue; N worker threads pull
;;;; jobs off and run them in parallel. Fan-in: each worker pushes its outcome
;;;; to a shared results list; the manager reads the list after all workers
;;;; finish.
;;;;
;;;; The queue is protected by a single mutex — Common Lisp has no lock-free
;;;; queue in the standard library, so the mutex IS the queue's contract.
;;;; Workers never block each other for long because the critical section
;;;; is tiny: one pop or one push.
;;;;
;;;; SBCL is assumed for threading.

(defpackage #:download-manager
  (:use #:cl)
  (:export #:make-manager
           #:enqueue
           #:cancel
           #:run-with
           #:queued-count
           #:result-count))

(in-package #:download-manager)

(defstruct download
  (url "" :type string)
  (expected-bytes 0 :type integer))

(defstruct result
  (url "" :type string)
  (status :pending :type keyword)       ; :completed :cancelled :failed
  (bytes nil :type (or null integer))
  (error nil :type (or null string)))

(defstruct manager
  (worker-count 1 :type integer)
  (queue nil :type list)                ; we use a list with a mutex
  (queue-lock (sb-thread:make-mutex))
  (results nil :type list)
  (results-lock (sb-thread:make-mutex))
  (cancel-flag nil :type boolean)
  (cancel-lock (sb-thread:make-mutex)))

(defun make-manager-with (worker-count)
  (when (< worker-count 1) (error "worker-count must be >= 1"))
  (make-manager :worker-count worker-count))

(defun enqueue (mgr download)
  (sb-thread:with-mutex ((manager-queue-lock mgr))
    (setf (manager-queue mgr)
          (append (manager-queue mgr) (list download)))))

(defun cancel (mgr)
  (sb-thread:with-mutex ((manager-cancel-lock mgr))
    (setf (manager-cancel-flag mgr) t)))

(defun %cancelled-p (mgr)
  (sb-thread:with-mutex ((manager-cancel-lock mgr))
    (manager-cancel-flag mgr)))

(defun %pop-job (mgr)
  (sb-thread:with-mutex ((manager-queue-lock mgr))
    (let ((job (first (manager-queue mgr))))
      (when job (setf (manager-queue mgr) (rest (manager-queue mgr))))
      job)))

(defun %push-result (mgr r)
  (sb-thread:with-mutex ((manager-results-lock mgr))
    (push r (manager-results mgr))))

(defun queued-count (mgr)
  (sb-thread:with-mutex ((manager-queue-lock mgr))
    (length (manager-queue mgr))))

(defun result-count (mgr)
  (sb-thread:with-mutex ((manager-results-lock mgr))
    (length (manager-results mgr))))

(defun %worker-loop (mgr simulate)
  (loop
    (when (%cancelled-p mgr) (return))
    (let ((d (%pop-job mgr)))
      (unless d (return))
      (let ((r (if (%cancelled-p mgr)
                   (make-result :url (download-url d) :status :cancelled)
                   (handler-case
                       (let ((bytes (funcall simulate d)))
                         (make-result :url (download-url d)
                                      :status :completed
                                      :bytes bytes))
                     (error (c)
                       (make-result :url (download-url d)
                                    :status :failed
                                    :error (princ-to-string c)))))))
        (%push-result mgr r)))))

(defun run-with (mgr simulate)
  "Spawn WORKER-COUNT threads, each running SIMULATE over popped jobs until
   the queue is empty or cancellation is observed. Blocks until all workers
   finish. Returns the results list (completion order, reversed from push)."
  (let ((threads (loop repeat (manager-worker-count mgr)
                       collect (sb-thread:make-thread
                                (lambda () (%worker-loop mgr simulate))))))
    (dolist (th threads) (sb-thread:join-thread th))
    (sb-thread:with-mutex ((manager-results-lock mgr))
      (nreverse (copy-list (manager-results mgr))))))
