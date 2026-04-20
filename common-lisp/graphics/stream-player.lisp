;;;; Stream Player — segmented video with buffer-based adaptive bitrate.

(defpackage #:stream-player
  (:use #:cl)
  (:export #:make-segment #:make-stream #:make-player
           #:stream-total-duration-ms #:stream-pick-bitrate
           #:player-attach #:player-set-bandwidth #:player-load-next-segment
           #:player-tick #:player-pause #:player-play #:player-seek-to-segment
           #:player-state #:player-buffer-ms #:player-played-ms
           #:player-current-segment #:player-current-bitrate #:player-events))

(in-package #:stream-player)

(defstruct segment
  (index 0 :type integer)
  (duration-ms 0 :type integer)
  (sizes-by-bitrate nil :type list))   ; alist of (bitrate . bytes)

(defun segment-size-for (s bitrate)
  (cdr (assoc bitrate (segment-sizes-by-bitrate s))))

(defstruct sp-stream
  (bitrates-bps nil :type list)
  (segments nil :type list)
  (safety-factor 0.85 :type single-float)
  (target-buffer-ms 6000 :type integer))

(defun stream-total-duration-ms (s)
  (reduce #'+ (sp-stream-segments s) :key #'segment-duration-ms :initial-value 0))

(defun stream-pick-bitrate (stream buffer-ms bandwidth-bps segment)
  (let* ((effective-bw (max 1.0 (* bandwidth-bps (sp-stream-safety-factor stream))))
         (raw-budget (max buffer-ms (segment-duration-ms segment)))
         (budget-ms (min raw-budget (* (sp-stream-target-buffer-ms stream) 2)))
         (chosen (first (sp-stream-bitrates-bps stream))))
    (dolist (bitrate (sp-stream-bitrates-bps stream))
      (let ((bytes (segment-size-for segment bitrate)))
        (when bytes
          (let ((download-ms (* (/ (* bytes 8.0) effective-bw) 1000.0)))
            (when (<= download-ms budget-ms)
              (setf chosen bitrate))))))
    chosen))

(defstruct player-event
  (kind :state-changed :type keyword)
  (detail "" :type string))

(defstruct player
  (state :idle :type keyword)
  (current-segment 0 :type integer)
  (buffer-ms 0 :type integer)
  (played-ms 0 :type integer)
  (bandwidth-bps 1000000 :type integer)
  (current-bitrate 0 :type integer)
  (events nil :type list)
  (stream nil :type (or null sp-stream)))

(defun player-set-state (p state)
  (unless (eq (player-state p) state)
    (let ((old (player-state p)))
      (setf (player-state p) state)
      (setf (player-events p)
            (append (player-events p)
                    (list (make-player-event
                           :kind :state-changed
                           :detail (format nil "~A -> ~A" old state))))))))

(defun player-attach (p stream)
  (setf (player-stream p) stream)
  (setf (player-current-bitrate p) (first (sp-stream-bitrates-bps stream)))
  (player-set-state p :buffering))

(defun player-set-bandwidth (p bps)
  (setf (player-bandwidth-bps p) bps))

(defun player-load-next-segment (p)
  (let ((stream (player-stream p)))
    (unless stream (return-from player-load-next-segment nil))
    (when (>= (player-current-segment p) (length (sp-stream-segments stream)))
      (return-from player-load-next-segment nil))
    (let* ((seg (nth (player-current-segment p) (sp-stream-segments stream)))
           (new-bitrate (stream-pick-bitrate stream
                                               (player-buffer-ms p)
                                               (player-bandwidth-bps p)
                                               seg)))
      (when (/= new-bitrate (player-current-bitrate p))
        (setf (player-events p)
              (append (player-events p)
                      (list (make-player-event
                             :kind :bitrate-switched
                             :detail (format nil "~A -> ~A"
                                              (player-current-bitrate p)
                                              new-bitrate)))))
        (setf (player-current-bitrate p) new-bitrate))
      (incf (player-buffer-ms p) (segment-duration-ms seg))
      (incf (player-current-segment p))
      (setf (player-events p)
            (append (player-events p)
                    (list (make-player-event
                           :kind :segment-loaded
                           :detail (format nil "segment ~D @ ~Abps"
                                            (segment-index seg)
                                            new-bitrate)))))
      (when (and (eq (player-state p) :buffering)
                  (>= (player-buffer-ms p) (sp-stream-target-buffer-ms stream)))
        (player-set-state p :playing))
      t)))

(defun player-tick (p ms)
  (unless (eq (player-state p) :playing) (return-from player-tick))
  (unless (player-stream p) (return-from player-tick))
  (let ((drained (min ms (player-buffer-ms p))))
    (decf (player-buffer-ms p) drained)
    (incf (player-played-ms p) drained)
    (let* ((stream (player-stream p))
           (at-end (>= (player-current-segment p)
                        (length (sp-stream-segments stream)))))
      (cond
        ((and (zerop (player-buffer-ms p)) at-end)
         (player-set-state p :ended))
        ((and (zerop (player-buffer-ms p)) (not at-end))
         (setf (player-events p)
               (append (player-events p)
                       (list (make-player-event
                              :kind :rebuffer
                              :detail (format nil "buffer empty at ~Ams"
                                               (player-played-ms p))))))
         (player-set-state p :buffering))))))

(defun player-pause (p)
  (when (eq (player-state p) :playing) (player-set-state p :paused)))

(defun player-play (p)
  (when (eq (player-state p) :paused) (player-set-state p :playing)))

(defun player-seek-to-segment (p segment-idx)
  (let ((stream (player-stream p)))
    (unless stream (return-from player-seek-to-segment nil))
    (when (or (< segment-idx 0)
              (>= segment-idx (length (sp-stream-segments stream))))
      (return-from player-seek-to-segment nil))
    (setf (player-current-segment p) segment-idx)
    (setf (player-buffer-ms p) 0)
    (setf (player-played-ms p)
          (reduce #'+ (subseq (sp-stream-segments stream) 0 segment-idx)
                   :key #'segment-duration-ms :initial-value 0))
    (setf (player-events p)
          (append (player-events p)
                  (list (make-player-event
                         :kind :seeked
                         :detail (format nil "segment ~D" segment-idx)))))
    (player-set-state p :buffering)
    t))

(defun demo ()
  (let* ((stream (make-sp-stream
                   :bitrates-bps '(500000 1000000 2000000 4000000)
                   :segments (loop for i from 0 below 5
                                    collect (make-segment
                                             :index i :duration-ms 2000
                                             :sizes-by-bitrate
                                             '((500000 . 125000)
                                               (1000000 . 250000)
                                               (2000000 . 500000)
                                               (4000000 . 1000000))))))
         (p (make-player)))
    (player-attach p stream)
    (player-set-bandwidth p 10000000)
    (dotimes (i 3) (player-load-next-segment p))
    (format t "after 3 loads: state=~A bitrate=~A buffer=~Ams~%"
            (player-state p) (player-current-bitrate p) (player-buffer-ms p))
    (player-tick p 6000)
    (format t "after drain: state=~A played=~Ams buffer=~Ams~%"
            (player-state p) (player-played-ms p) (player-buffer-ms p))))
