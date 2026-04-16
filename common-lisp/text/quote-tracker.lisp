;;;; Quote Tracker — collect, categorize, and retrieve quotes.

(defpackage :text/quote-tracker
  (:use :cl)
  (:export #:quote-entry
           #:make-quote-entry
           #:quote-text
           #:quote-author
           #:quote-source
           #:quote-tags
           #:quote-tracker
           #:make-quote-tracker
           #:add-quote
           #:random-quote
           #:search-by-author
           #:search-by-tag
           #:list-authors
           #:list-tags
           #:quote-count))

(in-package :text/quote-tracker)

;;; --- Data structures ---

(defstruct quote-entry
  (text "" :type string)
  (author "" :type string)
  (source "" :type string)
  (tags nil :type list))

(defstruct quote-tracker
  (quotes nil :type list))

;;; --- Operations ---

(defun add-quote (tracker text author &key (source "") (tags nil))
  "Add a quote to the tracker. Returns the new quote."
  (let ((q (make-quote-entry :text text :author author :source source :tags tags)))
    (push q (quote-tracker-quotes tracker))
    q))

(defun random-quote (tracker)
  "Return a random quote, or NIL if the collection is empty."
  (let ((quotes (quote-tracker-quotes tracker)))
    (when quotes
      (nth (random (length quotes)) quotes))))

(defun search-by-author (tracker author)
  "Return all quotes whose author contains AUTHOR (case-insensitive)."
  (let ((needle (string-downcase author)))
    (remove-if-not
     (lambda (q)
       (search needle (string-downcase (quote-entry-author q))))
     (quote-tracker-quotes tracker))))

(defun search-by-tag (tracker tag)
  "Return all quotes that have TAG (case-insensitive)."
  (let ((needle (string-downcase tag)))
    (remove-if-not
     (lambda (q)
       (member needle (quote-entry-tags q)
               :test #'string-equal))
     (quote-tracker-quotes tracker))))

(defun list-authors (tracker)
  "Return a sorted list of unique author names."
  (sort (remove-duplicates
         (mapcar #'quote-entry-author (quote-tracker-quotes tracker))
         :test #'string=)
        #'string<))

(defun list-tags (tracker)
  "Return a sorted list of unique tags."
  (sort (remove-duplicates
         (apply #'append
                (mapcar #'quote-entry-tags (quote-tracker-quotes tracker)))
         :test #'string-equal)
        #'string<))

(defun quote-count (tracker)
  "Return total number of quotes."
  (length (quote-tracker-quotes tracker)))

;;; --- Display ---

(defmethod print-object ((q quote-entry) stream)
  (format stream "\"~A\" — ~A" (quote-entry-text q) (quote-entry-author q))
  (when (and (quote-entry-source q)
             (not (string= (quote-entry-source q) "")))
    (format stream " (~A)" (quote-entry-source q)))
  (when (quote-entry-tags q)
    (format stream " [~{~A~^, ~}]" (quote-entry-tags q))))
