;;;; TV Show Tracker — hierarchical show/season/episode state + next-up.

(defpackage #:tv-tracker
  (:use #:cl)
  (:export #:make-episode #:make-show #:make-library #:make-watchlist
           #:library-add-show #:library-add-episode #:library-get
           #:library-airing-since
           #:watchlist-mark-watched #:watchlist-mark-unwatched #:watchlist-is-watched
           #:watchlist-mark-show-watched #:watchlist-mark-season-watched
           #:watchlist-watched-count
           #:show-total-episodes #:show-sorted-episodes
           #:show-total-runtime-min #:show-seasons
           #:show-progress #:next-up #:currently-watching
           #:episode-show-id #:episode-season #:episode-episode #:episode-title
           #:episode-duration-min #:episode-aired-ms
           #:show-id #:show-title #:show-episodes
           #:progress-watched #:progress-total #:progress-percent))

(in-package #:tv-tracker)

(defstruct episode
  (show-id 0 :type integer)
  (season 0 :type integer)
  (episode 0 :type integer)
  (title "" :type string)
  (duration-min 0 :type integer)
  (aired-ms 0 :type integer))

(defun episode-key (e)
  (list (episode-show-id e) (episode-season e) (episode-episode e)))

(defstruct show
  (id 0 :type integer)
  (title "" :type string)
  (episodes nil :type list))

(defun show-total-episodes (s)
  (length (show-episodes s)))

(defun show-sorted-episodes (s)
  (sort (copy-list (show-episodes s))
        (lambda (a b)
          (cond ((< (episode-season a) (episode-season b)) t)
                ((> (episode-season a) (episode-season b)) nil)
                (t (< (episode-episode a) (episode-episode b)))))))

(defun show-total-runtime-min (s)
  (reduce #'+ (show-episodes s) :key #'episode-duration-min :initial-value 0))

(defun show-seasons (s)
  (sort (remove-duplicates (mapcar #'episode-season (show-episodes s))) #'<))

(defstruct library
  (shows (make-hash-table) :type hash-table))

(defun library-add-show (lib show)
  (setf (gethash (show-id show) (library-shows lib)) show))

(defun library-add-episode (lib ep)
  (let ((s (gethash (episode-show-id ep) (library-shows lib))))
    (unless s (error "unknown show: ~A" (episode-show-id ep)))
    (let ((key (episode-key ep)))
      (setf (show-episodes s)
            (append (remove-if (lambda (e) (equal (episode-key e) key))
                                (show-episodes s))
                    (list ep))))))

(defun library-get (lib id)
  (gethash id (library-shows lib)))

(defun library-airing-since (lib since-ms)
  (let ((out nil))
    (maphash (lambda (id show) (declare (ignore id))
               (dolist (e (show-episodes show))
                 (when (>= (episode-aired-ms e) since-ms)
                   (push e out))))
             (library-shows lib))
    (sort out (lambda (a b)
                (cond ((< (episode-aired-ms a) (episode-aired-ms b)) t)
                      ((> (episode-aired-ms a) (episode-aired-ms b)) nil)
                      ((< (episode-show-id a) (episode-show-id b)) t)
                      ((> (episode-show-id a) (episode-show-id b)) nil)
                      ((< (episode-season a) (episode-season b)) t)
                      ((> (episode-season a) (episode-season b)) nil)
                      (t (< (episode-episode a) (episode-episode b))))))))

(defstruct watchlist
  (watched (make-hash-table :test 'equal) :type hash-table))

(defun watchlist-mark-watched (wl show-id season episode)
  (setf (gethash (list show-id season episode) (watchlist-watched wl)) t))

(defun watchlist-mark-unwatched (wl show-id season episode)
  (remhash (list show-id season episode) (watchlist-watched wl)))

(defun watchlist-is-watched (wl show-id season episode)
  (gethash (list show-id season episode) (watchlist-watched wl)))

(defun watchlist-mark-show-watched (wl show)
  (dolist (e (show-episodes show))
    (setf (gethash (episode-key e) (watchlist-watched wl)) t)))

(defun watchlist-mark-season-watched (wl show season)
  (dolist (e (show-episodes show))
    (when (= (episode-season e) season)
      (setf (gethash (episode-key e) (watchlist-watched wl)) t))))

(defun watchlist-watched-count (wl show-id)
  (let ((count 0))
    (maphash (lambda (k v) (declare (ignore v))
               (when (= (first k) show-id) (incf count)))
             (watchlist-watched wl))
    count))

(defstruct progress
  (show-id 0 :type integer)
  (watched 0 :type integer)
  (total 0 :type integer)
  (percent 0 :type integer))

(defun show-progress (show watchlist)
  (let* ((total (show-total-episodes show))
         (watched (watchlist-watched-count watchlist (show-id show)))
         (percent (if (zerop total) 0 (floor (* watched 100) total))))
    (make-progress :show-id (show-id show) :watched watched
                    :total total :percent percent)))

(defun next-up (show watchlist)
  (find-if (lambda (e)
             (not (watchlist-is-watched watchlist
                                          (episode-show-id e)
                                          (episode-season e)
                                          (episode-episode e))))
           (show-sorted-episodes show)))

(defun currently-watching (library watchlist)
  (let ((out nil))
    (maphash (lambda (id show) (declare (ignore id))
               (let ((w (watchlist-watched-count watchlist (show-id show))))
                 (when (and (> w 0) (< w (show-total-episodes show)))
                   (push show out))))
             (library-shows library))
    (sort out (lambda (a b) (< (show-id a) (show-id b))))))

(defun demo ()
  (let ((lib (make-library))
        (show (make-show :id 1 :title "Breaking Bad")))
    (loop for season from 1 to 2
          do (loop for episode from 1 to 3
                   do (setf (show-episodes show)
                            (append (show-episodes show)
                                    (list (make-episode
                                           :show-id 1 :season season :episode episode
                                           :title (format nil "S~DE~D" season episode)
                                           :duration-min 47
                                           :aired-ms (+ 1000000
                                                         (* season 86400000 7)
                                                         (* episode 86400000))))))))
    (library-add-show lib show)

    (let ((wl (make-watchlist)))
      (watchlist-mark-watched wl 1 1 1)
      (watchlist-mark-watched wl 1 1 2)
      (let ((p (show-progress show wl)))
        (format t "progress: ~D/~D = ~D%~%"
                (progress-watched p) (progress-total p) (progress-percent p)))
      (let ((n (next-up show wl)))
        (format t "next up: S~DE~D — ~A~%"
                (episode-season n) (episode-episode n) (episode-title n)))
      (let ((cw (currently-watching lib wl)))
        (format t "currently watching: ~A~%"
                (mapcar #'show-title cw))))))
