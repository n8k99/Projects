;;;; Transaction Averages — grouped aggregation over financial records.

(defpackage #:transaction-averages
  (:use #:cl)
  (:export #:parse-transaction #:parse-transactions #:group-by
           #:by-category #:by-month #:total-stats #:top-n-by-mean
           #:make-transaction #:transaction-date #:transaction-category
           #:transaction-amount-cents
           #:stats-count #:stats-sum-cents #:stats-mean-cents
           #:stats-min-cents #:stats-max-cents))

(in-package #:transaction-averages)

(defstruct transaction
  (date "" :type string)
  (category "" :type string)
  (amount-cents 0 :type integer))

(defstruct stats
  (count 0 :type integer)
  (sum-cents 0 :type integer)
  (mean-cents 0.0 :type single-float)
  (min-cents most-positive-fixnum :type integer)
  (max-cents most-negative-fixnum :type integer))

(defun stats-observe (s amount)
  (incf (stats-count s))
  (incf (stats-sum-cents s) amount)
  (when (< amount (stats-min-cents s)) (setf (stats-min-cents s) amount))
  (when (> amount (stats-max-cents s)) (setf (stats-max-cents s) amount))
  s)

(defun stats-finalise (s)
  (setf (stats-mean-cents s)
        (coerce (if (zerop (stats-count s)) 0.0
                    (/ (stats-sum-cents s) (stats-count s)))
                'single-float))
  s)

(defun split-by (s ch)
  (loop for start = 0 then (1+ end)
        for end = (position ch s :start start)
        collect (subseq s start end)
        while end))

(defun parse-transaction (line)
  (let ((parts (split-by line #\,)))
    (when (>= (length parts) 3)
      (let* ((date (string-trim " " (first parts)))
             (category (string-trim " " (second parts)))
             (rest-of-amount (format nil "~{~A~^,~}" (cddr parts)))
             (amount-str (string-trim " " rest-of-amount))
             (amount (ignore-errors (parse-integer amount-str))))
        (when (and amount (> (length date) 0) (> (length category) 0))
          (make-transaction :date date :category category :amount-cents amount))))))

(defun parse-transactions (text)
  (let ((out nil))
    (dolist (line (split-by text #\Newline))
      (unless (zerop (length line))
        (let ((tx (parse-transaction line)))
          (when tx (push tx out)))))
    (nreverse out)))

(defun transaction-month (tx)
  (let ((d (transaction-date tx)))
    (if (>= (length d) 7) (subseq d 0 7) d)))

(defun group-by (txns key-fn)
  (let ((groups (make-hash-table :test 'equal)))
    (dolist (tx txns)
      (let* ((k (funcall key-fn tx))
             (existing (gethash k groups)))
        (unless existing
          (setf existing (make-stats))
          (setf (gethash k groups) existing))
        (stats-observe existing (transaction-amount-cents tx))))
    (let ((keys (sort (loop for k being the hash-keys of groups collect k)
                      #'string<)))
      (loop for k in keys
            for s = (gethash k groups)
            do (stats-finalise s)
            collect (cons k s)))))

(defun by-category (txns)
  (group-by txns #'transaction-category))

(defun by-month (txns)
  (group-by txns #'transaction-month))

(defun total-stats (txns)
  (let ((s (make-stats)))
    (dolist (tx txns) (stats-observe s (transaction-amount-cents tx)))
    (stats-finalise s)
    s))

(defun top-n-by-mean (groups n)
  (let ((sorted (sort (copy-list groups)
                      (lambda (a b) (> (stats-mean-cents (cdr a))
                                        (stats-mean-cents (cdr b)))))))
    (subseq sorted 0 (min n (length sorted)))))

(defun demo ()
  (let* ((sample (format nil "2026-01-03,groceries,4500~%~
                              2026-01-10,groceries,3200~%~
                              2026-01-12,rent,120000~%~
                              2026-02-01,rent,120000~%~
                              2026-02-14,groceries,5500~%~
                              2026-02-22,books,2200~%"))
         (txns (parse-transactions sample)))
    (format t "=== ~D transactions ===~%~%by category:~%" (length txns))
    (dolist (kv (by-category txns))
      (let ((k (car kv)) (s (cdr kv)))
        (format t "  ~A count=~D sum=~D mean=~,2F min=~D max=~D~%"
                k (stats-count s) (stats-sum-cents s)
                (stats-mean-cents s) (stats-min-cents s) (stats-max-cents s))))
    (format t "~%by month:~%")
    (dolist (kv (by-month txns))
      (let ((k (car kv)) (s (cdr kv)))
        (format t "  ~A count=~D sum=~D mean=~,2F~%"
                k (stats-count s) (stats-sum-cents s) (stats-mean-cents s))))
    (format t "~%top 2 by mean:~%")
    (dolist (kv (top-n-by-mean (by-category txns) 2))
      (format t "  ~A mean=~,2F~%" (car kv) (stats-mean-cents (cdr kv))))))
