;;;; Event Scheduler and Calendar — intervals, recurrence, overlap detection.

(defpackage #:event-scheduler
  (:use #:cl)
  (:export #:make-event #:make-calendar #:calendar-add
           #:events-between #:find-conflicts
           #:event-id #:event-title #:event-start-ms #:event-end-ms
           #:event-recurrence #:event-series-end-ms
           #:occurrence-event-id #:occurrence-title
           #:occurrence-start-ms #:occurrence-end-ms))

(in-package #:event-scheduler)

(defparameter *ms-per-day* 86400000)

(defstruct event
  (id 0 :type integer)
  (title "" :type string)
  (start-ms 0 :type integer)
  (end-ms 0 :type integer)
  (recurrence :none :type keyword)       ; :none :daily :weekly :monthly
  (series-end-ms nil :type (or null integer)))

(defstruct occurrence
  (event-id 0 :type integer)
  (title "" :type string)
  (start-ms 0 :type integer)
  (end-ms 0 :type integer))

(defstruct calendar
  (events (make-hash-table) :type hash-table))

(defun calendar-add (cal event)
  (setf (gethash (event-id event) (calendar-events cal)) event))

(defun event-overlaps (event other-start other-end)
  (and (< (event-start-ms event) other-end)
       (< other-start (event-end-ms event))))

(defun event-duration (event)
  (- (event-end-ms event) (event-start-ms event)))

(defun recurrence-step (r)
  (case r
    (:daily *ms-per-day*)
    (:weekly (* 7 *ms-per-day*))
    (:monthly (* 30 *ms-per-day*))
    (t 0)))

(defun expand-event (event from-ms to-ms)
  (let ((out nil)
        (duration (event-duration event))
        (cutoff (or (event-series-end-ms event) most-positive-fixnum)))
    (cond
      ((eq (event-recurrence event) :none)
       (when (event-overlaps event from-ms to-ms)
         (push (make-occurrence :event-id (event-id event)
                                 :title (event-title event)
                                 :start-ms (event-start-ms event)
                                 :end-ms (event-end-ms event))
               out)))
      (t
       (let ((step (recurrence-step (event-recurrence event))))
         (when (> step 0)
           (let ((start (event-start-ms event)))
             (when (< start from-ms)
               (let ((skips (floor (max 0 (- (- from-ms start) duration)) step)))
                 (setf start (+ start (* skips step)))))
             (loop while (and (< start to-ms) (<= start cutoff))
                   do (let ((end (+ start duration)))
                        (when (> end from-ms)
                          (push (make-occurrence :event-id (event-id event)
                                                  :title (event-title event)
                                                  :start-ms start
                                                  :end-ms end)
                                out))
                        (setf start (+ start step)))))))))
    (nreverse out)))

(defun events-between (cal from-ms to-ms)
  (let ((all nil))
    (maphash (lambda (id event) (declare (ignore id))
               (dolist (occ (expand-event event from-ms to-ms))
                 (push occ all)))
             (calendar-events cal))
    (sort all (lambda (a b)
                (cond ((< (occurrence-start-ms a) (occurrence-start-ms b)) t)
                      ((> (occurrence-start-ms a) (occurrence-start-ms b)) nil)
                      (t (< (occurrence-event-id a) (occurrence-event-id b))))))))

(defun find-conflicts (cal start-ms end-ms)
  (let ((conflicts nil))
    (maphash (lambda (id event)
               (dolist (occ (expand-event event start-ms end-ms))
                 (when (and (< (occurrence-start-ms occ) end-ms)
                             (< start-ms (occurrence-end-ms occ)))
                   (pushnew id conflicts)
                   (return))))
             (calendar-events cal))
    (sort conflicts #'<)))

(defun demo ()
  (let ((cal (make-calendar)))
    (flet ((days (n) (* n *ms-per-day*)))
      (calendar-add cal (make-event :id 1 :title "Standup"
                                      :start-ms (days 10)
                                      :end-ms (+ (days 10) 1800000)
                                      :recurrence :daily
                                      :series-end-ms (days 30)))
      (calendar-add cal (make-event :id 2 :title "Weekly Review"
                                      :start-ms (+ (days 14) (* 15 3600000))
                                      :end-ms (+ (days 14) (* 16 3600000))
                                      :recurrence :weekly))
      (calendar-add cal (make-event :id 3 :title "One-off"
                                      :start-ms (days 12)
                                      :end-ms (+ (days 12) (* 2 3600000))))
      (format t "=== events between day 10 and day 16 ===~%")
      (dolist (o (events-between cal (days 10) (days 16)))
        (format t "  #~D ~A start=~Dd end=~Dd~%"
                (occurrence-event-id o) (occurrence-title o)
                (floor (occurrence-start-ms o) *ms-per-day*)
                (floor (occurrence-end-ms o) *ms-per-day*)))
      (format t "~%=== conflicts for a new event on day 12 midday ===~%")
      (format t "  ~A~%"
              (find-conflicts cal (days 12) (+ (days 12) (* 12 3600000)))))))
