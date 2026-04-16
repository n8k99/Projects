;;;; G034 — Get Atomic Time from Internet Clock
;;;; Time synchronization protocol: in-process NTP-style client/server.

(defpackage #:networking.atomic-time
  (:use #:cl)
  (:export #:make-time-server
           #:make-time-server-with-offset
           #:server-get-time
           #:make-time-client
           #:make-time-client-with-drift
           #:client-local-time
           #:client-query-time
           #:client-get-offset
           #:client-sync
           #:client-get-synced-time
           #:client-synced-p))

(in-package #:networking.atomic-time)

;;; --- TimeServer ---

(defstruct (time-server (:constructor %make-time-server))
  "Authoritative time source. Wraps system clock with optional offset."
  (offset-seconds 0.0d0 :type double-float))

(defun make-time-server ()
  "Create a time server using the system clock."
  (%make-time-server))

(defun make-time-server-with-offset (offset)
  "Create a time server whose clock is shifted by OFFSET seconds."
  (%make-time-server :offset-seconds (coerce offset 'double-float)))

(defun system-epoch-time ()
  "Return current time as epoch seconds (double-float).
   Uses universal time offset from 1900 to 1970."
  (coerce (- (get-universal-time) 2208988800) 'double-float))

(defun server-get-time (server)
  "Return authoritative timestamp from SERVER (epoch seconds)."
  (+ (system-epoch-time)
     (time-server-offset-seconds server)))

;;; --- TimeClient ---

(defstruct (time-client (:constructor %make-time-client))
  "NTP-style client that queries a TimeServer and tracks sync state."
  (server nil)
  (local-offset 0.0d0 :type double-float)
  (sync-offset 0.0d0 :type double-float)
  (synced nil :type boolean)
  (samples nil :type list))

(defun make-time-client (server)
  "Create a time client connected to SERVER."
  (%make-time-client :server server))

(defun make-time-client-with-drift (server drift)
  "Create a client whose local clock is off by DRIFT seconds."
  (%make-time-client :server server
                     :local-offset (coerce drift 'double-float)))

(defun client-local-time (client)
  "Return the client's uncorrected local time."
  (+ (system-epoch-time)
     (time-client-local-offset client)))

(defun client-query-time (client)
  "Query the server and return its authoritative timestamp."
  (let* ((t-send (client-local-time client))
         (server-time (server-get-time (time-client-server client)))
         (t-recv (client-local-time client))
         (round-trip (- t-recv t-send))
         (estimated-server-now (+ server-time (/ round-trip 2.0d0)))
         (sample (- estimated-server-now t-recv)))
    (push sample (time-client-samples client))
    server-time))

(defun client-get-offset (client)
  "Return measured offset between local clock and server clock.
   Positive means local clock is behind server (needs to advance)."
  (when (null (time-client-samples client))
    (client-query-time client))
  (let ((samples (time-client-samples client)))
    (/ (reduce #'+ samples) (length samples))))

(defun client-sync (client)
  "Adjust local time estimate by applying the measured offset."
  (setf (time-client-sync-offset client) (client-get-offset client))
  (setf (time-client-synced client) t))

(defun client-get-synced-time (client)
  "Return local time corrected by the synchronization offset."
  (if (time-client-synced client)
      (+ (client-local-time client)
         (time-client-sync-offset client))
      (client-local-time client)))

(defun client-synced-p (client)
  "Return whether the client has been synchronized."
  (time-client-synced client))

;;; --- Demo ---

(defun demo ()
  "Demonstrate atomic time synchronization."
  (format t "=== G034: Atomic Time Synchronization ===~%~%")
  (let* ((server (make-time-server))
         (drift -3.5d0)
         (client (make-time-client-with-drift server drift)))
    (format t "Server time:       ~,3F~%" (server-get-time server))
    (format t "Client local time: ~,3F~%" (client-local-time client))
    (format t "Deliberate drift:  ~,1F s~%~%" drift)
    (format t "Querying server (3 samples)...~%")
    (dotimes (i 3)
      (let ((ts (client-query-time client)))
        (format t "  Sample ~D: server reports ~,3F~%" (1+ i) ts)))
    (let ((offset (client-get-offset client)))
      (format t "~%Measured offset: ~,4F s~%" offset))
    (client-sync client)
    (format t "~%After sync:~%")
    (format t "  Server time:  ~,3F~%" (server-get-time server))
    (format t "  Synced time:  ~,3F~%" (client-get-synced-time client))
    (format t "  Local time:   ~,3F~%" (client-local-time client))
    (let ((error-val (abs (- (client-get-synced-time client)
                             (server-get-time server)))))
      (format t "~%  Sync error: ~,1F ms~%" (* error-val 1000))
      (format t "  Synced: ~A~%" (client-synced-p client)))))
