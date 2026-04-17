;;;; Whois Search Tool — query and parse domain registration data.

(defpackage #:whois-search
  (:use #:cl)
  (:export #:make-whois-client
           #:add-simulated
           #:whois-query
           #:whois-summary
           #:parse-whois-text
           #:compare-records
           #:registrar-breakdown
           #:whois-record-domain
           #:whois-record-registrar
           #:whois-record-name-servers
           #:whois-record-created-date
           #:whois-record-expiry-date
           #:whois-record-status
           #:whois-contact-name
           #:whois-contact-organization))

(in-package #:whois-search)

;;; --- Data Structures ---

(defstruct whois-contact
  (name "" :type string)
  (organization "" :type string)
  (email "" :type string)
  (city "" :type string)
  (state "" :type string)
  (country "" :type string))

(defun whois-contact-summary (c)
  (let ((parts (remove "" (list (whois-contact-name c)
                                 (whois-contact-organization c)
                                 (whois-contact-city c)
                                 (whois-contact-country c))
                       :test #'string=)))
    (if parts (format nil "~{~A~^, ~}" parts) "(redacted)")))

(defstruct whois-record
  (domain "" :type string)
  (registrar "" :type string)
  (registrant nil)
  (name-servers nil :type list)
  (status nil :type list)
  (created-date "" :type string)
  (updated-date "" :type string)
  (expiry-date "" :type string)
  (dnssec "" :type string)
  (raw-text "" :type string))

(defun whois-record-tld (r)
  (let ((pos (position #\. (whois-record-domain r) :from-end t)))
    (if pos (subseq (whois-record-domain r) (1+ pos)) "")))

;;; --- Parsing ---

(defun %find-field (raw key)
  "Find value for KEY: in raw whois text."
  (let ((key-lower (string-downcase key)))
    (dolist (line (uiop:split-string raw :separator '(#\Newline)))
      (let ((trimmed (string-trim '(#\Space #\Tab #\Return) line)))
        (when (and (>= (length trimmed) (length key-lower))
                   (string-equal key-lower (subseq (string-downcase trimmed) 0 (length key-lower))))
          (let ((colon (position #\: trimmed)))
            (when colon
              (return (string-trim '(#\Space #\Tab) (subseq trimmed (1+ colon)))))))))
    ""))

(defun %find-all-fields (raw key)
  (let ((key-lower (string-downcase key))
        results)
    (dolist (line (uiop:split-string raw :separator '(#\Newline)) (nreverse results))
      (let ((trimmed (string-trim '(#\Space #\Tab #\Return) line)))
        (when (and (>= (length trimmed) (length key-lower))
                   (string-equal key-lower (subseq (string-downcase trimmed) 0 (length key-lower))))
          (let ((colon (position #\: trimmed)))
            (when colon
              (let ((v (string-trim '(#\Space #\Tab) (subseq trimmed (1+ colon)))))
                (unless (string= v "") (push v results))))))))))

(defun parse-whois-text (domain raw)
  (let ((r (make-whois-record :domain (string-downcase domain) :raw-text raw)))
    (setf (whois-record-registrar r) (%find-field raw "Registrar"))
    (setf (whois-record-dnssec r) (%find-field raw "DNSSEC"))
    (setf (whois-record-created-date r)
          (let ((v (%find-field raw "Creation Date")))
            (if (string= v "") (%find-field raw "Created Date") v)))
    (setf (whois-record-updated-date r) (%find-field raw "Updated Date"))
    (setf (whois-record-expiry-date r)
          (let ((v (%find-field raw "Registry Expiry Date")))
            (if (string= v "") (%find-field raw "Expiration Date") v)))
    (setf (whois-record-name-servers r)
          (mapcar #'string-downcase (%find-all-fields raw "Name Server")))
    (setf (whois-record-status r)
          (or (%find-all-fields raw "Domain Status")
              (%find-all-fields raw "Status")))
    (let ((reg (make-whois-contact
                :name (%find-field raw "Registrant Name")
                :organization (%find-field raw "Registrant Organization")
                :email (%find-field raw "Registrant Email")
                :country (%find-field raw "Registrant Country")
                :city (%find-field raw "Registrant City"))))
      (unless (and (string= (whois-contact-name reg) "")
                   (string= (whois-contact-organization reg) "")
                   (string= (whois-contact-email reg) ""))
        (setf (whois-record-registrant r) reg)))
    r))

(defun whois-summary (r)
  (with-output-to-string (s)
    (format s "Domain: ~A~%" (whois-record-domain r))
    (unless (string= (whois-record-registrar r) "")
      (format s "  Registrar: ~A~%" (whois-record-registrar r)))
    (when (whois-record-registrant r)
      (format s "  Registrant: ~A~%" (whois-contact-summary (whois-record-registrant r))))
    (when (whois-record-name-servers r)
      (format s "  Name Servers: ~{~A~^, ~}~%" (whois-record-name-servers r)))))

;;; --- Client ---

(defstruct whois-client
  (cache (make-hash-table :test #'equal) :type hash-table)
  (simulated (make-hash-table :test #'equal) :type hash-table))

(defun add-simulated (client domain raw)
  (setf (gethash (string-downcase domain) (whois-client-simulated client)) raw))

(defun whois-query (client domain)
  (let ((domain (string-downcase domain)))
    (or (gethash domain (whois-client-cache client))
        (let* ((raw (or (gethash domain (whois-client-simulated client)) ""))
               (record (parse-whois-text domain raw)))
          (setf (gethash domain (whois-client-cache client)) record)
          record))))

(defun registrar-breakdown (client domains)
  (let ((counts (make-hash-table :test #'equal)))
    (dolist (d domains)
      (let* ((r (whois-query client d))
             (key (if (string= (whois-record-registrar r) "") "unknown"
                      (whois-record-registrar r))))
        (incf (gethash key counts 0))))
    (let (result)
      (maphash (lambda (k v) (push (cons k v) result)) counts)
      result)))

(defun compare-records (before after)
  (let (changes)
    (unless (string= (whois-record-registrar before) (whois-record-registrar after))
      (push (list :registrar (whois-record-registrar before) (whois-record-registrar after)) changes))
    (unless (equal (whois-record-name-servers before) (whois-record-name-servers after))
      (push (list :name-servers (whois-record-name-servers before) (whois-record-name-servers after)) changes))
    (unless (string= (whois-record-expiry-date before) (whois-record-expiry-date after))
      (push (list :expiry-date (whois-record-expiry-date before) (whois-record-expiry-date after)) changes))
    changes))
