;;;; WYSIWYG Editor — document model where edit and render are the same structure.
;;;;
;;;; The document is a list of RUNs, each a contiguous span sharing an attribute
;;;; set. Operations reshape runs — splitting where attrs change, merging where
;;;; adjacent runs end up identical. Positions are character offsets; runs are
;;;; the implementation detail.

(defpackage #:wysiwyg
  (:use #:cl)
  (:export #:make-document
           #:doc-length #:doc-insert #:doc-delete
           #:doc-apply-style #:doc-remove-style
           #:doc-to-plain #:doc-to-html #:doc-runs))

(in-package #:wysiwyg)

(defstruct run
  (text "" :type string)
  (attrs nil :type list))        ; sorted list of keyword attributes

(defstruct document
  (runs nil :type list))

(defun %attrs-equal (a b) (equal (sort (copy-list a) #'string<)
                                 (sort (copy-list b) #'string<)))

(defun doc-length (doc)
  (reduce #'+ (document-runs doc) :key (lambda (r) (length (run-text r)))))

(defun %split-runs (runs pos)
  "Return (values LEFT RIGHT) split at character offset POS."
  (let ((left nil) (right nil) (seen 0))
    (dolist (r runs)
      (let ((n (length (run-text r))))
        (cond
          ((>= pos (+ seen n)) (push r left))
          ((<= pos seen) (push r right))
          (t
           (let ((split (- pos seen)))
             (push (make-run :text (subseq (run-text r) 0 split)
                             :attrs (copy-list (run-attrs r))) left)
             (push (make-run :text (subseq (run-text r) split)
                             :attrs (copy-list (run-attrs r))) right))))
        (incf seen n)))
    (values (nreverse left) (nreverse right))))

(defun %normalize (runs)
  (let ((out nil))
    (dolist (r runs)
      (when (plusp (length (run-text r)))
        (if (and out (%attrs-equal (run-attrs (first out)) (run-attrs r)))
            (setf (run-text (first out))
                  (concatenate 'string (run-text (first out)) (run-text r)))
            (push (copy-run r) out))))
    (nreverse out)))

(defun doc-insert (doc pos text attrs)
  (multiple-value-bind (left right) (%split-runs (document-runs doc) pos)
    (setf (document-runs doc)
          (%normalize (append left (list (make-run :text text :attrs (copy-list attrs)))
                              right)))))

(defun doc-delete (doc start end)
  (when (< start end)
    (multiple-value-bind (left rest) (%split-runs (document-runs doc) start)
      (multiple-value-bind (_ right) (%split-runs rest (- end start))
        (declare (ignore _))
        (setf (document-runs doc) (%normalize (append left right)))))))

(defun %modify-attrs (doc start end fn)
  (when (< start end)
    (multiple-value-bind (left rest) (%split-runs (document-runs doc) start)
      (multiple-value-bind (middle right) (%split-runs rest (- end start))
        (let ((new-middle (mapcar (lambda (r)
                                    (make-run :text (run-text r)
                                              :attrs (funcall fn (run-attrs r))))
                                  middle)))
          (setf (document-runs doc)
                (%normalize (append left new-middle right))))))))

(defun doc-apply-style (doc start end attr)
  (%modify-attrs doc start end
                 (lambda (attrs) (adjoin attr attrs :test #'equal))))

(defun doc-remove-style (doc start end attr)
  (%modify-attrs doc start end
                 (lambda (attrs) (remove attr attrs :test #'equal))))

(defun doc-to-plain (doc)
  (format nil "~{~A~}" (mapcar #'run-text (document-runs doc))))

(defun %html-escape (s)
  (with-output-to-string (out)
    (loop for c across s do
      (case c
        (#\& (write-string "&amp;" out))
        (#\< (write-string "&lt;" out))
        (#\> (write-string "&gt;" out))
        (t (write-char c out))))))

(defparameter *block-attrs* '(:h1 :h2))
(defparameter *inline-attrs* '(:bold :italic :underline))
(defparameter *attr-tag* '((:h1 . "h1") (:h2 . "h2")
                            (:bold . "b") (:italic . "i") (:underline . "u")))

(defun doc-to-html (doc)
  (with-output-to-string (out)
    (dolist (r (document-runs doc))
      (let ((open nil) (close nil))
        (dolist (a *block-attrs*)
          (when (member a (run-attrs r))
            (push (format nil "<~A>" (cdr (assoc a *attr-tag*))) open)
            (push (format nil "</~A>" (cdr (assoc a *attr-tag*))) close)))
        (dolist (a *inline-attrs*)
          (when (member a (run-attrs r))
            (push (format nil "<~A>" (cdr (assoc a *attr-tag*))) open)
            (push (format nil "</~A>" (cdr (assoc a *attr-tag*))) close)))
        (format out "~{~A~}~A~{~A~}"
                (nreverse open)
                (%html-escape (run-text r))
                close)))))

(defun doc-runs (doc) (document-runs doc))
