;;;; Screen Saver — idle FSM + bouncing-line physics + dim ramp + activity reset.

(defpackage #:screen-saver
  (:use #:cl)
  (:export #:make-config #:make-endpoint #:make-bouncing-line #:make-ss-rgb
           #:make-screen-saver #:saver-add-line #:saver-tick
           #:saver-record-activity #:saver-unlock #:saver-dim-opacity
           #:saver-state #:saver-lines #:saver-elapsed-ms #:saver-events
           #:endpoint-x #:endpoint-y #:endpoint-vx #:endpoint-vy
           #:bouncing-line-p1 #:bouncing-line-p2))

(in-package #:screen-saver)

(defstruct config
  (idle-ms 60000 :type integer)
  (dim-ms 3000 :type integer)
  (auto-lock-ms 30000 :type integer)
  (wake-ms 500 :type integer)
  (canvas-width 1920 :type integer)
  (canvas-height 1080 :type integer))

(defstruct endpoint
  (x 0 :type integer)
  (y 0 :type integer)
  (vx 0 :type integer)
  (vy 0 :type integer))

(defstruct ss-rgb
  (r 0 :type integer)
  (g 0 :type integer)
  (b 0 :type integer))

(defstruct bouncing-line
  (p1 nil :type (or null endpoint))
  (p2 nil :type (or null endpoint))
  (color nil :type (or null ss-rgb)))

(defstruct saver-event
  (kind :state-changed :type keyword)
  (at-ms 0 :type integer)
  (detail "" :type string))

(defstruct screen-saver
  (config nil :type (or null config))
  (state :active :type keyword)
  (lines nil :type list)
  (elapsed-ms 0 :type integer)
  (last-activity-ms 0 :type integer)
  (state-entered-ms 0 :type integer)
  (events nil :type list))

(defun make-new-saver (config)
  (make-screen-saver :config config))

(defun saver-add-line (s line)
  (setf (screen-saver-lines s) (append (screen-saver-lines s) (list line))))

(defun saver-state (s) (screen-saver-state s))
(defun saver-lines (s) (screen-saver-lines s))
(defun saver-elapsed-ms (s) (screen-saver-elapsed-ms s))
(defun saver-events (s) (screen-saver-events s))

(defun saver-dim-opacity (s)
  (if (not (eq (screen-saver-state s) :dimming))
      0
      (let* ((dim-ms (config-dim-ms (screen-saver-config s)))
             (since (max 0 (- (screen-saver-elapsed-ms s)
                                (screen-saver-state-entered-ms s)))))
        (if (zerop dim-ms) 255
            (floor (* (min since dim-ms) 255) dim-ms)))))

(defun transition-to (s new-state)
  (unless (eq (screen-saver-state s) new-state)
    (setf (screen-saver-events s)
          (append (screen-saver-events s)
                  (list (make-saver-event
                         :kind :state-changed
                         :at-ms (screen-saver-elapsed-ms s)
                         :detail (format nil "~A -> ~A"
                                          (screen-saver-state s) new-state)))))
    (setf (screen-saver-state s) new-state)
    (setf (screen-saver-state-entered-ms s) (screen-saver-elapsed-ms s))))

(defun saver-record-activity (s)
  (setf (screen-saver-last-activity-ms s) (screen-saver-elapsed-ms s))
  (setf (screen-saver-events s)
        (append (screen-saver-events s)
                (list (make-saver-event
                       :kind :activity-reset-timer
                       :at-ms (screen-saver-elapsed-ms s)))))
  (when (member (screen-saver-state s) '(:dimming :saver))
    (transition-to s :waking)))

(defun saver-unlock (s)
  (when (eq (screen-saver-state s) :locked)
    (transition-to s :waking)
    (setf (screen-saver-last-activity-ms s) (screen-saver-elapsed-ms s))))

(defun reflect-endpoint (s ep idx ms cw ch)
  (incf (endpoint-x ep) (floor (* (endpoint-vx ep) ms) 100))
  (incf (endpoint-y ep) (floor (* (endpoint-vy ep) ms) 100))
  (when (< (endpoint-x ep) 0)
    (setf (endpoint-x ep) (- (endpoint-x ep)))
    (setf (endpoint-vx ep) (- (endpoint-vx ep)))
    (setf (screen-saver-events s)
          (append (screen-saver-events s)
                  (list (make-saver-event
                         :kind :line-bounced
                         :at-ms (screen-saver-elapsed-ms s)
                         :detail (format nil "line ~D axis x" idx))))))
  (when (> (endpoint-x ep) cw)
    (setf (endpoint-x ep) (- (* 2 cw) (endpoint-x ep)))
    (setf (endpoint-vx ep) (- (endpoint-vx ep)))
    (setf (screen-saver-events s)
          (append (screen-saver-events s)
                  (list (make-saver-event
                         :kind :line-bounced
                         :at-ms (screen-saver-elapsed-ms s)
                         :detail (format nil "line ~D axis x" idx))))))
  (when (< (endpoint-y ep) 0)
    (setf (endpoint-y ep) (- (endpoint-y ep)))
    (setf (endpoint-vy ep) (- (endpoint-vy ep)))
    (setf (screen-saver-events s)
          (append (screen-saver-events s)
                  (list (make-saver-event
                         :kind :line-bounced
                         :at-ms (screen-saver-elapsed-ms s)
                         :detail (format nil "line ~D axis y" idx))))))
  (when (> (endpoint-y ep) ch)
    (setf (endpoint-y ep) (- (* 2 ch) (endpoint-y ep)))
    (setf (endpoint-vy ep) (- (endpoint-vy ep)))
    (setf (screen-saver-events s)
          (append (screen-saver-events s)
                  (list (make-saver-event
                         :kind :line-bounced
                         :at-ms (screen-saver-elapsed-ms s)
                         :detail (format nil "line ~D axis y" idx)))))))

(defun integrate-physics (s ms)
  (let ((cw (* (config-canvas-width (screen-saver-config s)) 100))
        (ch (* (config-canvas-height (screen-saver-config s)) 100))
        (idx 0))
    (dolist (line (screen-saver-lines s))
      (reflect-endpoint s (bouncing-line-p1 line) idx ms cw ch)
      (reflect-endpoint s (bouncing-line-p2 line) idx ms cw ch)
      (incf idx))))

(defun saver-tick (s ms)
  (incf (screen-saver-elapsed-ms s) ms)
  (let ((since-state (- (screen-saver-elapsed-ms s)
                         (screen-saver-state-entered-ms s))))
    (case (screen-saver-state s)
      (:active
       (let ((idle (max 0 (- (screen-saver-elapsed-ms s)
                              (screen-saver-last-activity-ms s)))))
         (when (>= idle (config-idle-ms (screen-saver-config s)))
           (transition-to s :dimming))))
      (:dimming
       (when (>= since-state (config-dim-ms (screen-saver-config s)))
         (transition-to s :saver)))
      (:saver
       (when (>= since-state (config-auto-lock-ms (screen-saver-config s)))
         (transition-to s :locked))
       (integrate-physics s ms))
      (:waking
       (when (>= since-state (config-wake-ms (screen-saver-config s)))
         (transition-to s :active)
         (setf (screen-saver-last-activity-ms s)
               (screen-saver-elapsed-ms s)))))))

(defun demo ()
  (let* ((cfg (make-config :idle-ms 1000 :dim-ms 200
                             :auto-lock-ms 500 :wake-ms 100
                             :canvas-width 100 :canvas-height 100))
         (s (make-new-saver cfg)))
    (saver-add-line s (make-bouncing-line
                        :p1 (make-endpoint :x 9500 :y 5000 :vx 10000 :vy 0)
                        :p2 (make-endpoint :x 5000 :y 5000 :vx 0 :vy 0)
                        :color (make-ss-rgb :r 255 :g 255 :b 255)))

    (format t "initial: state=~A~%" (screen-saver-state s))
    (saver-tick s 500)
    (format t "at 500ms: state=~A~%" (screen-saver-state s))
    (saver-tick s 600)
    (format t "at 1100ms: state=~A dim_opacity=~D~%"
            (screen-saver-state s) (saver-dim-opacity s))
    (saver-tick s 100)
    (format t "at 1200ms: state=~A dim_opacity=~D~%"
            (screen-saver-state s) (saver-dim-opacity s))
    (saver-tick s 100)
    (format t "at 1300ms: state=~A~%" (screen-saver-state s))

    (saver-tick s 10)
    (let ((line (first (screen-saver-lines s))))
      (format t "at 1310ms: state=~A p1.x=~D p1.vx=~D~%"
              (screen-saver-state s)
              (endpoint-x (bouncing-line-p1 line))
              (endpoint-vx (bouncing-line-p1 line))))

    (saver-tick s 600)
    (format t "at 1910ms: state=~A~%" (screen-saver-state s))

    (saver-record-activity s)
    (format t "after activity: state=~A~%" (screen-saver-state s))
    (saver-unlock s)
    (format t "after unlock: state=~A~%" (screen-saver-state s))
    (saver-tick s 150)
    (format t "after 150ms in Waking: state=~A~%" (screen-saver-state s))))
