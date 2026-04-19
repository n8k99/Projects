;;;; Media Player Widget — playback FSM + playlist with repeat/shuffle modes.

(defpackage #:media-player
  (:use #:cl)
  (:export #:make-track #:make-player
           #:player-load-playlist
           #:player-state #:player-position-ms #:player-volume
           #:player-repeat #:player-shuffle-p #:player-current-track
           #:player-play #:player-pause #:player-stop
           #:player-next-track #:player-previous-track
           #:player-seek-ms #:player-set-volume
           #:player-set-repeat #:player-set-shuffle
           #:player-tick
           #:track-id #:track-title #:track-duration-ms))

(in-package #:media-player)

(defstruct track
  (id 0 :type integer)
  (title "" :type string)
  (artist "" :type string)
  (duration-ms 0 :type integer))

(defstruct player
  (playlist nil :type list)
  (state :stopped :type keyword)       ; :stopped :playing :paused
  (current-idx nil :type (or null integer))
  (position-ms 0 :type integer)
  (volume 1.0 :type real)
  (repeat :none :type keyword)         ; :none :one :all
  (shuffle-p nil :type boolean)
  (shuffle-order nil :type list)
  (rng #xdeadbeef :type integer))

(defun %rebuild-shuffle-order (p)
  (let ((n (length (player-playlist p))))
    (setf (player-shuffle-order p) (loop for i from 0 below n collect i))
    (when (and (player-shuffle-p p) (> n 1))
      (let ((arr (coerce (player-shuffle-order p) 'vector)))
        (loop for i from (1- n) downto 1 do
          (setf (player-rng p)
                (logand (+ (* (player-rng p) 6364136223846793005)
                           1442695040888963407) #xFFFFFFFFFFFFFFFF))
          (let ((j (mod (ash (player-rng p) -32) (1+ i))))
            (rotatef (aref arr i) (aref arr j))))
        (setf (player-shuffle-order p) (coerce arr 'list))))))

(defun player-load-playlist (p tracks)
  (setf (player-playlist p) tracks
        (player-state p) :stopped
        (player-current-idx p) nil
        (player-position-ms p) 0)
  (%rebuild-shuffle-order p))

(defun player-current-track (p)
  (let ((idx (player-current-idx p)))
    (when idx
      (let ((pl-idx (if (player-shuffle-p p)
                        (nth idx (player-shuffle-order p))
                        idx)))
        (when (and pl-idx (< pl-idx (length (player-playlist p))))
          (nth pl-idx (player-playlist p)))))))

(defun player-set-volume (p v) (setf (player-volume p) (max 0.0 (min 1.0 v))))
(defun player-set-repeat (p mode) (setf (player-repeat p) mode))

(defun player-set-shuffle (p on)
  (when (eq on (player-shuffle-p p)) (return-from player-set-shuffle))
  (let ((prev-idx (player-current-idx p)))
    (setf (player-shuffle-p p) on)
    (%rebuild-shuffle-order p)
    (when prev-idx
      (setf (player-current-idx p)
            (if on
                (position prev-idx (player-shuffle-order p))
                (nth prev-idx (player-shuffle-order p)))))))

(defun player-play (p)
  (case (player-state p)
    (:playing nil)
    (:paused (setf (player-state p) :playing)
             (list (list :state-changed :playing)))
    (:stopped
     (when (player-playlist p)
       (setf (player-current-idx p) 0
             (player-position-ms p) 0
             (player-state p) :playing)
       (let ((t1 (player-current-track p)))
         (append (when t1 (list (list :track-started (track-id t1))))
                 (list (list :state-changed :playing))))))))

(defun player-pause (p)
  (when (eq (player-state p) :playing)
    (setf (player-state p) :paused)
    (list (list :state-changed :paused))))

(defun player-stop (p)
  (unless (eq (player-state p) :stopped)
    (setf (player-state p) :stopped
          (player-position-ms p) 0
          (player-current-idx p) nil)
    (list (list :state-changed :stopped))))

(defun player-next-track (p)
  (let ((cur (player-current-idx p)))
    (unless cur (return-from player-next-track (player-play p)))
    (when (eq (player-repeat p) :one)
      (setf (player-position-ms p) 0)
      (let ((t1 (player-current-track p)))
        (return-from player-next-track
          (when t1 (list (list :track-started (track-id t1)))))))
    (let ((nxt (1+ cur)))
      (cond
        ((< nxt (length (player-playlist p)))
         (setf (player-current-idx p) nxt (player-position-ms p) 0)
         (let ((t1 (player-current-track p)))
           (when t1 (list (list :track-started (track-id t1))))))
        ((eq (player-repeat p) :all)
         (when (player-shuffle-p p) (%rebuild-shuffle-order p))
         (setf (player-current-idx p) 0 (player-position-ms p) 0)
         (let ((t1 (player-current-track p)))
           (when t1 (list (list :track-started (track-id t1))))))
        (t
         (setf (player-state p) :stopped
               (player-current-idx p) nil
               (player-position-ms p) 0)
         (list '(:playlist-ended) '(:state-changed :stopped)))))))

(defun player-previous-track (p)
  (let ((cur (player-current-idx p)))
    (unless cur (return-from player-previous-track nil))
    (if (zerop cur)
        (progn (setf (player-position-ms p) 0)
               (let ((t1 (player-current-track p)))
                 (when t1 (list (list :track-started (track-id t1))))))
        (progn (setf (player-current-idx p) (1- cur) (player-position-ms p) 0)
               (let ((t1 (player-current-track p)))
                 (when t1 (list (list :track-started (track-id t1)))))))))

(defun player-seek-ms (p pos)
  (let ((t1 (player-current-track p)))
    (when (and t1 (<= pos (track-duration-ms t1)))
      (setf (player-position-ms p) pos)
      t)))

(defun player-tick (p delta-ms)
  (unless (eq (player-state p) :playing) (return-from player-tick nil))
  (let ((events nil) (remaining delta-ms))
    (loop while (plusp remaining) do
      (let ((cur (player-current-track p)))
        (unless cur (return))
        (let ((time-left (max 0 (- (track-duration-ms cur) (player-position-ms p)))))
          (cond
            ((< remaining time-left)
             (incf (player-position-ms p) remaining)
             (setf remaining 0))
            (t
             (decf remaining time-left)
             (setf (player-position-ms p) (track-duration-ms cur))
             (push (list :track-ended (track-id cur)) events)
             (dolist (ev (player-next-track p))
               (push ev events))
             (when (eq (player-state p) :stopped) (return)))))))
    (nreverse events)))
