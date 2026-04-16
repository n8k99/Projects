;;;; Regex Query Tool — pattern matching, extraction, and replacement.

(defpackage :text/regex-query
  (:use :cl)
  (:export :find-matches
           :find-with-positions
           :regex-replace
           :regex-split
           :is-match
           :extract-groups
           :+email-pattern+
           :+url-pattern+
           :+phone-pattern+
           :+date-pattern+
           :+ip-address-pattern+
           :find-emails
           :find-urls
           :find-dates
           :find-phones
           :find-ip-addresses))

(in-package :text/regex-query)

;;; Pre-built patterns for common structured data.

(defparameter +email-pattern+ "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}")
(defparameter +url-pattern+ "https?://[^\\s<>\"']+")
(defparameter +phone-pattern+ "\\+?1?[-.\\s]?\\(?\\d{3}\\)?[-.\\s]?\\d{3}[-.\\s]?\\d{4}")
(defparameter +date-pattern+ "\\d{4}-\\d{2}-\\d{2}")
(defparameter +ip-address-pattern+ "\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b")

;;; Core operations.

(defun find-matches (text pattern)
  "Return all substrings of TEXT matching PATTERN."
  (cl-ppcre:all-matches-as-strings pattern text))

(defun find-with-positions (text pattern)
  "Return matches with positions as list of (match start end) triples."
  (let ((results '()))
    (cl-ppcre:do-scans (match-start match-end reg-starts reg-ends
                        (cl-ppcre:create-scanner pattern) text
                        (nreverse results))
      (push (list (subseq text match-start match-end)
                  match-start
                  match-end)
            results))))

(defun regex-replace (text pattern replacement)
  "Replace all occurrences of PATTERN in TEXT with REPLACEMENT."
  (cl-ppcre:regex-replace-all pattern text replacement))

(defun regex-split (text pattern)
  "Split TEXT on every occurrence of PATTERN."
  (cl-ppcre:split pattern text))

(defun is-match (text pattern)
  "Return T if PATTERN matches anywhere in TEXT."
  (not (null (cl-ppcre:scan pattern text))))

(defun extract-groups (text pattern)
  "Return capture-group lists for every match of PATTERN in TEXT.
Each element is a list of the captured group strings."
  (let ((results '()))
    (cl-ppcre:do-scans (match-start match-end reg-starts reg-ends
                        (cl-ppcre:create-scanner pattern) text
                        (nreverse results))
      (let ((groups '()))
        (loop for i from (1- (length reg-starts)) downto 0
              do (push (if (aref reg-starts i)
                           (subseq text (aref reg-starts i) (aref reg-ends i))
                           "")
                       groups))
        (push groups results)))))

;;; Convenience wrappers for pre-built patterns.

(defun find-emails (text)
  "Extract email addresses from TEXT."
  (find-matches text +email-pattern+))

(defun find-urls (text)
  "Extract URLs from TEXT."
  (find-matches text +url-pattern+))

(defun find-dates (text)
  "Extract YYYY-MM-DD dates from TEXT."
  (find-matches text +date-pattern+))

(defun find-phones (text)
  "Extract phone numbers from TEXT."
  (find-matches text +phone-pattern+))

(defun find-ip-addresses (text)
  "Extract IPv4 addresses from TEXT."
  (find-matches text +ip-address-pattern+))
