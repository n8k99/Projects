;;;; Progress Bar for Downloads — the first concurrent producer-consumer.
;;;;
;;;; A producer thread advances a byte counter; a consumer thread reads it to
;;;; render a display. A lock protects the compound transitions (bytes and
;;;; status together); simple advance is serialised on the same lock.
;;;;
;;;; Cancellation is cooperative: the controller calls CANCEL, setting the
;;;; status to :CANCELLED. The worker checks IS-CANCELLED between chunks
;;;; and exits. Completion, cancellation, and failure are STATES — a small
;;;; FSM observed from another thread — not boolean flags.
;;;;
;;;; SBCL is assumed for threading. The same code compiles under any CL
;;;; with bordeaux-threads aliased to sb-thread, but we keep the deps minimal.

(defpackage #:progress-bar
  (:use #:cl)
  (:export #:make-tracker
           #:advance
           #:cancel
           #:complete
           #:fail
           #:is-cancelled
           #:snapshot
           #:snapshot-bytes-done
           #:snapshot-total-bytes
           #:snapshot-status
           #:snapshot-elapsed-seconds
           #:snapshot-percent
           #:snapshot-bytes-per-second
           #:snapshot-eta-seconds
           #:render-bar))

(in-package #:progress-bar)

(defstruct tracker
  (lock (sb-thread:make-mutex))
  (bytes-done 0 :type integer)
  (total-bytes 0 :type integer)
  (status :running :type keyword)      ; :running :completed :cancelled :failed
  (start-time (get-internal-real-time) :type integer))

(defstruct snapshot
  (bytes-done 0 :type integer)
  (total-bytes 0 :type integer)
  (status :running :type keyword)
  (elapsed-seconds 0.0 :type real))

(defun advance (tracker delta)
  (sb-thread:with-mutex ((tracker-lock tracker))
    (incf (tracker-bytes-done tracker) delta)))

(defun %maybe-terminal (tracker new-status)
  "Only transition out of :RUNNING; terminal states are sticky."
  (sb-thread:with-mutex ((tracker-lock tracker))
    (when (eq (tracker-status tracker) :running)
      (setf (tracker-status tracker) new-status))))

(defun cancel   (tracker) (%maybe-terminal tracker :cancelled))
(defun complete (tracker) (%maybe-terminal tracker :completed))
(defun fail     (tracker) (%maybe-terminal tracker :failed))

(defun is-cancelled (tracker)
  (sb-thread:with-mutex ((tracker-lock tracker))
    (eq (tracker-status tracker) :cancelled)))

(defun snapshot (tracker)
  (sb-thread:with-mutex ((tracker-lock tracker))
    (make-snapshot
     :bytes-done (tracker-bytes-done tracker)
     :total-bytes (tracker-total-bytes tracker)
     :status (tracker-status tracker)
     :elapsed-seconds (float
                       (/ (- (get-internal-real-time) (tracker-start-time tracker))
                          internal-time-units-per-second)))))

(defun snapshot-percent (s)
  (if (zerop (snapshot-total-bytes s)) 100.0
      (* 100.0 (/ (snapshot-bytes-done s) (snapshot-total-bytes s)))))

(defun snapshot-bytes-per-second (s)
  (if (<= (snapshot-elapsed-seconds s) 0.0) 0.0
      (/ (snapshot-bytes-done s) (snapshot-elapsed-seconds s))))

(defun snapshot-eta-seconds (s)
  (let ((rate (snapshot-bytes-per-second s)))
    (if (or (<= rate 0.0)
            (>= (snapshot-bytes-done s) (snapshot-total-bytes s)))
        nil
        (/ (- (snapshot-total-bytes s) (snapshot-bytes-done s)) rate))))

(defun render-bar (snapshot width)
  (let* ((pct (snapshot-percent snapshot))
         (filled (min width (round (* (/ pct 100.0) width))))
         (empty (- width filled)))
    (with-output-to-string (s)
      (format s "[~A~A] ~6,2F%"
              (make-string filled :initial-element #\#)
              (make-string empty :initial-element #\-)
              pct))))
