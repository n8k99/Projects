;;;; Database Translation — dialect-aware DDL emission.

(defpackage #:db-translator
  (:use #:cl)
  (:export #:make-column-type #:make-default-value #:make-db-column #:make-table
           #:render-type #:render-default #:emit-create-table #:emit-schema))

(in-package #:db-translator)

(defstruct column-type
  (kind :integer :type keyword)
  (length 0 :type integer)
  (precision 0 :type integer)
  (scale 0 :type integer))

(defun render-type (ty dialect)
  (case (column-type-kind ty)
    (:smallint
     (case dialect (:mysql "TINYINT") (:postgres "SMALLINT") (:sqlite "INTEGER")))
    (:integer "INTEGER")
    (:bigint (if (eq dialect :sqlite) "INTEGER" "BIGINT"))
    (:text "TEXT")
    (:varchar (format nil "VARCHAR(~D)" (column-type-length ty)))
    (:boolean
     (case dialect (:mysql "TINYINT(1)") (:postgres "BOOLEAN") (:sqlite "INTEGER")))
    (:timestamp
     (case dialect (:mysql "DATETIME") (:postgres "TIMESTAMP") (:sqlite "TEXT")))
    (:date (if (eq dialect :sqlite) "TEXT" "DATE"))
    (:float "REAL")
    (:decimal (format nil "DECIMAL(~D,~D)"
                       (column-type-precision ty) (column-type-scale ty)))))

(defstruct default-value
  (kind :literal :type keyword)
  (literal "" :type string))

(defun render-default (d dialect)
  (case (default-value-kind d)
    (:literal (default-value-literal d))
    (:now (if (eq dialect :mysql) "NOW()" "CURRENT_TIMESTAMP"))))

(defstruct db-column
  (name "" :type string)
  (ty nil :type (or null column-type))
  (nullable t :type boolean)
  (primary-key nil :type boolean)
  (unique nil :type boolean)
  (auto-increment nil :type boolean)
  (default nil :type (or null default-value)))

(defstruct table
  (name "" :type string)
  (columns nil :type list))

(defun auto-increment-syntax (dialect)
  (case dialect
    (:mysql "AUTO_INCREMENT")
    (:postgres "GENERATED ALWAYS AS IDENTITY")
    (:sqlite "AUTOINCREMENT")))

(defun render-column (col dialect)
  (let ((parts (list (db-column-name col)
                      (render-type (db-column-ty col) dialect))))
    (when (db-column-primary-key col)
      (setf parts (append parts (list "PRIMARY KEY"))))
    (when (and (not (db-column-nullable col))
                (not (db-column-primary-key col)))
      (setf parts (append parts (list "NOT NULL"))))
    (when (and (db-column-unique col) (not (db-column-primary-key col)))
      (setf parts (append parts (list "UNIQUE"))))
    (when (db-column-auto-increment col)
      (cond
        ((and (eq dialect :sqlite) (db-column-primary-key col))
         (setf parts (append parts (list "AUTOINCREMENT"))))
        ((not (eq dialect :sqlite))
         (setf parts (append parts (list (auto-increment-syntax dialect)))))))
    (when (db-column-default col)
      (setf parts (append parts
                           (list (format nil "DEFAULT ~A"
                                           (render-default (db-column-default col)
                                                             dialect))))))
    (format nil "~{~A~^ ~}" parts)))

(defun emit-create-table (tbl dialect)
  (with-output-to-string (out)
    (format out "CREATE TABLE ~A (~%" (table-name tbl))
    (let ((cols (mapcar (lambda (c)
                          (format nil "  ~A" (render-column c dialect)))
                         (table-columns tbl))))
      (format out "~{~A~^,~%~}~%" cols))
    (format out ");~%")))

(defun emit-schema (tables dialect)
  (format nil "~{~A~}" (mapcar (lambda (t1) (emit-create-table t1 dialect)) tables)))

(defun demo ()
  (let ((users (make-table
                :name "users"
                :columns
                (list
                 (make-db-column :name "id"
                                   :ty (make-column-type :kind :integer)
                                   :primary-key t :nullable nil :auto-increment t)
                 (make-db-column :name "email"
                                   :ty (make-column-type :kind :varchar :length 255)
                                   :nullable nil :unique t)
                 (make-db-column :name "name"
                                   :ty (make-column-type :kind :text)
                                   :nullable nil)
                 (make-db-column :name "active"
                                   :ty (make-column-type :kind :boolean)
                                   :nullable nil
                                   :default (make-default-value :kind :literal :literal "1"))
                 (make-db-column :name "created_at"
                                   :ty (make-column-type :kind :timestamp)
                                   :nullable nil
                                   :default (make-default-value :kind :now))))))
    (dolist (dialect '(:mysql :postgres :sqlite))
      (format t "=== ~A ===~%" dialect)
      (format t "~A~%" (emit-create-table users dialect)))))
