;;;; Fortune Teller — randomized wisdom from categorized pools.

(defpackage :text/fortune-teller
  (:use :cl)
  (:export #:fortune-teller
           #:make-fortune-teller
           #:tell-fortune
           #:tell-fortune-by-category
           #:add-fortune
           #:ask-question
           #:fortune-count))

(in-package :text/fortune-teller)

;;; --- Data structures ---

(defstruct fortune-teller
  (pools (make-hash-table :test #'equal) :type hash-table))

;;; --- Default fortunes ---

(defun default-fortunes ()
  "Return an alist of (category . fortunes) for pre-loading."
  '(("wisdom" . ("The obstacle is the path."
                  "What you resist persists."
                  "Still water runs deep."
                  "The map is not the territory."
                  "Every expert was once a beginner."))
    ("humor" . ("You will step on a Lego in the dark tonight."
                "A closed mouth gathers no foot."
                "Today is a good day to avoid making eye contact."
                "Your socks will never match again."))
    ("warning" . ("Beware the shortcut that saves no time."
                  "Not every open door is an invitation."
                  "The comfortable path leads to the boring destination."
                  "Trust your instincts — they remember what you forgot."))
    ("motivation" . ("You are closer than you think."
                     "The best time to start was yesterday. The second best is now."
                     "Doubt kills more dreams than failure ever will."
                     "Small steps still move you forward."
                     "Discipline is choosing what you want most over what you want now."))
    ("mystery" . ("Something lost will find you when you stop looking."
                  "The answer you seek is in a room you haven't entered yet."
                  "A stranger already knows your name."
                  "The next full moon brings clarity."))))

(defun load-defaults (ft)
  "Pre-load the fortune teller with default fortunes."
  (loop for (cat . fortunes) in (default-fortunes)
        do (setf (gethash cat (fortune-teller-pools ft))
                 (copy-list fortunes)))
  ft)

;;; --- Operations ---

(defun all-fortunes (ft)
  "Gather all fortunes across all categories into a single list."
  (let ((result nil))
    (maphash (lambda (k v)
               (declare (ignore k))
               (setf result (append v result)))
             (fortune-teller-pools ft))
    result))

(defun tell-fortune (ft)
  "Return a random fortune from all categories."
  (let ((all (all-fortunes ft)))
    (if (null all)
        "The spirits are silent."
        (nth (random (length all)) all))))

(defun tell-fortune-by-category (ft category)
  "Return a random fortune from the specified category, or NIL."
  (let ((pool (gethash category (fortune-teller-pools ft))))
    (when pool
      (nth (random (length pool)) pool))))

(defun add-fortune (ft text category)
  "Add a fortune to a category pool."
  (push text (gethash category (fortune-teller-pools ft)))
  text)

(defun ask-question (ft question)
  "Acknowledge the question, then return a random fortune regardless."
  (declare (ignore question))
  (tell-fortune ft))

(defun fortune-count (ft)
  "Return the total number of fortunes across all categories."
  (let ((total 0))
    (maphash (lambda (k v)
               (declare (ignore k))
               (incf total (length v)))
             (fortune-teller-pools ft))
    total))

;;; --- Constructor with defaults ---

(defun create-fortune-teller ()
  "Create a fortune teller pre-loaded with default fortunes."
  (load-defaults (make-fortune-teller)))
