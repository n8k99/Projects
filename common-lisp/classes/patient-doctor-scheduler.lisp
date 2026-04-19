;;;; Patient / Doctor Scheduler — dual-resource availability, rescheduling, slot search.

(defpackage #:patient-doctor-scheduler
  (:use #:cl)
  (:export #:make-doctor #:doctor-id #:doctor-name #:doctor-specialty
           #:make-room #:room-id #:room-label
           #:make-patient #:patient-id #:patient-name #:patient-primary-doctor-id
           #:make-appointment
           #:appointment-id #:appointment-patient-id #:appointment-doctor-id
           #:appointment-room-id #:appointment-start #:appointment-end
           #:appointment-reason #:appointment-status
           #:appointment-active-p
           #:make-scheduler
           #:add-doctor #:add-room #:add-patient
           #:doctor-conflicts #:room-conflicts
           #:schedule-appointment
           #:cancel-appointment
           #:complete-appointment
           #:mark-no-show
           #:reschedule-appointment
           #:find-slot
           #:doctor-schedule
           #:room-schedule
           #:patient-history))

(in-package #:patient-doctor-scheduler)

(defstruct doctor
  (id "" :type string)
  (name "" :type string)
  (specialty "" :type string))

(defstruct room
  (id "" :type string)
  (label "" :type string))

(defstruct patient
  (id "" :type string)
  (name "" :type string)
  (primary-doctor-id nil :type (or null string)))

(defstruct appointment
  (id 0 :type integer)
  (patient-id "" :type string)
  (doctor-id "" :type string)
  (room-id "" :type string)
  (start 0 :type integer)
  (end 0 :type integer)
  (reason "" :type string)
  (status :scheduled :type keyword))   ; :scheduled :completed :cancelled :no-show

(defun appointment-active-p (a)
  (eq (appointment-status a) :scheduled))

(defun appointment-overlaps-p (a start end)
  (and (< (appointment-start a) end)
       (< start (appointment-end a))))

(defstruct scheduler
  (doctors (make-hash-table :test 'equal))
  (rooms (make-hash-table :test 'equal))
  (patients (make-hash-table :test 'equal))
  (appointments (make-hash-table :test 'eql))
  (next-id 1 :type integer))

(defun add-doctor (sch d)
  (when (gethash (doctor-id d) (scheduler-doctors sch))
    (error "duplicate doctor: ~A" (doctor-id d)))
  (setf (gethash (doctor-id d) (scheduler-doctors sch)) d)
  d)

(defun add-room (sch r)
  (when (gethash (room-id r) (scheduler-rooms sch))
    (error "duplicate room: ~A" (room-id r)))
  (setf (gethash (room-id r) (scheduler-rooms sch)) r)
  r)

(defun add-patient (sch p)
  (when (gethash (patient-id p) (scheduler-patients sch))
    (error "duplicate patient: ~A" (patient-id p)))
  (when (and (patient-primary-doctor-id p)
             (null (gethash (patient-primary-doctor-id p)
                            (scheduler-doctors sch))))
    (error "unknown doctor: ~A" (patient-primary-doctor-id p)))
  (setf (gethash (patient-id p) (scheduler-patients sch)) p)
  p)

(defun doctor-conflicts (sch doctor-id start end &key exclude-id)
  (unless (gethash doctor-id (scheduler-doctors sch))
    (error "unknown doctor: ~A" doctor-id))
  (let ((result nil))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (and (string= (appointment-doctor-id a) doctor-id)
                          (appointment-active-p a)
                          (or (null exclude-id)
                              (not (eql (appointment-id a) exclude-id)))
                          (appointment-overlaps-p a start end))
                 (push a result)))
             (scheduler-appointments sch))
    result))

(defun room-conflicts (sch room-id start end &key exclude-id)
  (unless (gethash room-id (scheduler-rooms sch))
    (error "unknown room: ~A" room-id))
  (let ((result nil))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (and (string= (appointment-room-id a) room-id)
                          (appointment-active-p a)
                          (or (null exclude-id)
                              (not (eql (appointment-id a) exclude-id)))
                          (appointment-overlaps-p a start end))
                 (push a result)))
             (scheduler-appointments sch))
    result))

(defun schedule-appointment (sch patient-id doctor-id room-id start end
                             &optional (reason ""))
  (unless (gethash patient-id (scheduler-patients sch))
    (error "unknown patient: ~A" patient-id))
  (unless (< start end)
    (error "start must be before end"))
  (let ((d-conf (doctor-conflicts sch doctor-id start end))
        (r-conf (room-conflicts sch room-id start end)))
    (when d-conf
      (error "doctor ~A busy: ~D conflict(s)" doctor-id (length d-conf)))
    (when r-conf
      (error "room ~A busy: ~D conflict(s)" room-id (length r-conf))))
  (let* ((id (scheduler-next-id sch))
         (a (make-appointment :id id
                              :patient-id patient-id
                              :doctor-id doctor-id
                              :room-id room-id
                              :start start :end end
                              :reason reason
                              :status :scheduled)))
    (setf (gethash id (scheduler-appointments sch)) a)
    (incf (scheduler-next-id sch))
    a))

(defun %transition (sch id to)
  (let ((a (or (gethash id (scheduler-appointments sch))
               (error "unknown appointment: ~A" id))))
    (unless (eq (appointment-status a) :scheduled)
      (error "appointment ~A is ~A" id (appointment-status a)))
    (setf (appointment-status a) to)
    a))

(defun cancel-appointment (sch id) (%transition sch id :cancelled))
(defun complete-appointment (sch id) (%transition sch id :completed))
(defun mark-no-show (sch id) (%transition sch id :no-show))

(defun reschedule-appointment (sch id new-start new-end &key new-room-id)
  (unless (< new-start new-end)
    (error "start must be before end"))
  (let ((a (or (gethash id (scheduler-appointments sch))
               (error "unknown appointment: ~A" id))))
    (unless (eq (appointment-status a) :scheduled)
      (error "appointment ~A is ~A" id (appointment-status a)))
    (let ((target-room (or new-room-id (appointment-room-id a))))
      (unless (gethash target-room (scheduler-rooms sch))
        (error "unknown room: ~A" target-room))
      (let ((d-conf (doctor-conflicts sch (appointment-doctor-id a)
                                      new-start new-end :exclude-id id))
            (r-conf (room-conflicts sch target-room
                                    new-start new-end :exclude-id id)))
        (when d-conf
          (error "doctor ~A busy: ~D conflict(s)"
                 (appointment-doctor-id a) (length d-conf)))
        (when r-conf
          (error "room ~A busy: ~D conflict(s)" target-room (length r-conf))))
      (setf (appointment-start a) new-start
            (appointment-end a) new-end
            (appointment-room-id a) target-room))
    a))

(defparameter *seconds-per-minute* 60)

(defun find-slot (sch doctor-id duration-seconds from until
                  &key (step-seconds (* 15 *seconds-per-minute*)))
  (unless (gethash doctor-id (scheduler-doctors sch))
    (error "unknown doctor: ~A" doctor-id))
  (when (or (<= duration-seconds 0) (>= from until) (<= step-seconds 0))
    (error "invalid interval or step"))
  (let ((t0 from))
    (loop
      (when (> (+ t0 duration-seconds) until)
        (return-from find-slot nil))
      (let ((end (+ t0 duration-seconds)))
        (when (null (doctor-conflicts sch doctor-id t0 end))
          (maphash (lambda (id r)
                     (declare (ignore id))
                     (when (null (room-conflicts sch (room-id r) t0 end))
                       (return-from find-slot (values t0 r))))
                   (scheduler-rooms sch))))
      (incf t0 step-seconds))))

(defun doctor-schedule (sch doctor-id)
  (let ((result nil))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (and (string= (appointment-doctor-id a) doctor-id)
                          (appointment-active-p a))
                 (push a result)))
             (scheduler-appointments sch))
    (sort result #'< :key #'appointment-start)))

(defun room-schedule (sch room-id)
  (let ((result nil))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (and (string= (appointment-room-id a) room-id)
                          (appointment-active-p a))
                 (push a result)))
             (scheduler-appointments sch))
    (sort result #'< :key #'appointment-start)))

(defun patient-history (sch patient-id)
  (let ((result nil))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (string= (appointment-patient-id a) patient-id)
                 (push a result)))
             (scheduler-appointments sch))
    (sort result #'< :key #'appointment-start)))
