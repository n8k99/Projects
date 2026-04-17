;;;; Packet Sniffer — capture and analyze network packets with filtering and statistics.

(defpackage #:packet-sniffer
  (:use #:cl)
  (:export #:make-packet
           #:make-capture-filter
           #:make-packet-capture
           #:ingest
           #:query-packets
           #:total-packets
           #:total-bytes
           #:protocol-breakdown
           #:top-talkers
           #:unique-ips
           #:capture-summary
           #:detect-syn-flood
           #:detect-port-scan
           #:packet-src-ip
           #:packet-dst-ip
           #:packet-protocol
           #:packet-size
           #:packet-flags))

(in-package #:packet-sniffer)

;;; --- Data Structures ---

(defstruct packet
  (timestamp-ms 0 :type integer)
  (src-ip "" :type string)
  (dst-ip "" :type string)
  (src-port 0 :type integer)
  (dst-port 0 :type integer)
  (protocol :unknown)   ; :tcp :udp :icmp :arp :dns :http :unknown
  (size 0 :type integer)
  (flags nil :type list)
  (payload nil)
  (ttl 64 :type integer))

(defstruct capture-filter
  (src-ip "" :type string)
  (dst-ip "" :type string)
  (protocol nil)
  (port nil)
  (min-size 0 :type integer)
  (max-size 0 :type integer))

(defun filter-matches (f pkt)
  "Test if PKT matches filter F."
  (and (or (string= (capture-filter-src-ip f) "")
           (string= (capture-filter-src-ip f) (packet-src-ip pkt)))
       (or (string= (capture-filter-dst-ip f) "")
           (string= (capture-filter-dst-ip f) (packet-dst-ip pkt)))
       (or (null (capture-filter-protocol f))
           (eq (capture-filter-protocol f) (packet-protocol pkt)))
       (or (null (capture-filter-port f))
           (= (capture-filter-port f) (packet-src-port pkt))
           (= (capture-filter-port f) (packet-dst-port pkt)))
       (or (zerop (capture-filter-min-size f))
           (>= (packet-size pkt) (capture-filter-min-size f)))
       (or (zerop (capture-filter-max-size f))
           (<= (packet-size pkt) (capture-filter-max-size f)))))

;;; --- Connection tracking ---

(defun %conn-key (pkt)
  "Bidirectional connection key as a list."
  (let ((a (list (packet-src-ip pkt) (packet-src-port pkt)))
        (b (list (packet-dst-ip pkt) (packet-dst-port pkt))))
    (if (string<= (format nil "~A:~D" (first a) (second a))
                  (format nil "~A:~D" (first b) (second b)))
        (list (first a) (second a) (first b) (second b) (packet-protocol pkt))
        (list (first b) (second b) (first a) (second a) (packet-protocol pkt)))))

(defstruct conn-stats
  (key nil)
  (packet-count 0 :type integer)
  (total-bytes 0 :type integer)
  (first-seen 0 :type integer)
  (last-seen 0 :type integer))

;;; --- Capture ---

(defstruct packet-capture
  (filter nil)
  (packets nil :type list)
  (connections (make-hash-table :test #'equal) :type hash-table))

(defun ingest (capture pkt)
  "Ingest a packet. Returns T if it passed the filter."
  (when (and (packet-capture-filter capture)
             (not (filter-matches (packet-capture-filter capture) pkt)))
    (return-from ingest nil))
  (push pkt (packet-capture-packets capture))
  (let* ((key (%conn-key pkt))
         (conn (gethash key (packet-capture-connections capture))))
    (if conn
        (progn
          (incf (conn-stats-packet-count conn))
          (incf (conn-stats-total-bytes conn) (packet-size pkt))
          (when (< (packet-timestamp-ms pkt) (conn-stats-first-seen conn))
            (setf (conn-stats-first-seen conn) (packet-timestamp-ms pkt)))
          (when (> (packet-timestamp-ms pkt) (conn-stats-last-seen conn))
            (setf (conn-stats-last-seen conn) (packet-timestamp-ms pkt))))
        (setf (gethash key (packet-capture-connections capture))
              (make-conn-stats :key key :packet-count 1
                               :total-bytes (packet-size pkt)
                               :first-seen (packet-timestamp-ms pkt)
                               :last-seen (packet-timestamp-ms pkt)))))
  t)

(defun query-packets (capture filter)
  (remove-if-not (lambda (p) (filter-matches filter p))
                 (packet-capture-packets capture)))

(defun total-packets (capture)
  (length (packet-capture-packets capture)))

(defun total-bytes (capture)
  (reduce #'+ (packet-capture-packets capture) :key #'packet-size :initial-value 0))

(defun protocol-breakdown (capture)
  "Returns alist of (protocol . count)."
  (let ((counts (make-hash-table)))
    (dolist (p (packet-capture-packets capture))
      (incf (gethash (packet-protocol p) counts 0)))
    (let (result)
      (maphash (lambda (k v) (push (cons k v) result)) counts)
      (sort result #'> :key #'cdr))))

(defun top-talkers (capture &optional (n 10))
  "Returns list of (ip . count) sorted by count."
  (let ((counts (make-hash-table :test #'equal)))
    (dolist (p (packet-capture-packets capture))
      (incf (gethash (packet-src-ip p) counts 0)))
    (let (result)
      (maphash (lambda (k v) (push (cons k v) result)) counts)
      (subseq (sort result #'> :key #'cdr) 0 (min n (length result))))))

(defun unique-ips (capture)
  (let ((ips (make-hash-table :test #'equal)))
    (dolist (p (packet-capture-packets capture))
      (setf (gethash (packet-src-ip p) ips) t)
      (setf (gethash (packet-dst-ip p) ips) t))
    (let (result)
      (maphash (lambda (k v) (declare (ignore v)) (push k result)) ips)
      result)))

(defun capture-summary (capture)
  (with-output-to-string (s)
    (format s "Capture: ~D packets, ~D bytes~%"
            (total-packets capture) (total-bytes capture))
    (format s "  Unique IPs: ~D~%" (length (unique-ips capture)))
    (format s "  Connections: ~D~%" (hash-table-count (packet-capture-connections capture)))
    (format s "  Protocol breakdown:~%")
    (dolist (entry (protocol-breakdown capture))
      (format s "    ~8A: ~D~%" (car entry) (cdr entry)))))

;;; --- Detection ---

(defun detect-syn-flood (capture &optional (threshold 100))
  (let ((syn-counts (make-hash-table :test #'equal))
        (ack-counts (make-hash-table :test #'equal)))
    (dolist (p (packet-capture-packets capture))
      (when (eq (packet-protocol p) :tcp)
        (when (and (member "SYN" (packet-flags p) :test #'string=)
                   (not (member "ACK" (packet-flags p) :test #'string=)))
          (incf (gethash (packet-src-ip p) syn-counts 0)))
        (when (member "ACK" (packet-flags p) :test #'string=)
          (incf (gethash (packet-src-ip p) ack-counts 0)))))
    (let (suspects)
      (maphash (lambda (ip syns)
                 (let ((acks (gethash ip ack-counts 0)))
                   (when (and (> syns threshold)
                              (or (zerop acks) (> (floor syns acks) 10)))
                     (push ip suspects))))
               syn-counts)
      suspects)))

(defun detect-port-scan (capture &optional (threshold 20))
  (let ((src-dst-ports (make-hash-table :test #'equal)))
    (dolist (p (packet-capture-packets capture))
      (when (and (eq (packet-protocol p) :tcp)
                 (member "SYN" (packet-flags p) :test #'string=))
        (let ((key (cons (packet-src-ip p) (packet-dst-ip p))))
          (unless (gethash key src-dst-ports)
            (setf (gethash key src-dst-ports) (make-hash-table)))
          (setf (gethash (packet-dst-port p) (gethash key src-dst-ports)) t))))
    (let (suspects)
      (maphash (lambda (key ports)
                 (when (>= (hash-table-count ports) threshold)
                   (pushnew (car key) suspects :test #'string=)))
               src-dst-ports)
      suspects)))
