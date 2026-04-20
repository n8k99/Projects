;;;; Budget Tracker — envelope allocation, variance analysis, alert thresholds.

(defpackage #:budget-tracker
  (:use #:cl)
  (:export #:make-envelope #:make-budget #:budget-add-envelope #:budget-record
           #:budget-status-all #:budget-alerts
           #:make-transaction
           #:envelope-category #:envelope-budgeted-cents #:envelope-alert-threshold-pct
           #:status-category #:status-spent-cents #:status-remaining-cents
           #:status-variance-cents #:status-variance-pct
           #:status-over-budget #:status-alert
           #:budget-total-budgeted #:budget-total-spent-between))

(in-package #:budget-tracker)

(defstruct envelope
  (category "" :type string)
  (budgeted-cents 0 :type integer)
  (rollover :reset :type keyword)
  (alert-threshold-pct 80 :type integer))

(defstruct transaction
  (timestamp-ms 0 :type integer)
  (category "" :type string)
  (amount-cents 0 :type integer)
  (note "" :type string))

(defstruct category-status
  (category "" :type string)
  (budgeted-cents 0 :type integer)
  (spent-cents 0 :type integer)
  (remaining-cents 0 :type integer)
  (variance-cents 0 :type integer)
  (variance-pct 0 :type integer)
  (over-budget nil :type boolean)
  (alert nil :type boolean))

(defstruct budget
  (envelopes (make-hash-table :test 'equal) :type hash-table)
  (transactions nil :type list))

(defun budget-add-envelope (b e)
  (setf (gethash (envelope-category e) (budget-envelopes b)) e))

(defun budget-record (b t1)
  (setf (budget-transactions b) (append (budget-transactions b) (list t1))))

(defun budget-total-budgeted (b)
  (let ((sum 0))
    (maphash (lambda (k v) (declare (ignore k))
               (incf sum (envelope-budgeted-cents v)))
             (budget-envelopes b))
    sum))

(defun budget-total-spent-between (b from-ms to-ms)
  (let ((sum 0))
    (dolist (tx (budget-transactions b))
      (when (and (>= (transaction-timestamp-ms tx) from-ms)
                 (< (transaction-timestamp-ms tx) to-ms))
        (incf sum (- (transaction-amount-cents tx)))))
    sum))

(defun status-spent-between (b from-ms to-ms)
  (let ((spent (make-hash-table :test 'equal)))
    (dolist (tx (budget-transactions b))
      (when (and (>= (transaction-timestamp-ms tx) from-ms)
                 (< (transaction-timestamp-ms tx) to-ms))
        (incf (gethash (transaction-category tx) spent 0)
              (- (transaction-amount-cents tx)))))
    spent))

(defun budget-status-between (b from-ms to-ms)
  (let ((spent (status-spent-between b from-ms to-ms))
        (out nil))
    (maphash (lambda (cat env)
               (declare (ignore cat))
               (let* ((s (gethash (envelope-category env) spent 0))
                      (budgeted (envelope-budgeted-cents env))
                      (remaining (- budgeted s))
                      (variance (- s budgeted))
                      (var-pct (if (zerop budgeted) -101
                                   (floor (* variance 100) budgeted)))
                      (over (> s budgeted))
                      (alert (and (> budgeted 0)
                                  (>= (max 0 (floor (* s 100) budgeted))
                                      (envelope-alert-threshold-pct env)))))
                 (push (make-category-status
                        :category (envelope-category env)
                        :budgeted-cents budgeted
                        :spent-cents s
                        :remaining-cents remaining
                        :variance-cents variance
                        :variance-pct var-pct
                        :over-budget over
                        :alert alert)
                       out)))
             (budget-envelopes b))
    ;; Uncategorised synthetics
    (maphash (lambda (cat s)
               (unless (gethash cat (budget-envelopes b))
                 (push (make-category-status
                        :category cat
                        :budgeted-cents 0
                        :spent-cents s
                        :remaining-cents (- s)
                        :variance-cents s
                        :variance-pct -101
                        :over-budget (> s 0)
                        :alert nil)
                       out)))
             spent)
    (sort out (lambda (a b) (string< (category-status-category a)
                                       (category-status-category b))))))

(defun budget-status-all (b)
  (budget-status-between b most-negative-fixnum most-positive-fixnum))

(defun budget-alerts (b)
  (remove-if-not #'category-status-alert (budget-status-all b)))

(defun demo ()
  (let ((b (make-budget)))
    (budget-add-envelope b (make-envelope :category "groceries"
                                            :budgeted-cents 50000))
    (budget-add-envelope b (make-envelope :category "rent"
                                            :budgeted-cents 200000
                                            :alert-threshold-pct 90))
    (budget-add-envelope b (make-envelope :category "entertainment"
                                            :budgeted-cents 10000
                                            :alert-threshold-pct 50))
    (budget-record b (make-transaction :timestamp-ms 100 :category "groceries"
                                        :amount-cents -30000))
    (budget-record b (make-transaction :timestamp-ms 200 :category "groceries"
                                        :amount-cents -15000))
    (budget-record b (make-transaction :timestamp-ms 300 :category "rent"
                                        :amount-cents -200000))
    (budget-record b (make-transaction :timestamp-ms 400 :category "entertainment"
                                        :amount-cents -8000))
    (budget-record b (make-transaction :timestamp-ms 500 :category "vacation"
                                        :amount-cents -5000))

    (format t "=== status ===~%")
    (dolist (s (budget-status-all b))
      (format t "  ~A ~14A spent=$~,2F / budget=$~,2F  variance=~D%~A~%"
              (if (category-status-over-budget s) "OVER" "OK  ")
              (category-status-category s)
              (/ (category-status-spent-cents s) 100.0)
              (/ (category-status-budgeted-cents s) 100.0)
              (category-status-variance-pct s)
              (if (category-status-alert s) " WARN" "")))

    (format t "~%=== alerts ===~%")
    (dolist (s (budget-alerts b))
      (format t "  ~A: ~,1F% spent~%"
              (category-status-category s)
              (* (/ (category-status-spent-cents s)
                    (category-status-budgeted-cents s))
                 100.0)))))
