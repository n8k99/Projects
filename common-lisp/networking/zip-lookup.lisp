;;;; Zip / Postal Code Lookup — resolve postal codes to location data.

(defpackage #:zip-lookup
  (:use #:cl)
  (:export #:make-postal-database
           #:postal-lookup
           #:search-by-city
           #:search-by-state
           #:postal-distance-between
           #:codes-within-radius
           #:state-breakdown
           #:postal-result-postal-code
           #:postal-result-city
           #:postal-result-state
           #:postal-result-state-abbr
           #:postal-result-country-code
           #:postal-result-summary))

(in-package #:zip-lookup)

(defstruct postal-entry
  (postal-code "" :type string)
  (country-code "" :type string)
  (country-name "" :type string)
  (state "" :type string)
  (state-abbr "" :type string)
  (city "" :type string)
  (latitude 0.0 :type float)
  (longitude 0.0 :type float)
  (timezone "" :type string))

(defstruct postal-result
  (postal-code "" :type string)
  (country-code "" :type string)
  (country-name "" :type string)
  (state "" :type string)
  (state-abbr "" :type string)
  (city "" :type string)
  (latitude 0.0 :type float)
  (longitude 0.0 :type float)
  (timezone "" :type string))

(defun postal-result-summary (r)
  (let* ((st (if (string= (postal-result-state-abbr r) "")
                 (postal-result-state r) (postal-result-state-abbr r)))
         (parts (remove "" (list (postal-result-city r) st (postal-result-country-code r))
                        :test #'string=)))
    (format nil "~A -> ~{~A~^, ~} [~,4F, ~,4F]"
            (postal-result-postal-code r) parts
            (postal-result-latitude r) (postal-result-longitude r))))

(defun %entry-to-result (e)
  (make-postal-result
   :postal-code (postal-entry-postal-code e) :country-code (postal-entry-country-code e)
   :country-name (postal-entry-country-name e) :state (postal-entry-state e)
   :state-abbr (postal-entry-state-abbr e) :city (postal-entry-city e)
   :latitude (postal-entry-latitude e) :longitude (postal-entry-longitude e)
   :timezone (postal-entry-timezone e)))

(defun %normalize (code)
  (string-upcase (remove #\Space code)))

(defun %make-entry (code cc cn st sa city lat lon tz)
  (make-postal-entry :postal-code code :country-code cc :country-name cn
                     :state st :state-abbr sa :city city
                     :latitude lat :longitude lon :timezone tz))

(defparameter *builtin-entries*
  (list
   (%make-entry "32202" "US" "United States" "Florida" "FL" "Jacksonville" 30.328 -81.655 "America/New_York")
   (%make-entry "32084" "US" "United States" "Florida" "FL" "St. Augustine" 29.8947 -81.3145 "America/New_York")
   (%make-entry "33101" "US" "United States" "Florida" "FL" "Miami" 25.7617 -80.1918 "America/New_York")
   (%make-entry "10001" "US" "United States" "New York" "NY" "New York" 40.7484 -73.9967 "America/New_York")
   (%make-entry "90210" "US" "United States" "California" "CA" "Beverly Hills" 34.0901 -118.4065 "America/Los_Angeles")
   (%make-entry "SW1A1AA" "GB" "United Kingdom" "England" "" "London" 51.5014 -0.1419 "Europe/London")
   (%make-entry "75001" "FR" "France" "Île-de-France" "" "Paris" 48.8606 2.3376 "Europe/Paris")))

(defstruct postal-database
  (by-code (make-hash-table :test #'equal) :type hash-table)
  (entries nil :type list))

(defun make-postal-database (&optional (entries *builtin-entries*))
  (let ((db (make-postal-database :entries entries)))
    (dolist (e entries db)
      (setf (gethash (%normalize (postal-entry-postal-code e))
                     (postal-database-by-code db))
            e))))

(defun postal-lookup (db code)
  (let ((e (gethash (%normalize code) (postal-database-by-code db))))
    (when e (%entry-to-result e))))

(defun search-by-city (db city)
  (let ((city-lower (string-downcase city)))
    (mapcar #'%entry-to-result
            (remove-if-not (lambda (e) (string-equal city-lower (string-downcase (postal-entry-city e))))
                           (postal-database-entries db)))))

(defun search-by-state (db state)
  (let ((s (string-downcase state)))
    (mapcar #'%entry-to-result
            (remove-if-not (lambda (e)
                             (or (string-equal s (string-downcase (postal-entry-state e)))
                                 (string-equal s (string-downcase (postal-entry-state-abbr e)))))
                           (postal-database-entries db)))))

(defun %haversine (lat1 lon1 lat2 lon2)
  (let* ((r 6371.0)
         (dlat (* (- lat2 lat1) (/ pi 180)))
         (dlon (* (- lon2 lon1) (/ pi 180)))
         (a (+ (expt (sin (/ dlat 2)) 2)
               (* (cos (* lat1 (/ pi 180))) (cos (* lat2 (/ pi 180)))
                  (expt (sin (/ dlon 2)) 2)))))
    (* r 2 (atan (sqrt a) (sqrt (- 1 a))))))

(defun postal-distance-between (db code-a code-b)
  (let ((a (postal-lookup db code-a))
        (b (postal-lookup db code-b)))
    (when (and a b)
      (%haversine (postal-result-latitude a) (postal-result-longitude a)
                  (postal-result-latitude b) (postal-result-longitude b)))))

(defun codes-within-radius (db center-code radius-km)
  (let ((c (postal-lookup db center-code)))
    (when c
      (let ((center-key (%normalize center-code))
            results)
        (dolist (e (postal-database-entries db))
          (unless (string= (%normalize (postal-entry-postal-code e)) center-key)
            (let ((d (%haversine (postal-result-latitude c) (postal-result-longitude c)
                                 (postal-entry-latitude e) (postal-entry-longitude e))))
              (when (<= d radius-km)
                (push (cons (%entry-to-result e) d) results)))))
        (sort results #'< :key #'cdr)))))

(defun state-breakdown (db codes)
  (let ((counts (make-hash-table :test #'equal)))
    (dolist (code codes)
      (let* ((r (postal-lookup db code))
             (key (if r
                      (let ((sa (postal-result-state-abbr r)))
                        (if (string= sa "") (postal-result-state r) sa))
                      "unknown")))
        (incf (gethash key counts 0))))
    (let (result)
      (maphash (lambda (k v) (push (cons k v) result)) counts)
      result)))
