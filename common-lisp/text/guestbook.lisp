(defpackage :text/guestbook
  (:use :cl)
  (:export #:make-journal
           #:add-entry
           #:get-entries
           #:get-entries-by-author
           #:get-entries-in-range
           #:entry-count
           #:export-text
           #:entry-author
           #:entry-content
           #:entry-timestamp))

(in-package :text/guestbook)

;;; ── Entry (immutable) ──────────────────────────────────────────────

(defstruct (entry (:constructor %make-entry))
  "An immutable journal entry with author, content, and timestamp."
  (author    "" :type string :read-only t)
  (content   "" :type string :read-only t)
  (timestamp 0  :type integer :read-only t))  ; universal-time

(defun format-timestamp (universal-time)
  "Format a universal-time as YYYY-MM-DD HH:MM:SS UTC."
  (multiple-value-bind (sec min hour day month year)
      (decode-universal-time universal-time 0)
    (format nil "~4,'0D-~2,'0D-~2,'0D ~2,'0D:~2,'0D:~2,'0D UTC"
            year month day hour min sec)))

(defun entry-to-text (entry)
  "Format an entry as [timestamp] author: content."
  (format nil "[~A] ~A: ~A"
          (format-timestamp (entry-timestamp entry))
          (entry-author entry)
          (entry-content entry)))

;;; ── Journal (append-only) ──────────────────────────────────────────

(defstruct journal
  "Chronological append-only journal / guestbook."
  (entries nil :type list))  ; list of entry, oldest first

(defun make-journal ()
  "Create an empty journal."
  (make-journal :entries nil))

(defun add-entry (journal author content &optional timestamp)
  "Append a new entry to the journal. Returns the new entry.
   TIMESTAMP defaults to current time if not provided."
  (let ((entry (%make-entry
                :author author
                :content content
                :timestamp (or timestamp (get-universal-time)))))
    (setf (journal-entries journal)
          (append (journal-entries journal) (list entry)))
    entry))

(defun get-entries (journal)
  "Return all entries, newest first."
  (reverse (journal-entries journal)))

(defun get-entries-by-author (journal author)
  "Return entries by a specific author, newest first."
  (remove-if-not (lambda (e) (string= (entry-author e) author))
                 (get-entries journal)))

(defun get-entries-in-range (journal start-time end-time)
  "Return entries within [START-TIME, END-TIME] (universal-time), newest first."
  (remove-if-not (lambda (e)
                   (let ((ts (entry-timestamp e)))
                     (and (<= start-time ts) (<= ts end-time))))
                 (get-entries journal)))

(defun entry-count (journal)
  "Return total number of entries."
  (length (journal-entries journal)))

(defun export-text (journal)
  "Export the full journal as plain text in chronological order."
  (format nil "~{~A~^~%~}"
          (mapcar #'entry-to-text (journal-entries journal))))
