;;;; Bulk Renamer and Organizer — pattern-driven rename + preview + undo log.

(defpackage #:bulk-renamer
  (:use #:cl)
  (:export #:make-brn-directory #:brn-directory-names #:preview #:apply-ops #:undo-ops
           #:make-replace-rule #:make-prefix-rule #:make-suffix-rule
           #:make-sequence-rule #:make-rename-op
           #:rename-op-from #:rename-op-to))

(in-package #:bulk-renamer)

(defstruct rule
  (kind :replace :type keyword)
  (find "" :type string)
  (replace "" :type string)
  (prefix "" :type string)
  (suffix "" :type string)
  (template "" :type string))

(defun make-replace-rule (find replace)
  (make-rule :kind :replace :find find :replace replace))
(defun make-prefix-rule (prefix)
  (make-rule :kind :prefix :prefix prefix))
(defun make-suffix-rule (suffix)
  (make-rule :kind :suffix-before-ext :suffix suffix))
(defun make-sequence-rule (template)
  (make-rule :kind :sequence :template template))

(defstruct rename-op
  (from "" :type string)
  (to "" :type string))

(defstruct brn-directory
  (entries (make-hash-table :test 'equal) :type hash-table))

(defun brn-directory-add (d name)
  (setf (gethash name (brn-directory-entries d)) t)
  d)

(defun brn-directory-names (d)
  (sort (loop for k being the hash-keys of (brn-directory-entries d) collect k)
        #'string<))

(defun make-brn-directory-with-names (names)
  (let ((d (make-brn-directory)))
    (dolist (n names) (brn-directory-add d n))
    d))

(defun apply-rule (name rule index)
  (case (rule-kind rule)
    (:replace
     (let ((pos (search (rule-find rule) name)))
       (if pos
           (concatenate 'string
                        (subseq name 0 pos)
                        (rule-replace rule)
                        (subseq name (+ pos (length (rule-find rule)))))
           name)))
    (:prefix (concatenate 'string (rule-prefix rule) name))
    (:suffix-before-ext
     (let ((dot (position #\. name :from-end t)))
       (if dot
           (concatenate 'string (subseq name 0 dot) (rule-suffix rule)
                        (subseq name dot))
           (concatenate 'string name (rule-suffix rule)))))
    (:sequence
     (replace-all (rule-template rule) "{n}" (write-to-string (1+ index))))))

(defun replace-all (s find replace)
  (let ((pos (search find s)))
    (if pos
        (concatenate 'string (subseq s 0 pos) replace
                     (replace-all (subseq s (+ pos (length find))) find replace))
        s)))

(defun preview (d rule)
  (let ((ops nil)
        (seen-targets (make-hash-table :test 'equal)))
    (loop for name in (brn-directory-names d)
          for i from 0
          for new-name = (apply-rule name rule i)
          unless (string= new-name name)
          do (when (gethash new-name seen-targets)
               (error "collision: ~A and ~A both → ~A"
                      (gethash new-name seen-targets) name new-name))
             (setf (gethash new-name seen-targets) name)
             (push (make-rename-op :from name :to new-name) ops))
    (setf ops (nreverse ops))
    (let ((rename-sources (loop for op in ops collect (rename-op-from op))))
      (dolist (op ops)
        (when (and (gethash (rename-op-to op) (brn-directory-entries d))
                   (not (member (rename-op-to op) rename-sources :test #'string=)))
          (error "target exists: ~A" (rename-op-to op)))))
    ops))

(defun apply-ops (d ops)
  (dolist (op ops)
    (unless (gethash (rename-op-from op) (brn-directory-entries d))
      (error "from missing: ~A" (rename-op-from op))))
  (dolist (op ops)
    (remhash (rename-op-from op) (brn-directory-entries d)))
  (dolist (op ops)
    (setf (gethash (rename-op-to op) (brn-directory-entries d)) t))
  ;; Return undo log.
  (mapcar (lambda (op) (make-rename-op :from (rename-op-to op)
                                        :to (rename-op-from op)))
          ops))

(defun undo-ops (d log)
  (apply-ops d log))

(defun demo ()
  (let ((d (make-brn-directory-with-names
            '("img_01.png" "img_02.png" "img_03.png" "notes.txt"))))
    (format t "before: ~A~%" (brn-directory-names d))
    (let* ((rule (make-replace-rule "img_" "photo_"))
           (ops (preview d rule)))
      (format t "preview: ~A~%"
              (mapcar (lambda (op) (list (rename-op-from op) (rename-op-to op))) ops))
      (let ((undo-log (apply-ops d ops)))
        (format t "after: ~A~%" (brn-directory-names d))
        (undo-ops d undo-log)
        (format t "undone: ~A~%" (brn-directory-names d))))
    (format t "~%=== sequence rename ===~%")
    (let ((d2 (make-brn-directory-with-names '("track_a.wav" "track_b.wav" "track_c.wav"))))
      (let* ((rule (make-sequence-rule "song_{n}.mp3"))
             (ops (preview d2 rule)))
        (format t "preview: ~A~%"
                (mapcar (lambda (op) (list (rename-op-from op) (rename-op-to op))) ops))
        (apply-ops d2 ops)
        (format t "after: ~A~%" (brn-directory-names d2))))))
