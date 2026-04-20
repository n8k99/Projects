;;;; Travel Planner System — leg-chained itineraries with integrity validation.

(defpackage #:travel-planner
  (:use #:cl)
  (:export #:make-leg #:make-trip #:trip-add-leg #:validate-trip #:layovers
           #:leg-duration-ms #:trip-total-span-ms #:trip-in-transit-ms
           #:trip-total-cost-cents #:trip-total-layover-ms
           #:leg-origin #:leg-destination #:leg-depart-ms #:leg-arrive-ms
           #:leg-cost-cents #:leg-carrier #:leg-mode #:leg-id
           #:trip-legs #:trip-title #:trip-min-layover-minutes
           #:layover-location #:layover-arrive-ms #:layover-depart-ms #:layover-duration-ms
           #:issue-severity #:issue-kind #:issue-leg-idx #:issue-detail))

(in-package #:travel-planner)

(defstruct leg
  (id 0 :type integer)
  (mode :flight :type keyword)
  (origin "" :type string)
  (destination "" :type string)
  (depart-ms 0 :type integer)
  (arrive-ms 0 :type integer)
  (cost-cents 0 :type integer)
  (carrier "" :type string))

(defun leg-duration-ms (l)
  (- (leg-arrive-ms l) (leg-depart-ms l)))

(defstruct trip
  (id 0 :type integer)
  (title "" :type string)
  (legs nil :type list)
  (min-layover-minutes 30 :type integer))

(defun trip-add-leg (trip leg)
  (setf (trip-legs trip) (append (trip-legs trip) (list leg))))

(defun trip-total-span-ms (trip)
  (if (null (trip-legs trip))
      0
      (- (leg-arrive-ms (car (last (trip-legs trip))))
         (leg-depart-ms (first (trip-legs trip))))))

(defun trip-in-transit-ms (trip)
  (reduce #'+ (trip-legs trip) :key #'leg-duration-ms :initial-value 0))

(defun trip-total-cost-cents (trip)
  (reduce #'+ (trip-legs trip) :key #'leg-cost-cents :initial-value 0))

(defun trip-total-layover-ms (trip)
  (- (trip-total-span-ms trip) (trip-in-transit-ms trip)))

(defstruct layover
  (location "" :type string)
  (arrive-ms 0 :type integer)
  (depart-ms 0 :type integer)
  (duration-ms 0 :type integer))

(defun layovers (trip)
  (let ((out nil)
        (legs (trip-legs trip)))
    (loop for (prev next) on legs
          while next
          do (push (make-layover :location (leg-destination prev)
                                   :arrive-ms (leg-arrive-ms prev)
                                   :depart-ms (leg-depart-ms next)
                                   :duration-ms (- (leg-depart-ms next)
                                                    (leg-arrive-ms prev)))
                   out))
    (nreverse out)))

(defstruct issue
  (severity :info :type keyword)
  (kind :info :type keyword)
  (leg-idx 0 :type integer)
  (detail "" :type string))

(defun validate-trip (trip)
  (let ((issues nil)
        (legs (trip-legs trip)))
    (loop for leg in legs
          for i from 0
          for d = (leg-duration-ms leg)
          do (cond
               ((zerop d)
                (push (make-issue :severity :warning :kind :zero-duration-leg
                                   :leg-idx i) issues))
               ((< d 0)
                (push (make-issue :severity :error :kind :negative-duration-leg
                                   :leg-idx i) issues))))
    (loop for (prev next) on legs
          for i from 0
          while next
          do (let ((idx (1+ i)))
               (unless (string= (leg-destination prev) (leg-origin next))
                 (push (make-issue :severity :error :kind :location-mismatch
                                     :leg-idx idx
                                     :detail (format nil "~A -> ~A"
                                                       (leg-destination prev)
                                                       (leg-origin next)))
                       issues))
               (cond
                 ((< (leg-depart-ms next) (leg-arrive-ms prev))
                  (push (make-issue :severity :error :kind :time-conflict
                                      :leg-idx idx
                                      :detail (format nil "arrive=~D depart=~D"
                                                        (leg-arrive-ms prev)
                                                        (leg-depart-ms next)))
                        issues))
                 (t
                  (let* ((layover-ms (- (leg-depart-ms next) (leg-arrive-ms prev)))
                         (layover-min (floor layover-ms 60000)))
                    (when (and (> layover-ms 0)
                                (< layover-min (trip-min-layover-minutes trip)))
                      (push (make-issue :severity :warning :kind :tight-layover
                                          :leg-idx idx
                                          :detail (format nil "~D min" layover-min))
                            issues)))))))
    (nreverse issues)))

(defun demo ()
  (let ((trip (make-trip :id 1 :title "NYC -> SFO -> LAX")))
    (trip-add-leg trip (make-leg :id 1 :origin "JFK" :destination "SFO"
                                   :depart-ms 100000000 :arrive-ms 120000000
                                   :cost-cents 30000 :carrier "ACME Air"))
    (trip-add-leg trip (make-leg :id 2 :origin "SFO" :destination "LAX"
                                   :depart-ms 130000000 :arrive-ms 135000000
                                   :cost-cents 15000 :carrier "ACME Air"))
    (format t "=== ~A ===~%" (trip-title trip))
    (format t "  total span:    ~,1F min~%" (/ (trip-total-span-ms trip) 60000.0))
    (format t "  in transit:    ~,1F min~%" (/ (trip-in-transit-ms trip) 60000.0))
    (format t "  layovers:      ~,1F min~%" (/ (trip-total-layover-ms trip) 60000.0))
    (format t "  total cost:    $~,2F~%" (/ (trip-total-cost-cents trip) 100.0))
    (format t "~%=== layovers ===~%")
    (dolist (lo (layovers trip))
      (format t "  ~A: ~,1F min~%" (layover-location lo)
              (/ (layover-duration-ms lo) 60000.0)))
    (format t "~%=== validation ===~%")
    (dolist (is (validate-trip trip))
      (format t "  [~A] ~A on leg ~D: ~A~%"
              (issue-severity is) (issue-kind is)
              (issue-leg-idx is) (issue-detail is)))))
