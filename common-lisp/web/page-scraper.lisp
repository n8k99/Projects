;;;; Page Scraper — parse HTML-like markup into a tree, query with selectors.
;;;;
;;;; Forgiving parser: unknown constructs become text, stray close tags drop,
;;;; recovery at the next valid boundary. Selectors are a CSS subset: tag,
;;;; .class, #id, combinations, descendant combinator via whitespace.

(defpackage #:page-scraper
  (:use #:cl)
  (:export #:parse #:select #:text-of
           #:element-tag #:element-attrs #:element-children
           #:text-p #:element-p #:text-text))

(in-package #:page-scraper)

(defstruct element
  (tag "" :type string)
  (attrs nil :type list)           ; alist of (name . value)
  (children nil :type list))

(defstruct text
  (text "" :type string))

(defparameter *void-tags*
  '("area" "base" "br" "col" "embed" "hr" "img"
    "input" "link" "meta" "source" "track" "wbr"))

(defstruct %parser (s "" :type string) (i 0 :type integer))

(defun %eof-p (p) (>= (%parser-i p) (length (%parser-s p))))
(defun %peek (p) (and (not (%eof-p p)) (char (%parser-s p) (%parser-i p))))
(defun %starts-with (p s)
  (let ((end (+ (%parser-i p) (length s))))
    (and (<= end (length (%parser-s p)))
         (string= (subseq (%parser-s p) (%parser-i p) end) s))))

(defun %consume-while (p pred)
  (let ((start (%parser-i p)))
    (loop while (and (not (%eof-p p)) (funcall pred (%peek p)))
          do (incf (%parser-i p)))
    (subseq (%parser-s p) start (%parser-i p))))

(defun %consume-ws (p) (%consume-while p (lambda (c) (member c '(#\Space #\Tab #\Newline #\Return)))))

(defun %alnum-dash-p (c) (or (alphanumericp c) (char= c #\-)))
(defun %attr-name-char-p (c) (or (alphanumericp c) (member c '(#\- #\_))))

(defun %parse-attrs (p)
  (let ((attrs nil))
    (loop
      (%consume-ws p)
      (when (or (%eof-p p) (char= (%peek p) #\>) (char= (%peek p) #\/))
        (return attrs))
      (let ((name (string-downcase (%consume-while p #'%attr-name-char-p))))
        (cond
          ((zerop (length name))
           (incf (%parser-i p)))
          (t
           (%consume-ws p)
           (let ((value ""))
             (when (%starts-with p "=")
               (incf (%parser-i p))
               (%consume-ws p)
               (let ((q (%peek p)))
                 (cond
                   ((or (eql q #\") (eql q #\'))
                    (incf (%parser-i p))
                    (setf value (%consume-while p (lambda (c) (char/= c q))))
                    (unless (%eof-p p) (incf (%parser-i p))))
                   (t
                    (setf value (%consume-while p
                                  (lambda (c)
                                    (not (or (member c '(#\Space #\Tab #\Newline))
                                             (char= c #\>) (char= c #\/))))))))))
             (push (cons name value) attrs))))))))

(defun %parse-nodes (p parent-tag)
  (let ((out nil))
    (loop
      (when (%eof-p p) (return (nreverse out)))
      (when (and parent-tag (%starts-with p (format nil "</~A" parent-tag)))
        (return (nreverse out)))
      (cond
        ((%starts-with p "</")
         (%consume-while p (lambda (c) (char/= c #\>)))
         (unless (%eof-p p) (incf (%parser-i p))))
        ((and (eql (%peek p) #\<)
              (< (1+ (%parser-i p)) (length (%parser-s p)))
              (alpha-char-p (char (%parser-s p) (1+ (%parser-i p)))))
         (let ((el (%parse-element p)))
           (when el (push el out))))
        (t
         (let ((txt (%consume-while p (lambda (c) (char/= c #\<)))))
           (when (plusp (length txt))
             (push (make-text :text txt) out))))))))

(defun %parse-element (p)
  (incf (%parser-i p))               ; consume <
  (let ((tag (string-downcase (%consume-while p #'%alnum-dash-p))))
    (when (zerop (length tag)) (return-from %parse-element nil))
    (let ((attrs (%parse-attrs p))
          (self-closing nil))
      (cond
        ((%starts-with p "/>") (incf (%parser-i p) 2) (setf self-closing t))
        ((%starts-with p ">") (incf (%parser-i p)))
        (t (return-from %parse-element nil)))
      (when (or self-closing (member tag *void-tags* :test #'string=))
        (return-from %parse-element
          (make-element :tag tag :attrs attrs :children nil)))
      (let ((children (%parse-nodes p tag)))
        (when (%starts-with p (format nil "</~A" tag))
          (%consume-while p (lambda (c) (char/= c #\>)))
          (unless (%eof-p p) (incf (%parser-i p))))
        (make-element :tag tag :attrs attrs :children children)))))

(defun parse (html)
  (%parse-nodes (make-%parser :s html) nil))

(defstruct %simple (tag nil) (cls nil) (id nil))

(defun %parse-simple (s)
  (let ((simp (make-%simple)) (buf "") (kind #\space))
    (flet ((flush ()
             (unless (zerop (length buf))
               (case kind
                 (#\. (setf (%simple-cls simp) buf))
                 (#\# (setf (%simple-id simp) buf))
                 (t (setf (%simple-tag simp) (string-downcase buf))))
               (setf buf ""))))
      (loop for c across s do
        (cond
          ((or (char= c #\.) (char= c #\#)) (flush) (setf kind c))
          (t (setf buf (concatenate 'string buf (string c))))))
      (flush))
    simp))

(defun %matches (simp el)
  (and (or (null (%simple-tag simp))
           (string= (%simple-tag simp) (element-tag el)))
       (or (null (%simple-id simp))
           (let ((id (cdr (assoc "id" (element-attrs el) :test #'string=))))
             (and id (string= id (%simple-id simp)))))
       (or (null (%simple-cls simp))
           (let ((cls (cdr (assoc "class" (element-attrs el) :test #'string=))))
             (and cls
                  (find (%simple-cls simp) (split-ws cls) :test #'string=))))))

(defun split-ws (s)
  (loop with parts = nil
        with start = nil
        for i from 0 below (length s)
        for c = (char s i)
        do (cond
             ((or (char= c #\Space) (char= c #\Tab))
              (when start (push (subseq s start i) parts) (setf start nil)))
             (t (unless start (setf start i))))
        finally (when start (push (subseq s start) parts))
                (return (nreverse parts))))

(defun select (nodes selector)
  (let ((chain (mapcar #'%parse-simple (split-ws selector)))
        (out nil))
    (labels ((rec (ns idx)
               (dolist (n ns)
                 (when (element-p n)
                   (let ((match (%matches (nth idx chain) n)))
                     (cond
                       ((and match (= idx (1- (length chain))))
                        (push n out)
                        (rec (element-children n) idx))
                       (match (rec (element-children n) (1+ idx)))
                       (t (rec (element-children n) idx))))))))
      (rec nodes 0)
      (nreverse out))))

(defun text-of (node)
  (cond
    ((text-p node) (text-text node))
    ((element-p node)
     (apply #'concatenate 'string (mapcar #'text-of (element-children node))))
    (t "")))
