;;;; Scheduled Auto Login and Action — cron-like task scheduler.

(defpackage #:scheduler
  (:use #:cl)
  (:export #:make-scheduler #:make-schedule-every-ms #:make-schedule-at-ms
           #:make-schedule-daily
           #:add-task #:remove-task #:set-enabled
           #:scheduler-tasks #:get-task #:history #:history-for
           #:next-fires #:tick
           #:run-task-id #:run-status #:run-message
           #:task-id #:task-name #:task-enabled #:task-credential-ref))

(in-package #:scheduler)

(defconstant +day-ms+ 86400000)

(defstruct schedule
  (kind :every-ms :type keyword)
  (interval-ms 0 :type integer)
  (at-ms 0 :type integer)
  (hour 0 :type integer)
  (minute 0 :type integer))

(defun make-schedule-every-ms (ms) (make-schedule :kind :every-ms :interval-ms ms))
(defun make-schedule-at-ms (t0) (make-schedule :kind :at-ms :at-ms t0))
(defun make-schedule-daily (h m) (make-schedule :kind :daily :hour h :minute m))

(defstruct task
  (id 0 :type integer)
  (name "" :type string)
  (schedule (make-schedule) :type schedule)
  (action "" :type string)
  (credential-ref nil :type (or null string))
  (next-fire-ms 0 :type integer)
  (enabled t :type boolean))

(defstruct run
  (task-id 0 :type integer)
  (started-ms 0 :type integer)
  (finished-ms 0 :type integer)
  (status :succeeded :type keyword)
  (message "" :type string))

(defstruct scheduler
  (tasks nil :type list)
  (history nil :type list)
  (next-id 1 :type integer))

(defun %next-daily (base-ms hour minute)
  (let* ((day (floor base-ms +day-ms+))
         (day-start (* day +day-ms+))
         (target-in-day (+ (* hour 3600000) (* minute 60000)))
         (candidate (+ day-start target-in-day)))
    (if (> candidate base-ms) candidate (+ candidate +day-ms+))))

(defun %compute-initial-fire (sched base-ms)
  (ecase (schedule-kind sched)
    (:every-ms (+ base-ms (schedule-interval-ms sched)))
    (:at-ms (schedule-at-ms sched))
    (:daily (%next-daily base-ms (schedule-hour sched) (schedule-minute sched)))))

(defun %schedule-next (sched prev-fire-ms now-ms)
  (ecase (schedule-kind sched)
    (:every-ms
     (let ((nxt (+ prev-fire-ms (schedule-interval-ms sched))))
       (loop while (<= nxt now-ms)
             do (incf nxt (schedule-interval-ms sched)))
       (values nxt nil)))
    (:at-ms (values most-positive-fixnum t))
    (:daily (values (%next-daily now-ms (schedule-hour sched) (schedule-minute sched)) nil))))

(defun add-task (s name sched action credential-ref first-fire-base-ms)
  (let ((id (scheduler-next-id s)))
    (incf (scheduler-next-id s))
    (setf (scheduler-tasks s)
          (append (scheduler-tasks s)
                  (list (make-task :id id :name name :schedule sched
                                   :action action :credential-ref credential-ref
                                   :next-fire-ms (%compute-initial-fire sched first-fire-base-ms)))))
    id))

(defun remove-task (s id)
  (let ((pos (position id (scheduler-tasks s) :key #'task-id)))
    (when pos
      (setf (scheduler-tasks s)
            (append (subseq (scheduler-tasks s) 0 pos)
                    (subseq (scheduler-tasks s) (1+ pos))))
      t)))

(defun set-enabled (s id enabled)
  (let ((task (find id (scheduler-tasks s) :key #'task-id)))
    (when task (setf (task-enabled task) enabled) t)))

(defun get-task (s id) (find id (scheduler-tasks s) :key #'task-id))
(defun history (s) (scheduler-history s))
(defun history-for (s id)
  (remove-if-not (lambda (r) (= (run-task-id r) id)) (scheduler-history s)))

(defun next-fires (s)
  (mapcar (lambda (t1) (cons (task-id t1) (task-next-fire-ms t1)))
          (scheduler-tasks s)))

(defun tick (s now-ms dispatch-fn)
  "DISPATCH-FN takes a task, returns (values succeeded-p message)."
  (let ((new-runs nil)
        (due (remove-if-not (lambda (t1)
                              (and (task-enabled t1) (<= (task-next-fire-ms t1) now-ms)))
                            (scheduler-tasks s))))
    (dolist (task due)
      (multiple-value-bind (ok msg) (funcall dispatch-fn task)
        (let ((r (make-run :task-id (task-id task)
                           :started-ms now-ms :finished-ms now-ms
                           :status (if ok :succeeded :failed)
                           :message msg)))
          (setf (scheduler-history s)
                (append (scheduler-history s) (list r)))
          (push r new-runs))
        (multiple-value-bind (next disable) (%schedule-next (task-schedule task)
                                                             (task-next-fire-ms task)
                                                             now-ms)
          (setf (task-next-fire-ms task) next)
          (when disable (setf (task-enabled task) nil)))))
    (nreverse new-runs)))
