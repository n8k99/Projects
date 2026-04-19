;;;; Password Safe — encrypted-at-rest credential store with session FSM.
;;;; Not real crypto; byte-sum + XOR demonstrates the STRUCTURE of
;;;; encryption-at-rest without pretending to be secure.

(defpackage #:password-safe
  (:use #:cl)
  (:export #:make-safe #:safe-locked-p
           #:unlock-safe #:lock-safe #:check-auto-lock
           #:add-entry #:get-password #:remove-entry
           #:search-entries #:get-summary #:entry-count
           #:generate-password #:password-strength
           #:summary-id #:summary-service #:summary-username #:summary-notes))

(in-package #:password-safe)

(defstruct entry
  (id 0 :type integer)
  (service "" :type string)
  (username "" :type string)
  (encrypted nil :type vector)
  (notes "" :type string)
  (created-ms 0 :type integer)
  (modified-ms 0 :type integer))

(defstruct summary
  (id 0 :type integer)
  (service "" :type string)
  (username "" :type string)
  (notes "" :type string))

(defstruct safe
  (salt (make-array 16 :element-type '(unsigned-byte 8)
                         :initial-contents (loop for i from 0 below 16 collect i)))
  (verifier nil :type (or null vector))
  (entries nil :type list)
  (next-id 1 :type integer)
  (key nil :type (or null vector))
  (last-activity-ms 0 :type integer)
  (auto-lock-ms 0 :type integer))

(defun %verifier-hash (master salt)
  (let ((bytes (map 'list #'char-code master))
        (out (make-array 32 :element-type '(unsigned-byte 8))))
    (loop for r from 0 below 32 do
      (let ((h r))
        (dolist (b bytes) (setf h (logand (+ (* h 131) b) #xFFFFFFFFFFFFFFFF)))
        (loop for b across salt do
          (setf h (logand (+ (* h 131) b) #xFFFFFFFFFFFFFFFF)))
        (setf (aref out r) (logand h #xFF))))
    out))

(defun %derive-key (master salt)
  (let ((bytes (map 'list #'char-code master))
        (out (make-array 64 :element-type '(unsigned-byte 8))))
    (loop for i from 0 below 64 do
      (let ((h (+ 128 i)))
        (dolist (b bytes) (setf h (logand (+ (* h 197) b) #xFFFFFFFFFFFFFFFF)))
        (loop for b across salt do
          (setf h (logand (+ (* h 197) b) #xFFFFFFFFFFFFFFFF)))
        (setf (aref out i) (logand h #xFF))))
    out))

(defun %xor (data key)
  (let* ((n (length data))
         (out (make-array n :element-type '(unsigned-byte 8))))
    (loop for i from 0 below n do
      (setf (aref out i) (logxor (aref data i) (aref key (mod i (length key))))))
    out))

(defun make-safe-with (master auto-lock-ms)
  (let ((s (make-safe :auto-lock-ms auto-lock-ms)))
    (setf (safe-verifier s) (%verifier-hash master (safe-salt s)))
    s))

(defun safe-locked-p (s) (null (safe-key s)))

(defun unlock-safe (s master now-ms)
  (if (equalp (%verifier-hash master (safe-salt s)) (safe-verifier s))
      (progn
        (setf (safe-key s) (%derive-key master (safe-salt s))
              (safe-last-activity-ms s) now-ms)
        t)
      nil))

(defun lock-safe (s) (setf (safe-key s) nil))

(defun check-auto-lock (s now-ms)
  (when (and (safe-key s)
             (>= now-ms (+ (safe-last-activity-ms s) (safe-auto-lock-ms s))))
    (lock-safe s)))

(defun %touch (s now-ms)
  (when (safe-key s)
    (setf (safe-last-activity-ms s) now-ms)
    (safe-key s)))

(defun %string-to-bytes (s)
  (map 'vector #'char-code s))

(defun %bytes-to-string (b)
  (map 'string #'code-char b))

(defun add-entry (s service username password notes now-ms)
  (let ((key (%touch s now-ms)))
    (unless key (return-from add-entry nil))
    (let ((id (safe-next-id s)))
      (incf (safe-next-id s))
      (setf (safe-entries s)
            (append (safe-entries s)
                    (list (make-entry :id id :service service :username username
                                       :encrypted (%xor (%string-to-bytes password) key)
                                       :notes notes
                                       :created-ms now-ms :modified-ms now-ms))))
      id)))

(defun get-password (s id now-ms)
  (let ((key (%touch s now-ms)))
    (unless key (return-from get-password nil))
    (let ((e (find id (safe-entries s) :key #'entry-id)))
      (when e (%bytes-to-string (%xor (entry-encrypted e) key))))))

(defun remove-entry (s id now-ms)
  (unless (%touch s now-ms) (return-from remove-entry nil))
  (let ((pos (position id (safe-entries s) :key #'entry-id)))
    (when pos
      (setf (safe-entries s)
            (append (subseq (safe-entries s) 0 pos)
                    (subseq (safe-entries s) (1+ pos))))
      t)))

(defun get-summary (s id)
  (let ((e (find id (safe-entries s) :key #'entry-id)))
    (when e (make-summary :id (entry-id e) :service (entry-service e)
                          :username (entry-username e) :notes (entry-notes e)))))

(defun search-entries (s query now-ms)
  (unless (%touch s now-ms) (return-from search-entries nil))
  (let ((q (string-downcase query)))
    (loop for e in (safe-entries s)
          when (or (search q (string-downcase (entry-service e)))
                   (search q (string-downcase (entry-username e)))
                   (search q (string-downcase (entry-notes e))))
            collect (make-summary :id (entry-id e) :service (entry-service e)
                                   :username (entry-username e) :notes (entry-notes e)))))

(defun entry-count (s) (length (safe-entries s)))

(defun generate-password (length lower-p upper-p digits-p symbols-p seed)
  (let ((alphabet ""))
    (when lower-p (setf alphabet (concatenate 'string alphabet "abcdefghijklmnopqrstuvwxyz")))
    (when upper-p (setf alphabet (concatenate 'string alphabet "ABCDEFGHIJKLMNOPQRSTUVWXYZ")))
    (when digits-p (setf alphabet (concatenate 'string alphabet "0123456789")))
    (when symbols-p (setf alphabet (concatenate 'string alphabet "!@#$%^&*()-_=+[]{}")))
    (when (zerop (length alphabet)) (return-from generate-password ""))
    (let ((state (logand (+ (* seed 6364136223846793005) 1442695040888963407)
                         #xFFFFFFFFFFFFFFFF)))
      (with-output-to-string (out)
        (loop repeat length do
          (setf state (logand (+ (* state 6364136223846793005) 1442695040888963407)
                              #xFFFFFFFFFFFFFFFF))
          (write-char (char alphabet (mod (ash state -32) (length alphabet))) out))))))

(defun password-strength (pw)
  (let ((len (length pw))
        (classes 0))
    (when (some (lambda (c) (and (char<= #\a c) (char<= c #\z))) pw) (incf classes))
    (when (some (lambda (c) (and (char<= #\A c) (char<= c #\Z))) pw) (incf classes))
    (when (some (lambda (c) (and (char<= #\0 c) (char<= c #\9))) pw) (incf classes))
    (when (some (lambda (c) (not (alphanumericp c))) pw) (incf classes))
    (cond
      ((< len 8) :weak)
      ((and (< len 12) (< classes 3)) :weak)
      ((>= len 20) :very-strong)
      ((and (>= len 14) (>= classes 3)) :strong)
      ((>= classes 4) :strong)
      (t :fair))))
