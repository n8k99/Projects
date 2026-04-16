;;;; CD Key Generator -- generate and validate software license keys.

(defpackage :cd-key-generator
  (:use :cl)
  (:export #:generate-key
           #:validate-key
           #:generate-batch))

(in-package :cd-key-generator)

(defconstant +charset+ "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")
(defconstant +group-len+ 5)
(defconstant +data-groups+ 4)
(defconstant +prime+ 65521)

(defun compute-checksum (groups)
  "Compute a 5-char checksum from a list of group strings."
  (let* ((raw (apply #'concatenate 'string groups))
         (h (mod (loop for i from 0 below (length raw)
                       sum (* (char-code (char raw i)) (1+ i)))
                 +prime+))
         (chars (make-string +group-len+)))
    (dotimes (i +group-len+)
      (setf (char chars i)
            (char +charset+ (mod h (length +charset+))))
      (setf h (floor h (length +charset+))))
    chars))

(defun random-group ()
  "Generate a random group of 5 alphanumeric uppercase characters."
  (let ((group (make-string +group-len+)))
    (dotimes (i +group-len+)
      (setf (char group i)
            (char +charset+ (random (length +charset+)))))
    group))

(defun generate-key ()
  "Generate a CD key in XXXXX-XXXXX-XXXXX-XXXXX-XXXXX format."
  (let ((groups (loop repeat +data-groups+ collect (random-group))))
    (let ((checksum (compute-checksum groups)))
      (format nil "~{~A~^-~}" (append groups (list checksum))))))

(defun validate-key (key)
  "Validate a CD key by recomputing its checksum."
  (let* ((upper (string-upcase (string-trim '(#\Space #\Newline) key)))
         (parts (split-by-dash upper)))
    (when (/= (length parts) (1+ +data-groups+))
      (return-from validate-key nil))
    (dolist (p parts)
      (when (/= (length p) +group-len+)
        (return-from validate-key nil))
      (dotimes (i (length p))
        (unless (position (char p i) +charset+)
          (return-from validate-key nil))))
    (let ((expected (compute-checksum (subseq parts 0 +data-groups+))))
      (string= (nth +data-groups+ parts) expected))))

(defun split-by-dash (str)
  "Split a string by dash characters."
  (let ((parts '())
        (start 0))
    (loop for i from 0 to (length str)
          do (when (or (= i (length str))
                       (char= (char str i) #\-))
               (push (subseq str start i) parts)
               (setf start (1+ i))))
    (nreverse parts)))

(defun generate-batch (n)
  "Generate a batch of N CD keys."
  (loop repeat n collect (generate-key)))

;;; Demo
(defun demo ()
  (format t "=== CD Key Generator ===~%~%")
  (let ((keys (generate-batch 5)))
    (dolist (key keys)
      (format t "  ~A  valid=~A~%" key (validate-key key)))
    (let* ((first-key (first keys))
           (tampered (concatenate 'string
                       (if (char= (char first-key 0) #\Z) "A" "Z")
                       (subseq first-key 1))))
      (format t "~%  Tampered: ~A  valid=~A~%" tampered (validate-key tampered)))))
