;;;; Calculator — evaluate arithmetic expressions.
;;;; CL has READ for s-expressions, but we implement infix parsing explicitly.

(defpackage :numbers/calculator
  (:use :cl)
  (:export :evaluate))

(in-package :numbers/calculator)

;;; Tokenizer

(defstruct token
  (kind nil :type keyword)    ; :number :plus :minus :star :slash :lparen :rparen
  (value 0.0d0 :type double-float))

(defun tokenize (expr)
  "Tokenize an expression string into a list of token structs."
  (let ((tokens '())
        (chars (coerce expr 'list))
        (i 0)
        (len (length expr)))
    (loop while (< i len)
          for c = (nth i chars)
          do (cond
               ((member c '(#\Space #\Tab))
                (incf i))
               ((char= c #\+)
                (push (make-token :kind :plus) tokens)
                (incf i))
               ((char= c #\-)
                (push (make-token :kind :minus) tokens)
                (incf i))
               ((char= c #\*)
                (push (make-token :kind :star) tokens)
                (incf i))
               ((char= c #\/)
                (push (make-token :kind :slash) tokens)
                (incf i))
               ((char= c #\()
                (push (make-token :kind :lparen) tokens)
                (incf i))
               ((char= c #\))
                (push (make-token :kind :rparen) tokens)
                (incf i))
               ((or (digit-char-p c) (char= c #\.))
                (let ((start i))
                  (loop while (and (< i len)
                                   (let ((ch (nth i chars)))
                                     (or (digit-char-p ch) (char= ch #\.))))
                        do (incf i))
                  (let ((numstr (subseq expr start i)))
                    (push (make-token :kind :number
                                      :value (coerce (read-from-string numstr) 'double-float))
                          tokens))))
               (t (error "Unexpected character: ~a" c))))
    (nreverse tokens)))

;;; Recursive descent parser
;;;
;;; Grammar:
;;;   expr   -> term (('+' | '-') term)*
;;;   term   -> unary (('*' | '/') unary)*
;;;   unary  -> '-' unary | atom
;;;   atom   -> NUMBER | '(' expr ')'

(defvar *tokens* nil)
(defvar *pos* 0)

(defun peek-token ()
  (when (< *pos* (length *tokens*))
    (nth *pos* *tokens*)))

(defun consume-token ()
  (prog1 (nth *pos* *tokens*)
    (incf *pos*)))

(defun expect-token (kind)
  (let ((tok (peek-token)))
    (unless (and tok (eq (token-kind tok) kind))
      (error "Expected ~a, got ~a" kind (if tok (token-kind tok) "end of input")))
    (consume-token)))

(defun parse-expr ()
  (let ((left (parse-term)))
    (loop while (and (peek-token)
                     (member (token-kind (peek-token)) '(:plus :minus)))
          do (let ((op (token-kind (consume-token)))
                   (right (parse-term)))
               (setf left (if (eq op :plus)
                              (+ left right)
                              (- left right)))))
    left))

(defun parse-term ()
  (let ((left (parse-unary)))
    (loop while (and (peek-token)
                     (member (token-kind (peek-token)) '(:star :slash)))
          do (let ((op (token-kind (consume-token)))
                   (right (parse-unary)))
               (setf left (if (eq op :star)
                              (* left right)
                              (progn
                                (when (zerop right)
                                  (error "Division by zero"))
                                (/ left right))))))
    left))

(defun parse-unary ()
  (cond
    ((and (peek-token) (eq (token-kind (peek-token)) :minus))
     (consume-token)
     (- (parse-unary)))
    ((and (peek-token) (eq (token-kind (peek-token)) :plus))
     (consume-token)
     (parse-unary))
    (t (parse-atom))))

(defun parse-atom ()
  (let ((tok (peek-token)))
    (unless tok
      (error "Unexpected end of input"))
    (case (token-kind tok)
      (:lparen
       (consume-token)
       (let ((result (parse-expr)))
         (expect-token :rparen)
         result))
      (:number
       (consume-token)
       (token-value tok))
      (otherwise
       (error "Unexpected token: ~a" (token-kind tok))))))

(defun evaluate (expr-string)
  "Evaluate an arithmetic expression string and return the result as a double-float."
  (let* ((*tokens* (tokenize expr-string))
         (*pos* 0))
    (when (null *tokens*)
      (error "Empty expression"))
    (let ((result (parse-expr)))
      (when (< *pos* (length *tokens*))
        (error "Unexpected token after expression: ~a"
               (token-kind (peek-token))))
      result)))
