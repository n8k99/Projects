;;;; Port Scanner — probe a host's ports to discover listening services.

(defpackage #:port-scanner
  (:use #:cl)
  (:export #:make-port-scanner
           #:scan-port
           #:scan-range
           #:scan-common
           #:scan-ports
           #:compare-scans
           #:lookup-service
           #:scan-report-summary
           #:port-result-port
           #:port-result-status
           #:port-result-service
           #:scan-report-host
           #:scan-report-open-ports
           #:scan-report-closed-ports
           #:scan-report-filtered-ports
           #:scan-report-ports-scanned))

(in-package #:port-scanner)

;;; --- Well-known service names ---

(defparameter *well-known-ports*
  '((20 . "ftp-data") (21 . "ftp") (22 . "ssh") (23 . "telnet") (25 . "smtp")
    (53 . "dns") (80 . "http") (110 . "pop3") (123 . "ntp") (135 . "msrpc")
    (139 . "netbios-ssn") (143 . "imap") (443 . "https") (445 . "microsoft-ds")
    (465 . "smtps") (587 . "submission") (993 . "imaps") (995 . "pop3s")
    (1433 . "mssql") (1521 . "oracle") (3306 . "mysql") (3389 . "ms-wbt-server")
    (5432 . "postgresql") (5433 . "postgresql-alt") (5672 . "amqp")
    (5900 . "vnc") (6379 . "redis") (8080 . "http-proxy") (8443 . "https-alt")
    (9200 . "elasticsearch") (27017 . "mongodb")))

(defun lookup-service (port)
  "Return the well-known service name for PORT, or empty string."
  (or (cdr (assoc port *well-known-ports*)) ""))

;;; --- Data Structures ---

(defstruct port-result
  (port 0 :type integer)
  (status :closed)    ; :open :closed :filtered
  (service "" :type string)
  (banner "" :type string)
  (latency-ms 0.0 :type float))

(defstruct scan-report
  (host "" :type string)
  (ports-scanned 0 :type integer)
  (open-ports nil :type list)
  (closed-ports 0 :type integer)
  (filtered-ports 0 :type integer)
  (duration-ms 0.0 :type float))

(defstruct port-scanner
  (timeout-ms 1000 :type integer)
  (max-workers 100 :type integer))

;;; --- Scanning ---
;;; Note: actual TCP connect requires usocket or sb-bsd-sockets.
;;; This module provides the data model, report generation, and a simulated
;;; scan function. Callers with socket access can substitute the probe function.

(defun %simulate-port-probe (host port timeout-ms)
  "Simulated port probe — returns :closed for all ports.
Replace with real socket connect for live scanning."
  (declare (ignore host timeout-ms))
  (values :closed 0.0 ""))

(defun scan-port (scanner host port &key (probe-fn #'%simulate-port-probe))
  "Scan a single port. Returns a port-result."
  (multiple-value-bind (status latency banner)
      (funcall probe-fn host port (port-scanner-timeout-ms scanner))
    (make-port-result
     :port port
     :status status
     :service (if (eq status :open) (lookup-service port) "")
     :banner (or banner "")
     :latency-ms (or latency 0.0))))

(defun scan-ports (scanner host ports &key (probe-fn #'%simulate-port-probe))
  "Scan a list of ports. Returns a scan-report."
  (let ((start (get-internal-real-time))
        (results (mapcar (lambda (p) (scan-port scanner host p :probe-fn probe-fn))
                         ports)))
    (let ((elapsed-ms (* 1000.0 (/ (- (get-internal-real-time) start)
                                    internal-time-units-per-second))))
      (make-scan-report
       :host host
       :ports-scanned (length ports)
       :open-ports (remove-if-not (lambda (r) (eq (port-result-status r) :open)) results)
       :closed-ports (count :closed results :key #'port-result-status)
       :filtered-ports (count :filtered results :key #'port-result-status)
       :duration-ms elapsed-ms))))

(defun scan-range (scanner host start-port end-port &key (probe-fn #'%simulate-port-probe))
  "Scan a contiguous range of ports."
  (scan-ports scanner host
              (loop for p from start-port to end-port collect p)
              :probe-fn probe-fn))

(defun scan-common (scanner host &key (probe-fn #'%simulate-port-probe))
  "Scan well-known ports only."
  (let ((ports (sort (mapcar #'car *well-known-ports*) #'<)))
    (scan-ports scanner host ports :probe-fn probe-fn)))

;;; --- Reporting ---

(defun scan-report-summary (report)
  "Return a human-readable summary string."
  (with-output-to-string (s)
    (format s "Scan report for ~A~%" (scan-report-host report))
    (format s "  Scanned ~D ports in ~,0Fms~%"
            (scan-report-ports-scanned report)
            (scan-report-duration-ms report))
    (format s "  Open: ~D, Closed: ~D, Filtered: ~D~%"
            (length (scan-report-open-ports report))
            (scan-report-closed-ports report)
            (scan-report-filtered-ports report))
    (when (scan-report-open-ports report)
      (format s "  Open ports:~%")
      (dolist (p (scan-report-open-ports report))
        (format s "    ~5D/~A~@[ (~A)~] (~,1Fms)~%"
                (port-result-port p)
                (port-result-status p)
                (unless (string= (port-result-service p) "") (port-result-service p))
                (port-result-latency-ms p))))))

;;; --- Compare ---

(defun compare-scans (before after)
  "Compare two scan reports. Returns alist of :newly-opened :newly-closed :still-open."
  (let ((before-open (mapcar #'port-result-port (scan-report-open-ports before)))
        (after-open (mapcar #'port-result-port (scan-report-open-ports after))))
    (list
     (cons :newly-opened (sort (set-difference after-open before-open) #'<))
     (cons :newly-closed (sort (set-difference before-open after-open) #'<))
     (cons :still-open (sort (intersection before-open after-open) #'<)))))
