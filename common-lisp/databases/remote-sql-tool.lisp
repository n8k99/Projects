;;;; Remote SQL Tool — in-memory table engine + execution over parsed queries.

(defpackage #:remote-sql-tool
  (:use #:cl)
  (:export #:make-connection #:create-table #:rst-insert #:select-query
           #:begin-txn #:commit-txn #:rollback-txn
           #:make-column #:make-cell-int #:make-cell-text #:make-cell-null
           #:make-select-query #:make-where-expr #:make-order-by-expr
           #:result-set-columns #:result-set-rows
           #:cell-is-int #:cell-int-val #:cell-text-val #:cell-is-null))

(in-package #:remote-sql-tool)

(defstruct cell
  (is-null nil :type boolean)
  (is-int nil :type boolean)
  (int-val 0 :type integer)
  (text-val "" :type string))

(defun make-cell-int (n) (make-cell :is-int t :int-val n))
(defun make-cell-text (s) (make-cell :text-val s))
(defun make-cell-null () (make-cell :is-null t))

(defstruct column
  (name "" :type string)
  (ty :int :type keyword))

(defstruct rst-table
  (name "" :type string)
  (columns nil :type list)
  (rows nil :type list))

(defstruct connection
  (tables (make-hash-table :test 'equal) :type hash-table)
  (pending nil :type (or null hash-table)))

(defstruct where-expr
  (column "" :type string)
  (op "" :type string)
  (value nil :type (or null cell)))

(defstruct order-by-expr
  (column "" :type string)
  (desc nil :type boolean))

(defstruct select-query
  (project-all nil :type boolean)
  (columns nil :type list)
  (table "" :type string)
  (where nil :type (or null where-expr))
  (order-by nil :type (or null order-by-expr))
  (limit nil :type (or null integer)))

(defstruct result-set
  (columns nil :type list)
  (rows nil :type list))

(defun cell-type-of (c)
  (cond ((cell-is-null c) nil)
        ((cell-is-int c) :int)
        (t :text)))

(defun column-index (table col-name)
  (position col-name (rst-table-columns table)
            :key #'column-name :test #'string=))

(defun create-table (conn name cols)
  (when (gethash name (connection-tables conn))
    (error "table exists: ~A" name))
  (setf (gethash name (connection-tables conn))
        (make-rst-table :name name :columns cols)))

(defun rst-insert (conn table values)
  (let ((t1 (gethash table (connection-tables conn))))
    (unless t1 (error "unknown table: ~A" table))
    (unless (= (length values) (length (rst-table-columns t1)))
      (error "expected ~D values, got ~D"
             (length (rst-table-columns t1)) (length values)))
    (loop for col in (rst-table-columns t1)
          for val in values
          for vt = (cell-type-of val)
          when (and vt (not (eq vt (column-ty col))))
          do (error "type mismatch on column ~A" (column-name col)))
    (cond
      ((connection-pending conn)
       (let ((existing (gethash table (connection-pending conn))))
         (setf (gethash table (connection-pending conn))
               (append (or existing nil) (list values)))))
      (t (setf (rst-table-rows t1)
               (append (rst-table-rows t1) (list values)))))))

(defun begin-txn (conn)
  (setf (connection-pending conn) (make-hash-table :test 'equal)))

(defun commit-txn (conn)
  (unless (connection-pending conn) (error "no transaction"))
  (maphash (lambda (name rows)
             (let ((t1 (gethash name (connection-tables conn))))
               (when t1
                 (setf (rst-table-rows t1) (append (rst-table-rows t1) rows)))))
           (connection-pending conn))
  (setf (connection-pending conn) nil))

(defun rollback-txn (conn)
  (unless (connection-pending conn) (error "no transaction"))
  (setf (connection-pending conn) nil))

(defun rows-of (conn table-name)
  (let* ((t1 (gethash table-name (connection-tables conn)))
         (base (if t1 (copy-list (rst-table-rows t1)) nil)))
    (when (connection-pending conn)
      (let ((p (gethash table-name (connection-pending conn))))
        (when p (setf base (append base (copy-list p))))))
    base))

(defun compare-cells (a b)
  (cond
    ((and (cell-is-null a) (cell-is-null b)) 0)
    ((cell-is-null a) -1)
    ((cell-is-null b) 1)
    ((and (cell-is-int a) (cell-is-int b))
     (cond ((< (cell-int-val a) (cell-int-val b)) -1)
           ((> (cell-int-val a) (cell-int-val b)) 1)
           (t 0)))
    ((and (not (cell-is-int a)) (not (cell-is-int b)))
     (cond ((string< (cell-text-val a) (cell-text-val b)) -1)
           ((string> (cell-text-val a) (cell-text-val b)) 1)
           (t 0)))
    (t 0)))

(defun eval-pred (cell op rhs)
  (let ((c (compare-cells cell rhs)))
    (cond ((string= op "=") (= c 0))
          ((string= op "!=") (/= c 0))
          ((string= op "<") (< c 0))
          ((string= op "<=") (<= c 0))
          ((string= op ">") (> c 0))
          ((string= op ">=") (>= c 0))
          (t nil))))

(defun select-query (conn q)
  (let ((t1 (gethash (select-query-table q) (connection-tables conn))))
    (unless t1 (error "unknown table: ~A" (select-query-table q)))
    (let* ((proj-indices
             (if (select-query-project-all q)
                 (loop for i from 0 below (length (rst-table-columns t1)) collect i)
                 (loop for c in (select-query-columns q)
                       for i = (column-index t1 c)
                       unless i do (error "unknown column: ~A" c)
                       collect i)))
           (where-idx
             (when (select-query-where q)
               (let ((i (column-index t1 (where-expr-column (select-query-where q)))))
                 (unless i (error "unknown column: ~A"
                                  (where-expr-column (select-query-where q))))
                 i)))
           (order-info
             (when (select-query-order-by q)
               (let ((i (column-index t1 (order-by-expr-column (select-query-order-by q)))))
                 (unless i (error "unknown column: ~A"
                                  (order-by-expr-column (select-query-order-by q))))
                 (cons i (order-by-expr-desc (select-query-order-by q))))))
           (base-rows (rows-of conn (select-query-table q)))
           (filtered
             (remove-if-not
               (lambda (r)
                 (or (not (select-query-where q))
                     (eval-pred (nth where-idx r)
                                 (where-expr-op (select-query-where q))
                                 (where-expr-value (select-query-where q)))))
               base-rows)))
      (when order-info
        (let ((i (car order-info)) (desc (cdr order-info)))
          (setf filtered
                (stable-sort filtered
                             (lambda (a b)
                               (let ((c (compare-cells (nth i a) (nth i b))))
                                 (if desc (> c 0) (< c 0))))))))
      (let ((projected (mapcar (lambda (r)
                                 (mapcar (lambda (i) (nth i r)) proj-indices))
                               filtered)))
        (when (select-query-limit q)
          (setf projected (subseq projected 0 (min (length projected)
                                                    (select-query-limit q)))))
        (make-result-set
         :columns (mapcar (lambda (i) (column-name (nth i (rst-table-columns t1))))
                          proj-indices)
         :rows projected)))))

(defun cell-display (c)
  (cond ((cell-is-null c) "NULL")
        ((cell-is-int c) (write-to-string (cell-int-val c)))
        (t (cell-text-val c))))

(defun demo ()
  (let ((conn (make-connection)))
    (create-table conn "users"
                  (list (make-column :name "id" :ty :int)
                        (make-column :name "name" :ty :text)
                        (make-column :name "age" :ty :int)))
    (rst-insert conn "users"
                 (list (make-cell-int 1) (make-cell-text "Alice") (make-cell-int 30)))
    (rst-insert conn "users"
                 (list (make-cell-int 2) (make-cell-text "Bob") (make-cell-int 25)))
    (rst-insert conn "users"
                 (list (make-cell-int 3) (make-cell-text "Carol") (make-cell-int 35)))
    (format t "=== SELECT name FROM users WHERE age > 26 ORDER BY age DESC LIMIT 2 ===~%")
    (let ((r (select-query conn
                           (make-select-query
                            :project-all nil
                            :columns '("name")
                            :table "users"
                            :where (make-where-expr :column "age" :op ">"
                                                     :value (make-cell-int 26))
                            :order-by (make-order-by-expr :column "age" :desc t)
                            :limit 2))))
      (format t "  columns: ~A~%" (result-set-columns r))
      (dolist (row (result-set-rows r))
        (format t "  ~A~%" (mapcar #'cell-display row))))
    (format t "~%=== transaction: BEGIN; INSERT Dave; ROLLBACK ===~%")
    (begin-txn conn)
    (rst-insert conn "users"
                 (list (make-cell-int 4) (make-cell-text "Dave") (make-cell-int 40)))
    (format t "  before rollback: ~D rows~%" (length (rows-of conn "users")))
    (rollback-txn conn)
    (format t "  after rollback: ~D rows~%" (length (rows-of conn "users")))))
