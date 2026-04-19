;;;; Library Catalog — three-layer identity, hierarchical subjects, FIFO holds, renewal.

(defpackage #:library-catalog
  (:use #:cl)
  (:export #:make-title
           #:title-id #:title-name #:title-authors #:title-subject-path
           #:make-copy
           #:copy-id #:copy-title-id #:copy-location #:copy-status
           #:make-patron
           #:patron-id #:patron-name
           #:make-loan
           #:loan-id #:loan-copy-id #:loan-patron-id
           #:loan-checked-out-at #:loan-due-at #:loan-returned-at
           #:loan-renewal-count
           #:loan-active-p
           #:loan-overdue-p
           #:make-hold
           #:hold-id #:hold-title-id #:hold-patron-id
           #:hold-placed-at #:hold-status #:hold-fulfilled-copy-id
           #:make-library-catalog
           #:add-title #:add-copy #:add-patron
           #:check-out
           #:return-copy
           #:renew
           #:place-hold
           #:cancel-hold
           #:hold-queue
           #:copies-of
           #:available-copies
           #:is-available
           #:search-by-subject
           #:search-by-author
           #:overdue))

(in-package #:library-catalog)

(defstruct title
  (id "" :type string)
  (name "" :type string)
  (authors nil :type list)
  (subject-path "" :type string)
  (isbn "" :type string))

(defstruct copy
  (id "" :type string)
  (title-id "" :type string)
  (location "" :type string)
  (status :available :type keyword))    ; :available :checked-out :lost :withdrawn

(defstruct patron
  (id "" :type string)
  (name "" :type string))

(defstruct loan
  (id 0 :type integer)
  (copy-id "" :type string)
  (patron-id "" :type string)
  (checked-out-at 0 :type integer)
  (due-at 0 :type integer)
  (returned-at nil :type (or null integer))
  (renewal-count 0 :type integer))

(defun loan-active-p (l) (null (loan-returned-at l)))

(defun loan-overdue-p (l now)
  (and (loan-active-p l) (> now (loan-due-at l))))

(defstruct hold
  (id 0 :type integer)
  (title-id "" :type string)
  (patron-id "" :type string)
  (placed-at 0 :type integer)
  (status :waiting :type keyword)      ; :waiting :fulfilled :cancelled
  (fulfilled-copy-id nil :type (or null string)))

(defstruct library-catalog
  (titles (make-hash-table :test 'equal))
  (copies (make-hash-table :test 'equal))
  (patrons (make-hash-table :test 'equal))
  (loans (make-hash-table :test 'eql))
  (holds (make-hash-table :test 'eql))
  (next-loan-id 1 :type integer)
  (next-hold-id 1 :type integer))

(defparameter *default-loan-days* 21)
(defparameter *max-renewals* 2)
(defparameter *seconds-per-day* 86400)

(defun add-title (cat t0)
  (when (gethash (title-id t0) (library-catalog-titles cat))
    (error "duplicate title: ~A" (title-id t0)))
  (setf (gethash (title-id t0) (library-catalog-titles cat)) t0)
  t0)

(defun add-copy (cat c)
  (unless (gethash (copy-title-id c) (library-catalog-titles cat))
    (error "unknown title: ~A" (copy-title-id c)))
  (when (gethash (copy-id c) (library-catalog-copies cat))
    (error "duplicate copy: ~A" (copy-id c)))
  (setf (gethash (copy-id c) (library-catalog-copies cat)) c)
  c)

(defun add-patron (cat p)
  (when (gethash (patron-id p) (library-catalog-patrons cat))
    (error "duplicate patron: ~A" (patron-id p)))
  (setf (gethash (patron-id p) (library-catalog-patrons cat)) p)
  p)

(defun %copy (cat id)
  (or (gethash id (library-catalog-copies cat))
      (error "unknown copy: ~A" id)))

(defun %loan (cat id)
  (or (gethash id (library-catalog-loans cat))
      (error "unknown loan: ~A" id)))

(defun check-out (cat copy-id patron-id &key (duration-days *default-loan-days*)
                                              (now (get-universal-time)))
  (unless (gethash patron-id (library-catalog-patrons cat))
    (error "unknown patron: ~A" patron-id))
  (let ((c (%copy cat copy-id)))
    (unless (eq (copy-status c) :available)
      (error "copy ~A not available (status ~A)" copy-id (copy-status c)))
    (setf (copy-status c) :checked-out)
    (let* ((id (library-catalog-next-loan-id cat))
           (l (make-loan :id id
                         :copy-id copy-id
                         :patron-id patron-id
                         :checked-out-at now
                         :due-at (+ now (* duration-days *seconds-per-day*)))))
      (setf (gethash id (library-catalog-loans cat)) l)
      (incf (library-catalog-next-loan-id cat))
      l)))

(defun %fulfill-next-hold (cat title-id copy-id)
  (let ((candidates nil))
    (maphash (lambda (k h)
               (declare (ignore k))
               (when (and (string= (hold-title-id h) title-id)
                          (eq (hold-status h) :waiting))
                 (push h candidates)))
             (library-catalog-holds cat))
    (when candidates
      (let ((first (first (sort candidates #'< :key #'hold-placed-at))))
        (setf (hold-status first) :fulfilled
              (hold-fulfilled-copy-id first) copy-id)
        first))))

(defun return-copy (cat loan-id &optional (now (get-universal-time)))
  "Return a copy. Returns (values loan fulfilled-hold-or-nil)."
  (let ((l (%loan cat loan-id)))
    (when (loan-returned-at l)
      (error "loan ~A already returned" loan-id))
    (setf (loan-returned-at l) now)
    (let* ((c (%copy cat (loan-copy-id l)))
           (title-id (copy-title-id c)))
      (setf (copy-status c) :available)
      (values l (%fulfill-next-hold cat title-id (copy-id c))))))

(defun renew (cat loan-id &key (additional-days *default-loan-days*))
  (let ((l (%loan cat loan-id)))
    (unless (loan-active-p l)
      (error "loan ~A is not active" loan-id))
    (when (>= (loan-renewal-count l) *max-renewals*)
      (error "loan ~A reached max renewals" loan-id))
    (let* ((c (%copy cat (loan-copy-id l)))
           (title-id (copy-title-id c))
           (blocked nil))
      (maphash (lambda (k h)
                 (declare (ignore k))
                 (when (and (string= (hold-title-id h) title-id)
                            (eq (hold-status h) :waiting))
                   (setf blocked t)))
               (library-catalog-holds cat))
      (when blocked
        (error "cannot renew ~A: other patrons have holds on this title" loan-id))
      (incf (loan-due-at l) (* additional-days *seconds-per-day*))
      (incf (loan-renewal-count l))
      l)))

(defun place-hold (cat title-id patron-id &optional (now (get-universal-time)))
  (unless (gethash title-id (library-catalog-titles cat))
    (error "unknown title: ~A" title-id))
  (unless (gethash patron-id (library-catalog-patrons cat))
    (error "unknown patron: ~A" patron-id))
  (let* ((id (library-catalog-next-hold-id cat))
         (h (make-hold :id id :title-id title-id :patron-id patron-id
                       :placed-at now :status :waiting)))
    (setf (gethash id (library-catalog-holds cat)) h)
    (incf (library-catalog-next-hold-id cat))
    h))

(defun cancel-hold (cat hold-id)
  (let ((h (or (gethash hold-id (library-catalog-holds cat))
               (error "unknown hold: ~A" hold-id))))
    (unless (eq (hold-status h) :waiting)
      (error "hold ~A is not waiting" hold-id))
    (setf (hold-status h) :cancelled)
    h))

(defun hold-queue (cat title-id)
  (unless (gethash title-id (library-catalog-titles cat))
    (error "unknown title: ~A" title-id))
  (let ((queue nil))
    (maphash (lambda (k h)
               (declare (ignore k))
               (when (and (string= (hold-title-id h) title-id)
                          (eq (hold-status h) :waiting))
                 (push h queue)))
             (library-catalog-holds cat))
    (sort queue #'< :key #'hold-placed-at)))

(defun copies-of (cat title-id)
  (let ((result nil))
    (maphash (lambda (id c)
               (declare (ignore id))
               (when (string= (copy-title-id c) title-id)
                 (push c result)))
             (library-catalog-copies cat))
    result))

(defun available-copies (cat title-id)
  (remove-if-not (lambda (c) (eq (copy-status c) :available))
                 (copies-of cat title-id)))

(defun is-available (cat title-id)
  (not (null (available-copies cat title-id))))

(defun %subject-prefix-match-p (path prefix)
  (or (string= path prefix)
      (and (> (length path) (length prefix))
           (string= (subseq path 0 (length prefix)) prefix)
           (char= (char path (length prefix)) #\.))))

(defun search-by-subject (cat prefix)
  (let ((result nil))
    (maphash (lambda (id t0)
               (declare (ignore id))
               (when (%subject-prefix-match-p (title-subject-path t0) prefix)
                 (push t0 result)))
             (library-catalog-titles cat))
    result))

(defun search-by-author (cat author)
  (let ((result nil))
    (maphash (lambda (id t0)
               (declare (ignore id))
               (when (member author (title-authors t0) :test #'string=)
                 (push t0 result)))
             (library-catalog-titles cat))
    result))

(defun overdue (cat &optional (now (get-universal-time)))
  (let ((result nil))
    (maphash (lambda (id l)
               (declare (ignore id))
               (when (loan-overdue-p l now)
                 (push l result)))
             (library-catalog-loans cat))
    result))
