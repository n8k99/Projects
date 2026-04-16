;;;; Text to HTML Generator — convert formatted plain text to HTML.

(defpackage :text/text-to-html
  (:use :cl)
  (:export :text-to-html))

(in-package :text/text-to-html)

(defun escape-html (s)
  "Replace &, <, > with their HTML entities."
  (let ((result s))
    (setf result (cl-ppcre:regex-replace-all "&" result "&amp;"))
    (setf result (cl-ppcre:regex-replace-all "<" result "&lt;"))
    (setf result (cl-ppcre:regex-replace-all ">" result "&gt;"))
    result))

(defun apply-inline (text)
  "Apply inline formatting: code, bold, italic, links."
  (let ((result text))
    ;; Code spans
    (setf result (cl-ppcre:regex-replace-all "`([^`]+)`" result "<code>\\1</code>"))
    ;; Bold
    (setf result (cl-ppcre:regex-replace-all "\\*\\*(.+?)\\*\\*" result "<strong>\\1</strong>"))
    ;; Italic
    (setf result (cl-ppcre:regex-replace-all "\\*(.+?)\\*" result "<em>\\1</em>"))
    ;; Links
    (setf result (cl-ppcre:regex-replace-all "\\[([^\\]]+)\\]\\(([^)]+)\\)" result "<a href=\"\\2\">\\1</a>"))
    result))

(defun split-lines (s)
  "Split string S on newline characters."
  (cl-ppcre:split "\\n" s))

(defun starts-with-p (s prefix)
  "Return T if S starts with PREFIX."
  (and (>= (length s) (length prefix))
       (string= s prefix :end1 (length prefix))))

(defun string-trim-whitespace (s)
  "Trim leading and trailing whitespace."
  (string-trim '(#\Space #\Tab #\Return) s))

(defun header-level (line)
  "Return (VALUES level content) if LINE is a header, NIL otherwise."
  (let ((trimmed (string-trim-whitespace line)))
    (when (and (> (length trimmed) 0) (char= (char trimmed 0) #\#))
      (let ((hashes (loop for i from 0 below (length trimmed)
                          while (char= (char trimmed i) #\#)
                          count t)))
        (when (and (<= hashes 3)
                   (> (length trimmed) (1+ hashes))
                   (char= (char trimmed hashes) #\Space))
          (values hashes (subseq trimmed (1+ hashes))))))))

(defun flush-paragraph (lines)
  "Join LINES into a paragraph with <br> between them."
  (when lines
    (let ((joined (format nil "~{~A~^<br>~%~}" (reverse lines))))
      (format nil "<p>~A</p>" (apply-inline joined)))))

(defun flush-list (items)
  "Wrap ITEMS in <ul><li> tags."
  (when items
    (let ((inner (format nil "~{<li>~A</li>~^~%~}"
                         (mapcar #'apply-inline (reverse items)))))
      (format nil "<ul>~%~A~%</ul>" inner))))

(defun text-to-html (text)
  "Convert formatted plain text to HTML.

Supports paragraphs, bold, italic, headers (H1-H3), links,
unordered lists, inline code, line breaks, and HTML entity escaping."
  (let ((lines (split-lines text))
        (blocks nil)
        (paragraph nil)
        (list-items nil))
    (flet ((do-flush-paragraph ()
             (let ((p (flush-paragraph paragraph)))
               (when p (push p blocks)))
             (setf paragraph nil))
           (do-flush-list ()
             (let ((l (flush-list list-items)))
               (when l (push l blocks)))
             (setf list-items nil)))
      (dolist (line lines)
        (let ((trimmed (string-trim-whitespace line)))
          (cond
            ;; Blank line
            ((string= trimmed "")
             (do-flush-list)
             (do-flush-paragraph))

            ;; Header
            ((multiple-value-bind (level content) (header-level trimmed)
               (when level
                 (do-flush-list)
                 (do-flush-paragraph)
                 (let ((escaped (escape-html content)))
                   (push (format nil "<h~D>~A</h~D>" level (apply-inline escaped) level)
                         blocks))
                 t)))

            ;; List item
            ((starts-with-p trimmed "- ")
             (do-flush-paragraph)
             (push (escape-html (subseq trimmed 2)) list-items))

            ;; Regular text
            (t
             (when list-items (do-flush-list))
             (push (escape-html trimmed) paragraph)))))

      ;; Flush remaining
      (do-flush-list)
      (do-flush-paragraph))

    (format nil "~{~A~^~%~}" (reverse blocks))))
