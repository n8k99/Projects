;;;; Wallpaper Manager — multi-monitor rotation + resolution best-fit + recency.

(defpackage #:wallpaper-manager
  (:use #:cl)
  (:export #:make-resolution #:resolution-width #:resolution-height
           #:resolution-fit-score #:best-variant-for-monitor
           #:make-monitor #:make-wallpaper #:make-variant
           #:wp-wallpaper-id #:wp-wallpaper-variants
           #:make-manager #:manager-add-monitor #:manager-add-wallpaper
           #:manager-tick #:manager-rotate-monitor
           #:manager-current-variant-for-monitor
           #:manager-assignments #:manager-events #:manager-elapsed-ms
           #:make-schedule-interval #:make-schedule-fixed-list
           #:make-schedule-tag-window
           #:monitor-state-current-wallpaper-id
           #:monitor-state-history))

(in-package #:wallpaper-manager)

(defstruct resolution
  (width 0 :type integer)
  (height 0 :type integer))

(defun resolution-equal-p (a b)
  (and (= (resolution-width a) (resolution-width b))
       (= (resolution-height a) (resolution-height b))))

(defun resolution-aspect-ratio (r)
  (/ (resolution-width r) (resolution-height r) 1.0))

(defun resolution-pixel-count (r)
  (* (resolution-width r) (resolution-height r)))

(defstruct monitor
  (id 0 :type integer)
  (resolution nil :type (or null resolution)))

(defstruct variant
  (resolution nil :type (or null resolution))
  (path "" :type string))

(defstruct (wp-wallpaper (:conc-name wp-wallpaper-))
  (id 0 :type integer)
  (name "" :type string)
  (tags nil :type list)
  (variants nil :type list))

(defun resolution-fit-score (monitor variant)
  (if (resolution-equal-p monitor variant)
      1000
      (let* ((m-ar (resolution-aspect-ratio monitor))
             (v-ar (resolution-aspect-ratio variant))
             (ar-diff (abs (- m-ar v-ar)))
             (ar-score (floor (* (/ (- 0.5 (min ar-diff 0.5)) 0.5) 500)))
             (m-px (resolution-pixel-count monitor))
             (v-px (resolution-pixel-count variant))
             (ratio (/ v-px m-px 1.0))
             (px-score (if (>= ratio 1.0)
                            (let ((denom (max 1.0 (min 2.0 (max 1.0 (log ratio 2))))))
                              (floor (/ 300 denom)))
                            (floor (* (max 0.0 (- ratio 0.5)) 600)))))
        (+ ar-score px-score))))

(defun best-variant-for-monitor (wallpaper monitor-res)
  (when (wp-wallpaper-variants wallpaper)
    (let ((best (first (wp-wallpaper-variants wallpaper)))
          (best-score (resolution-fit-score monitor-res
                                              (variant-resolution
                                               (first (wp-wallpaper-variants wallpaper))))))
      (dolist (v (rest (wp-wallpaper-variants wallpaper)))
        (let ((s (resolution-fit-score monitor-res (variant-resolution v))))
          (when (> s best-score)
            (setf best v best-score s))))
      best)))

;; Schedule as a plist with :kind and kind-specific fields.
(defun make-schedule-interval (ms) (list :kind :interval :ms ms))
(defun make-schedule-fixed-list (times) (list :kind :fixed-list :times times))
(defun make-schedule-tag-window (tag start-ms end-ms inner-ms)
  (list :kind :tag-window :tag tag :start-ms start-ms
         :end-ms end-ms :inner-ms inner-ms))

(defstruct manager-event
  (kind :rotated :type keyword)
  (monitor-id 0 :type integer)
  (at-ms 0 :type integer)
  (wallpaper-id nil :type (or null integer)))

(defstruct monitor-state
  (current-wallpaper-id nil :type (or null integer))
  (last-rotation-ms 0 :type integer)
  (history nil :type list))

(defconstant +lcg-mul+ 6364136223846793005)
(defconstant +lcg-inc+ 1442695040888963407)
(defconstant +u64+ (1- (expt 2 64)))

(defstruct manager
  (schedule nil :type list)
  (recency-window 0 :type integer)
  (lcg-state 1 :type integer)
  (monitors nil :type list)
  (wallpapers nil :type list)
  ;; assignments kept as assoc list: (monitor-id . monitor-state)
  (assignments nil :type list)
  (elapsed-ms 0 :type integer)
  (events nil :type list))

(defun manager-new (schedule recency-window seed)
  (make-manager :schedule schedule
                 :recency-window recency-window
                 :lcg-state (logand (1+ seed) +u64+)))

(defun manager-add-monitor (mgr m)
  (setf (manager-monitors mgr) (append (manager-monitors mgr) (list m)))
  (setf (manager-assignments mgr)
        (append (manager-assignments mgr)
                (list (cons (monitor-id m) (make-monitor-state))))))

(defun manager-add-wallpaper (mgr w)
  (setf (manager-wallpapers mgr) (append (manager-wallpapers mgr) (list w))))

(defun manager-state-for (mgr monitor-id)
  (cdr (assoc monitor-id (manager-assignments mgr))))

(defun lcg-next (mgr)
  (setf (manager-lcg-state mgr)
        (logand (+ (* (manager-lcg-state mgr) +lcg-mul+) +lcg-inc+) +u64+))
  (manager-lcg-state mgr))

(defun matches-tag-p (w tag-filter)
  (or (null tag-filter)
      (member tag-filter (wp-wallpaper-tags w) :test #'string=)))

(defun manager-pick-next-for-monitor (mgr monitor-id tag-filter)
  (let* ((state (manager-state-for mgr monitor-id))
         (history (monitor-state-history state))
         (recent (subseq (reverse history)
                          0 (min (length history) (manager-recency-window mgr))))
         (eligible (loop for w in (manager-wallpapers mgr)
                         when (and (not (member (wp-wallpaper-id w) recent))
                                    (matches-tag-p w tag-filter))
                         collect (wp-wallpaper-id w))))
    (cond
      (eligible
       (let ((r (lcg-next mgr)))
         (nth (mod r (length eligible)) eligible)))
      (t
       (let ((fallback (loop for w in (manager-wallpapers mgr)
                              when (matches-tag-p w tag-filter)
                              collect (wp-wallpaper-id w))))
         (if fallback
             (let ((r (lcg-next mgr)))
               (nth (mod r (length fallback)) fallback))
             nil))))))

(defun manager-rotate-monitor (mgr monitor-id &optional tag-filter)
  (let ((pick (manager-pick-next-for-monitor mgr monitor-id tag-filter)))
    (cond
      ((null pick)
       (setf (manager-events mgr)
             (append (manager-events mgr)
                     (list (make-manager-event
                            :kind :skipped-no-eligible
                            :monitor-id monitor-id
                            :at-ms (manager-elapsed-ms mgr)))))
       nil)
      (t
       (let ((state (manager-state-for mgr monitor-id)))
         (setf (monitor-state-current-wallpaper-id state) pick)
         (setf (monitor-state-last-rotation-ms state) (manager-elapsed-ms mgr))
         (setf (monitor-state-history state)
               (append (monitor-state-history state) (list pick))))
       (setf (manager-events mgr)
             (append (manager-events mgr)
                     (list (make-manager-event
                            :kind :rotated
                            :monitor-id monitor-id
                            :at-ms (manager-elapsed-ms mgr)
                            :wallpaper-id pick))))
       t))))

(defun should-rotate-p (mgr monitor-id)
  (let ((state (manager-state-for mgr monitor-id))
        (s (manager-schedule mgr)))
    (case (getf s :kind)
      (:interval
       (or (null (monitor-state-current-wallpaper-id state))
           (>= (- (manager-elapsed-ms mgr)
                   (monitor-state-last-rotation-ms state))
               (getf s :ms))))
      (:fixed-list
       (some (lambda (tt)
                (and (< (monitor-state-last-rotation-ms state) tt)
                     (<= tt (manager-elapsed-ms mgr))))
              (getf s :times)))
      (:tag-window
       (or (null (monitor-state-current-wallpaper-id state))
           (>= (- (manager-elapsed-ms mgr)
                   (monitor-state-last-rotation-ms state))
               (getf s :inner-ms)))))))

(defun active-tag (mgr)
  (let ((s (manager-schedule mgr)))
    (when (eq (getf s :kind) :tag-window)
      (let* ((day-ms (* 24 60 60 1000))
             (in-day (mod (manager-elapsed-ms mgr) day-ms)))
        (when (and (<= (getf s :start-ms) in-day)
                    (< in-day (getf s :end-ms)))
          (getf s :tag))))))

(defun manager-tick (mgr ms)
  (incf (manager-elapsed-ms mgr) ms)
  (dolist (m (manager-monitors mgr))
    (when (should-rotate-p mgr (monitor-id m))
      (manager-rotate-monitor mgr (monitor-id m) (active-tag mgr)))))

(defun manager-current-variant-for-monitor (mgr monitor-id)
  (let* ((state (manager-state-for mgr monitor-id))
         (wid (and state (monitor-state-current-wallpaper-id state))))
    (when wid
      (let ((monitor (find monitor-id (manager-monitors mgr) :key #'monitor-id))
            (wallpaper (find wid (manager-wallpapers mgr) :key #'wp-wallpaper-id)))
        (when (and monitor wallpaper)
          (best-variant-for-monitor wallpaper (monitor-resolution monitor)))))))

(defun demo ()
  (let ((mgr (manager-new (make-schedule-interval 1000) 2 42)))
    (manager-add-monitor mgr (make-monitor :id 1
                                             :resolution (make-resolution :width 1920 :height 1080)))
    (manager-add-monitor mgr (make-monitor :id 2
                                             :resolution (make-resolution :width 2560 :height 1440)))
    (dolist (spec '((10 ("nature")) (11 ("abstract")) (12 ("nature"))
                     (13 ("abstract")) (14 ("nature"))))
      (manager-add-wallpaper mgr
        (make-wp-wallpaper
         :id (first spec)
         :name (format nil "wp~D" (first spec))
         :tags (second spec)
         :variants (list (make-variant :resolution (make-resolution :width 1920 :height 1080)
                                         :path (format nil "/wp/~D/1080.png" (first spec)))
                          (make-variant :resolution (make-resolution :width 2560 :height 1440)
                                         :path (format nil "/wp/~D/1440.png" (first spec)))))))

    (manager-tick mgr 1)
    (format t "t=1ms: m1=~A m2=~A~%"
            (monitor-state-current-wallpaper-id (manager-state-for mgr 1))
            (monitor-state-current-wallpaper-id (manager-state-for mgr 2)))
    (manager-tick mgr 1100)
    (format t "t=1101ms: m1=~A m2=~A~%"
            (monitor-state-current-wallpaper-id (manager-state-for mgr 1))
            (monitor-state-current-wallpaper-id (manager-state-for mgr 2)))
    (manager-tick mgr 1100)
    (format t "t=2201ms: m1=~A m2=~A~%"
            (monitor-state-current-wallpaper-id (manager-state-for mgr 1))
            (monitor-state-current-wallpaper-id (manager-state-for mgr 2)))

    (let ((v (manager-current-variant-for-monitor mgr 2)))
      (format t "m2 variant: ~Dx~D~%"
              (resolution-width (variant-resolution v))
              (resolution-height (variant-resolution v))))

    (let ((m1080 (make-resolution :width 1920 :height 1080)))
      (format t "score 1920x1080 vs 1920x1080: ~D~%"
              (resolution-fit-score m1080 (make-resolution :width 1920 :height 1080)))
      (format t "score 1920x1080 vs 1280x720:  ~D~%"
              (resolution-fit-score m1080 (make-resolution :width 1280 :height 720)))
      (format t "score 1920x1080 vs 2000x1500: ~D~%"
              (resolution-fit-score m1080 (make-resolution :width 2000 :height 1500))))

    (format t "history m1: ~A~%" (monitor-state-history (manager-state-for mgr 1)))
    (format t "history m2: ~A~%" (monitor-state-history (manager-state-for mgr 2)))))
