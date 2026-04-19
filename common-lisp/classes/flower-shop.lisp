;;;; Flower Shop Ordering — build-up/commit cart with frozen prices and reversible orders.

(defpackage #:flower-shop
  (:use #:cl)
  (:export #:make-flower
           #:flower-sku #:flower-name #:flower-unit-price-cents #:flower-stock
           #:make-cart-line
           #:cart-line-sku #:cart-line-quantity #:cart-line-unit-price-cents
           #:cart-line-total-cents
           #:make-discount-rule
           #:discount-rule-kind #:discount-rule-value
           #:discount-rule-min-subtotal-cents
           #:discount-cents
           #:make-cart
           #:cart-id #:cart-owner-id #:cart-lines
           #:cart-discount-rules #:cart-shipping-cents #:cart-tax-rate-bps
           #:cart-status
           #:make-order
           #:order-id #:order-owner-id #:order-lines #:order-discount-rules
           #:order-shipping-cents #:order-tax-rate-bps #:order-status
           #:make-shop
           #:add-flower #:stock-flower
           #:new-cart
           #:add-to-cart
           #:update-quantity
           #:remove-from-cart
           #:apply-discount
           #:subtotal-cents
           #:total-discount-cents
           #:tax-cents
           #:total-cents
           #:checkout
           #:cancel-order
           #:fulfill-order))

(in-package #:flower-shop)

(defstruct flower
  (sku "" :type string)
  (name "" :type string)
  (unit-price-cents 0 :type integer)
  (stock 0 :type integer))

(defstruct cart-line
  (sku "" :type string)
  (quantity 0 :type integer)
  (unit-price-cents 0 :type integer))       ; frozen

(defun cart-line-total-cents (l)
  (* (cart-line-quantity l) (cart-line-unit-price-cents l)))

(defstruct discount-rule
  (kind :percentage-off :type keyword)     ; :percentage-off :fixed-off :free-shipping
  (value 0 :type integer)
  (min-subtotal-cents 0 :type integer)
  (note "" :type string))

(defun discount-cents (rule subtotal shipping)
  (if (< subtotal (discount-rule-min-subtotal-cents rule))
      0
      (ecase (discount-rule-kind rule)
        (:percentage-off (floor (* subtotal (discount-rule-value rule)) 100))
        (:fixed-off      (min (discount-rule-value rule) subtotal))
        (:free-shipping  (if (>= subtotal (discount-rule-value rule)) shipping 0)))))

(defstruct cart
  (id "" :type string)
  (owner-id "" :type string)
  (lines nil :type list)
  (discount-rules nil :type list)
  (shipping-cents 0 :type integer)
  (tax-rate-bps 0 :type integer)
  (status :draft :type keyword))           ; :draft :placed :fulfilled :cancelled

(defstruct order
  (id 0 :type integer)
  (owner-id "" :type string)
  (lines nil :type list)
  (discount-rules nil :type list)
  (shipping-cents 0 :type integer)
  (tax-rate-bps 0 :type integer)
  (status :placed :type keyword))

(defstruct shop
  (flowers (make-hash-table :test 'equal))
  (carts (make-hash-table :test 'equal))
  (orders (make-hash-table :test 'eql))
  (next-order-id 1 :type integer))

(defun add-flower (shop f)
  (when (gethash (flower-sku f) (shop-flowers shop))
    (error "duplicate flower: ~A" (flower-sku f)))
  (setf (gethash (flower-sku f) (shop-flowers shop)) f)
  f)

(defun stock-flower (shop sku quantity)
  (when (minusp quantity) (error "non-negative quantity required"))
  (let ((f (or (gethash sku (shop-flowers shop))
               (error "unknown flower: ~A" sku))))
    (incf (flower-stock f) quantity)))

(defun new-cart (shop cart-id owner-id &key (shipping-cents 0) (tax-rate-bps 0))
  (when (gethash cart-id (shop-carts shop))
    (error "duplicate cart: ~A" cart-id))
  (let ((c (make-cart :id cart-id :owner-id owner-id
                      :shipping-cents shipping-cents
                      :tax-rate-bps tax-rate-bps)))
    (setf (gethash cart-id (shop-carts shop)) c)
    c))

(defun %require-draft (shop cart-id)
  (let ((c (or (gethash cart-id (shop-carts shop))
               (error "unknown cart: ~A" cart-id))))
    (unless (eq (cart-status c) :draft)
      (error "cart ~A is ~A, not draft" cart-id (cart-status c)))
    c))

(defun add-to-cart (shop cart-id sku quantity)
  (unless (plusp quantity) (error "quantity must be positive"))
  (let* ((flower (or (gethash sku (shop-flowers shop))
                     (error "unknown flower: ~A" sku)))
         (cart (%require-draft shop cart-id))
         (already (loop for l in (cart-lines cart)
                        when (string= (cart-line-sku l) sku)
                          sum (cart-line-quantity l))))
    (when (< (flower-stock flower) (+ already quantity))
      (error "out of stock: ~A (want ~A, stock ~A)"
             sku (+ already quantity) (flower-stock flower)))
    (let ((existing (find-if (lambda (l) (string= (cart-line-sku l) sku))
                             (cart-lines cart))))
      (if existing
          (incf (cart-line-quantity existing) quantity)
          (push (make-cart-line :sku sku :quantity quantity
                                :unit-price-cents (flower-unit-price-cents flower))
                (cart-lines cart))))
    cart))

(defun update-quantity (shop cart-id sku new-quantity)
  (when (minusp new-quantity) (error "quantity must be non-negative"))
  (let* ((flower (or (gethash sku (shop-flowers shop))
                     (error "unknown flower: ~A" sku)))
         (cart (%require-draft shop cart-id)))
    (when (and (plusp new-quantity)
               (< (flower-stock flower) new-quantity))
      (error "out of stock: ~A" sku))
    (let ((existing (find-if (lambda (l) (string= (cart-line-sku l) sku))
                             (cart-lines cart))))
      (unless existing (error "~A not in cart ~A" sku cart-id))
      (if (zerop new-quantity)
          (setf (cart-lines cart)
                (remove-if (lambda (l) (string= (cart-line-sku l) sku))
                           (cart-lines cart)))
          (setf (cart-line-quantity existing) new-quantity)))
    cart))

(defun remove-from-cart (shop cart-id sku)
  (update-quantity shop cart-id sku 0))

(defun apply-discount (shop cart-id rule)
  (let ((cart (%require-draft shop cart-id)))
    (push rule (cart-discount-rules cart))
    cart))

(defun subtotal-cents (cart-or-order)
  (reduce #'+ (if (cart-p cart-or-order)
                  (cart-lines cart-or-order)
                  (order-lines cart-or-order))
          :key #'cart-line-total-cents
          :initial-value 0))

(defun %rules-and-shipping (cart-or-order)
  (if (cart-p cart-or-order)
      (values (cart-discount-rules cart-or-order)
              (cart-shipping-cents cart-or-order)
              (cart-tax-rate-bps cart-or-order))
      (values (order-discount-rules cart-or-order)
              (order-shipping-cents cart-or-order)
              (order-tax-rate-bps cart-or-order))))

(defun total-discount-cents (cart-or-order)
  (let ((sub (subtotal-cents cart-or-order)))
    (multiple-value-bind (rules shipping) (%rules-and-shipping cart-or-order)
      (reduce #'+ rules
              :key (lambda (r) (discount-cents r sub shipping))
              :initial-value 0))))

(defun tax-cents (cart-or-order)
  (multiple-value-bind (rules shipping bps) (%rules-and-shipping cart-or-order)
    (declare (ignore rules shipping))
    (let ((taxable (max 0 (- (subtotal-cents cart-or-order)
                             (total-discount-cents cart-or-order)))))
      (floor (* taxable bps) 10000))))

(defun total-cents (cart-or-order)
  (multiple-value-bind (rules shipping bps) (%rules-and-shipping cart-or-order)
    (declare (ignore rules bps))
    (max 0 (+ (- (subtotal-cents cart-or-order)
                 (total-discount-cents cart-or-order))
              (tax-cents cart-or-order)
              shipping))))

(defun checkout (shop cart-id)
  (let ((cart (%require-draft shop cart-id)))
    (unless (cart-lines cart) (error "cart ~A is empty" cart-id))
    ;; Pre-flight.
    (dolist (l (cart-lines cart))
      (let ((f (or (gethash (cart-line-sku l) (shop-flowers shop))
                   (error "unknown flower: ~A" (cart-line-sku l)))))
        (when (< (flower-stock f) (cart-line-quantity l))
          (error "out of stock at checkout: ~A" (cart-line-sku l)))))
    ;; Commit.
    (dolist (l (cart-lines cart))
      (decf (flower-stock (gethash (cart-line-sku l) (shop-flowers shop)))
            (cart-line-quantity l)))
    (let* ((id (shop-next-order-id shop))
           (o (make-order :id id
                          :owner-id (cart-owner-id cart)
                          :lines (copy-list (cart-lines cart))
                          :discount-rules (copy-list (cart-discount-rules cart))
                          :shipping-cents (cart-shipping-cents cart)
                          :tax-rate-bps (cart-tax-rate-bps cart)
                          :status :placed)))
      (setf (gethash id (shop-orders shop)) o)
      (incf (shop-next-order-id shop))
      (setf (cart-status cart) :placed)
      o)))

(defun cancel-order (shop order-id)
  (let ((o (or (gethash order-id (shop-orders shop))
               (error "unknown order: ~A" order-id))))
    (unless (eq (order-status o) :placed)
      (error "order ~A is ~A, only :placed can be cancelled"
             order-id (order-status o)))
    (setf (order-status o) :cancelled)
    ;; Reverse inventory decrement.
    (dolist (l (order-lines o))
      (let ((f (gethash (cart-line-sku l) (shop-flowers shop))))
        (when f (incf (flower-stock f) (cart-line-quantity l)))))
    o))

(defun fulfill-order (shop order-id)
  (let ((o (or (gethash order-id (shop-orders shop))
               (error "unknown order: ~A" order-id))))
    (unless (eq (order-status o) :placed)
      (error "order ~A is ~A, only :placed can be fulfilled"
             order-id (order-status o)))
    (setf (order-status o) :fulfilled)
    o))
