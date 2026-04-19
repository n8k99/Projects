;;;; Bandwidth Monitor — time-series rate calculation over samples.

(defpackage #:bandwidth-monitor
  (:use #:cl)
  (:export #:make-monitor #:record-sample
           #:sample-count #:latest #:total-bytes
           #:instantaneous-rate #:average-rate #:peak-rate))

(in-package #:bandwidth-monitor)

(defstruct sample
  (timestamp-ms 0 :type integer)
  (bytes-total 0 :type integer))

(defstruct monitor
  (samples nil :type list)       ; oldest first
  (capacity 2 :type integer))

(defun make-monitor-with (capacity)
  (when (< capacity 2) (error "need at least 2 samples to compute a rate"))
  (make-monitor :capacity capacity))

(defun record-sample (m ts bytes)
  (setf (monitor-samples m)
        (append (monitor-samples m)
                (list (make-sample :timestamp-ms ts :bytes-total bytes))))
  (when (> (length (monitor-samples m)) (monitor-capacity m))
    (setf (monitor-samples m) (rest (monitor-samples m)))))

(defun sample-count (m) (length (monitor-samples m)))
(defun latest (m) (car (last (monitor-samples m))))

(defun total-bytes (m)
  (let ((total 0) (prev nil))
    (dolist (s (monitor-samples m))
      (when (and prev (>= (sample-bytes-total s) (sample-bytes-total prev)))
        (incf total (- (sample-bytes-total s) (sample-bytes-total prev))))
      (setf prev s))
    total))

(defun instantaneous-rate (m)
  (let ((n (length (monitor-samples m))))
    (if (< n 2) 0.0
        (let* ((a (nth (- n 2) (monitor-samples m)))
               (b (nth (- n 1) (monitor-samples m))))
          (if (or (<= (sample-timestamp-ms b) (sample-timestamp-ms a))
                  (< (sample-bytes-total b) (sample-bytes-total a)))
              0.0
              (/ (* 1000.0 (- (sample-bytes-total b) (sample-bytes-total a)))
                 (- (sample-timestamp-ms b) (sample-timestamp-ms a))))))))

(defun average-rate (m window-ms)
  (let ((samples (monitor-samples m)))
    (if (null samples) 0.0
        (let* ((latest-s (car (last samples)))
               (min-ts (max 0 (- (sample-timestamp-ms latest-s) window-ms)))
               (earliest nil))
          (loop for s in (reverse samples)
                while (>= (sample-timestamp-ms s) min-ts)
                do (setf earliest s))
          (cond
            ((or (null earliest) (eq earliest latest-s)) 0.0)
            ((< (sample-bytes-total latest-s) (sample-bytes-total earliest)) 0.0)
            (t
             (let ((dt (- (sample-timestamp-ms latest-s) (sample-timestamp-ms earliest))))
               (if (<= dt 0) 0.0
                   (/ (* 1000.0 (- (sample-bytes-total latest-s) (sample-bytes-total earliest)))
                      dt)))))))))

(defun peak-rate (m window-ms)
  (let ((samples (monitor-samples m)))
    (if (null samples) 0.0
        (let* ((latest-s (car (last samples)))
               (min-ts (max 0 (- (sample-timestamp-ms latest-s) window-ms)))
               (prev nil) (peak 0.0))
          (dolist (s samples peak)
            (when (and prev
                       (>= (sample-timestamp-ms s) min-ts)
                       (> (sample-timestamp-ms s) (sample-timestamp-ms prev))
                       (>= (sample-bytes-total s) (sample-bytes-total prev)))
              (let ((rate (/ (* 1000.0 (- (sample-bytes-total s) (sample-bytes-total prev)))
                             (- (sample-timestamp-ms s) (sample-timestamp-ms prev)))))
                (when (> rate peak) (setf peak rate))))
            (setf prev s))))))
