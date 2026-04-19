;;;; Product Inventory — entities with identity, movements as a ledger, current quantity as aggregate.

(defpackage #:product-inventory
  (:use #:cl)
  (:export #:make-product
           #:product-sku
           #:product-name
           #:product-price
           #:product-category
           #:product-reorder-point
           #:make-inventory
           #:add-product
           #:receive-stock
           #:sell-stock
           #:quantity-on-hand
           #:history-for
           #:total-value
           #:restock-needed
           #:by-category))

(in-package #:product-inventory)

(defstruct product
  (sku "" :type string)
  (name "" :type string)
  (price 0.0 :type number)
  (category "" :type string)
  (reorder-point 0 :type integer))

(defstruct movement
  (sku "" :type string)
  (kind :receive :type keyword)  ; :receive or :sell
  (quantity 0 :type integer)
  (note "" :type string))

(defun movement-signed (m)
  "Signed effect of a movement: +quantity for :receive, -quantity for :sell."
  (if (eq (movement-kind m) :receive)
      (movement-quantity m)
      (- (movement-quantity m))))

(defstruct inventory
  (products (make-hash-table :test 'equal))
  (movements nil))

(defun add-product (inv product)
  "Register a product. Error if sku already present."
  (when (gethash (product-sku product) (inventory-products inv))
    (error "duplicate sku: ~A" (product-sku product)))
  (setf (gethash (product-sku product) (inventory-products inv)) product)
  product)

(defun lookup (inv sku)
  (or (gethash sku (inventory-products inv))
      (error "unknown sku: ~A" sku)))

(defun receive-stock (inv sku quantity &optional (note ""))
  (lookup inv sku)
  (unless (plusp quantity)
    (error "receive quantity must be positive"))
  (push (make-movement :sku sku :kind :receive :quantity quantity :note note)
        (inventory-movements inv))
  quantity)

(defun sell-stock (inv sku quantity &optional (note ""))
  (lookup inv sku)
  (unless (plusp quantity)
    (error "sell quantity must be positive"))
  (let ((on-hand (quantity-on-hand inv sku)))
    (when (< on-hand quantity)
      (error "insufficient stock for ~A: on hand ~A, requested ~A"
             sku on-hand quantity)))
  (push (make-movement :sku sku :kind :sell :quantity quantity :note note)
        (inventory-movements inv))
  quantity)

(defun quantity-on-hand (inv sku)
  (lookup inv sku)
  (reduce #'+ (inventory-movements inv)
          :initial-value 0
          :key (lambda (m) (if (string= (movement-sku m) sku)
                               (movement-signed m)
                               0))))

(defun history-for (inv sku)
  (lookup inv sku)
  (reverse (remove-if-not (lambda (m) (string= (movement-sku m) sku))
                          (inventory-movements inv))))

(defun total-value (inv)
  (let ((total 0.0))
    (maphash (lambda (sku p)
               (incf total (* (product-price p)
                              (quantity-on-hand inv sku))))
             (inventory-products inv))
    total))

(defun restock-needed (inv)
  "Products whose current quantity is at or below their reorder point."
  (let ((needed nil))
    (maphash (lambda (sku p)
               (let ((point (product-reorder-point p)))
                 (when (and (plusp point)
                            (<= (quantity-on-hand inv sku) point))
                   (push p needed))))
             (inventory-products inv))
    needed))

(defun by-category (inv category)
  (let ((result nil))
    (maphash (lambda (sku p)
               (declare (ignore sku))
               (when (string= (product-category p) category)
                 (push p result)))
             (inventory-products inv))
    result))
