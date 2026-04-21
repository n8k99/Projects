;;;; Traffic Light — coordinated super-state FSM + preemption + adaptive timing.

(defpackage #:traffic-light
  (:use #:cl)
  (:export #:make-config #:make-junction
           #:junction-phase #:junction-phase-elapsed-ms
           #:junction-ns-light #:junction-ew-light
           #:junction-ped-signal
           #:junction-tick #:junction-set-ns-queue
           #:junction-set-ew-queue #:junction-request-ped-crossing
           #:junction-begin-emergency #:junction-end-emergency
           #:junction-phase-duration-ms #:junction-events
           #:junction-total-elapsed-ms))

(in-package #:traffic-light)

(defstruct config
  (ns-green-ms 20000 :type integer)
  (ns-yellow-ms 3000 :type integer)
  (ew-green-ms 20000 :type integer)
  (ew-yellow-ms 3000 :type integer)
  (all-red-ms 2000 :type integer)
  (max-green-extension-ms 10000 :type integer)
  (ped-walk-ms 8000 :type integer)
  (ped-flashing-ms 5000 :type integer))

(defstruct junction-event
  (kind :phase-changed :type keyword)
  (at-ms 0 :type integer)
  (detail "" :type string))

(defstruct junction
  (phase :ns-green :type keyword)
  (phase-elapsed-ms 0 :type integer)
  (phase-duration-ms 0 :type integer)
  (cycle-side :ew :type keyword) ; after AllRed, go to this direction
  (config nil :type (or null config))
  (ns-queue 0 :type integer)
  (ew-queue 0 :type integer)
  (ped-requested nil :type boolean)
  (ped-state :inactive :type keyword)
  (ped-elapsed-ms 0 :type integer)
  (saved-phase nil :type list)
  (total-elapsed-ms 0 :type integer)
  (events nil :type list))

(defun duration-for (config phase)
  (case phase
    (:ns-green (config-ns-green-ms config))
    (:ns-yellow (config-ns-yellow-ms config))
    (:ew-green (config-ew-green-ms config))
    (:ew-yellow (config-ew-yellow-ms config))
    (:all-red (config-all-red-ms config))
    (:emergency-all-red (expt 10 18))))

(defun init-junction (i)
  (setf (junction-phase-duration-ms i)
        (duration-for (junction-config i) (junction-phase i)))
  i)

(defun make-new-junction (config)
  (init-junction
   (make-junction :config config :phase :ns-green :cycle-side :ew)))

(defun junction-ns-light (i)
  (case (junction-phase i)
    (:ns-green :green)
    (:ns-yellow :yellow)
    (t :red)))

(defun junction-ew-light (i)
  (case (junction-phase i)
    (:ew-green :green)
    (:ew-yellow :yellow)
    (t :red)))

(defun junction-ped-signal (i)
  (case (junction-ped-state i)
    (:inactive :dont-walk)
    (:walk :walk)
    (:flashing :flashing)))

(defun junction-set-ns-queue (i n) (setf (junction-ns-queue i) n))
(defun junction-set-ew-queue (i n) (setf (junction-ew-queue i) n))

(defun junction-request-ped-crossing (i)
  (when (and (eq (junction-ped-state i) :inactive)
              (not (junction-ped-requested i)))
    (setf (junction-ped-requested i) t)
    (setf (junction-events i)
          (append (junction-events i)
                  (list (make-junction-event
                         :kind :ped-requested
                         :at-ms (junction-total-elapsed-ms i)))))))

(defun junction-begin-emergency (i)
  (unless (eq (junction-phase i) :emergency-all-red)
    (setf (junction-saved-phase i)
          (list (junction-phase i)
                (junction-phase-elapsed-ms i)
                (junction-cycle-side i)))
    (let ((at (junction-total-elapsed-ms i))
          (from (junction-phase i)))
      (setf (junction-phase i) :emergency-all-red)
      (setf (junction-phase-elapsed-ms i) 0)
      (setf (junction-phase-duration-ms i) (expt 10 18))
      (setf (junction-ped-state i) :inactive)
      (setf (junction-events i)
            (append (junction-events i)
                    (list (make-junction-event
                           :kind :emergency-started :at-ms at)
                           (make-junction-event
                            :kind :phase-changed :at-ms at
                            :detail (format nil "~A -> emergency-all-red" from))))))))

(defun junction-end-emergency (i)
  (when (eq (junction-phase i) :emergency-all-red)
    (let ((at (junction-total-elapsed-ms i))
          (from (junction-phase i))
          (saved (junction-saved-phase i)))
      (cond
        (saved
         (setf (junction-phase i) (first saved))
         (setf (junction-phase-elapsed-ms i) (second saved))
         (setf (junction-phase-duration-ms i)
               (duration-for (junction-config i) (first saved)))
         (setf (junction-cycle-side i) (third saved))
         (setf (junction-saved-phase i) nil))
        (t
         (setf (junction-phase i) :all-red)
         (setf (junction-phase-elapsed-ms i) 0)
         (setf (junction-phase-duration-ms i)
               (config-all-red-ms (junction-config i)))))
      (setf (junction-events i)
            (append (junction-events i)
                    (list (make-junction-event :kind :emergency-ended :at-ms at)
                          (make-junction-event
                           :kind :phase-changed :at-ms at
                           :detail (format nil "~A -> ~A" from
                                            (junction-phase i)))))))))

(defun consider-extension (i)
  (let ((direction (case (junction-phase i)
                      (:ns-green :ns)
                      (:ew-green :ew))))
    (when direction
      (let ((queue (if (eq direction :ns)
                        (junction-ns-queue i)
                        (junction-ew-queue i))))
        (when (plusp queue)
          (let* ((config (junction-config i))
                 (base (if (eq direction :ns)
                            (config-ns-green-ms config)
                            (config-ew-green-ms config)))
                 (already (max 0 (- (junction-phase-duration-ms i) base)))
                 (remaining-cap (max 0 (- (config-max-green-extension-ms config)
                                            already))))
            (when (plusp remaining-cap)
              (let ((grant (min (* queue 2000) remaining-cap)))
                (incf (junction-phase-duration-ms i) grant)
                (setf (junction-events i)
                      (append (junction-events i)
                              (list (make-junction-event
                                     :kind :green-extended
                                     :at-ms (junction-total-elapsed-ms i)
                                     :detail (format nil "~A +~Dms" direction grant)))))))))))))

(defun next-phase (i)
  (case (junction-phase i)
    (:ns-green (list :ns-yellow (junction-cycle-side i)))
    (:ns-yellow (list :all-red :ew))
    (:all-red (if (eq (junction-cycle-side i) :ew)
                   (list :ew-green :ns)
                   (list :ns-green :ew)))
    (:ew-green (list :ew-yellow (junction-cycle-side i)))
    (:ew-yellow (list :all-red :ns))
    (:emergency-all-red (list :emergency-all-red (junction-cycle-side i)))))

(defun transition-phase (i)
  (destructuring-bind (new-phase new-side) (next-phase i)
    (let ((from (junction-phase i)))
      (setf (junction-phase i) new-phase)
      (setf (junction-cycle-side i) new-side)
      (setf (junction-phase-elapsed-ms i) 0)
      (setf (junction-phase-duration-ms i)
            (duration-for (junction-config i) new-phase))
      (setf (junction-events i)
            (append (junction-events i)
                    (list (make-junction-event
                           :kind :phase-changed
                           :at-ms (junction-total-elapsed-ms i)
                           :detail (format nil "~A -> ~A" from new-phase)))))
      (when (and (eq new-phase :all-red) (junction-ped-requested i))
        (setf (junction-ped-requested i) nil)
        (setf (junction-ped-state i) :walk)
        (setf (junction-ped-elapsed-ms i) 0)
        (setf (junction-events i)
              (append (junction-events i)
                      (list (make-junction-event
                             :kind :ped-walk-started
                             :at-ms (junction-total-elapsed-ms i)))))
        (return-from transition-phase t)))
    nil))

(defun advance-pedestrian (i ms)
  (unless (eq (junction-ped-state i) :inactive)
    (incf (junction-ped-elapsed-ms i) ms)
    (cond
      ((and (eq (junction-ped-state i) :walk)
             (>= (junction-ped-elapsed-ms i)
                 (config-ped-walk-ms (junction-config i))))
       (setf (junction-ped-state i) :flashing)
       (setf (junction-ped-elapsed-ms i) 0))
      ((and (eq (junction-ped-state i) :flashing)
             (>= (junction-ped-elapsed-ms i)
                 (config-ped-flashing-ms (junction-config i))))
       (setf (junction-ped-state i) :inactive)
       (setf (junction-ped-elapsed-ms i) 0)
       (setf (junction-events i)
             (append (junction-events i)
                     (list (make-junction-event
                            :kind :ped-walk-ended
                            :at-ms (junction-total-elapsed-ms i)))))))))

(defun junction-tick (i ms)
  (incf (junction-total-elapsed-ms i) ms)
  (when (eq (junction-phase i) :emergency-all-red) (return-from junction-tick))
  (incf (junction-phase-elapsed-ms i) ms)
  (when (and (member (junction-phase i) '(:ns-green :ew-green))
              (>= (junction-phase-elapsed-ms i)
                  (junction-phase-duration-ms i)))
    (consider-extension i))
  (let ((ped-just-started-remaining nil))
    (loop while (and (>= (junction-phase-elapsed-ms i)
                          (junction-phase-duration-ms i))
                      (not (eq (junction-phase i) :emergency-all-red)))
          do
          (let* ((overflow (- (junction-phase-elapsed-ms i)
                               (junction-phase-duration-ms i)))
                 (started (transition-phase i)))
            (setf (junction-phase-elapsed-ms i)
                  (min overflow (junction-phase-duration-ms i)))
            (when started (setf ped-just-started-remaining overflow))))
    (let ((ped-advance
            (cond
              ((eq (junction-ped-state i) :inactive) 0)
              (ped-just-started-remaining ped-just-started-remaining)
              (t ms))))
      (advance-pedestrian i ped-advance))))

(defun demo ()
  (let ((cfg (make-config
               :ns-green-ms 1000 :ns-yellow-ms 200
               :ew-green-ms 1000 :ew-yellow-ms 200
               :all-red-ms 100 :max-green-extension-ms 500
               :ped-walk-ms 500 :ped-flashing-ms 300)))
    (let ((i (make-new-junction cfg)))
      (format t "initial: phase=~A ns=~A ew=~A~%"
              (junction-phase i)
              (junction-ns-light i)
              (junction-ew-light i))
      (junction-tick i 1000)
      (format t "after 1000ms: phase=~A~%" (junction-phase i))
      (junction-tick i 200)
      (format t "after +200ms: phase=~A~%" (junction-phase i))
      (junction-tick i 100)
      (format t "after +100ms: phase=~A~%" (junction-phase i)))

    (let ((i (make-new-junction cfg)))
      (junction-set-ns-queue i 3)
      (junction-tick i 1000)
      (format t "with queue: phase=~A duration=~D~%"
              (junction-phase i) (junction-phase-duration-ms i)))

    (let ((i (make-new-junction cfg)))
      (junction-tick i 300)
      (junction-begin-emergency i)
      (format t "emergency started: phase=~A~%" (junction-phase i))
      (junction-tick i 10000)
      (format t "frozen: phase=~A~%" (junction-phase i))
      (junction-end-emergency i)
      (format t "resumed: phase=~A elapsed=~D~%"
              (junction-phase i) (junction-phase-elapsed-ms i)))

    (let ((i (make-new-junction cfg)))
      (junction-request-ped-crossing i)
      (format t "ped requested, signal=~A~%" (junction-ped-signal i))
      (junction-tick i 1200)
      (format t "after 1200ms: phase=~A ped=~A~%"
              (junction-phase i) (junction-ped-signal i))
      (junction-tick i 500)
      (format t "after +500ms: ped=~A~%" (junction-ped-signal i))
      (junction-tick i 300)
      (format t "after +300ms: ped=~A~%" (junction-ped-signal i)))))
