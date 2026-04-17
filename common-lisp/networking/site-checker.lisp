;;;; Site Checker with Time Scheduling — periodic health monitoring of web endpoints.

(defpackage #:site-checker
  (:use #:cl)
  (:export #:make-site-monitor
           #:add-monitored-site
           #:record-check
           #:site-dashboard
           #:sites-by-status
           #:make-check-config
           #:make-site-check-result
           #:site-history-summary
           #:site-history-uptime-pct
           #:site-history-current-status
           #:site-history-consecutive-failures))

(in-package #:site-checker)

(defstruct check-config
  (url "" :type string)
  (name "" :type string)
  (interval-secs 60 :type integer)
  (timeout-ms 10000 :type integer)
  (expected-status 200 :type integer)
  (max-response-ms 5000.0 :type float))

(defstruct site-check-result
  (url "" :type string)
  (status :unknown)  ; :up :down :degraded :unknown
  (http-status 0 :type integer)
  (response-time-ms 0.0 :type float)
  (error "" :type string)
  (checked-at (get-universal-time) :type integer))

(defstruct site-history
  (config (make-check-config) :type check-config)
  (checks nil :type list)
  (consecutive-failures 0 :type integer))

(defun site-history-current-status (h)
  (if (site-history-checks h)
      (site-check-result-status (car (last (site-history-checks h))))
      :unknown))

(defun site-history-uptime-pct (h)
  (let ((checks (site-history-checks h)))
    (if (null checks) 0.0
        (let ((up (count :up checks :key #'site-check-result-status)))
          (* (/ (float up) (float (length checks))) 100.0)))))

(defun site-history-avg-response (h)
  (let ((times (mapcar #'site-check-result-response-time-ms
                       (remove-if-not (lambda (c) (eq (site-check-result-status c) :up))
                                      (site-history-checks h)))))
    (if times (/ (reduce #'+ times) (float (length times))) 0.0)))

(defun %status-name (s)
  (case s (:up "up") (:down "down") (:degraded "degraded") (t "unknown")))

(defun record-check (history result)
  "Add a check result. Returns status change message or NIL."
  (let ((prev (site-history-current-status history)))
    (if (member (site-check-result-status result) '(:down :degraded))
        (incf (site-history-consecutive-failures history))
        (setf (site-history-consecutive-failures history) 0))
    (setf (site-history-checks history)
          (append (site-history-checks history) (list result)))
    (when (and (not (eq prev (site-check-result-status result)))
               (not (eq prev :unknown)))
      (format nil "~A: ~A -> ~A"
              (check-config-name (site-history-config history))
              (%status-name prev)
              (%status-name (site-check-result-status result))))))

(defun site-history-summary (h)
  (format nil "~A: ~A | uptime ~,1F% | avg ~,0Fms | ~D checks | ~D consecutive failures"
          (check-config-name (site-history-config h))
          (%status-name (site-history-current-status h))
          (site-history-uptime-pct h)
          (site-history-avg-response h)
          (length (site-history-checks h))
          (site-history-consecutive-failures h)))

;;; --- Monitor ---

(defstruct site-monitor
  (histories (make-hash-table :test #'equal) :type hash-table))

(defun add-monitored-site (monitor config)
  (when (string= (check-config-name config) "")
    (setf (check-config-name config) (check-config-url config)))
  (setf (gethash (check-config-url config) (site-monitor-histories monitor))
        (make-site-history :config config)))

(defun sites-by-status (monitor status)
  (let (result)
    (maphash (lambda (url h)
               (when (eq (site-history-current-status h) status)
                 (push url result)))
             (site-monitor-histories monitor))
    (sort result #'string<)))

(defun site-dashboard (monitor)
  (with-output-to-string (s)
    (format s "Site Monitor Dashboard~%")
    (format s "~60,,,'-A~%" "")
    (let (urls)
      (maphash (lambda (url h) (declare (ignore h)) (push url urls))
               (site-monitor-histories monitor))
      (dolist (url (sort urls #'string<))
        (format s "~A~%" (site-history-summary
                          (gethash url (site-monitor-histories monitor))))))))
