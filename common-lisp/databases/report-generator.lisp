;;;; Report Generator — grouped reports with subtotal and grand-total rows.

(defpackage #:report-generator
  (:use #:cl)
  (:export #:make-column-def #:make-report-spec #:make-record
           #:generate-report #:render-report
           #:make-value-int #:make-value-text #:make-value-null
           #:report-row-kind #:report-row-cells #:report-row-group-label
           #:report-title #:report-rows))

(in-package #:report-generator)

(defstruct rpt-value
  (kind :text :type keyword)
  (int-val 0 :type integer)
  (text-val "" :type string))

(defun make-value-int (n) (make-rpt-value :kind :int :int-val n))
(defun make-value-text (s) (make-rpt-value :kind :text :text-val s))
(defun make-value-null () (make-rpt-value :kind :null))

(defun value-as-int (v)
  (when (eq (rpt-value-kind v) :int) (rpt-value-int-val v)))

(defun value-as-text (v)
  (case (rpt-value-kind v)
    (:text (rpt-value-text-val v))
    (:int (write-to-string (rpt-value-int-val v)))
    (t nil)))

(defstruct column-def
  (name "" :type string)
  (field "" :type string)
  (kind :plain :type keyword)        ; :plain | :sum-aggregate
  (format-cents nil :type boolean))

(defstruct report-spec
  (title "" :type string)
  (columns nil :type list)
  (group-by nil :type (or null string)))

(defstruct report-row
  (kind :data :type keyword)          ; :header | :data | :subtotal | :total
  (cells nil :type list)
  (group-label nil :type (or null string)))

(defstruct report
  (title "" :type string)
  (rows nil :type list))

(defun make-record (&rest pairs)
  (let ((h (make-hash-table :test 'equal)))
    (loop for (k v) on pairs by #'cddr
          do (setf (gethash k h) v))
    h))

(defun record-get (record field)
  (gethash field record))

(defun format-int (n as-cents)
  (if (not as-cents)
      (write-to-string n)
      (let* ((abs-n (abs n))
             (dollars (floor abs-n 100))
             (cents (mod abs-n 100))
             (sign (if (< n 0) "-" "")))
        (format nil "$~A~D.~2,'0D" sign dollars cents))))

(defun format-cell (value col)
  (cond ((or (null value) (eq (rpt-value-kind value) :null)) "")
        ((eq (rpt-value-kind value) :int)
         (format-int (rpt-value-int-val value) (column-def-format-cents col)))
        (t (rpt-value-text-val value))))

(defun generate-report (records spec)
  (let ((rows (list (make-report-row
                     :kind :header
                     :cells (mapcar #'column-def-name (report-spec-columns spec))))))
    (let* ((groups
             (if (report-spec-group-by spec)
                 (let ((m (make-hash-table :test 'equal)))
                   (dolist (r records)
                     (let* ((v (record-get r (report-spec-group-by spec)))
                            (key (or (and v (value-as-text v)) "(none)")))
                       (setf (gethash key m) (append (gethash key m) (list r)))))
                   (sort (loop for k being the hash-keys of m
                               collect (cons k (gethash k m)))
                         (lambda (a b) (string< (car a) (car b)))))
                 (list (cons "__all__" records))))
           (grand-totals (make-hash-table :test 'equal)))

      (dolist (pair groups)
        (let ((group-key (car pair))
              (group-records (cdr pair)))
          ;; Data rows
          (dolist (record group-records)
            (let ((cells (mapcar (lambda (c)
                                    (format-cell (record-get record (column-def-field c)) c))
                                  (report-spec-columns spec))))
              (push (make-report-row :kind :data :cells cells
                                      :group-label (when (report-spec-group-by spec) group-key))
                    rows)))
          ;; Subtotal row
          (when (report-spec-group-by spec)
            (let ((subtotals (make-hash-table :test 'equal)))
              (dolist (record group-records)
                (dolist (c (report-spec-columns spec))
                  (when (eq (column-def-kind c) :sum-aggregate)
                    (let* ((v (record-get record (column-def-field c)))
                           (n (and v (value-as-int v))))
                      (when n
                        (setf (gethash (column-def-field c) subtotals)
                              (+ (gethash (column-def-field c) subtotals 0) n))
                        (setf (gethash (column-def-field c) grand-totals)
                              (+ (gethash (column-def-field c) grand-totals 0) n)))))))
              (let ((cells nil)
                    (idx 0))
                (dolist (c (report-spec-columns spec))
                  (cond
                    ((= idx 0) (push (format nil "~A subtotal" group-key) cells))
                    ((eq (column-def-kind c) :sum-aggregate)
                     (push (format-int (gethash (column-def-field c) subtotals 0)
                                        (column-def-format-cents c)) cells))
                    (t (push "" cells)))
                  (incf idx))
                (push (make-report-row :kind :subtotal
                                        :cells (nreverse cells)
                                        :group-label group-key) rows))))))

      (when (some (lambda (c) (eq (column-def-kind c) :sum-aggregate))
                   (report-spec-columns spec))
        (unless (report-spec-group-by spec)
          (dolist (record records)
            (dolist (c (report-spec-columns spec))
              (when (eq (column-def-kind c) :sum-aggregate)
                (let* ((v (record-get record (column-def-field c)))
                       (n (and v (value-as-int v))))
                  (when n
                    (setf (gethash (column-def-field c) grand-totals)
                          (+ (gethash (column-def-field c) grand-totals 0) n))))))))
        (let ((cells nil) (idx 0))
          (dolist (c (report-spec-columns spec))
            (cond
              ((= idx 0) (push "TOTAL" cells))
              ((eq (column-def-kind c) :sum-aggregate)
               (push (format-int (gethash (column-def-field c) grand-totals 0)
                                  (column-def-format-cents c)) cells))
              (t (push "" cells)))
            (incf idx))
          (push (make-report-row :kind :total :cells (nreverse cells)) rows))))
    (make-report :title (report-spec-title spec) :rows (nreverse rows))))

(defun render-report (report)
  (with-output-to-string (out)
    (when (> (length (report-title report)) 0)
      (format out "~A~%" (report-title report)))
    (dolist (row (report-rows report))
      (let ((prefix (case (report-row-kind row)
                       (:header "#") (:data " ")
                       (:subtotal "~") (:total "="))))
        (format out "~A	~{~A~^	~}~%" prefix (report-row-cells row))))))

(defun demo ()
  (let ((records nil)
        (spec (make-report-spec
               :title "Sales Report"
               :columns (list (make-column-def :name "Region" :field "region" :kind :plain)
                              (make-column-def :name "Product" :field "product" :kind :plain)
                              (make-column-def :name "Qty" :field "qty" :kind :sum-aggregate)
                              (make-column-def :name "Revenue" :field "revenue"
                                                :kind :sum-aggregate :format-cents t))
               :group-by "region")))
    (dolist (row '(("East" "Widget" 3 1500) ("East" "Gadget" 2 2000)
                   ("West" "Widget" 5 2500) ("West" "Gizmo" 1 750)))
      (push (make-record "region" (make-value-text (first row))
                          "product" (make-value-text (second row))
                          "qty" (make-value-int (third row))
                          "revenue" (make-value-int (fourth row)))
            records))
    (setf records (nreverse records))
    (format t "~A" (render-report (generate-report records spec)))))
