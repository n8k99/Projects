;;;; Image Gallery — external storage refs, content-addressed dedup, set-algebra tag queries.

(defpackage #:image-gallery
  (:use #:cl)
  (:export #:make-blob-ref
           #:blob-ref-path #:blob-ref-size-bytes #:blob-ref-exists
           #:make-image
           #:image-id #:image-filename #:image-source
           #:image-thumbnail #:image-preview
           #:image-width #:image-height #:image-mime-type
           #:image-content-hash #:image-tags
           #:make-album
           #:album-id #:album-name #:album-image-ids #:album-curator
           #:make-gallery
           #:add-image #:remove-image
           #:tag-image #:untag-image
           #:all-tags
           #:tag-counts
           #:with-all-tags
           #:with-any-tag
           #:without-tags
           #:duplicates
           #:find-by-hash
           #:create-album
           #:add-to-album
           #:remove-from-album
           #:album-contents
           #:albums-containing
           #:total-bytes))

(in-package #:image-gallery)

(defstruct blob-ref
  (path "" :type string)
  (size-bytes 0 :type integer)
  (exists t :type boolean))

(defstruct image
  (id "" :type string)
  (filename "" :type string)
  (source nil :type (or null blob-ref))
  (thumbnail nil :type (or null blob-ref))
  (preview nil :type (or null blob-ref))
  (width 0 :type integer)
  (height 0 :type integer)
  (mime-type "image/jpeg" :type string)
  (content-hash "" :type string)
  (tags nil :type list))             ; tags stored as list of strings (treated as set)

(defstruct album
  (id "" :type string)
  (name "" :type string)
  (description "" :type string)
  (image-ids nil :type list)
  (curator "" :type string))

(defstruct gallery
  (images (make-hash-table :test 'equal))
  (albums (make-hash-table :test 'equal)))

(defun add-image (g image)
  (when (gethash (image-id image) (gallery-images g))
    (error "duplicate image: ~A" (image-id image)))
  (setf (gethash (image-id image) (gallery-images g)) image)
  image)

(defun remove-image (g image-id)
  (let ((image (gethash image-id (gallery-images g))))
    (unless image (error "unknown image: ~A" image-id))
    (remhash image-id (gallery-images g))
    ;; Orphan references from all albums.
    (maphash (lambda (aid a)
               (declare (ignore aid))
               (setf (album-image-ids a)
                     (remove image-id (album-image-ids a) :test #'string=)))
             (gallery-albums g))
    image))

(defun %require-image (g image-id)
  (or (gethash image-id (gallery-images g))
      (error "unknown image: ~A" image-id)))

(defun tag-image (g image-id &rest tags)
  (let ((image (%require-image g image-id)))
    (dolist (t0 tags)
      (pushnew t0 (image-tags image) :test #'string=))
    image))

(defun untag-image (g image-id &rest tags)
  (let ((image (%require-image g image-id)))
    (dolist (t0 tags)
      (setf (image-tags image) (remove t0 (image-tags image) :test #'string=)))
    image))

(defun all-tags (g)
  (let ((seen (make-hash-table :test 'equal)))
    (maphash (lambda (id image)
               (declare (ignore id))
               (dolist (t0 (image-tags image))
                 (setf (gethash t0 seen) t)))
             (gallery-images g))
    (loop for k being the hash-keys of seen collect k)))

(defun tag-counts (g)
  (let ((counts (make-hash-table :test 'equal)))
    (maphash (lambda (id image)
               (declare (ignore id))
               (dolist (t0 (image-tags image))
                 (incf (gethash t0 counts 0))))
             (gallery-images g))
    counts))

(defun %has-all (tags image-tags)
  (every (lambda (t0) (member t0 image-tags :test #'string=)) tags))

(defun %has-any (tags image-tags)
  (some (lambda (t0) (member t0 image-tags :test #'string=)) tags))

(defun with-all-tags (g tags)
  "Intersection: images containing every tag in TAGS."
  (let ((result nil))
    (maphash (lambda (id image)
               (declare (ignore id))
               (when (or (null tags) (%has-all tags (image-tags image)))
                 (push image result)))
             (gallery-images g))
    result))

(defun with-any-tag (g tags)
  "Union: images containing any tag in TAGS."
  (if (null tags)
      nil
      (let ((result nil))
        (maphash (lambda (id image)
                   (declare (ignore id))
                   (when (%has-any tags (image-tags image))
                     (push image result)))
                 (gallery-images g))
        result)))

(defun without-tags (g include exclude)
  "Images matching every tag in INCLUDE and none in EXCLUDE."
  (let ((base (with-all-tags g include)))
    (remove-if (lambda (image) (%has-any exclude (image-tags image))) base)))

(defun duplicates (g)
  "Hash-table of content-hash -> (list of image ids) for hashes seen more than once."
  (let ((buckets (make-hash-table :test 'equal)))
    (maphash (lambda (id image)
               (declare (ignore id))
               (let ((h (image-content-hash image)))
                 (when (and h (not (string= h "")))
                   (push (image-id image) (gethash h buckets)))))
             (gallery-images g))
    (let ((dups (make-hash-table :test 'equal)))
      (maphash (lambda (h ids)
                 (when (> (length ids) 1)
                   (setf (gethash h dups) (nreverse ids))))
               buckets)
      dups)))

(defun find-by-hash (g content-hash)
  (let ((result nil))
    (maphash (lambda (id image)
               (declare (ignore id))
               (when (string= (image-content-hash image) content-hash)
                 (push image result)))
             (gallery-images g))
    result))

(defun create-album (g album)
  (when (gethash (album-id album) (gallery-albums g))
    (error "duplicate album: ~A" (album-id album)))
  (dolist (iid (album-image-ids album))
    (unless (gethash iid (gallery-images g))
      (error "unknown image in album: ~A" iid)))
  (setf (gethash (album-id album) (gallery-albums g)) album)
  album)

(defun add-to-album (g album-id image-id)
  (let ((album (or (gethash album-id (gallery-albums g))
                   (error "unknown album: ~A" album-id))))
    (%require-image g image-id)
    (unless (member image-id (album-image-ids album) :test #'string=)
      (setf (album-image-ids album)
            (append (album-image-ids album) (list image-id))))
    album))

(defun remove-from-album (g album-id image-id)
  (let ((album (or (gethash album-id (gallery-albums g))
                   (error "unknown album: ~A" album-id))))
    (setf (album-image-ids album)
          (remove image-id (album-image-ids album) :test #'string=))
    album))

(defun album-contents (g album-id)
  (let ((album (or (gethash album-id (gallery-albums g))
                   (error "unknown album: ~A" album-id))))
    (mapcar (lambda (iid) (gethash iid (gallery-images g)))
            (remove-if-not (lambda (iid) (gethash iid (gallery-images g)))
                           (album-image-ids album)))))

(defun albums-containing (g image-id)
  (%require-image g image-id)
  (let ((result nil))
    (maphash (lambda (id album)
               (declare (ignore id))
               (when (member image-id (album-image-ids album) :test #'string=)
                 (push album result)))
             (gallery-albums g))
    result))

(defun total-bytes (g)
  (let ((total 0))
    (maphash (lambda (id image)
               (declare (ignore id))
               (when (image-source image)
                 (incf total (blob-ref-size-bytes (image-source image))))
               (when (image-thumbnail image)
                 (incf total (blob-ref-size-bytes (image-thumbnail image))))
               (when (image-preview image)
                 (incf total (blob-ref-size-bytes (image-preview image)))))
             (gallery-images g))
    total))
