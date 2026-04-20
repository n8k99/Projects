;;;; PDF Generator — document pagination via block flow layout.

(defpackage #:pdf-generator
  (:use #:cl)
  (:export #:make-document #:heading #:paragraph #:spacer #:page-break
           #:render #:page-count
           #:page-number #:page-lines #:wrap-text
           #:rendered-line-text #:rendered-line-y #:rendered-line-is-heading))

(in-package #:pdf-generator)

(defstruct block-data
  (kind :paragraph :type keyword)
  (text "" :type string)
  (level 1 :type integer)
  (height 0 :type integer))

(defstruct document
  (width-chars 80 :type integer)
  (height-lines 66 :type integer)
  (blocks nil :type list))

(defstruct rendered-line
  (text "" :type string)
  (y 0 :type integer)
  (is-heading nil :type boolean))

(defstruct page
  (lines nil :type list)
  (number 1 :type integer))

(defun heading (doc text &optional (level 1))
  (setf (document-blocks doc)
        (append (document-blocks doc)
                (list (make-block-data :kind :heading :text text :level level))))
  doc)

(defun paragraph (doc text)
  (setf (document-blocks doc)
        (append (document-blocks doc)
                (list (make-block-data :kind :paragraph :text text))))
  doc)

(defun spacer (doc height)
  (setf (document-blocks doc)
        (append (document-blocks doc)
                (list (make-block-data :kind :spacer :height height))))
  doc)

(defun page-break (doc)
  (setf (document-blocks doc)
        (append (document-blocks doc)
                (list (make-block-data :kind :page-break))))
  doc)

(defun split-words (s)
  (let ((words nil)
        (current (make-array 0 :element-type 'character
                               :adjustable t :fill-pointer 0)))
    (loop for c across s do
          (if (or (char= c #\Space) (char= c #\Newline) (char= c #\Tab))
              (when (> (length current) 0)
                (push (copy-seq current) words)
                (setf (fill-pointer current) 0))
              (vector-push-extend c current)))
    (when (> (length current) 0)
      (push (copy-seq current) words))
    (mapcar (lambda (w) (coerce w 'string)) (nreverse words))))

(defun wrap-text (text width)
  (let ((lines nil) (current ""))
    (dolist (word (split-words text))
      (cond
        ((zerop (length current)) (setf current word))
        ((<= (+ (length current) 1 (length word)) width)
         (setf current (concatenate 'string current " " word)))
        (t (push current lines)
           (setf current word))))
    (when (> (length current) 0)
      (push current lines))
    (if lines (nreverse lines) (list ""))))

(defun render-heading (text level)
  (case level
    (1 (format nil "# ~A" text))
    (2 (format nil "## ~A" text))
    (t (format nil "### ~A" text))))

(defun render (doc)
  (let ((pages nil)
        (current (make-page :number 1))
        (y 0))
    (labels ((push-page ()
               (when (page-lines current)
                 (push current pages)
                 (setf current (make-page :number (1+ (page-number current))))
                 (setf y 0))))
      (dolist (block (document-blocks doc))
        (case (block-data-kind block)
          (:page-break (push-page))
          (:spacer
           (if (> (+ y (block-data-height block)) (document-height-lines doc))
               (push-page)
               (incf y (block-data-height block))))
          (:heading
           (when (> (+ y 2) (document-height-lines doc))
             (push-page))
           (setf (page-lines current)
                 (append (page-lines current)
                         (list (make-rendered-line
                                :text (render-heading (block-data-text block)
                                                       (block-data-level block))
                                :y y :is-heading t))))
           (incf y 2))
          (:paragraph
           (dolist (line (wrap-text (block-data-text block) (document-width-chars doc)))
             (when (>= y (document-height-lines doc))
               (push-page))
             (setf (page-lines current)
                   (append (page-lines current)
                           (list (make-rendered-line :text line :y y :is-heading nil))))
             (incf y))
           (when (< y (document-height-lines doc))
             (incf y)))))
      (when (page-lines current) (push current pages))
      (nreverse pages))))

(defun page-count (doc)
  (length (render doc)))

(defun demo ()
  (let ((doc (make-document :width-chars 40 :height-lines 10)))
    (heading doc "Rosetta Stone" 1)
    (paragraph doc
               (concatenate 'string
                            "The quick brown fox jumps over the lazy dog. "
                            "Pack my box with five dozen liquor jugs. "
                            "How vexingly quick daft zebras jump."))
    (heading doc "Section 1" 2)
    (paragraph doc "Short second paragraph about layout.")
    (page-break doc)
    (heading doc "New Page" 1)
    (paragraph doc "Content that starts on a fresh page.")
    (let ((pages (render doc)))
      (format t "=== ~D page(s) rendered ===~%" (length pages))
      (dolist (p pages)
        (format t "--- Page ~D ---~%" (page-number p))
        (dolist (line (page-lines p))
          (format t "  [~A] y=~2D  ~A~%"
                  (if (rendered-line-is-heading line) "H" " ")
                  (rendered-line-y line)
                  (rendered-line-text line)))
        (format t "~%")))))
