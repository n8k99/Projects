;;;; Alarm Clock — schedule and trigger time-based alerts.

(defpackage :numbers/alarm-clock
  (:use :cl)
  (:export :make-alarm-clock
           :set-alarm
           :check-alarms
           :pending-alarms
           :cancel-alarm))

(in-package :numbers/alarm-clock)

;;; An alarm is a plist: (:hour H :minute M :second S :label L)

(defun make-alarm-clock ()
  "Create a new alarm clock (a list that holds alarms)."
  (list nil))

(defun alarms-of (clock)
  "Access the alarm list inside the clock container."
  (car clock))

(defun (setf alarms-of) (new-val clock)
  (setf (car clock) new-val))

(defun time-to-seconds (h m s)
  (+ (* h 3600) (* m 60) s))

(defun set-alarm (clock hour minute &key (second 0) (label ""))
  "Schedule an alarm for the given time with an optional label.
Returns the alarm plist that was added."
  (unless (and (<= 0 hour 23) (<= 0 minute 59) (<= 0 second 59))
    (error "Invalid time: ~2,'0D:~2,'0D:~2,'0D" hour minute second))
  (let ((alarm (list :hour hour :minute minute :second second :label label)))
    (push alarm (alarms-of clock))
    alarm))

(defun check-alarms (clock now-h now-m now-s)
  "Check all pending alarms against the given time (H, M, S).
Returns a list of triggered alarms. Triggered alarms are removed from pending."
  (let ((now-secs (time-to-seconds now-h now-m now-s))
        (triggered nil)
        (remaining nil))
    (dolist (alarm (alarms-of clock))
      (let ((alarm-secs (time-to-seconds (getf alarm :hour)
                                          (getf alarm :minute)
                                          (getf alarm :second))))
        (if (<= alarm-secs now-secs)
            (push alarm triggered)
            (push alarm remaining))))
    (setf (alarms-of clock) (nreverse remaining))
    (nreverse triggered)))

(defun pending-alarms (clock)
  "Return a list of all alarms that have not yet triggered."
  (copy-list (alarms-of clock)))

(defun cancel-alarm (clock label)
  "Cancel the first pending alarm with the given label.
Returns T if an alarm was cancelled, NIL otherwise."
  (let ((alarms (alarms-of clock)))
    (loop for prev = nil then cell
          for cell on alarms
          when (string= (getf (car cell) :label) label)
          do (if prev
                 (setf (cdr prev) (cdr cell))
                 (setf (alarms-of clock) (cdr cell)))
             (return t)
          finally (return nil))))
