;;;; YouTube Downloader — concurrency-limited queue with resumable downloads.

(defpackage #:youtube-downloader
  (:use #:cl)
  (:export #:make-video-format #:video-format-id #:video-format-resolution
           #:video-format-size-bytes
           #:pick-format #:make-retry-policy #:retry-policy-backoff-for-attempt
           #:make-manager #:manager-enqueue #:manager-tick
           #:manager-report-failure #:manager-download #:manager-active-count
           #:manager-pending-queue #:manager-events #:manager-elapsed-ms
           #:download-state-kind #:download-bytes-done #:download-bytes-total
           #:download-attempts))

(in-package #:youtube-downloader)

(defstruct video-format
  (id "" :type string)
  (resolution 0 :type integer)
  (codec "" :type string)
  (container "" :type string)
  (size-bytes 0 :type integer)
  (bitrate-bps 0 :type integer))

(defun pick-format (formats policy)
  "policy is (:max-quality) | (:lowest) | (:best-under cap)."
  (when formats
    (case (first policy)
      (:max-quality
       (reduce (lambda (a b) (if (> (video-format-resolution a)
                                     (video-format-resolution b))
                                  a b))
               formats))
      (:lowest
       (reduce (lambda (a b) (if (< (video-format-resolution a)
                                     (video-format-resolution b))
                                  a b))
               formats))
      (:best-under
       (let* ((cap (second policy))
              (candidates (remove-if (lambda (f) (> (video-format-size-bytes f) cap))
                                      formats)))
         (when candidates
           (reduce (lambda (a b) (if (> (video-format-resolution a)
                                         (video-format-resolution b))
                                      a b))
                   candidates)))))))

(defstruct retry-policy
  (max-attempts 3 :type integer)
  (initial-ms 1000 :type integer)
  (multiplier 2 :type integer))

(defun retry-policy-backoff-for-attempt (policy n)
  (if (zerop n)
      0
      (let ((ms (retry-policy-initial-ms policy)))
        (loop repeat (1- n) do (setf ms (* ms (retry-policy-multiplier policy))))
        ms)))

(defstruct download-state
  (kind :pending :type keyword)
  (ready-at-ms 0 :type integer))

(defstruct download
  (id 0 :type integer)
  (video-id "" :type string)
  (format nil :type (or null video-format))
  (bytes-done 0 :type integer)
  (bytes-total 0 :type integer)
  (attempts 0 :type integer)
  (state (make-download-state) :type download-state))

(defstruct manager-event
  (kind :enqueued :type keyword)
  (id 0 :type integer)
  (detail "" :type string))

(defstruct manager
  (max-concurrent 1 :type integer)
  (bandwidth-bps 0 :type integer)
  (retry-policy (make-retry-policy) :type retry-policy)
  (downloads nil :type list)
  (pending-queue nil :type list)
  (elapsed-ms 0 :type integer)
  (events nil :type list)
  (next-id 1 :type integer))

(defun manager-enqueue (mgr video-id format)
  (let ((id (manager-next-id mgr)))
    (incf (manager-next-id mgr))
    (let ((d (make-download :id id :video-id video-id :format format
                             :bytes-total (video-format-size-bytes format))))
      (setf (manager-downloads mgr) (append (manager-downloads mgr) (list d)))
      (setf (manager-pending-queue mgr)
            (append (manager-pending-queue mgr) (list id)))
      (setf (manager-events mgr)
            (append (manager-events mgr)
                    (list (make-manager-event :kind :enqueued :id id)))))
    id))

(defun manager-download (mgr id)
  (find id (manager-downloads mgr) :key #'download-id))

(defun manager-active-count (mgr)
  (count-if (lambda (d) (eq (download-state-kind (download-state d)) :in-flight))
            (manager-downloads mgr)))

(defun promote-retries (mgr)
  (let ((now (manager-elapsed-ms mgr)))
    (dolist (d (manager-downloads mgr))
      (let ((s (download-state d)))
        (when (and (eq (download-state-kind s) :retrying)
                    (<= (download-state-ready-at-ms s) now))
          (setf (download-state d) (make-download-state :kind :pending))
          (setf (manager-pending-queue mgr)
                (append (manager-pending-queue mgr) (list (download-id d)))))))))

(defun fill-slots (mgr)
  (loop while (and (< (manager-active-count mgr) (manager-max-concurrent mgr))
                    (manager-pending-queue mgr))
        do
        (let* ((id (pop (manager-pending-queue mgr)))
               (d (manager-download mgr id)))
          (when (and d (eq (download-state-kind (download-state d)) :pending))
            (incf (download-attempts d))
            (setf (download-state d) (make-download-state :kind :in-flight))
            (setf (manager-events mgr)
                  (append (manager-events mgr)
                          (list (make-manager-event
                                 :kind :started :id id
                                 :detail (format nil "attempt ~D"
                                                  (download-attempts d))))))))))

(defun manager-report-failure (mgr id)
  (let ((d (manager-download mgr id)))
    (when (and d (eq (download-state-kind (download-state d)) :in-flight))
      (let ((policy (manager-retry-policy mgr)))
        (cond
          ((< (download-attempts d) (retry-policy-max-attempts policy))
           (let* ((backoff (retry-policy-backoff-for-attempt policy (download-attempts d)))
                  (ready (+ (manager-elapsed-ms mgr) backoff)))
             (setf (download-state d)
                   (make-download-state :kind :retrying :ready-at-ms ready))
             (setf (manager-events mgr)
                   (append (manager-events mgr)
                           (list (make-manager-event
                                  :kind :failed :id id
                                  :detail (format nil "attempt ~D"
                                                   (download-attempts d)))
                                  (make-manager-event
                                   :kind :retry-scheduled :id id
                                   :detail (format nil "ready ~D" ready)))))))
          (t
           (setf (download-state d) (make-download-state :kind :failed))
           (setf (manager-events mgr)
                 (append (manager-events mgr)
                         (list (make-manager-event
                                :kind :failed :id id
                                :detail (format nil "attempt ~D"
                                                 (download-attempts d)))
                                (make-manager-event
                                 :kind :abandoned :id id))))))))))

(defun manager-tick (mgr ms)
  (incf (manager-elapsed-ms mgr) ms)
  (promote-retries mgr)
  (fill-slots mgr)
  (let ((active (remove-if-not
                  (lambda (d)
                    (eq (download-state-kind (download-state d)) :in-flight))
                  (manager-downloads mgr))))
    (when active
      (let* ((per-stream-bps (floor (manager-bandwidth-bps mgr) (length active)))
             (bytes-this-tick (floor (* per-stream-bps ms) 8000)))
        (dolist (d active)
          (let ((before (download-bytes-done d)))
            (setf (download-bytes-done d)
                  (min (+ (download-bytes-done d) bytes-this-tick)
                        (download-bytes-total d)))
            (when (> (download-bytes-done d) before)
              (setf (manager-events mgr)
                    (append (manager-events mgr)
                            (list (make-manager-event
                                   :kind :progress :id (download-id d)
                                   :detail (format nil "~D/~D"
                                                    (download-bytes-done d)
                                                    (download-bytes-total d)))))))
            (when (>= (download-bytes-done d) (download-bytes-total d))
              (setf (download-state d) (make-download-state :kind :succeeded))
              (setf (manager-events mgr)
                    (append (manager-events mgr)
                            (list (make-manager-event
                                   :kind :completed :id (download-id d))))))))))))

(defun demo ()
  (let ((formats (list (make-video-format :id "a" :resolution 480
                                            :codec "avc1" :container "mp4"
                                            :size-bytes 50000000 :bitrate-bps 500000)
                         (make-video-format :id "b" :resolution 720
                                            :codec "avc1" :container "mp4"
                                            :size-bytes 100000000 :bitrate-bps 1000000)
                         (make-video-format :id "c" :resolution 1080
                                            :codec "avc1" :container "mp4"
                                            :size-bytes 200000000 :bitrate-bps 2000000))))
    (format t "format picks:~%")
    (format t "  max_quality -> ~A~%"
            (video-format-id (pick-format formats '(:max-quality))))
    (format t "  lowest -> ~A~%"
            (video-format-id (pick-format formats '(:lowest))))
    (format t "  under 120MB -> ~A~%"
            (video-format-id (pick-format formats '(:best-under 120000000))))

    (let ((mgr (make-manager :max-concurrent 2 :bandwidth-bps 8000000
                              :retry-policy (make-retry-policy :max-attempts 3
                                                                 :initial-ms 1000
                                                                 :multiplier 2))))
      (manager-enqueue mgr "v1" (nth 1 formats))
      (manager-enqueue mgr "v2" (nth 0 formats))
      (manager-enqueue mgr "v3" (nth 2 formats))

      (manager-tick mgr 1000)
      (format t "after 1s: active=~D pending=~D~%"
              (manager-active-count mgr) (length (manager-pending-queue mgr)))
      (dolist (i '(1 2 3))
        (let ((d (manager-download mgr i)))
          (format t "  [~D] ~A ~D/~D~%" i
                  (download-state-kind (download-state d))
                  (download-bytes-done d) (download-bytes-total d))))

      (manager-report-failure mgr 1)
      (let ((d1 (manager-download mgr 1)))
        (format t "after failure: [1] ~A ready_at=~D attempts=~D~%"
                (download-state-kind (download-state d1))
                (download-state-ready-at-ms (download-state d1))
                (download-attempts d1)))

      (format t "backoff for retry 1: ~Dms~%"
              (retry-policy-backoff-for-attempt (manager-retry-policy mgr) 1))
      (format t "backoff for retry 2: ~Dms~%"
              (retry-policy-backoff-for-attempt (manager-retry-policy mgr) 2))
      (format t "backoff for retry 3: ~Dms~%"
              (retry-policy-backoff-for-attempt (manager-retry-policy mgr) 3)))))
