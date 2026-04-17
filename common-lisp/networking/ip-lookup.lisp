;;;; Country from IP Lookup — resolve IP addresses to geographic location data.

(defpackage #:ip-lookup
  (:use #:cl)
  (:export #:make-geo-ip-database
           #:lookup
           #:bulk-lookup
           #:country-breakdown
           #:nearest-server
           #:is-private-ip
           #:ip-to-int
           #:int-to-ip
           #:haversine-km
           #:geo-result-ip
           #:geo-result-country-code
           #:geo-result-country-name
           #:geo-result-city
           #:geo-result-is-private
           #:geo-result-summary))

(in-package #:ip-lookup)

;;; --- IP conversion ---

(defun ip-to-int (ip)
  "Convert dotted-quad IP string to integer."
  (let ((parts (mapcar #'parse-integer (uiop:split-string ip :separator "."))))
    (when (= (length parts) 4)
      (+ (ash (first parts) 24) (ash (second parts) 16)
         (ash (third parts) 8) (fourth parts)))))

(defun int-to-ip (n)
  "Convert integer to dotted-quad IP string."
  (format nil "~D.~D.~D.~D"
          (ldb (byte 8 24) n) (ldb (byte 8 16) n)
          (ldb (byte 8 8) n) (ldb (byte 8 0) n)))

(defun is-private-ip (ip)
  (let ((n (ip-to-int ip)))
    (or (<= #x0A000000 n #x0AFFFFFF)
        (<= #xAC100000 n #xAC1FFFFF)
        (<= #xC0A80000 n #xC0A8FFFF)
        (<= #x7F000000 n #x7FFFFFFF)
        (<= n #x00FFFFFF))))

;;; --- Data Structures ---

(defstruct ip-range
  (start 0 :type integer)
  (end 0 :type integer)
  (country-code "" :type string)
  (country-name "" :type string)
  (region "" :type string)
  (city "" :type string)
  (latitude 0.0 :type float)
  (longitude 0.0 :type float)
  (timezone "" :type string))

(defstruct geo-result
  (ip "" :type string)
  (country-code "" :type string)
  (country-name "" :type string)
  (region "" :type string)
  (city "" :type string)
  (latitude 0.0 :type float)
  (longitude 0.0 :type float)
  (timezone "" :type string)
  (is-private nil))

(defun geo-result-summary (r)
  (if (geo-result-is-private r)
      (format nil "~A -> private/reserved" (geo-result-ip r))
      (let ((parts (remove "" (list (geo-result-city r) (geo-result-region r)
                                     (geo-result-country-name r))
                           :test #'string=)))
        (format nil "~A -> ~{~A~^, ~} (~A) [~,4F, ~,4F]"
                (geo-result-ip r) parts (geo-result-country-code r)
                (geo-result-latitude r) (geo-result-longitude r)))))

;;; --- Built-in ranges ---

(defun %make-range (s e cc cn rg ct lat lon tz)
  (make-ip-range :start (ip-to-int s) :end (ip-to-int e)
                 :country-code cc :country-name cn :region rg :city ct
                 :latitude lat :longitude lon :timezone tz))

(defparameter *builtin-ranges*
  (sort (list
         (%make-range "1.0.0.0" "1.0.0.255" "AU" "Australia" "Queensland" "Brisbane" -27.4679 153.0281 "Australia/Brisbane")
         (%make-range "1.1.1.0" "1.1.1.255" "AU" "Australia" "New South Wales" "Sydney" -33.8688 151.2093 "Australia/Sydney")
         (%make-range "8.8.4.0" "8.8.8.255" "US" "United States" "California" "Mountain View" 37.3861 -122.0839 "America/Los_Angeles")
         (%make-range "9.0.0.0" "9.255.255.255" "US" "United States" "New York" "Armonk" 41.1085 -73.7205 "America/New_York")
         (%make-range "13.0.0.0" "13.255.255.255" "US" "United States" "Virginia" "Ashburn" 39.0438 -77.4874 "America/New_York")
         (%make-range "144.126.0.0" "144.126.255.255" "US" "United States" "New York" "New York" 40.7128 -74.006 "America/New_York")
         (%make-range "77.0.0.0" "77.255.255.255" "DE" "Germany" "Hessen" "Frankfurt" 50.1109 8.6821 "Europe/Berlin")
         (%make-range "212.0.0.0" "212.255.255.255" "GB" "United Kingdom" "England" "London" 51.5074 -0.1278 "Europe/London"))
        #'< :key #'ip-range-start))

;;; --- Database ---

(defstruct geo-ip-database
  (ranges *builtin-ranges* :type list))

(defun %binary-search (ranges ip-int)
  (let ((lo 0) (hi (1- (length ranges))))
    (loop while (<= lo hi) do
      (let* ((mid (floor (+ lo hi) 2))
             (r (nth mid ranges)))
        (cond
          ((< ip-int (ip-range-start r)) (setf hi (1- mid)))
          ((> ip-int (ip-range-end r)) (setf lo (1+ mid)))
          (t (return-from %binary-search r)))))
    nil))

(defun lookup (db ip)
  "Look up geographic data for an IP."
  (if (is-private-ip ip)
      (make-geo-result :ip ip :is-private t)
      (let ((r (%binary-search (geo-ip-database-ranges db) (ip-to-int ip))))
        (if r
            (make-geo-result :ip ip
                             :country-code (ip-range-country-code r)
                             :country-name (ip-range-country-name r)
                             :region (ip-range-region r)
                             :city (ip-range-city r)
                             :latitude (ip-range-latitude r)
                             :longitude (ip-range-longitude r)
                             :timezone (ip-range-timezone r))
            (make-geo-result :ip ip)))))

(defun bulk-lookup (db ips)
  (mapcar (lambda (ip) (lookup db ip)) ips))

(defun country-breakdown (db ips)
  "Returns alist of (country-code . count)."
  (let ((counts (make-hash-table :test #'equal)))
    (dolist (ip ips)
      (let* ((r (lookup db ip))
             (key (cond ((geo-result-is-private r) "private")
                        ((string= (geo-result-country-code r) "") "unknown")
                        (t (geo-result-country-code r)))))
        (incf (gethash key counts 0))))
    (let (result)
      (maphash (lambda (k v) (push (cons k v) result)) counts)
      result)))

;;; --- Haversine ---

(defun haversine-km (lat1 lon1 lat2 lon2)
  (let* ((r 6371.0)
         (dlat (* (- lat2 lat1) (/ pi 180)))
         (dlon (* (- lon2 lon1) (/ pi 180)))
         (a (+ (expt (sin (/ dlat 2)) 2)
               (* (cos (* lat1 (/ pi 180)))
                  (cos (* lat2 (/ pi 180)))
                  (expt (sin (/ dlon 2)) 2)))))
    (* r 2 (atan (sqrt a) (sqrt (- 1 a))))))

(defun nearest-server (client-ip server-ips db)
  "Find geographically nearest server to client."
  (let ((client (lookup db client-ip)))
    (when (or (geo-result-is-private client) (string= (geo-result-country-code client) ""))
      (return-from nearest-server nil))
    (let ((best-ip nil) (best-dist most-positive-double-float))
      (dolist (sip server-ips best-ip)
        (let ((srv (lookup db sip)))
          (unless (or (geo-result-is-private srv) (string= (geo-result-country-code srv) ""))
            (let ((d (haversine-km (geo-result-latitude client) (geo-result-longitude client)
                                   (geo-result-latitude srv) (geo-result-longitude srv))))
              (when (< d best-dist)
                (setf best-dist d best-ip sip)))))))))
