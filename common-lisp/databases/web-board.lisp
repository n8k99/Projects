;;;; Web Board / Forum — threaded posts with vote tallies + hot-score ranking.

(defpackage #:web-board
  (:use #:cl)
  (:export #:make-board #:make-thread-entry #:make-post
           #:board-add-thread #:board-add-post
           #:post-score #:hot-score #:thread-score #:thread-hot-score
           #:board-top-level-posts #:board-children-of #:board-thread-tree
           #:board-list-threads #:board-post-count
           #:post-id #:post-thread-id #:post-parent-id #:post-author
           #:post-body #:post-created-ms #:post-upvotes #:post-downvotes
           #:thread-entry-id #:thread-entry-title #:thread-entry-created-ms
           #:thread-entry-pinned #:thread-entry-locked
           #:post-with-depth-post #:post-with-depth-depth))

(in-package #:web-board)

(defstruct thread-entry
  (id 0 :type integer)
  (title "" :type string)
  (author "" :type string)
  (created-ms 0 :type integer)
  (pinned nil :type boolean)
  (locked nil :type boolean))

(defstruct post
  (id 0 :type integer)
  (thread-id 0 :type integer)
  (parent-id nil :type (or null integer))
  (author "" :type string)
  (body "" :type string)
  (created-ms 0 :type integer)
  (upvotes 0 :type integer)
  (downvotes 0 :type integer))

(defun post-score (p)
  (- (post-upvotes p) (post-downvotes p)))

(defun hot-score (p now-ms)
  (let* ((s (post-score p))
         (sign (cond ((> s 0) 1.0) ((< s 0) -1.0) (t 0.0)))
         (magnitude (log (max 1 (abs s)) 10))
         (age-days (/ (max 0 (- now-ms (post-created-ms p))) 86400000.0)))
    (- (* sign magnitude) (* age-days 0.2))))

(defstruct board
  (threads (make-hash-table) :type hash-table)
  (posts (make-hash-table) :type hash-table))

(defun board-add-thread (b t1)
  (setf (gethash (thread-entry-id t1) (board-threads b)) t1))

(defun board-add-post (b p)
  (unless (gethash (post-thread-id p) (board-threads b))
    (error "unknown thread ~A" (post-thread-id p)))
  (when (post-parent-id p)
    (let ((parent (gethash (post-parent-id p) (board-posts b))))
      (unless parent (error "unknown parent ~A" (post-parent-id p)))
      (unless (= (post-thread-id parent) (post-thread-id p))
        (error "parent in different thread"))))
  (setf (gethash (post-id p) (board-posts b)) p))

(defun thread-score (b thread-id)
  (let ((sum 0))
    (maphash (lambda (id p) (declare (ignore id))
               (when (= (post-thread-id p) thread-id)
                 (incf sum (post-score p))))
             (board-posts b))
    sum))

(defun thread-hot-score (b thread-id now-ms)
  (let ((sum 0.0))
    (maphash (lambda (id p) (declare (ignore id))
               (when (= (post-thread-id p) thread-id)
                 (incf sum (hot-score p now-ms))))
             (board-posts b))
    sum))

(defun board-top-level-posts (b thread-id)
  (let ((out nil))
    (maphash (lambda (id p) (declare (ignore id))
               (when (and (= (post-thread-id p) thread-id)
                          (null (post-parent-id p)))
                 (push p out)))
             (board-posts b))
    (sort out #'< :key #'post-created-ms)))

(defun board-children-of (b post-id)
  (let ((out nil))
    (maphash (lambda (id p) (declare (ignore id))
               (when (and (post-parent-id p)
                          (= (post-parent-id p) post-id))
                 (push p out)))
             (board-posts b))
    (sort out #'< :key #'post-created-ms)))

(defstruct post-with-depth
  (post nil :type (or null post))
  (depth 0 :type integer))

(defun walk-subtree (b p depth out)
  (push (make-post-with-depth :post p :depth depth) (car out))
  (dolist (child (board-children-of b (post-id p)))
    (walk-subtree b child (1+ depth) out)))

(defun board-thread-tree (b thread-id)
  (let ((out (list nil)))
    (dolist (top (board-top-level-posts b thread-id))
      (walk-subtree b top 0 out))
    (nreverse (car out))))

(defun board-post-count (b thread-id)
  (let ((c 0))
    (maphash (lambda (id p) (declare (ignore id))
               (when (= (post-thread-id p) thread-id) (incf c)))
             (board-posts b))
    c))

(defun board-list-threads (b sort now-ms)
  (let ((all nil))
    (maphash (lambda (id t1) (declare (ignore id)) (push t1 all))
             (board-threads b))
    (sort all
          (lambda (a b)
            (cond
              ((and (thread-entry-pinned a) (not (thread-entry-pinned b))) t)
              ((and (not (thread-entry-pinned a)) (thread-entry-pinned b)) nil)
              (t (case sort
                   (:new (> (thread-entry-created-ms a) (thread-entry-created-ms b)))
                   (:top (> (thread-score b (thread-entry-id a))
                             (thread-score b (thread-entry-id b))))
                   (:hot (> (thread-hot-score b (thread-entry-id a) now-ms)
                             (thread-hot-score b (thread-entry-id b) now-ms))))))))))

(defun demo ()
  (let ((b (make-board)))
    (board-add-thread b (make-thread-entry :id 1 :title "Welcome" :author "alice"
                                             :created-ms 1000000 :pinned t))
    (board-add-thread b (make-thread-entry :id 2 :title "General discussion" :author "bob"
                                             :created-ms 2000000))
    (board-add-post b (make-post :id 10 :thread-id 1 :parent-id nil
                                  :author "alice" :body "hi everyone"
                                  :created-ms 1000000 :upvotes 10 :downvotes 1))
    (board-add-post b (make-post :id 11 :thread-id 1 :parent-id 10
                                  :author "bob" :body "hi alice"
                                  :created-ms 1100000 :upvotes 5 :downvotes 0))
    (board-add-post b (make-post :id 12 :thread-id 1 :parent-id 10
                                  :author "carol" :body "hello"
                                  :created-ms 1200000 :upvotes 3 :downvotes 0))
    (board-add-post b (make-post :id 13 :thread-id 1 :parent-id 11
                                  :author "alice" :body "hey bob"
                                  :created-ms 1300000 :upvotes 2 :downvotes 0))
    (board-add-post b (make-post :id 20 :thread-id 2 :parent-id nil
                                  :author "bob" :body "first!"
                                  :created-ms 2000000 :upvotes 100 :downvotes 5))
    (format t "=== thread 1 tree ===~%")
    (dolist (item (board-thread-tree b 1))
      (let ((p (post-with-depth-post item))
            (d (post-with-depth-depth item)))
        (format t "  ~A#~D by ~A (score=~D)~%"
                (make-string (* 2 d) :initial-element #\Space)
                (post-id p) (post-author p) (post-score p))))
    (format t "~%=== threads sorted TOP ===~%")
    (dolist (t1 (board-list-threads b :top 3000000))
      (format t "  ~D: ~A~A (score=~D)~%"
              (thread-entry-id t1) (thread-entry-title t1)
              (if (thread-entry-pinned t1) " [pinned]" "")
              (thread-score b (thread-entry-id t1))))))
