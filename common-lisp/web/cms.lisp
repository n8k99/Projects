;;;; Content Management System — lifecycle FSM + revisions + scheduled publish.

(defpackage #:cms
  (:use #:cl)
  (:export #:make-cms
           #:create-article #:save-revision
           #:submit-for-review #:approve-and-publish
           #:schedule-publish #:archive-article #:unarchive-article
           #:tick #:revert-to-revision
           #:set-article-tags
           #:article-by-id #:article-by-slug #:articles
           #:by-state #:by-tag #:by-author
           #:revisions-of
           #:article-id #:article-slug #:article-title #:article-body
           #:article-state #:article-author #:article-tags
           #:article-published-at-ms #:article-scheduled-publish-at-ms
           #:revision-version #:revision-body #:revision-title))

(in-package #:cms)

(defstruct article
  (id 0 :type integer)
  (slug "" :type string)
  (title "" :type string)
  (body "" :type string)
  (author "" :type string)
  (state :draft :type keyword)
  (tags nil :type list)
  (created-ms 0 :type integer)
  (modified-ms 0 :type integer)
  (scheduled-publish-at-ms nil :type (or null integer))
  (published-at-ms nil :type (or null integer)))

(defstruct revision
  (article-id 0 :type integer)
  (version 0 :type integer)
  (title "" :type string)
  (body "" :type string)
  (saved-at-ms 0 :type integer)
  (saved-by "" :type string))

(defstruct cms
  (articles nil :type list)
  (revisions (make-hash-table :test 'eql))
  (next-id 1 :type integer))

(define-condition cms-error (error) ())
(define-condition unknown-article (cms-error)
  ((id :initarg :id :reader err-id))
  (:report (lambda (c s) (format s "unknown article: ~A" (err-id c)))))
(define-condition unknown-revision (cms-error)
  ((id :initarg :id :reader err-id) (ver :initarg :ver :reader err-ver))
  (:report (lambda (c s) (format s "unknown revision: ~A v~A" (err-id c) (err-ver c)))))
(define-condition illegal-transition (cms-error)
  ((from :initarg :from :reader err-from)
   (action :initarg :action :reader err-action))
  (:report (lambda (c s) (format s "illegal ~A from ~A" (err-action c) (err-from c)))))
(define-condition archived-is-frozen (cms-error) ())
(define-condition slug-taken (cms-error)
  ((slug :initarg :slug :reader err-slug))
  (:report (lambda (c s) (format s "slug taken: ~A" (err-slug c)))))

(defun %require (cms id)
  (or (find id (cms-articles cms) :key #'article-id)
      (error 'unknown-article :id id)))

(defun create-article (cms author slug title body now-ms)
  (when (find slug (cms-articles cms) :key #'article-slug :test #'string=)
    (error 'slug-taken :slug slug))
  (let* ((id (cms-next-id cms))
         (a (make-article :id id :slug slug :title title :body body
                          :author author :state :draft
                          :created-ms now-ms :modified-ms now-ms)))
    (incf (cms-next-id cms))
    (setf (cms-articles cms) (append (cms-articles cms) (list a)))
    (setf (gethash id (cms-revisions cms))
          (list (make-revision :article-id id :version 1
                               :title title :body body
                               :saved-at-ms now-ms :saved-by author)))
    id))

(defun save-revision (cms id user title body now-ms)
  (let ((a (%require cms id)))
    (when (eq (article-state a) :archived) (error 'archived-is-frozen))
    (setf (article-title a) title (article-body a) body
          (article-modified-ms a) now-ms)
    (let* ((revs (gethash id (cms-revisions cms)))
           (version (1+ (if revs (revision-version (car (last revs))) 0)))
           (new-rev (make-revision :article-id id :version version
                                    :title title :body body
                                    :saved-at-ms now-ms :saved-by user)))
      (setf (gethash id (cms-revisions cms)) (append revs (list new-rev)))
      version)))

(defun submit-for-review (cms id now-ms)
  (let ((a (%require cms id)))
    (unless (eq (article-state a) :draft)
      (error 'illegal-transition :from (article-state a) :action "submit-for-review"))
    (setf (article-state a) :in-review (article-modified-ms a) now-ms)))

(defun approve-and-publish (cms id now-ms)
  (let ((a (%require cms id)))
    (unless (member (article-state a) '(:draft :in-review :scheduled))
      (error 'illegal-transition :from (article-state a) :action "approve-and-publish"))
    (setf (article-state a) :published
          (article-published-at-ms a) now-ms
          (article-modified-ms a) now-ms
          (article-scheduled-publish-at-ms a) nil)))

(defun schedule-publish (cms id at-ms now-ms)
  (let ((a (%require cms id)))
    (unless (member (article-state a) '(:draft :in-review :scheduled))
      (error 'illegal-transition :from (article-state a) :action "schedule-publish"))
    (setf (article-state a) :scheduled
          (article-scheduled-publish-at-ms a) at-ms
          (article-modified-ms a) now-ms)))

(defun archive-article (cms id now-ms)
  (let ((a (%require cms id)))
    (when (eq (article-state a) :archived)
      (error 'illegal-transition :from :archived :action "archive"))
    (setf (article-state a) :archived (article-modified-ms a) now-ms)))

(defun unarchive-article (cms id now-ms)
  (let ((a (%require cms id)))
    (unless (eq (article-state a) :archived)
      (error 'illegal-transition :from (article-state a) :action "unarchive"))
    (setf (article-state a) :draft (article-modified-ms a) now-ms)))

(defun tick (cms now-ms)
  (let ((published nil))
    (dolist (a (cms-articles cms))
      (when (and (eq (article-state a) :scheduled)
                 (article-scheduled-publish-at-ms a)
                 (<= (article-scheduled-publish-at-ms a) now-ms))
        (setf (article-state a) :published
              (article-published-at-ms a) now-ms
              (article-modified-ms a) now-ms
              (article-scheduled-publish-at-ms a) nil)
        (push (article-id a) published)))
    (nreverse published)))

(defun revert-to-revision (cms id version user now-ms)
  (let* ((a (%require cms id))
         (revs (gethash id (cms-revisions cms)))
         (target (find version revs :key #'revision-version)))
    (when (eq (article-state a) :archived) (error 'archived-is-frozen))
    (unless target (error 'unknown-revision :id id :ver version))
    (setf (article-title a) (revision-title target)
          (article-body a) (revision-body target)
          (article-modified-ms a) now-ms)
    (let* ((new-version (1+ (if revs (revision-version (car (last revs))) 0)))
           (new-rev (make-revision :article-id id :version new-version
                                    :title (revision-title target)
                                    :body (revision-body target)
                                    :saved-at-ms now-ms :saved-by user)))
      (setf (gethash id (cms-revisions cms)) (append revs (list new-rev)))
      new-version)))

(defun set-article-tags (cms id tags now-ms)
  (let ((a (%require cms id)))
    (setf (article-tags a) tags (article-modified-ms a) now-ms)))

(defun article-by-id (cms id) (find id (cms-articles cms) :key #'article-id))
(defun article-by-slug (cms slug) (find slug (cms-articles cms) :key #'article-slug :test #'string=))
(defun articles (cms) (cms-articles cms))
(defun by-state (cms state)
  (remove-if-not (lambda (a) (eq (article-state a) state)) (cms-articles cms)))
(defun by-tag (cms tag)
  (remove-if-not (lambda (a) (member tag (article-tags a) :test #'string=))
                 (cms-articles cms)))
(defun by-author (cms author)
  (remove-if-not (lambda (a) (string= (article-author a) author))
                 (cms-articles cms)))
(defun revisions-of (cms id) (gethash id (cms-revisions cms)))
