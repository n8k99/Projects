;;;; Vending Machine — a finite state machine with coin-constrained change-making.

(defpackage #:vending-machine
  (:use #:cl)
  (:export #:make-slot
           #:slot-id #:slot-product #:slot-price-cents #:slot-quantity
           #:make-machine
           #:machine-state #:machine-inserted-cents
           #:add-slot
           #:load-change
           #:insert-coin
           #:select-slot
           #:refund))

(in-package #:vending-machine)

(defparameter *denominations* '(100 25 10 5 1))

(defstruct slot
  (id "" :type string)
  (product "" :type string)
  (price-cents 0 :type integer)
  (quantity 0 :type integer))

(defstruct machine
  (slots (make-hash-table :test 'equal))
  (coin-inventory (make-hash-table :test 'eql))    ; denom -> count
  (inserted-coins (make-hash-table :test 'eql))
  (inserted-cents 0 :type integer)
  (state :idle :type keyword))                     ; :idle :accepting

(defun %init-coin-table ()
  (let ((h (make-hash-table :test 'eql)))
    (dolist (d *denominations*) (setf (gethash d h) 0))
    h))

(defun make-fresh-machine ()
  (make-machine :coin-inventory (%init-coin-table)
                :inserted-coins (%init-coin-table)))

(defun add-slot (machine slot)
  (when (gethash (slot-id slot) (machine-slots machine))
    (error "duplicate slot: ~A" (slot-id slot)))
  (unless (plusp (slot-price-cents slot))
    (error "price must be positive"))
  (setf (gethash (slot-id slot) (machine-slots machine)) slot)
  slot)

(defun %valid-denom-p (d) (member d *denominations*))

(defun load-change (machine denom count)
  (unless (%valid-denom-p denom)
    (error "invalid denomination: ~A" denom))
  (when (minusp count) (error "count must be non-negative"))
  (incf (gethash denom (machine-coin-inventory machine) 0) count))

(defun insert-coin (machine denom)
  (unless (%valid-denom-p denom)
    (error "invalid denomination: ~A" denom))
  (incf (machine-inserted-cents machine) denom)
  (incf (gethash denom (machine-inserted-coins machine) 0))
  (setf (machine-state machine) :accepting)
  (machine-inserted-cents machine))

(defun refund (machine)
  (let ((refund nil))
    (maphash (lambda (d c)
               (when (plusp c) (push (cons d c) refund))
               (setf (gethash d (machine-inserted-coins machine)) 0))
             (machine-inserted-coins machine))
    (setf (machine-inserted-cents machine) 0
          (machine-state machine) :idle)
    refund))

(defun %greedy-change (amount pool)
  "Given POOL (hash denom->count) return an alist of (denom . count) totalling
   AMOUNT, or NIL if no such combination is available."
  (let ((remaining amount)
        (result nil)
        (denoms (sort (copy-list *denominations*) #'>)))
    (dolist (d denoms)
      (when (<= d remaining)
        (let* ((available (gethash d pool 0))
               (use (min available (floor remaining d))))
          (when (plusp use)
            (push (cons d use) result)
            (decf remaining (* d use))))))
    (if (zerop remaining) (nreverse result) nil)))

(defun select-slot (machine slot-id)
  (unless (eq (machine-state machine) :accepting)
    (error "cannot select in state ~A" (machine-state machine)))
  (let ((slot (or (gethash slot-id (machine-slots machine))
                  (error "unknown slot: ~A" slot-id))))
    (when (zerop (slot-quantity slot))
      (error "sold out: ~A" slot-id))
    (when (< (machine-inserted-cents machine) (slot-price-cents slot))
      (error "insufficient funds: ~A costs ~A cents, inserted ~A"
             (slot-product slot) (slot-price-cents slot)
             (machine-inserted-cents machine)))
    (let* ((change-owed (- (machine-inserted-cents machine)
                           (slot-price-cents slot)))
           (pool (make-hash-table :test 'eql)))
      (maphash (lambda (d c) (setf (gethash d pool 0) c))
               (machine-coin-inventory machine))
      (maphash (lambda (d c) (incf (gethash d pool 0) c))
               (machine-inserted-coins machine))
      (let ((change (%greedy-change change-owed pool)))
        (unless (or (zerop change-owed) change)
          (error "cannot make change for ~A cents" change-owed))
        ;; Commit
        (dolist (entry change)
          (decf (gethash (car entry) pool) (cdr entry)))
        (clrhash (machine-coin-inventory machine))
        (maphash (lambda (d c) (setf (gethash d (machine-coin-inventory machine)) c))
                 pool)
        (decf (slot-quantity slot))
        (setf (machine-inserted-cents machine) 0)
        (maphash (lambda (d c)
                   (declare (ignore c))
                   (setf (gethash d (machine-inserted-coins machine)) 0))
                 (machine-inserted-coins machine))
        (setf (machine-state machine) :idle)
        (values (slot-product slot) change)))))
