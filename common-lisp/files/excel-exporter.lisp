;;;; Excel Spreadsheet Exporter — cell-addressed grid, formulae, CSV export.

(defpackage #:excel-exporter
  (:use #:cl)
  (:export #:make-sheet #:sheet-set-number #:sheet-set-text #:sheet-set-formula
           #:sheet-evaluate #:sheet-to-csv #:parse-address #:format-address))

(in-package #:excel-exporter)

(defun parse-address (addr)
  (when (and (stringp addr) (> (length addr) 1))
    (let ((first (char addr 0)))
      (when (alpha-char-p first)
        (let ((col (- (char-code (char-upcase first)) (char-code #\A)))
              (row-str (subseq addr 1)))
          (multiple-value-bind (row pos) (parse-integer row-str :junk-allowed t)
            (when (and row (= pos (length row-str)) (> row 0))
              (cons (1- row) col))))))))

(defun format-address (row col)
  (format nil "~A~D" (code-char (+ (char-code #\A) col)) (1+ row)))

(defstruct cell
  (kind :number :type keyword)          ; :number :text :formula :blank
  (number 0.0)
  (text "")
  (formula ""))

(defstruct sheet
  (cells (make-hash-table :test 'equal)))

(defstruct (value (:conc-name v-))
  (kind :empty :type keyword)
  (number 0.0)
  (text ""))

(defun v-empty () (make-value :kind :empty))
(defun v-num (n) (make-value :kind :number :number n))
(defun v-txt (s) (make-value :kind :text :text s))
(defun v-err (s) (make-value :kind :error :text s))

(defun v-as-number (v)
  (case (v-kind v)
    (:number (v-number v))
    (:text (ignore-errors (read-from-string (v-text v))))
    (t nil)))

(defun sheet-set-number (sheet addr n)
  (let ((pos (parse-address addr)))
    (unless pos (error "bad address: ~A" addr))
    (setf (gethash pos (sheet-cells sheet)) (make-cell :kind :number :number n))))

(defun sheet-set-text (sheet addr s)
  (let ((pos (parse-address addr)))
    (unless pos (error "bad address: ~A" addr))
    (setf (gethash pos (sheet-cells sheet)) (make-cell :kind :text :text s))))

(defun sheet-set-formula (sheet addr f)
  (let ((pos (parse-address addr)))
    (unless pos (error "bad address: ~A" addr))
    (setf (gethash pos (sheet-cells sheet)) (make-cell :kind :formula :formula f))))

(defun eval-cell (sheet pos visiting)
  (if (find pos visiting :test #'equal)
      (v-err "circular reference")
      (let ((c (gethash pos (sheet-cells sheet)))
            (new-visiting (cons pos visiting)))
        (cond
          ((null c) (v-empty))
          ((eq (cell-kind c) :blank) (v-empty))
          ((eq (cell-kind c) :number) (v-num (cell-number c)))
          ((eq (cell-kind c) :text) (v-txt (cell-text c)))
          ((eq (cell-kind c) :formula) (eval-formula sheet (cell-formula c) new-visiting))))))

(defun eval-formula (sheet f visiting)
  (let ((body (string-trim " " (if (and (> (length f) 0) (char= (char f 0) #\=))
                                    (subseq f 1) f))))
    ;; Function call?
    (let ((paren (position #\( body)))
      (when (and paren (char= (char body (1- (length body))) #\)))
        (return-from eval-formula
          (eval-function sheet (subseq body 0 paren)
                          (subseq body (1+ paren) (1- (length body)))
                          visiting))))
    ;; Binary op?
    (dolist (op '(#\+ #\- #\* #\/))
      (let ((pos (position op body :start 1)))
        (when pos
          (return-from eval-formula
            (apply-op op
                      (eval-operand sheet
                                     (string-trim " " (subseq body 0 pos))
                                     visiting)
                      (eval-operand sheet
                                     (string-trim " " (subseq body (1+ pos)))
                                     visiting))))))
    ;; Operand only
    (eval-operand sheet body visiting)))

(defun eval-operand (sheet s visiting)
  (let ((trimmed (string-trim " " s)))
    (let ((n (ignore-errors (read-from-string trimmed))))
      (if (numberp n)
          (v-num (float n))
          (let ((pos (parse-address trimmed)))
            (if pos
                (eval-cell sheet pos visiting)
                (v-err (format nil "bad operand: ~A" trimmed))))))))

(defun expand-range (sheet args visiting)
  (let ((out nil))
    (dolist (part (loop for start = 0 then (1+ end)
                        for end = (position #\, args :start start)
                        collect (subseq args start end)
                        while end))
      (let* ((trimmed (string-trim " " part))
             (colon (position #\: trimmed)))
        (if colon
            (let ((start-pos (parse-address (subseq trimmed 0 colon)))
                  (end-pos (parse-address (subseq trimmed (1+ colon)))))
              (when (and start-pos end-pos)
                (loop for r from (car start-pos) to (car end-pos) do
                      (loop for c from (cdr start-pos) to (cdr end-pos) do
                            (push (eval-cell sheet (cons r c) visiting) out)))))
            (push (eval-operand sheet trimmed visiting) out))))
    (nreverse out)))

(defun eval-function (sheet name args visiting)
  (let* ((values (expand-range sheet args visiting))
         (nums (loop for v in values
                     for n = (v-as-number v)
                     when n collect n))
         (upper (string-upcase name)))
    (cond
      ((string= upper "SUM") (v-num (reduce #'+ nums :initial-value 0.0)))
      ((or (string= upper "AVG") (string= upper "AVERAGE"))
       (if (null nums)
           (v-err "empty range for AVG")
           (v-num (/ (reduce #'+ nums) (length nums)))))
      ((string= upper "MIN")
       (if (null nums) (v-err "empty range") (v-num (apply #'min nums))))
      ((string= upper "MAX")
       (if (null nums) (v-err "empty range") (v-num (apply #'max nums))))
      ((string= upper "COUNT") (v-num (length nums)))
      (t (v-err (format nil "unknown function: ~A" name))))))

(defun apply-op (op l r)
  (let ((a (v-as-number l)) (b (v-as-number r)))
    (if (and a b)
        (case op
          (#\+ (v-num (+ a b)))
          (#\- (v-num (- a b)))
          (#\* (v-num (* a b)))
          (#\/ (if (zerop b) (v-err "div by zero") (v-num (/ a b))))
          (t (v-err "bad op")))
        (v-err "non-numeric operand"))))

(defun sheet-evaluate (sheet addr)
  (let ((pos (parse-address addr)))
    (if pos
        (eval-cell sheet pos nil)
        (v-err (format nil "bad address: ~A" addr)))))

(defun format-value (v)
  (case (v-kind v)
    (:empty "")
    (:number
     (let ((n (v-number v)))
       (if (and (= n (floor n)) (< (abs n) 1.0e15))
           (write-to-string (floor n))
           (write-to-string n))))
    (:text (v-text v))
    (:error (format nil "#ERR: ~A" (v-text v)))))

(defun csv-escape (s)
  (if (or (position #\, s) (position #\" s) (position #\Newline s))
      (format nil "\"~A\""
              (with-output-to-string (out)
                (loop for c across s do
                      (when (char= c #\") (write-char #\" out))
                      (write-char c out))))
      s))

(defun sheet-to-csv (sheet)
  (if (zerop (hash-table-count (sheet-cells sheet)))
      ""
      (let ((max-row 0) (max-col 0))
        (maphash (lambda (k v) (declare (ignore v))
                   (when (> (car k) max-row) (setf max-row (car k)))
                   (when (> (cdr k) max-col) (setf max-col (cdr k))))
                 (sheet-cells sheet))
        (with-output-to-string (out)
          (loop for row from 0 to max-row do
                (loop for col from 0 to max-col do
                      (when (> col 0) (write-char #\, out))
                      (write-string (csv-escape
                                     (format-value
                                      (sheet-evaluate sheet (format-address row col))))
                                    out))
                (write-char #\Newline out))))))

(defun demo ()
  (let ((s (make-sheet)))
    (sheet-set-text s "A1" "name")
    (sheet-set-text s "B1" "age")
    (sheet-set-text s "C1" "squared")
    (sheet-set-text s "A2" "Alice")
    (sheet-set-number s "B2" 30)
    (sheet-set-formula s "C2" "=B2 * B2")
    (sheet-set-text s "A3" "Bob")
    (sheet-set-number s "B3" 25)
    (sheet-set-formula s "C3" "=B3 * B3")
    (sheet-set-text s "A4" "Total")
    (sheet-set-formula s "B4" "=SUM(B2:B3)")
    (sheet-set-formula s "C4" "=SUM(C2:C3)")
    (format t "=== sheet CSV ===~%~A" (sheet-to-csv s))
    (format t "B4 = ~A~%" (v-number (sheet-evaluate s "B4")))
    (format t "C4 = ~A~%" (v-number (sheet-evaluate s "C4")))))
