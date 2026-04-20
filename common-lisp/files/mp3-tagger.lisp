;;;; Mp3 Tagger — file metadata as a queryable, indexed tag store.

(defpackage #:mp3-tagger
  (:use #:cl)
  (:export #:make-tag-store #:store-insert #:store-get #:store-remove
           #:store-query #:make-tag
           #:tag-title #:tag-artist #:tag-album #:tag-year #:tag-genre #:tag-track
           #:make-query #:tag-missing-fields))

(in-package #:mp3-tagger)

(defstruct tag
  (title nil :type (or null string))
  (artist nil :type (or null string))
  (album nil :type (or null string))
  (year nil :type (or null integer))
  (genre nil :type (or null string))
  (track nil :type (or null integer)))

(defun tag-missing-fields (tag)
  (let ((out nil))
    (unless (tag-title tag) (push "title" out))
    (unless (tag-artist tag) (push "artist" out))
    (unless (tag-album tag) (push "album" out))
    (unless (tag-year tag) (push "year" out))
    (unless (tag-genre tag) (push "genre" out))
    (unless (tag-track tag) (push "track" out))
    (nreverse out)))

(defstruct tag-store
  (tags (make-hash-table :test 'equal) :type hash-table)
  (artist-idx (make-hash-table :test 'equal) :type hash-table)
  (album-idx (make-hash-table :test 'equal) :type hash-table)
  (year-idx (make-hash-table) :type hash-table))

(defstruct query
  (kind :by-artist :type keyword)
  (artist "" :type string)
  (album "" :type string)
  (year 0 :type integer)
  (year-from 0 :type integer)
  (year-to 0 :type integer)
  (needle "" :type string)
  (field-name "" :type string))

(defun index-add (idx key path)
  (push path (gethash key idx)))

(defun index-remove (idx key path)
  (setf (gethash key idx)
        (remove path (gethash key idx) :test #'string=)))

(defun store-remove-indexes (store path)
  (let ((tag (gethash path (tag-store-tags store))))
    (when tag
      (when (tag-artist tag)
        (index-remove (tag-store-artist-idx store) (tag-artist tag) path))
      (when (tag-album tag)
        (index-remove (tag-store-album-idx store) (tag-album tag) path))
      (when (tag-year tag)
        (index-remove (tag-store-year-idx store) (tag-year tag) path)))))

(defun store-insert (store path tag)
  (when (gethash path (tag-store-tags store))
    (store-remove-indexes store path))
  (when (tag-artist tag)
    (index-add (tag-store-artist-idx store) (tag-artist tag) path))
  (when (tag-album tag)
    (index-add (tag-store-album-idx store) (tag-album tag) path))
  (when (tag-year tag)
    (index-add (tag-store-year-idx store) (tag-year tag) path))
  (setf (gethash path (tag-store-tags store)) tag))

(defun store-get (store path)
  (gethash path (tag-store-tags store)))

(defun store-remove (store path)
  (store-remove-indexes store path)
  (remhash path (tag-store-tags store)))

(defun field-missing-p (tag name)
  (cond
    ((string= name "title") (null (tag-title tag)))
    ((string= name "artist") (null (tag-artist tag)))
    ((string= name "album") (null (tag-album tag)))
    ((string= name "year") (null (tag-year tag)))
    ((string= name "genre") (null (tag-genre tag)))
    ((string= name "track") (null (tag-track tag)))
    (t nil)))

(defun store-query (store q)
  (let ((paths
         (case (query-kind q)
           (:by-artist
            (copy-list (gethash (query-artist q) (tag-store-artist-idx store))))
           (:by-album
            (copy-list (gethash (query-album q) (tag-store-album-idx store))))
           (:by-year
            (copy-list (gethash (query-year q) (tag-store-year-idx store))))
           (:year-range
            (let ((out nil))
              (maphash (lambda (y paths)
                         (when (and (>= y (query-year-from q))
                                    (<= y (query-year-to q)))
                           (dolist (p paths) (push p out))))
                       (tag-store-year-idx store))
              out))
           (:title-contains
            (let ((needle (string-downcase (query-needle q)))
                  (out nil))
              (maphash (lambda (path tag)
                         (when (and (tag-title tag)
                                    (search needle (string-downcase (tag-title tag))))
                           (push path out)))
                       (tag-store-tags store))
              out))
           (:missing-field
            (let ((field (query-field-name q))
                  (out nil))
              (maphash (lambda (path tag)
                         (when (field-missing-p tag field)
                           (push path out)))
                       (tag-store-tags store))
              out))
           (t nil))))
    (sort paths #'string<)))

(defun demo ()
  (let ((s (make-tag-store)))
    (store-insert s "a.mp3" (make-tag :title "Aeon" :artist "Björk"
                                       :album "Vespertine" :year 2001
                                       :genre "Electronic" :track 1))
    (store-insert s "b.mp3" (make-tag :title "Pagan Poetry" :artist "Björk"
                                       :album "Vespertine" :year 2001
                                       :genre "Electronic" :track 3))
    (store-insert s "c.mp3" (make-tag :title "Unplayed"))
    (store-insert s "d.mp3" (make-tag :title "Hyperballad" :artist "Björk"
                                       :album "Post" :year 1995 :track 2))

    (format t "=== by artist: Björk ===~%")
    (dolist (p (store-query s (make-query :kind :by-artist :artist "Björk")))
      (format t "  ~A~%" p))
    (format t "~%=== year range 2000-2010 ===~%")
    (dolist (p (store-query s (make-query :kind :year-range :year-from 2000 :year-to 2010)))
      (format t "  ~A~%" p))
    (format t "~%=== title contains 'aeon' ===~%")
    (dolist (p (store-query s (make-query :kind :title-contains :needle "aeon")))
      (format t "  ~A~%" p))
    (format t "~%=== missing genre ===~%")
    (dolist (p (store-query s (make-query :kind :missing-field :field-name "genre")))
      (format t "  ~A~%" p))))
