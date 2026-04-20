;;;; SQL Query Analyzer — recursive-descent parser + static analysis.

(defpackage #:sql-query-analyzer
  (:use #:cl)
  (:export #:tokenise #:parse-sql #:analyse
           #:make-schema #:schema-add-table
           #:query-projection-all #:query-projection-columns
           #:query-table #:query-where #:query-order-by #:query-limit
           #:finding-severity #:finding-code #:finding-message
           #:analysis-tables #:analysis-columns #:analysis-findings))

(in-package #:sql-query-analyzer)

(defstruct token
  (kind :word :type keyword)
  (word "" :type string)
  (number 0 :type integer)
  (string "" :type string)
  (op "" :type string))

(defun char-alpha-or-underscore-p (c)
  (or (alpha-char-p c) (char= c #\_)))

(defun char-alnum-or-underscore-p (c)
  (or (alphanumericp c) (char= c #\_)))

(defun tokenise (input)
  (let ((tokens nil)
        (i 0)
        (n (length input)))
    (loop while (< i n) do
          (let ((c (char input i)))
            (cond
              ((or (char= c #\Space) (char= c #\Tab) (char= c #\Newline))
               (incf i))
              ((char= c #\,) (push (make-token :kind :comma) tokens) (incf i))
              ((char= c #\;) (push (make-token :kind :semicolon) tokens) (incf i))
              ((char= c #\*) (push (make-token :kind :star) tokens) (incf i))
              ((char= c #\')
               (incf i)
               (let ((start i))
                 (loop while (and (< i n) (not (char= (char input i) #\'))) do (incf i))
                 (when (>= i n) (error "unterminated string"))
                 (push (make-token :kind :string :string (subseq input start i)) tokens)
                 (incf i)))
              ((char= c #\=)
               (push (make-token :kind :op :op "=") tokens) (incf i))
              ((or (char= c #\<) (char= c #\>) (char= c #\!))
               (let ((op (string c)))
                 (incf i)
                 (when (and (< i n) (char= (char input i) #\=))
                   (setf op (concatenate 'string op "="))
                   (incf i))
                 (when (string= op "!") (error "bad operator: !"))
                 (push (make-token :kind :op :op op) tokens)))
              ((or (digit-char-p c)
                   (and (char= c #\-) (< (1+ i) n) (digit-char-p (char input (1+ i)))))
               (let ((start i))
                 (when (char= c #\-) (incf i))
                 (loop while (and (< i n) (digit-char-p (char input i))) do (incf i))
                 (let ((num (parse-integer (subseq input start i))))
                   (push (make-token :kind :number :number num) tokens))))
              ((char-alpha-or-underscore-p c)
               (let ((start i))
                 (loop while (and (< i n) (char-alnum-or-underscore-p (char input i))) do (incf i))
                 (push (make-token :kind :word
                                    :word (string-downcase (subseq input start i)))
                       tokens)))
              (t (error "unexpected char: ~A" c)))))
    (nreverse tokens)))

(defstruct sql-value
  (is-num t :type boolean)
  (num 0 :type integer)
  (text "" :type string))

(defstruct where-clause
  (column "" :type string)
  (op "" :type string)
  (value nil :type (or null sql-value)))

(defstruct order-by
  (column "" :type string)
  (order :asc :type keyword))

(defstruct query
  (projection-all nil :type boolean)
  (projection-columns nil :type list)
  (table "" :type string)
  (where nil :type (or null where-clause))
  (order-by nil :type (or null order-by))
  (limit nil :type (or null integer)))

(defstruct parser
  (tokens nil :type list)
  (pos 0 :type integer))

(defun parser-peek (p)
  (nth (parser-pos p) (parser-tokens p)))

(defun parser-bump (p)
  (let ((t1 (parser-peek p)))
    (incf (parser-pos p))
    t1))

(defun parser-expect-word (p word)
  (let ((tok (parser-bump p)))
    (unless (and tok (eq (token-kind tok) :word) (string= (token-word tok) word))
      (error "expected '~A', got ~A" word tok))))

(defun parser-peek-word (p)
  (let ((tok (parser-peek p)))
    (when (and tok (eq (token-kind tok) :word))
      (token-word tok))))

(defun parser-parse-identifier (p)
  (let ((tok (parser-bump p)))
    (unless (and tok (eq (token-kind tok) :word))
      (error "expected identifier, got ~A" tok))
    (token-word tok)))

(defun parser-parse-projection (p)
  (let ((tok (parser-peek p)))
    (cond
      ((and tok (eq (token-kind tok) :star))
       (parser-bump p)
       (values t nil))
      (t
       (let ((cols (list (parser-parse-identifier p))))
         (loop while (let ((n (parser-peek p)))
                       (and n (eq (token-kind n) :comma)))
               do (parser-bump p)
                  (push (parser-parse-identifier p) cols))
         (values nil (nreverse cols)))))))

(defun parser-parse-where (p)
  (let* ((column (parser-parse-identifier p))
         (op-tok (parser-bump p)))
    (unless (and op-tok (eq (token-kind op-tok) :op))
      (error "expected operator, got ~A" op-tok))
    (let ((val-tok (parser-bump p)))
      (cond
        ((and val-tok (eq (token-kind val-tok) :number))
         (make-where-clause :column column :op (token-op op-tok)
                             :value (make-sql-value :is-num t
                                                     :num (token-number val-tok))))
        ((and val-tok (eq (token-kind val-tok) :string))
         (make-where-clause :column column :op (token-op op-tok)
                             :value (make-sql-value :is-num nil
                                                     :text (token-string val-tok))))
        (t (error "expected value, got ~A" val-tok))))))

(defun parser-parse-order-by (p)
  (let ((column (parser-parse-identifier p))
        (order :asc))
    (let ((w (parser-peek-word p)))
      (cond
        ((equal w "asc") (parser-bump p))
        ((equal w "desc") (parser-bump p) (setf order :desc))))
    (make-order-by :column column :order order)))

(defun parser-parse-query (p)
  (parser-expect-word p "select")
  (multiple-value-bind (all cols) (parser-parse-projection p)
    (parser-expect-word p "from")
    (let* ((table (parser-parse-identifier p))
           (q (make-query :projection-all all :projection-columns cols :table table)))
      (when (equal (parser-peek-word p) "where")
        (parser-bump p)
        (setf (query-where q) (parser-parse-where p)))
      (when (equal (parser-peek-word p) "order")
        (parser-bump p)
        (parser-expect-word p "by")
        (setf (query-order-by q) (parser-parse-order-by p)))
      (when (equal (parser-peek-word p) "limit")
        (parser-bump p)
        (let ((tok (parser-bump p)))
          (unless (and tok (eq (token-kind tok) :number) (>= (token-number tok) 0))
            (error "expected positive number after LIMIT"))
          (setf (query-limit q) (token-number tok))))
      q)))

(defun parse-sql (input)
  (let ((tokens (tokenise input)))
    (when (and tokens (eq (token-kind (car (last tokens))) :semicolon))
      (setf tokens (butlast tokens)))
    (let* ((p (make-parser :tokens tokens :pos 0))
           (q (parser-parse-query p)))
      (when (< (parser-pos p) (length (parser-tokens p)))
        (error "trailing tokens after query"))
      q)))

(defstruct schema
  (tables (make-hash-table :test 'equal) :type hash-table))

(defun schema-add-table (s name columns)
  (let ((set (make-hash-table :test 'equal)))
    (dolist (c columns) (setf (gethash c set) t))
    (setf (gethash name (schema-tables s)) set)))

(defstruct finding
  (severity :info :type keyword)
  (code "" :type string)
  (message "" :type string))

(defstruct analysis
  (tables nil :type list)
  (columns nil :type list)
  (findings nil :type list))

(defun analyse (query schema)
  (let ((findings nil)
        (tables (list (query-table query)))
        (columns nil)
        (known (gethash (query-table query) (schema-tables schema))))
    (unless known
      (push (make-finding :severity :error :code "UNKNOWN_TABLE"
                           :message (format nil "unknown table: ~A" (query-table query)))
            findings))
    (let ((referenced nil))
      (unless (query-projection-all query)
        (setf referenced (append referenced (query-projection-columns query))))
      (when (query-where query)
        (push (where-clause-column (query-where query)) referenced))
      (when (query-order-by query)
        (push (order-by-column (query-order-by query)) referenced))
      (dolist (col (remove-duplicates referenced :test #'string=))
        (push col columns)
        (when (and known (not (gethash col known)))
          (push (make-finding :severity :error :code "UNKNOWN_COLUMN"
                               :message (format nil "unknown column ~A in table ~A"
                                                 col (query-table query)))
                findings))))
    (when (query-projection-all query)
      (push (make-finding :severity :warning :code "SELECT_STAR"
                           :message "SELECT * pulls every column; name them explicitly")
            findings))
    (when (and (query-limit query) (not (query-order-by query)))
      (push (make-finding :severity :warning :code "LIMIT_WITHOUT_ORDER"
                           :message "LIMIT without ORDER BY returns arbitrary rows")
            findings))
    (when (and (not (query-where query)) (not (query-limit query)))
      (push (make-finding :severity :info :code "FULL_SCAN"
                           :message "no WHERE or LIMIT — query will scan every row")
            findings))
    (make-analysis :tables tables :columns (nreverse columns)
                    :findings (nreverse findings))))

(defun demo ()
  (let ((schema (make-schema)))
    (schema-add-table schema "users" '("id" "name" "email" "created_at"))
    (schema-add-table schema "orders" '("id" "user_id" "amount" "status"))
    (dolist (s '("SELECT id, name FROM users WHERE id = 42"
                 "SELECT * FROM users LIMIT 10"
                 "SELECT name FROM users ORDER BY created_at DESC LIMIT 5"
                 "SELECT birthday FROM widgets"))
      (format t "~%=== ~A ===~%" s)
      (handler-case
          (let* ((q (parse-sql s))
                 (a (analyse q schema)))
            (format t "  tables:  ~A~%" (analysis-tables a))
            (format t "  columns: ~A~%" (sort (copy-list (analysis-columns a)) #'string<))
            (dolist (f (analysis-findings a))
              (format t "  [~7A] ~A: ~A~%"
                      (finding-severity f) (finding-code f) (finding-message f))))
        (error (e) (format t "  parse error: ~A~%" e))))))
