;;;; Slide Show — presentation navigation with dual-track content.

(defpackage #:slide-show
  (:use #:cl)
  (:export #:make-slide #:make-presentation
           #:slide-heading #:slide-bullet #:slide-image #:slide-code #:slide-with-notes
           #:presentation-add-slide #:presentation-slide-count
           #:presentation-current #:presentation-advance #:presentation-back
           #:presentation-goto #:presentation-progress
           #:render-current #:render-handout
           #:presentation-search #:presentation-transition-stats))

(in-package #:slide-show)

(defstruct slide-element
  (kind :heading :type keyword)
  (text "" :type string)
  (level 1 :type integer)
  (indent 0 :type integer)
  (src "" :type string)
  (alt "" :type string)
  (language "" :type string)
  (lines 0 :type integer))

(defstruct slide
  (title "" :type string)
  (content nil :type list)
  (notes "" :type string)
  (transition :none :type keyword))

(defun slide-heading (s text level)
  (setf (slide-content s)
        (append (slide-content s)
                (list (make-slide-element :kind :heading :text text :level level))))
  s)

(defun slide-bullet (s text indent)
  (setf (slide-content s)
        (append (slide-content s)
                (list (make-slide-element :kind :bullet :text text :indent indent))))
  s)

(defun slide-image (s src alt)
  (setf (slide-content s)
        (append (slide-content s)
                (list (make-slide-element :kind :image :src src :alt alt))))
  s)

(defun slide-code (s text language)
  (setf (slide-content s)
        (append (slide-content s)
                (list (make-slide-element :kind :code :text text :language language))))
  s)

(defun slide-with-notes (s notes)
  (setf (slide-notes s) notes)
  s)

(defstruct presentation
  (title "" :type string)
  (slides nil :type list)
  (current-index 0 :type integer))

(defun presentation-add-slide (p s)
  (setf (presentation-slides p) (append (presentation-slides p) (list s))))

(defun presentation-slide-count (p)
  (length (presentation-slides p)))

(defun presentation-current (p)
  (when (and (presentation-slides p)
             (< (presentation-current-index p) (length (presentation-slides p))))
    (nth (presentation-current-index p) (presentation-slides p))))

(defun presentation-advance (p)
  (cond ((< (1+ (presentation-current-index p))
            (length (presentation-slides p)))
         (incf (presentation-current-index p)) t)
        (t nil)))

(defun presentation-back (p)
  (cond ((> (presentation-current-index p) 0)
         (decf (presentation-current-index p)) t)
        (t nil)))

(defun presentation-goto (p index)
  (cond ((and (>= index 0) (< index (length (presentation-slides p))))
         (setf (presentation-current-index p) index) t)
        (t nil)))

(defun presentation-progress (p)
  (if (null (presentation-slides p))
      '(0 0)
      (list (1+ (presentation-current-index p)) (length (presentation-slides p)))))

(defun element-text (e)
  (case (slide-element-kind e)
    (:heading (slide-element-text e))
    (:bullet (slide-element-text e))
    (:image (slide-element-alt e))
    (:code (slide-element-text e))
    (t "")))

(defun render-element (e)
  (case (slide-element-kind e)
    (:heading
     (format nil "~A ~A"
             (make-string (max 1 (min 6 (slide-element-level e)))
                           :initial-element #\#)
             (slide-element-text e)))
    (:bullet
     (format nil "~A- ~A"
             (make-string (* 2 (slide-element-indent e))
                           :initial-element #\Space)
             (slide-element-text e)))
    (:image
     (format nil "![~A](~A)" (slide-element-alt e) (slide-element-src e)))
    (:code
     (format nil "```~A~%~A~%```"
             (slide-element-language e) (slide-element-text e)))
    (:spacer (make-string (slide-element-lines e) :initial-element #\Newline))))

(defun render-slide (slide mode)
  (with-output-to-string (out)
    (format out "~A~%" (slide-title slide))
    (format out "~A~%"
            (make-string (min 80 (length (slide-title slide)))
                          :initial-element #\=))
    (dolist (e (slide-content slide))
      (format out "~A~%" (render-element e)))
    (when (and (or (eq mode :speaker) (eq mode :handout))
               (> (length (slide-notes slide)) 0))
      (format out "~%[Speaker notes]~%~A~%" (slide-notes slide)))))

(defun render-current (p mode)
  (let ((s (presentation-current p)))
    (if s (render-slide s mode) "")))

(defun render-handout (p)
  (with-output-to-string (out)
    (format out "# ~A~%~%" (presentation-title p))
    (loop for s in (presentation-slides p)
          for i from 1
          do (format out "--- Slide ~D ---~%" i)
             (format out "~A~%" (render-slide s :handout)))))

(defun string-contains-ci (haystack needle)
  (search (string-downcase needle) (string-downcase haystack)))

(defun presentation-search (p needle)
  (loop for s in (presentation-slides p)
        for i from 0
        when (or (string-contains-ci (slide-title s) needle)
                 (some (lambda (e) (string-contains-ci (element-text e) needle))
                        (slide-content s)))
        collect i))

(defun presentation-transition-stats (p)
  (let ((out nil))
    (dolist (s (presentation-slides p))
      (let ((k (string-downcase (symbol-name (slide-transition s)))))
        (let ((pair (assoc k out :test #'string=)))
          (if pair (incf (cdr pair))
              (push (cons k 1) out)))))
    (nreverse out)))

(defun demo ()
  (let ((p (make-presentation :title "My Talk")))
    (presentation-add-slide p
      (slide-with-notes
       (slide-bullet
        (slide-bullet
         (slide-heading (make-slide :title "Introduction") "Welcome" 1)
         "Agenda" 0)
        "Items" 1)
       "Remember to pause for laughs."))
    (presentation-add-slide p
      (slide-with-notes
       (slide-bullet
        (slide-bullet
         (slide-heading (make-slide :title "The Problem") "Pain points" 1)
         "Thing one" 0)
        "Thing two" 0)
       "Be careful not to get political."))
    (format t "=== audience view ===~%~A" (render-current p :audience))
    (format t "=== speaker view ===~%~A" (render-current p :speaker))
    (presentation-advance p)
    (format t "~%progress: ~A~%" (presentation-progress p))
    (format t "current: ~A~%" (slide-title (presentation-current p)))
    (format t "~%search 'pain': slides ~A~%" (presentation-search p "pain"))))
