;;;; Log File Maker — structured append-only log with level filtering and rotation.

(defpackage #:log-file-maker
  (:use #:cl)
  (:export #:make-logger #:log-entry #:query-level #:query-category
           #:query-message-contains #:query-time-range
           #:to-text #:parse-entries
           #:logger-active #:logger-archive #:logger-level-threshold
           #:make-entry #:entry-timestamp #:entry-level
           #:entry-category #:entry-message))

(in-package #:log-file-maker)

(defstruct entry
  (timestamp 0 :type integer)
  (level :info :type keyword)
  (category "" :type string)
  (message "" :type string))

(defstruct logger
  (level-threshold :debug :type keyword)
  (max-entries 100 :type integer)
  (active nil :type list)
  (archive nil :type list))

(defparameter *level-rank*
  '((:debug . 0) (:info . 1) (:warn . 2) (:error . 3)))

(defun level-rank (level) (cdr (assoc level *level-rank*)))

(defun level-geq (a b)
  (>= (level-rank a) (level-rank b)))

(defun level-to-str (level)
  (case level
    (:debug "DEBUG")
    (:info "INFO")
    (:warn "WARN")
    (:error "ERROR")
    (t "UNKNOWN")))

(defun level-from-str (s)
  (cond
    ((string= s "DEBUG") :debug)
    ((string= s "INFO") :info)
    ((string= s "WARN") :warn)
    ((string= s "ERROR") :error)
    (t nil)))

(defun entry-to-string (e)
  (format nil "~D~C~A~C~A~C~A"
          (entry-timestamp e) #\Tab
          (level-to-str (entry-level e)) #\Tab
          (entry-category e) #\Tab
          (entry-message e)))

(defun log-entry (logger ts level category message)
  (when (level-geq level (logger-level-threshold logger))
    (setf (logger-active logger)
          (append (logger-active logger)
                  (list (make-entry :timestamp ts :level level
                                    :category category :message message))))
    (loop while (> (length (logger-active logger)) (logger-max-entries logger))
          do (setf (logger-archive logger)
                   (append (logger-archive logger)
                           (list (car (logger-active logger)))))
             (setf (logger-active logger) (cdr (logger-active logger))))))

(defun query-level (logger min-level)
  (remove-if-not (lambda (e) (level-geq (entry-level e) min-level))
                 (logger-active logger)))

(defun query-category (logger category)
  (remove-if-not (lambda (e) (string= (entry-category e) category))
                 (logger-active logger)))

(defun query-message-contains (logger needle)
  (let ((lc (string-downcase needle)))
    (remove-if-not (lambda (e) (search lc (string-downcase (entry-message e))))
                   (logger-active logger))))

(defun query-time-range (logger from to)
  (remove-if-not (lambda (e)
                   (and (>= (entry-timestamp e) from)
                        (<= (entry-timestamp e) to)))
                 (logger-active logger)))

(defun to-text (logger)
  (if (null (logger-active logger))
      ""
      (concatenate 'string
                   (format nil "~{~A~^~%~}"
                           (mapcar #'entry-to-string (logger-active logger)))
                   (string #\Newline))))

(defun split-tab (line)
  (let ((parts nil)
        (start 0)
        (count 0)
        (max-parts 4))
    (loop for i from 0 below (length line)
          when (and (char= (char line i) #\Tab) (< count (1- max-parts)))
          do (push (subseq line start i) parts)
             (setf start (1+ i))
             (incf count))
    (push (subseq line start) parts)
    (nreverse parts)))

(defun parse-entries (text)
  (let ((entries nil))
    (dolist (line (remove-if (lambda (l) (zerop (length l)))
                             (loop for start = 0 then (1+ end)
                                   for end = (position #\Newline text :start start)
                                   collect (subseq text start end)
                                   while end)))
      (let ((parts (split-tab line)))
        (unless (= (length parts) 4) (return-from parse-entries nil))
        (let* ((ts (ignore-errors (parse-integer (first parts))))
               (level (level-from-str (second parts)))
               (category (third parts))
               (message (fourth parts)))
          (unless (and ts level) (return-from parse-entries nil))
          (push (make-entry :timestamp ts :level level
                             :category category :message message)
                entries))))
    (nreverse entries)))

(defun demo ()
  (let ((l (make-logger :level-threshold :debug :max-entries 100)))
    (log-entry l 100 :info "boot" "starting up")
    (log-entry l 200 :debug "boot" "loaded config")
    (log-entry l 300 :warn "net" "slow response")
    (log-entry l 400 :error "db" "connection lost")
    (log-entry l 500 :info "db" "reconnected")
    (format t "=== all entries ===~%~A" (to-text l))
    (format t "~%=== >= WARN ===~%")
    (dolist (e (query-level l :warn))
      (format t "  ~A~%" (entry-to-string e)))
    (format t "~%=== category db ===~%")
    (dolist (e (query-category l "db"))
      (format t "  ~A~%" (entry-to-string e)))
    (format t "~%=== parse round-trip ===~%")
    (let ((parsed (parse-entries (to-text l))))
      (format t "  ~D entries round-tripped~%" (length parsed)))
    (format t "~%=== rotation demo ===~%")
    (let ((l2 (make-logger :level-threshold :debug :max-entries 3)))
      (dotimes (i 6) (log-entry l2 (* i 10) :info "x" (format nil "m~D" i)))
      (format t "  active: ~A, archived: ~D~%"
              (mapcar #'entry-message (logger-active l2))
              (length (logger-archive l2))))))
