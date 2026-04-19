;;;; Recipe Creator and Manager — compositional entities, scaling, pantry checks, shopping lists.
;;;;
;;;; Common Lisp has exact rationals natively (1/3 is just a number), so we
;;;; don't need a separate Qty type — the language solves that piece for us.

(defpackage #:recipe-manager
  (:use #:cl)
  (:export #:make-ingredient
           #:ingredient-id #:ingredient-name #:ingredient-default-unit
           #:make-recipe-line
           #:recipe-line-ingredient-id #:recipe-line-quantity #:recipe-line-unit
           #:make-recipe
           #:recipe-id #:recipe-name #:recipe-lines #:recipe-serves #:recipe-steps
           #:make-pantry-item
           #:pantry-item-quantity #:pantry-item-unit
           #:make-shortage
           #:shortage-ingredient-id #:shortage-required #:shortage-available
           #:shortage-unit #:shortage-reason
           #:make-recipe-book
           #:add-ingredient
           #:add-recipe
           #:stock
           #:add-stock
           #:scale
           #:can-make
           #:shopping-list
           #:make-it
           #:recipes-using
           #:ingredients-used))

(in-package #:recipe-manager)

(defstruct ingredient
  (id "" :type string)
  (name "" :type string)
  (default-unit "" :type string))

(defstruct recipe-line
  (ingredient-id "" :type string)
  (quantity 0 :type (or integer rational))
  (unit "" :type string))

(defstruct recipe
  (id "" :type string)
  (name "" :type string)
  (lines nil :type list)
  (serves 1 :type integer)
  (steps nil :type list))

(defstruct pantry-item
  (quantity 0 :type (or integer rational))
  (unit "" :type string))

(defstruct shortage
  (ingredient-id "" :type string)
  (required 0 :type (or integer rational))
  (available 0 :type (or integer rational))
  (unit "" :type string)
  (reason :insufficient :type keyword))  ; :insufficient :unit-mismatch :not-stocked

(defstruct recipe-book
  (ingredients (make-hash-table :test 'equal))
  (recipes (make-hash-table :test 'equal))
  (pantry (make-hash-table :test 'equal)))

(defun add-ingredient (book i)
  (when (gethash (ingredient-id i) (recipe-book-ingredients book))
    (error "duplicate ingredient: ~A" (ingredient-id i)))
  (setf (gethash (ingredient-id i) (recipe-book-ingredients book)) i)
  i)

(defun add-recipe (book r)
  (when (gethash (recipe-id r) (recipe-book-recipes book))
    (error "duplicate recipe: ~A" (recipe-id r)))
  (dolist (line (recipe-lines r))
    (unless (gethash (recipe-line-ingredient-id line) (recipe-book-ingredients book))
      (error "unknown ingredient: ~A" (recipe-line-ingredient-id line))))
  (setf (gethash (recipe-id r) (recipe-book-recipes book)) r)
  r)

(defun stock (book ingredient-id quantity unit)
  "Set the pantry amount for an ingredient."
  (unless (gethash ingredient-id (recipe-book-ingredients book))
    (error "unknown ingredient: ~A" ingredient-id))
  (setf (gethash ingredient-id (recipe-book-pantry book))
        (make-pantry-item :quantity quantity :unit unit)))

(defun add-stock (book ingredient-id quantity unit)
  "Increase pantry stock; fails on unit mismatch."
  (unless (gethash ingredient-id (recipe-book-ingredients book))
    (error "unknown ingredient: ~A" ingredient-id))
  (let ((existing (gethash ingredient-id (recipe-book-pantry book))))
    (if (null existing)
        (setf (gethash ingredient-id (recipe-book-pantry book))
              (make-pantry-item :quantity quantity :unit unit))
        (progn
          (unless (string= (pantry-item-unit existing) unit)
            (error "unit mismatch for ~A: pantry ~A, adding ~A"
                   ingredient-id (pantry-item-unit existing) unit))
          (incf (pantry-item-quantity existing) quantity)))))

(defun %scale-line (line factor)
  (make-recipe-line :ingredient-id (recipe-line-ingredient-id line)
                    :quantity (* (recipe-line-quantity line) factor)
                    :unit (recipe-line-unit line)))

(defun scale (book recipe-id factor)
  (let ((r (or (gethash recipe-id (recipe-book-recipes book))
               (error "unknown recipe: ~A" recipe-id))))
    (make-recipe :id (recipe-id r)
                 :name (recipe-name r)
                 :lines (mapcar (lambda (l) (%scale-line l factor)) (recipe-lines r))
                 :serves (recipe-serves r)
                 :steps (copy-list (recipe-steps r)))))

(defun %required-lines (book recipe-id servings)
  "Return the scaled line list for SERVINGS (or the recipe's own serves if NIL)."
  (let ((r (or (gethash recipe-id (recipe-book-recipes book))
               (error "unknown recipe: ~A" recipe-id))))
    (cond
      ((or (null servings) (= servings (recipe-serves r)))
       (copy-list (recipe-lines r)))
      (t
       (let ((factor (/ servings (recipe-serves r))))
         (mapcar (lambda (l) (%scale-line l factor)) (recipe-lines r)))))))

(defun can-make (book recipe-id &optional servings)
  "Return (values ok-p shortages). Conjunction over every line."
  (let ((shortages nil))
    (dolist (line (%required-lines book recipe-id servings))
      (let ((p (gethash (recipe-line-ingredient-id line) (recipe-book-pantry book))))
        (cond
          ((null p)
           (push (make-shortage :ingredient-id (recipe-line-ingredient-id line)
                                :required (recipe-line-quantity line)
                                :available 0
                                :unit (recipe-line-unit line)
                                :reason :not-stocked)
                 shortages))
          ((not (string= (pantry-item-unit p) (recipe-line-unit line)))
           (push (make-shortage :ingredient-id (recipe-line-ingredient-id line)
                                :required (recipe-line-quantity line)
                                :available 0
                                :unit (recipe-line-unit line)
                                :reason :unit-mismatch)
                 shortages))
          ((< (pantry-item-quantity p) (recipe-line-quantity line))
           (push (make-shortage :ingredient-id (recipe-line-ingredient-id line)
                                :required (recipe-line-quantity line)
                                :available (pantry-item-quantity p)
                                :unit (recipe-line-unit line)
                                :reason :insufficient)
                 shortages)))))
    (values (null shortages) (nreverse shortages))))

(defun shopping-list (book recipe-id &optional servings)
  "The gap report: required minus available, per line. Only deficits appear."
  (let ((out nil))
    (dolist (line (%required-lines book recipe-id servings))
      (let ((p (gethash (recipe-line-ingredient-id line) (recipe-book-pantry book))))
        (cond
          ((or (null p)
               (not (string= (pantry-item-unit p) (recipe-line-unit line))))
           (push line out))
          (t
           (let ((deficit (- (recipe-line-quantity line) (pantry-item-quantity p))))
             (when (plusp deficit)
               (push (make-recipe-line :ingredient-id (recipe-line-ingredient-id line)
                                       :quantity deficit
                                       :unit (recipe-line-unit line))
                     out)))))))
    (nreverse out)))

(defun make-it (book recipe-id &optional servings)
  "Atomically consume required ingredients. All-or-nothing."
  (multiple-value-bind (ok shortages) (can-make book recipe-id servings)
    (unless ok
      (error "cannot make ~A: ~D shortage(s)" recipe-id (length shortages))))
  (dolist (line (%required-lines book recipe-id servings))
    (decf (pantry-item-quantity
           (gethash (recipe-line-ingredient-id line) (recipe-book-pantry book)))
          (recipe-line-quantity line)))
  (gethash recipe-id (recipe-book-recipes book)))

(defun recipes-using (book ingredient-id)
  (unless (gethash ingredient-id (recipe-book-ingredients book))
    (error "unknown ingredient: ~A" ingredient-id))
  (let ((result nil))
    (maphash (lambda (id r)
               (declare (ignore id))
               (when (some (lambda (l) (string= (recipe-line-ingredient-id l) ingredient-id))
                           (recipe-lines r))
                 (push r result)))
             (recipe-book-recipes book))
    result))

(defun ingredients-used (book)
  "The set of ingredient ids referenced by any recipe."
  (let ((seen (make-hash-table :test 'equal)))
    (maphash (lambda (id r)
               (declare (ignore id))
               (dolist (l (recipe-lines r))
                 (setf (gethash (recipe-line-ingredient-id l) seen) t)))
             (recipe-book-recipes book))
    (loop for k being the hash-keys of seen collect k)))
