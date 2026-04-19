;;;; Web Browser with Tabs — state model for tabbed navigation.
;;;;
;;;; Per-tab history (back/forward stacks) and browser-scoped closed-tab
;;;; recovery are two orthogonal undo-like stacks with different scopes.
;;;; Bookmarks are browser-scoped and survive tab close.

(defpackage #:browser
  (:use #:cl)
  (:export #:make-browser
           #:open-tab #:close-tab #:switch-to #:active-tab
           #:navigate #:go-back #:go-forward
           #:reopen-closed #:add-bookmark #:bookmarks
           #:tab-count #:closed-count
           #:tab-id #:tab-url #:tab-back #:tab-forward))

(in-package #:browser)

(defstruct tab
  (id 0 :type integer)
  (title "" :type string)
  (url "" :type string)
  (back nil :type list)
  (forward nil :type list))

(defstruct closed-tab
  (url "" :type string) (title "" :type string)
  (back nil :type list) (forward nil :type list))

(defstruct browser
  (tabs nil :type list)           ; list of tab
  (active nil :type (or null integer))
  (closed nil :type list)         ; stack (newest first)
  (bookmarks nil :type list)      ; list of (title . url)
  (next-id 1 :type integer))

(defun open-tab (b url)
  (let* ((id (browser-next-id b))
         (tab (make-tab :id id :title url :url url)))
    (incf (browser-next-id b))
    (setf (browser-tabs b) (append (browser-tabs b) (list tab)))
    (setf (browser-active b) id)
    id))

(defun %tab-position (b id)
  (position id (browser-tabs b) :key #'tab-id))

(defun close-tab (b id)
  (let ((pos (%tab-position b id)))
    (unless pos (return-from close-tab nil))
    (let ((old (nth pos (browser-tabs b))))
      (push (make-closed-tab :url (tab-url old) :title (tab-title old)
                              :back (tab-back old) :forward (tab-forward old))
            (browser-closed b))
      (setf (browser-tabs b)
            (append (subseq (browser-tabs b) 0 pos)
                    (subseq (browser-tabs b) (1+ pos))))
      (when (eql (browser-active b) id)
        (let ((len (length (browser-tabs b))))
          (setf (browser-active b)
                (cond ((< pos len) (tab-id (nth pos (browser-tabs b))))
                      ((plusp len) (tab-id (nth (1- len) (browser-tabs b))))
                      (t nil))))))
    t))

(defun switch-to (b id)
  (when (%tab-position b id)
    (setf (browser-active b) id)
    t))

(defun active-tab (b)
  (and (browser-active b)
       (find (browser-active b) (browser-tabs b) :key #'tab-id)))

(defun navigate (b url)
  (let ((tab (active-tab b)))
    (unless tab (return-from navigate nil))
    (push (tab-url tab) (tab-back tab))
    (setf (tab-back tab) (nreverse (tab-back tab)))         ; maintain chronological
    (setf (tab-back tab) (append (butlast (tab-back tab) 0) (list (tab-url tab))))
    ;; Simpler: rebuild properly.
    t))

;; Redefine navigate more cleanly
(defun navigate (b url)
  (let ((tab (active-tab b)))
    (unless tab (return-from navigate nil))
    (setf (tab-back tab) (append (tab-back tab) (list (tab-url tab))))
    (setf (tab-forward tab) nil)
    (setf (tab-url tab) url)
    (setf (tab-title tab) url)
    t))

(defun go-back (b)
  (let ((tab (active-tab b)))
    (unless (and tab (tab-back tab)) (return-from go-back nil))
    (let ((prev (car (last (tab-back tab)))))
      (setf (tab-back tab) (butlast (tab-back tab)))
      (setf (tab-forward tab) (append (tab-forward tab) (list (tab-url tab))))
      (setf (tab-url tab) prev)
      prev)))

(defun go-forward (b)
  (let ((tab (active-tab b)))
    (unless (and tab (tab-forward tab)) (return-from go-forward nil))
    (let ((next (car (last (tab-forward tab)))))
      (setf (tab-forward tab) (butlast (tab-forward tab)))
      (setf (tab-back tab) (append (tab-back tab) (list (tab-url tab))))
      (setf (tab-url tab) next)
      next)))

(defun reopen-closed (b)
  (unless (browser-closed b) (return-from reopen-closed nil))
  (let* ((ct (pop (browser-closed b)))
         (id (browser-next-id b))
         (tab (make-tab :id id :title (closed-tab-title ct)
                        :url (closed-tab-url ct)
                        :back (closed-tab-back ct)
                        :forward (closed-tab-forward ct))))
    (incf (browser-next-id b))
    (setf (browser-tabs b) (append (browser-tabs b) (list tab)))
    (setf (browser-active b) id)
    id))

(defun add-bookmark (b)
  (let ((tab (active-tab b)))
    (unless tab (return-from add-bookmark nil))
    (let ((entry (cons (tab-title tab) (tab-url tab))))
      (unless (find entry (browser-bookmarks b) :test #'equal)
        (setf (browser-bookmarks b)
              (append (browser-bookmarks b) (list entry)))))
    t))

(defun bookmarks (b) (browser-bookmarks b))
(defun tab-count (b) (length (browser-tabs b)))
(defun closed-count (b) (length (browser-closed b)))
