;;;; Mail Checker — monitor an inbox for new messages with filtering and notification.

(defpackage #:mail-checker
  (:use #:cl)
  (:export #:make-mail-server
           #:create-mailbox
           #:send-mail
           #:check-mail
           #:mark-read
           #:mark-flagged
           #:delete-message
           #:unread-count
           #:mailbox-stats
           #:make-mail-filter
           #:parse-email-address
           #:check-result-summary
           #:email-address-name
           #:email-address-address
           #:mail-message-uid
           #:mail-message-subject
           #:mail-message-sender
           #:check-result-total
           #:check-result-new-count
           #:check-result-matched))

(in-package #:mail-checker)

;;; --- Data Structures ---

(defstruct email-address
  (name "" :type string)
  (address "" :type string))

(defun parse-email-address (raw)
  "Parse 'Name <addr>' or plain 'addr'."
  (let ((raw (string-trim '(#\Space #\Tab) raw)))
    (let ((lt (position #\< raw))
          (gt (position #\> raw)))
      (if (and lt gt (< lt gt))
          (make-email-address
           :name (string-trim '(#\Space #\" ) (subseq raw 0 lt))
           :address (string-trim '(#\Space) (subseq raw (1+ lt) gt)))
          (make-email-address :address raw)))))

(defstruct mail-message
  (uid "" :type string)
  (sender (make-email-address) :type email-address)
  (recipients nil :type list)
  (subject "" :type string)
  (body "" :type string)
  (date (get-universal-time) :type integer)
  (is-read nil)
  (is-flagged nil))

(defstruct mail-filter
  (sender-contains "" :type string)
  (subject-contains "" :type string)
  (body-contains "" :type string)
  (is-unread-only nil)
  (since nil))

(defun %contains-ci (haystack needle)
  "Case-insensitive substring test."
  (search (string-downcase needle) (string-downcase haystack)))

(defun message-matches-filter (msg filter)
  "Test if MSG matches FILTER criteria."
  (and (or (string= (mail-filter-sender-contains filter) "")
           (%contains-ci (email-address-address (mail-message-sender msg))
                         (mail-filter-sender-contains filter)))
       (or (string= (mail-filter-subject-contains filter) "")
           (%contains-ci (mail-message-subject msg)
                         (mail-filter-subject-contains filter)))
       (or (string= (mail-filter-body-contains filter) "")
           (%contains-ci (mail-message-body msg)
                         (mail-filter-body-contains filter)))
       (or (not (mail-filter-is-unread-only filter))
           (not (mail-message-is-read msg)))
       (or (null (mail-filter-since filter))
           (>= (mail-message-date msg) (mail-filter-since filter)))))

;;; --- Check Result ---

(defstruct check-result
  (total 0 :type integer)
  (new-count 0 :type integer)
  (matched nil :type list)
  (checked-at (get-universal-time) :type integer))

(defun check-result-summary (result)
  (with-output-to-string (s)
    (format s "Mail check~%")
    (format s "  Total: ~D, New: ~D, Matched: ~D~%"
            (check-result-total result)
            (check-result-new-count result)
            (length (check-result-matched result)))
    (dolist (msg (subseq (check-result-matched result) 0
                         (min 10 (length (check-result-matched result)))))
      (format s "  ~A ~A ~A~%"
              (if (mail-message-is-read msg) " " "●")
              (email-address-address (mail-message-sender msg))
              (mail-message-subject msg)))))

;;; --- Mailbox ---

(defstruct mailbox
  (owner "" :type string)
  (messages (make-hash-table :test #'equal) :type hash-table)
  (seen-uids (make-hash-table :test #'equal) :type hash-table)
  (last-check nil))

(defun deliver (mailbox msg)
  "Deliver a message to the mailbox."
  (setf (gethash (mail-message-uid msg) (mailbox-messages mailbox)) msg))

(defun check-mail (mailbox &optional filter)
  "Check mailbox for messages, optionally filtered."
  (let ((new-count 0)
        (all-msgs nil))
    ;; Count new
    (maphash (lambda (uid msg)
               (declare (ignore msg))
               (unless (gethash uid (mailbox-seen-uids mailbox))
                 (incf new-count)
                 (setf (gethash uid (mailbox-seen-uids mailbox)) t)))
             (mailbox-messages mailbox))
    ;; Collect all
    (maphash (lambda (uid msg) (declare (ignore uid)) (push msg all-msgs))
             (mailbox-messages mailbox))
    (setf all-msgs (sort all-msgs #'> :key #'mail-message-date))
    (setf (mailbox-last-check mailbox) (get-universal-time))
    (let ((matched (if filter
                       (remove-if-not (lambda (m) (message-matches-filter m filter)) all-msgs)
                       all-msgs)))
      (make-check-result
       :total (hash-table-count (mailbox-messages mailbox))
       :new-count new-count
       :matched matched))))

(defun mark-read (mailbox uid)
  (let ((msg (gethash uid (mailbox-messages mailbox))))
    (when msg (setf (mail-message-is-read msg) t) t)))

(defun mark-flagged (mailbox uid &optional (flagged t))
  (let ((msg (gethash uid (mailbox-messages mailbox))))
    (when msg (setf (mail-message-is-flagged msg) flagged) t)))

(defun delete-message (mailbox uid)
  (when (gethash uid (mailbox-messages mailbox))
    (remhash uid (mailbox-messages mailbox))
    (remhash uid (mailbox-seen-uids mailbox))
    t))

(defun unread-count (mailbox)
  (let ((count 0))
    (maphash (lambda (uid msg)
               (declare (ignore uid))
               (unless (mail-message-is-read msg) (incf count)))
             (mailbox-messages mailbox))
    count))

(defun mailbox-stats (mailbox)
  (let ((total 0) (unread 0) (flagged 0))
    (maphash (lambda (uid msg)
               (declare (ignore uid))
               (incf total)
               (unless (mail-message-is-read msg) (incf unread))
               (when (mail-message-is-flagged msg) (incf flagged)))
             (mailbox-messages mailbox))
    (list (cons :total total) (cons :unread unread) (cons :flagged flagged))))

;;; --- Server ---

(defstruct mail-server
  (mailboxes (make-hash-table :test #'equal) :type hash-table)
  (next-uid 0 :type integer))

(defun create-mailbox (server owner)
  (let ((mb (make-mailbox :owner owner)))
    (setf (gethash owner (mail-server-mailboxes server)) mb)
    mb))

(defun send-mail (server sender recipients subject body)
  "Send a message. SENDER and each element of RECIPIENTS are email-address structs."
  (incf (mail-server-next-uid server))
  (let* ((uid (format nil "msg-~8,'0X" (mail-server-next-uid server)))
         (msg (make-mail-message
               :uid uid :sender sender :recipients recipients
               :subject subject :body body)))
    (dolist (r recipients)
      (let ((mb (gethash (email-address-address r) (mail-server-mailboxes server))))
        (when mb (deliver mb msg))))
    uid))
