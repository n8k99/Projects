;;;; Bookmark Collector and Sorter — multi-axis organisation over a URL-keyed collection.

(defpackage #:bookmarks
  (:use #:cl)
  (:export #:make-collection
           #:bookmark-add #:bookmark-remove #:bookmark-count #:bookmark-get
           #:bookmark-set-tags #:bookmark-move-folder
           #:bookmark-search #:bookmark-by-tag #:bookmark-by-folder
           #:bookmark-sort-by
           #:bookmark-all-tags #:bookmark-all-folders
           #:bookmark-id #:bookmark-url #:bookmark-title
           #:bookmark-tags #:bookmark-folder #:bookmark-added-at-ms))

(in-package #:bookmarks)

(defstruct bookmark
  (id 0 :type integer)
  (url "" :type string)
  (title "" :type string)
  (tags nil :type list)            ; sorted list of strings
  (folder "/" :type string)
  (added-at-ms 0 :type integer))

(defstruct collection
  (bookmarks nil :type list)
  (next-id 1 :type integer))

(defun %sorted-tags (tags)
  (sort (remove-duplicates (copy-list tags) :test #'string=) #'string<))

(defun bookmark-add (c url title folder tags added-at-ms)
  (let ((existing (find url (collection-bookmarks c)
                        :key #'bookmark-url :test #'string=)))
    (if existing
        (progn
          (setf (bookmark-title existing) title
                (bookmark-folder existing) folder
                (bookmark-tags existing) (%sorted-tags tags))
          (bookmark-id existing))
        (let* ((id (collection-next-id c))
               (b (make-bookmark :id id :url url :title title
                                  :folder folder
                                  :tags (%sorted-tags tags)
                                  :added-at-ms added-at-ms)))
          (incf (collection-next-id c))
          (setf (collection-bookmarks c)
                (append (collection-bookmarks c) (list b)))
          id))))

(defun bookmark-remove (c id)
  (let ((pos (position id (collection-bookmarks c) :key #'bookmark-id)))
    (when pos
      (setf (collection-bookmarks c)
            (append (subseq (collection-bookmarks c) 0 pos)
                    (subseq (collection-bookmarks c) (1+ pos))))
      t)))

(defun bookmark-count (c) (length (collection-bookmarks c)))

(defun bookmark-get (c id)
  (find id (collection-bookmarks c) :key #'bookmark-id))

(defun bookmark-set-tags (c id tags)
  (let ((b (bookmark-get c id)))
    (when b (setf (bookmark-tags b) (%sorted-tags tags)) t)))

(defun bookmark-move-folder (c id folder)
  (let ((b (bookmark-get c id)))
    (when b (setf (bookmark-folder b) folder) t)))

(defun %substring-p (needle haystack)
  (and (search (string-downcase needle) (string-downcase haystack)) t))

(defun bookmark-search (c query)
  (remove-if-not
   (lambda (b)
     (or (%substring-p query (bookmark-title b))
         (%substring-p query (bookmark-url b))
         (some (lambda (tag) (%substring-p query tag)) (bookmark-tags b))))
   (collection-bookmarks c)))

(defun bookmark-by-tag (c tag)
  (remove-if-not (lambda (b) (find tag (bookmark-tags b) :test #'string=))
                 (collection-bookmarks c)))

(defun bookmark-by-folder (c folder recursive)
  (remove-if-not
   (lambda (b)
     (if recursive
         (let ((prefix (if (char= (char folder (1- (length folder))) #\/)
                           folder (concatenate 'string folder "/"))))
           (or (string= (bookmark-folder b) folder)
               (and (>= (length (bookmark-folder b)) (length prefix))
                    (string= (subseq (bookmark-folder b) 0 (length prefix)) prefix))))
         (string= (bookmark-folder b) folder)))
   (collection-bookmarks c)))

(defun bookmark-sort-by (c criterion)
  (let ((key-fn
          (case criterion
            (:title #'bookmark-title)
            (:url #'bookmark-url)
            (:added-at #'bookmark-added-at-ms)
            (:folder #'bookmark-folder)
            (t (error "unknown sort criterion: ~A" criterion))))
        (comp
          (case criterion
            ((:title :url :folder) #'string<)
            (:added-at #'<)
            (t #'<))))
    (stable-sort (copy-list (collection-bookmarks c)) comp :key key-fn)))

(defun bookmark-all-tags (c)
  (sort (remove-duplicates
         (apply #'append (mapcar #'bookmark-tags (collection-bookmarks c)))
         :test #'string=)
        #'string<))

(defun bookmark-all-folders (c)
  (sort (remove-duplicates
         (mapcar #'bookmark-folder (collection-bookmarks c))
         :test #'string=)
        #'string<))
