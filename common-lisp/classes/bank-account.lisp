;;;; Bank Account Manager — double-entry ledger, atomic transactions, balance as aggregate.

(defpackage #:bank-account
  (:use #:cl)
  (:export #:make-customer
           #:customer-id #:customer-name
           #:make-account
           #:account-id #:account-holder-id #:account-kind #:account-status
           #:make-entry
           #:entry-account-id #:entry-amount-cents
           #:make-transaction
           #:transaction-id #:transaction-kind #:transaction-entries #:transaction-memo
           #:make-bank
           #:add-customer
           #:open-account
           #:close-account
           #:freeze-account
           #:unfreeze-account
           #:deposit
           #:withdraw
           #:transfer
           #:balance
           #:statement
           #:total-assets
           #:ledger-balanced-p))

(in-package #:bank-account)

(defstruct customer
  (id "" :type string)
  (name "" :type string))

(defstruct account
  (id "" :type string)
  (holder-id "" :type string)
  (kind :checking :type keyword)   ; :checking :savings :credit
  (status :open :type keyword))    ; :open :frozen :closed

(defstruct entry
  (account-id "" :type string)
  (amount-cents 0 :type integer))  ; +credit, -debit

(defstruct transaction
  (id 0 :type integer)
  (kind :deposit :type keyword)    ; :deposit :withdrawal :transfer :fee :interest
  (entries nil :type list)
  (memo "" :type string)
  (at 0 :type integer))            ; universal time

(defstruct bank
  (customers (make-hash-table :test 'equal))
  (accounts (make-hash-table :test 'equal))
  (transactions (make-hash-table :test 'eql))
  (next-tx-id 1 :type integer)
  (next-account-seq 1 :type integer))

(defun add-customer (bank c)
  (when (gethash (customer-id c) (bank-customers bank))
    (error "duplicate customer: ~A" (customer-id c)))
  (setf (gethash (customer-id c) (bank-customers bank)) c)
  c)

(defun open-account (bank holder-id kind &key account-id)
  (unless (gethash holder-id (bank-customers bank))
    (error "unknown customer: ~A" holder-id))
  (let ((id (or account-id
                (prog1 (format nil "A~6,'0D" (bank-next-account-seq bank))
                  (incf (bank-next-account-seq bank))))))
    (when (gethash id (bank-accounts bank))
      (error "duplicate account id: ~A" id))
    (let ((a (make-account :id id :holder-id holder-id :kind kind :status :open)))
      (setf (gethash id (bank-accounts bank)) a)
      a)))

(defun %require-account (bank account-id)
  (or (gethash account-id (bank-accounts bank))
      (error "unknown account: ~A" account-id)))

(defun %require-active (bank account-id)
  (let ((a (%require-account bank account-id)))
    (case (account-status a)
      (:frozen (error "account frozen: ~A" account-id))
      (:closed (error "account closed: ~A" account-id)))
    a))

(defun freeze-account (bank account-id)
  (let ((a (%require-account bank account-id)))
    (setf (account-status a) :frozen)
    a))

(defun unfreeze-account (bank account-id)
  (let ((a (%require-account bank account-id)))
    (when (eq (account-status a) :frozen)
      (setf (account-status a) :open))
    a))

(defun close-account (bank account-id)
  (let ((a (%require-account bank account-id))
        (bal (balance bank account-id)))
    (unless (zerop bal)
      (error "cannot close ~A with balance ~D cents" account-id bal))
    (setf (account-status a) :closed)
    a))

(defun %commit (bank kind entries memo)
  (let* ((id (bank-next-tx-id bank))
         (tx (make-transaction :id id :kind kind :entries entries
                               :memo memo :at (get-universal-time))))
    (setf (gethash id (bank-transactions bank)) tx)
    (incf (bank-next-tx-id bank))
    tx))

(defun deposit (bank account-id amount-cents &optional (memo ""))
  (unless (plusp amount-cents)
    (error "deposit amount must be positive cents"))
  (%require-active bank account-id)
  (%commit bank :deposit
           (list (make-entry :account-id account-id :amount-cents amount-cents))
           memo))

(defun withdraw (bank account-id amount-cents &optional (memo ""))
  (unless (plusp amount-cents)
    (error "withdraw amount must be positive cents"))
  (%require-active bank account-id)
  (let ((bal (balance bank account-id)))
    (when (< bal amount-cents)
      (error "insufficient funds: ~A has ~D, requested ~D"
             account-id bal amount-cents)))
  (%commit bank :withdrawal
           (list (make-entry :account-id account-id :amount-cents (- amount-cents)))
           memo))

(defun transfer (bank from-id to-id amount-cents &optional (memo ""))
  (unless (plusp amount-cents)
    (error "transfer amount must be positive cents"))
  (when (string= from-id to-id)
    (error "transfer requires distinct accounts"))
  (%require-active bank from-id)
  (%require-active bank to-id)
  (let ((bal (balance bank from-id)))
    (when (< bal amount-cents)
      (error "insufficient funds: ~A has ~D, requested ~D"
             from-id bal amount-cents)))
  (let ((entries (list (make-entry :account-id from-id :amount-cents (- amount-cents))
                       (make-entry :account-id to-id :amount-cents amount-cents))))
    (unless (zerop (reduce #'+ entries :key #'entry-amount-cents))
      (error "transfer entries must sum to zero"))
    (%commit bank :transfer entries memo)))

(defun balance (bank account-id)
  (%require-account bank account-id)
  (let ((total 0))
    (maphash (lambda (k tx)
               (declare (ignore k))
               (dolist (e (transaction-entries tx))
                 (when (string= (entry-account-id e) account-id)
                   (incf total (entry-amount-cents e)))))
             (bank-transactions bank))
    total))

(defun statement (bank account-id &key from-time to-time)
  (%require-account bank account-id)
  (let ((result nil))
    (maphash (lambda (k tx)
               (declare (ignore k))
               (when (and (some (lambda (e) (string= (entry-account-id e) account-id))
                                (transaction-entries tx))
                          (or (null from-time) (>= (transaction-at tx) from-time))
                          (or (null to-time) (<= (transaction-at tx) to-time)))
                 (push tx result)))
             (bank-transactions bank))
    (sort result #'< :key #'transaction-at)))

(defun total-assets (bank customer-id)
  (unless (gethash customer-id (bank-customers bank))
    (error "unknown customer: ~A" customer-id))
  (let ((total 0))
    (maphash (lambda (id a)
               (declare (ignore id))
               (when (string= (account-holder-id a) customer-id)
                 (incf total (balance bank (account-id a)))))
             (bank-accounts bank))
    total))

(defun ledger-balanced-p (bank)
  "Every transfer transaction's entries must sum to zero."
  (let ((balanced t))
    (maphash (lambda (k tx)
               (declare (ignore k))
               (when (eq (transaction-kind tx) :transfer)
                 (unless (zerop (reduce #'+ (transaction-entries tx)
                                         :key #'entry-amount-cents))
                   (setf balanced nil))))
             (bank-transactions bank))
    balanced))
