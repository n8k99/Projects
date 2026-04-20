;;;; Mp3 Player — playlist navigation with shuffle/repeat + position tracking.

(defpackage #:mp3-player
  (:use #:cl)
  (:export #:make-track #:make-library #:library-add #:library-get
           #:make-playlist #:make-player
           #:player-load-playlist #:player-play #:player-pause #:player-stop
           #:player-tick #:player-next #:player-prev
           #:player-set-repeat #:player-set-shuffle
           #:player-current-track-id
           #:player-state #:player-queue-order #:player-queue-position
           #:player-position-ms #:player-shuffle #:player-repeat
           #:player-history #:player-events))

(in-package #:mp3-player)

(defstruct track
  (id 0 :type integer)
  (title "" :type string)
  (artist "" :type string)
  (duration-ms 0 :type integer))

(defstruct library
  (tracks (make-hash-table) :type hash-table))

(defun library-add (lib tr)
  (setf (gethash (track-id tr) (library-tracks lib)) tr))

(defun library-get (lib id)
  (gethash id (library-tracks lib)))

(defstruct playlist
  (id 0 :type integer)
  (title "" :type string)
  (track-ids nil :type list))

(defstruct play-event
  (kind :state-changed :type keyword)
  (detail "" :type string))

(defstruct player
  (playlist nil :type (or null playlist))
  (queue-order nil :type list)
  (queue-position 0 :type integer)
  (position-ms 0 :type integer)
  (state :idle :type keyword)
  (repeat :off :type keyword)
  (shuffle nil :type boolean)
  (shuffle-seed 42 :type integer)
  (history nil :type list)
  (events nil :type list))

(defun player-current-track-id (p)
  (let ((order (player-queue-order p))
        (pos (player-queue-position p)))
    (when (and (>= pos 0) (< pos (length order)))
      (nth pos order))))

(defun player-set-state (p state)
  (unless (eq (player-state p) state)
    (let ((old (player-state p)))
      (setf (player-state p) state)
      (setf (player-events p)
            (append (player-events p)
                    (list (make-play-event
                           :kind :state-changed
                           :detail (format nil "~A -> ~A" old state))))))))

(defun emit-track-changed (p)
  (let ((id (player-current-track-id p)))
    (when id
      (setf (player-events p)
            (append (player-events p)
                    (list (make-play-event
                           :kind :track-changed
                           :detail (format nil "track ~D" id))))))))

(defun apply-shuffle (p)
  (let ((pl (player-playlist p)))
    (when pl
      (setf (player-queue-order p) (copy-list (playlist-track-ids pl)))
      ;; Fisher-Yates with LCG (same constants as G096)
      (let ((state (mod (1+ (player-shuffle-seed p)) (expt 2 64)))
            (order (player-queue-order p))
            (n (length (player-queue-order p))))
        (loop for i from (1- n) downto 1 do
              (setf state (mod (+ (* state 6364136223846793005)
                                    1442695040888963407)
                                (expt 2 64)))
              (let* ((r (ldb (byte 32 33) state))
                     (j (mod r (1+ i)))
                     (tmp (nth i order)))
                (setf (nth i order) (nth j order))
                (setf (nth j order) tmp)))
        (setf (player-queue-order p) order))
      ;; keep current track at position 0
      (let ((cur-id (player-current-track-id p)))
        (when cur-id
          (let ((pos (position cur-id (player-queue-order p))))
            (when pos
              (let ((tmp (nth 0 (player-queue-order p))))
                (setf (nth 0 (player-queue-order p))
                      (nth pos (player-queue-order p)))
                (setf (nth pos (player-queue-order p)) tmp)
                (setf (player-queue-position p) 0)))))))))

(defun unshuffle (p)
  (let ((pl (player-playlist p))
        (cur-id (player-current-track-id p)))
    (when pl
      (setf (player-queue-order p) (copy-list (playlist-track-ids pl)))
      (when cur-id
        (let ((pos (position cur-id (player-queue-order p))))
          (when pos (setf (player-queue-position p) pos)))))))

(defun player-load-playlist (p pl)
  (setf (player-playlist p) pl)
  (setf (player-queue-order p) (copy-list (playlist-track-ids pl)))
  (setf (player-queue-position p) 0)
  (setf (player-position-ms p) 0)
  (when (player-shuffle p) (apply-shuffle p))
  (player-set-state p :stopped))

(defun player-play (p)
  (when (player-queue-order p) (player-set-state p :playing)))

(defun player-pause (p)
  (when (eq (player-state p) :playing) (player-set-state p :paused)))

(defun player-stop (p)
  (player-set-state p :stopped)
  (setf (player-position-ms p) 0))

(defun compute-next (p)
  (let* ((order (player-queue-order p))
         (pos (player-queue-position p))
         (n (length order)))
    (case (player-repeat p)
      (:one (list :repeat 0))
      (:all (list :advance (mod (1+ pos) n)))
      (:off (if (< (1+ pos) n)
                (list :advance (1+ pos))
                (list :end 0))))))

(defun player-tick (p ms lib)
  (unless (and (eq (player-state p) :playing) (player-queue-order p))
    (return-from player-tick))
  (let* ((cur-id (player-current-track-id p))
         (tr (and cur-id (library-get lib cur-id))))
    (unless tr (return-from player-tick))
    (let ((remaining (max 0 (- (track-duration-ms tr) (player-position-ms p)))))
      (cond
        ((>= ms remaining)
         (setf (player-history p)
               (append (player-history p) (list cur-id)))
         (setf (player-position-ms p) 0)
         (let ((action (compute-next p)))
           (case (first action)
             (:advance
              (setf (player-queue-position p) (second action))
              (emit-track-changed p))
             (:repeat (emit-track-changed p))
             (:end (player-set-state p :stopped)))))
        (t (incf (player-position-ms p) ms))))))

(defun player-next (p lib)
  (declare (ignore lib))
  (let ((cur-id (player-current-track-id p)))
    (unless cur-id (return-from player-next nil))
    (setf (player-history p)
          (append (player-history p) (list cur-id)))
    (setf (player-position-ms p) 0)
    (let ((action (compute-next p)))
      (case (first action)
        (:advance
         (setf (player-queue-position p) (second action))
         (emit-track-changed p)
         t)
        (:repeat (emit-track-changed p) t)
        (:end (player-set-state p :stopped) nil)))))

(defun player-prev (p lib)
  (declare (ignore lib))
  (cond
    ((> (player-position-ms p) 3000)
     (setf (player-position-ms p) 0) t)
    ((player-history p)
     (let ((prev-id (car (last (player-history p)))))
       (setf (player-history p) (butlast (player-history p)))
       (let ((pos (position prev-id (player-queue-order p))))
         (when pos
           (setf (player-queue-position p) pos)
           (setf (player-position-ms p) 0)
           (emit-track-changed p)
           t))))
    (t (setf (player-position-ms p) 0) nil)))

(defun player-set-repeat (p mode)
  (unless (eq (player-repeat p) mode)
    (setf (player-repeat p) mode)
    (setf (player-events p)
          (append (player-events p)
                  (list (make-play-event :kind :repeat-changed
                                          :detail (format nil "~A" mode)))))))

(defun player-set-shuffle (p on)
  (unless (eq (player-shuffle p) on)
    (setf (player-shuffle p) on)
    (setf (player-events p)
          (append (player-events p)
                  (list (make-play-event :kind :shuffle-changed
                                          :detail (if on "on" "off")))))
    (if on (apply-shuffle p) (unshuffle p))))

(defun demo ()
  (let ((lib (make-library))
        (p (make-player)))
    (dolist (row '((1 "Alpha" 120000) (2 "Beta" 180000)
                    (3 "Gamma" 150000) (4 "Delta" 200000)))
      (library-add lib (make-track :id (first row) :title (second row)
                                    :artist "Various"
                                    :duration-ms (third row))))
    (player-load-playlist p (make-playlist :id 1 :title "Mix"
                                            :track-ids '(1 2 3 4)))
    (player-play p)
    (format t "start: track=~A state=~A~%"
            (player-current-track-id p) (player-state p))
    (player-tick p 125000 lib)
    (format t "after 125s: track=~A pos=~Ams~%"
            (player-current-track-id p) (player-position-ms p))
    (player-set-repeat p :all)
    (setf (player-queue-position p) 3)
    (setf (player-position-ms p) 0)
    (player-tick p 210000 lib)
    (format t "after wrap: track=~A state=~A~%"
            (player-current-track-id p) (player-state p))
    (player-set-shuffle p t)
    (format t "shuffle order: ~A~%" (player-queue-order p))))
