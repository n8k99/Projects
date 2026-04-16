;;;; Post it Notes Program — create, manage, and search notes.

(defpackage :text/post-it-notes
  (:use :cl)
  (:export #:note
           #:note-id
           #:note-title
           #:note-content
           #:note-created-at
           #:note-color
           #:note-board
           #:make-note-board
           #:add-note
           #:get-note
           #:update-note
           #:delete-note
           #:list-notes
           #:search-notes
           #:board-count))

(in-package :text/post-it-notes)

;;; --- Data structures ---

(defstruct note
  (id 0 :type fixnum)
  (title "" :type string)
  (content "" :type string)
  (created-at 0 :type integer)
  (color nil :type (or null string)))

(defstruct note-board
  (notes (make-hash-table :test #'eql) :type hash-table)
  (next-id 1 :type fixnum))

;;; --- Operations ---

(defun add-note (board title content &optional color)
  "Create a new note on BOARD with TITLE and CONTENT. Returns the assigned id."
  (let* ((id (note-board-next-id board))
         (note (make-note :id id
                          :title title
                          :content content
                          :created-at (get-universal-time)
                          :color color)))
    (setf (gethash id (note-board-notes board)) note)
    (incf (note-board-next-id board))
    id))

(defun get-note (board id)
  "Retrieve a note by ID from BOARD. Returns the note or NIL."
  (gethash id (note-board-notes board)))

(defun update-note (board id content)
  "Update the content of note ID on BOARD. Returns T if found, NIL otherwise."
  (let ((note (gethash id (note-board-notes board))))
    (when note
      (setf (note-content note) content)
      t)))

(defun delete-note (board id)
  "Delete note ID from BOARD. Returns T if found, NIL otherwise."
  (remhash id (note-board-notes board)))

(defun list-notes (board)
  "Return all notes from BOARD sorted by id."
  (let ((notes '()))
    (maphash (lambda (k v)
               (declare (ignore k))
               (push v notes))
             (note-board-notes board))
    (sort notes #'< :key #'note-id)))

(defun search-notes (board query)
  "Search notes on BOARD by title or content (case-insensitive)."
  (let ((q (string-downcase query))
        (results '()))
    (maphash (lambda (k v)
               (declare (ignore k))
               (when (or (search q (string-downcase (note-title v)))
                         (search q (string-downcase (note-content v))))
                 (push v results)))
             (note-board-notes board))
    results))

(defun board-count (board)
  "Return the number of notes on BOARD."
  (hash-table-count (note-board-notes board)))
