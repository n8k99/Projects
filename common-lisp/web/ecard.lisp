;;;; E-Card Generator — template + data rendering with slot validation.

(defpackage #:ecard
  (:use #:cl)
  (:export #:make-slot #:make-gallery
           #:add-template #:create-card
           #:render #:render-plain
           #:gallery-templates #:templates-in-category
           #:card-by-id #:cards-by-template
           #:template-id #:template-name #:template-slots
           #:card-id #:card-filled))

(in-package #:ecard)

(defstruct slot
  (name "" :type string)
  (kind :text :type keyword)          ; :text :long-text :image-url :color :date
  (required nil :type boolean)
  (default nil :type (or null string)))

(defstruct template
  (id 0 :type integer)
  (name "" :type string)
  (category "" :type string)
  (slots nil :type list)
  (html-template "" :type string))

(defstruct card
  (id 0 :type integer)
  (template-id 0 :type integer)
  (filled nil :type list)              ; alist (name . value)
  (recipient "" :type string)
  (sender "" :type string)
  (created-ms 0 :type integer))

(defstruct gallery
  (templates nil :type list)
  (cards nil :type list)
  (next-template-id 1 :type integer)
  (next-card-id 1 :type integer))

(define-condition validation-error (error) ())
(define-condition unknown-template (validation-error)
  ((id :initarg :id :reader err-id))
  (:report (lambda (c s) (format s "unknown template: ~A" (err-id c)))))
(define-condition missing-required (validation-error)
  ((slot-name :initarg :slot-name :reader err-slot))
  (:report (lambda (c s) (format s "missing required slot: ~A" (err-slot c)))))
(define-condition unknown-slot (validation-error)
  ((slot-name :initarg :slot-name :reader err-slot))
  (:report (lambda (c s) (format s "unknown slot: ~A" (err-slot c)))))

(defun add-template (g name category slots html-template)
  (let ((id (gallery-next-template-id g)))
    (incf (gallery-next-template-id g))
    (setf (gallery-templates g)
          (append (gallery-templates g)
                  (list (make-template :id id :name name :category category
                                        :slots slots :html-template html-template))))
    id))

(defun %find-template (g id)
  (find id (gallery-templates g) :key #'template-id))

(defun templates-in-category (g category)
  (remove-if-not (lambda (t1) (string= (template-category t1) category))
                 (gallery-templates g)))

(defun create-card (g template-id filled recipient sender now-ms)
  (let ((template (%find-template g template-id)))
    (unless template (error 'unknown-template :id template-id))
    (let ((known (mapcar #'slot-name (template-slots template))))
      ;; Unknown slot?
      (dolist (pair filled)
        (unless (member (car pair) known :test #'string=)
          (error 'unknown-slot :slot-name (car pair))))
      ;; Missing required?
      (dolist (sl (template-slots template))
        (when (and (slot-required sl) (not (assoc (slot-name sl) filled :test #'string=)))
          (error 'missing-required :slot-name (slot-name sl))))
      ;; Apply defaults.
      (let ((final filled))
        (dolist (sl (template-slots template))
          (when (and (not (assoc (slot-name sl) final :test #'string=))
                     (slot-default sl))
            (push (cons (slot-name sl) (slot-default sl)) final)))
        (let ((id (gallery-next-card-id g)))
          (incf (gallery-next-card-id g))
          (setf (gallery-cards g)
                (append (gallery-cards g)
                        (list (make-card :id id :template-id template-id
                                          :filled final :recipient recipient
                                          :sender sender :created-ms now-ms))))
          id)))))

(defun card-by-id (g id) (find id (gallery-cards g) :key #'card-id))
(defun cards-by-template (g tid)
  (remove-if-not (lambda (c) (= (card-template-id c) tid)) (gallery-cards g)))

(defun %replace-all (text needle replacement)
  (let ((start 0) (out "") (n (length needle)))
    (loop for pos = (search needle text :start2 start)
          while pos do
            (setf out (concatenate 'string out (subseq text start pos) replacement)
                  start (+ pos n)))
    (concatenate 'string out (subseq text start))))

(defun render (g card-id)
  (let ((c (card-by-id g card-id)))
    (unless c (return-from render nil))
    (let ((template (%find-template g (card-template-id c))))
      (unless template (return-from render nil))
      (let ((out (template-html-template template)))
        (dolist (pair (card-filled c))
          (setf out (%replace-all out
                                   (format nil "{{~A}}" (car pair))
                                   (cdr pair))))
        (setf out (%replace-all out "{{_recipient}}" (card-recipient c)))
        (setf out (%replace-all out "{{_sender}}" (card-sender c)))
        out))))

(defun %strip-tags (s)
  (let ((out "") (in-tag nil))
    (loop for c across s do
      (cond
        ((char= c #\<) (setf in-tag t))
        ((char= c #\>) (setf in-tag nil))
        ((not in-tag) (setf out (concatenate 'string out (string c))))))
    out))

(defun render-plain (g card-id)
  (let ((html (render g card-id)))
    (when html (%strip-tags html))))
