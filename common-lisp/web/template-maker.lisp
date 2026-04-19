;;;; Template Maker — meta-level tool for building and validating templates.
;;;; Loads G081's ecard package for Slot reuse.

(load "/home/n8k99/Development/projects/common-lisp/web/ecard.lisp")

(defpackage #:template-maker
  (:use #:cl)
  (:export #:make-maker #:new-template
           #:add-slot #:remove-slot #:update-html #:set-sample
           #:validate #:preview
           #:maker-builder #:builder-id #:builder-name
           #:builder-slots #:builder-html-draft #:builder-sample-data
           #:issue-kind #:issue-name))

(in-package #:template-maker)

(defstruct builder
  (id 0 :type integer)
  (name "" :type string)
  (category "" :type string)
  (slots nil :type list)         ; list of ecard:slot
  (html-draft "" :type string)
  (sample-data nil :type list))  ; alist (name . value)

(defstruct issue
  (kind :unused :type keyword)
  (name "" :type string))

(defstruct maker
  (builders nil :type list)
  (next-id 1 :type integer))

(define-condition unknown-builder (error)
  ((id :initarg :id :reader err-id))
  (:report (lambda (c s) (format s "unknown builder: ~A" (err-id c)))))
(define-condition slot-already-exists (error)
  ((name :initarg :name :reader err-name))
  (:report (lambda (c s) (format s "slot already exists: ~A" (err-name c)))))
(define-condition slot-not-found (error)
  ((name :initarg :name :reader err-name))
  (:report (lambda (c s) (format s "slot not found: ~A" (err-name c)))))

(defun new-template (m name category)
  (let ((id (maker-next-id m)))
    (incf (maker-next-id m))
    (setf (maker-builders m)
          (append (maker-builders m)
                  (list (make-builder :id id :name name :category category))))
    id))

(defun maker-builder (m id) (find id (maker-builders m) :key #'builder-id))

(defun %require (m id)
  (or (maker-builder m id) (error 'unknown-builder :id id)))

(defun add-slot (m id slot)
  (let ((b (%require m id)))
    (when (find (ecard::slot-name slot) (builder-slots b)
                :key #'ecard::slot-name :test #'string=)
      (error 'slot-already-exists :name (ecard::slot-name slot)))
    (setf (builder-slots b) (append (builder-slots b) (list slot)))))

(defun remove-slot (m id slot-name)
  (let* ((b (%require m id))
         (pos (position slot-name (builder-slots b)
                        :key #'ecard::slot-name :test #'string=)))
    (unless pos (error 'slot-not-found :name slot-name))
    (setf (builder-slots b)
          (append (subseq (builder-slots b) 0 pos)
                  (subseq (builder-slots b) (1+ pos))))
    (setf (builder-sample-data b)
          (remove slot-name (builder-sample-data b) :key #'car :test #'string=))))

(defun update-html (m id html)
  (setf (builder-html-draft (%require m id)) html))

(defun set-sample (m id slot-name value)
  (let ((b (%require m id)))
    (unless (find slot-name (builder-slots b)
                  :key #'ecard::slot-name :test #'string=)
      (error 'slot-not-found :name slot-name))
    (let ((existing (assoc slot-name (builder-sample-data b) :test #'string=)))
      (if existing
          (setf (cdr existing) value)
          (push (cons slot-name value) (builder-sample-data b))))))

(defun %extract-placeholders (html)
  (let ((out nil) (len (length html)) (i 0))
    (loop while (< i (1- len)) do
      (cond
        ((and (char= (char html i) #\{) (char= (char html (1+ i)) #\{))
         (let ((start (+ i 2)) (j (+ i 2)) (found nil))
           (loop while (and (< j (1- len)) (not found)) do
             (if (and (char= (char html j) #\})
                      (char= (char html (1+ j)) #\}))
                 (setf found t)
                 (incf j)))
           (cond
             (found
              (let ((name (subseq html start j)))
                (when (plusp (length name))
                  (pushnew name out :test #'string=)))
              (setf i (+ j 2)))
             (t (incf i)))))
        (t (incf i))))
    (nreverse out)))

(defun %valid-slot-name-p (name)
  (and (plusp (length name))
       (every (lambda (c)
                (or (alphanumericp c) (char= c #\_)))
              name)))

(defun validate (m id)
  (let ((b (maker-builder m id)))
    (unless b (return-from validate nil))
    (let ((issues nil) (seen nil))
      ;; Invalid / duplicate slot names
      (dolist (slot (builder-slots b))
        (unless (%valid-slot-name-p (ecard::slot-name slot))
          (push (make-issue :kind :invalid-slot-name :name (ecard::slot-name slot)) issues))
        (if (find (ecard::slot-name slot) seen :test #'string=)
            (push (make-issue :kind :duplicate-slot :name (ecard::slot-name slot)) issues)
            (push (ecard::slot-name slot) seen)))
      (let ((declared (mapcar #'ecard::slot-name (builder-slots b)))
            (referenced (%extract-placeholders (builder-html-draft b))))
        ;; Unused slot
        (dolist (slot (builder-slots b))
          (unless (find (ecard::slot-name slot) referenced :test #'string=)
            (push (make-issue :kind :unused-slot :name (ecard::slot-name slot)) issues)))
        ;; Undeclared placeholder (skip _-prefixed)
        (dolist (ref referenced)
          (unless (or (char= (char ref 0) #\_)
                      (find ref declared :test #'string=))
            (push (make-issue :kind :undeclared-placeholder :name ref) issues))))
      ;; Missing sample for required
      (dolist (slot (builder-slots b))
        (when (and (ecard::slot-required slot)
                   (not (assoc (ecard::slot-name slot) (builder-sample-data b) :test #'string=)))
          (push (make-issue :kind :missing-sample :name (ecard::slot-name slot)) issues)))
      (nreverse issues))))

(defun %replace-all (text needle replacement)
  (let ((start 0) (out "") (n (length needle)))
    (loop for pos = (search needle text :start2 start)
          while pos do
            (setf out (concatenate 'string out (subseq text start pos) replacement)
                  start (+ pos n)))
    (concatenate 'string out (subseq text start))))

(defun preview (m id)
  (let ((b (maker-builder m id)))
    (unless b (return-from preview nil))
    (let ((out (builder-html-draft b)))
      (dolist (slot (builder-slots b))
        (let* ((sample (cdr (assoc (ecard::slot-name slot) (builder-sample-data b) :test #'string=)))
               (value (or sample (ecard::slot-default slot))))
          (when value
            (setf out (%replace-all out (format nil "{{~A}}" (ecard::slot-name slot)) value)))))
      (setf out (%replace-all out "{{_recipient}}" "[preview recipient]"))
      (setf out (%replace-all out "{{_sender}}" "[preview sender]"))
      out)))
