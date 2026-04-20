;;;; Sort File Records Utility — parse delimited records, multi-key sort.

(defpackage #:sort-file-records
  (:use #:cl)
  (:export #:parse-record #:serialise-record #:record-fields
           #:parse-table #:serialise-table #:sort-table
           #:make-sort-key #:table-header #:table-rows #:field-index))

(in-package #:sort-file-records)

(defstruct record (fields nil :type list))

(defstruct table
  (header nil :type (or null list))
  (rows nil :type list))

(defstruct sort-key
  (field 0 :type integer)
  (order :asc :type keyword)
  (kind :str :type keyword))

(defun field-from-str (s)
  (let ((trimmed (string-trim " " s)))
    (multiple-value-bind (n pos) (parse-integer trimmed :junk-allowed t)
      (if (and n (= pos (length trimmed))) n trimmed))))

(defun field-as-str (f)
  (if (integerp f) (write-to-string f) f))

(defun split-by (s ch)
  (loop for start = 0 then (1+ end)
        for end = (position ch s :start start)
        collect (subseq s start end)
        while end))

(defun parse-record (line)
  (make-record :fields (mapcar #'field-from-str (split-by line #\,))))

(defun serialise-record (r)
  (format nil "~{~A~^,~}" (mapcar #'field-as-str (record-fields r))))

(defun parse-table (text has-header)
  (let ((lines (remove-if (lambda (l) (zerop (length l)))
                          (split-by text #\Newline))))
    (if has-header
        (make-table
         :header (mapcar (lambda (s) (string-trim " " s))
                         (split-by (first lines) #\,))
         :rows (mapcar #'parse-record (rest lines)))
        (make-table :header nil :rows (mapcar #'parse-record lines)))))

(defun serialise-table (tbl)
  (with-output-to-string (s)
    (when (table-header tbl)
      (format s "~{~A~^,~}~%" (table-header tbl)))
    (dolist (r (table-rows tbl))
      (format s "~A~%" (serialise-record r)))))

(defun field-at (r i)
  (let ((fields (record-fields r)))
    (when (< i (length fields)) (nth i fields))))

(defun field-index (tbl name)
  (position name (table-header tbl) :test #'string-equal))

(defun compare-fields (a b kind)
  (cond
    ((and (null a) (null b)) 0)
    ((null a) -1)  ; missing sorts first
    ((null b) 1)
    ((eq kind :int)
     (let ((ai (if (integerp a) a (or (parse-integer a :junk-allowed t) 0)))
           (bi (if (integerp b) b (or (parse-integer b :junk-allowed t) 0))))
       (cond ((< ai bi) -1) ((> ai bi) 1) (t 0))))
    (t
     (let ((as (if (integerp a) (write-to-string a) a))
           (bs (if (integerp b) (write-to-string b) b)))
       (cond ((string< as bs) -1) ((string> as bs) 1) (t 0))))))

(defun compare-records (a b spec)
  (loop for key in spec
        for cmp = (compare-fields (field-at a (sort-key-field key))
                                   (field-at b (sort-key-field key))
                                   (sort-key-kind key))
        for adjusted = (if (eq (sort-key-order key) :desc) (- cmp) cmp)
        when (/= adjusted 0) return adjusted
        finally (return 0)))

(defun sort-table (tbl spec)
  ;; SBCL's sort is not required to be stable; use stable-sort.
  (setf (table-rows tbl)
        (stable-sort (copy-list (table-rows tbl))
                     (lambda (a b) (< (compare-records a b spec) 0))))
  tbl)

(defun demo ()
  (let* ((sample (format nil "name,age,score~%Alice,30,85~%Bob,25,92~%Carol,30,78~%Dave,25,88~%"))
         (tbl (parse-table sample t)))
    (format t "header: ~A~%" (table-header tbl))
    (format t "rows before: ~A~%"
            (mapcar #'serialise-record (table-rows tbl)))
    (format t "~%=== sort by age asc, score desc ===~%")
    (sort-table tbl (list (make-sort-key :field 1 :order :asc :kind :int)
                          (make-sort-key :field 2 :order :desc :kind :int)))
    (dolist (r (table-rows tbl))
      (format t "  ~A~%" (serialise-record r)))
    (format t "~%=== round trip unchanged: ~A ===~%"
            (string= (serialise-table (parse-table sample t)) sample))))
