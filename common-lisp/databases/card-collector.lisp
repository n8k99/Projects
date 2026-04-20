;;;; Card Collector — multiset inventory with valuation and trade balancing.

(defpackage #:card-collector
  (:use #:cl)
  (:export #:make-card #:card-id #:card-value-cents
           #:make-catalogue #:catalogue-add #:catalogue-get #:catalogue-all-ids
           #:make-collection #:collection-add #:collection-remove
           #:collection-total-cards #:collection-unique-cards
           #:collection-total-value-cents #:collection-missing
           #:make-trade-bundle #:trade-bundle-add #:trade-bundle-value
           #:evaluate-trade))

(in-package #:card-collector)

(defparameter *rarity-mult*
  '((:common . 1.0) (:uncommon . 2.0) (:rare . 8.0) (:mythic . 25.0)))

(defparameter *condition-mult*
  '((:mint . 1.0) (:near-mint . 0.8) (:played . 0.4) (:damaged . 0.15)))

(defun rarity-mult (r) (cdr (assoc r *rarity-mult*)))
(defun condition-mult (c) (cdr (assoc c *condition-mult*)))

(defstruct card
  (set-code "" :type string)
  (number 0 :type integer)
  (name "" :type string)
  (rarity :common :type keyword)
  (base-price-cents 0 :type integer))

(defun card-id (c)
  (format nil "~A-~3,'0D" (card-set-code c) (card-number c)))

(defun card-value-cents (c condition)
  (round (* (card-base-price-cents c)
             (rarity-mult (card-rarity c))
             (condition-mult condition))))

(defstruct catalogue
  (cards (make-hash-table :test 'equal) :type hash-table))

(defun catalogue-add (cat card)
  (setf (gethash (card-id card) (catalogue-cards cat)) card))

(defun catalogue-get (cat id)
  (gethash id (catalogue-cards cat)))

(defun catalogue-all-ids (cat)
  (sort (loop for k being the hash-keys of (catalogue-cards cat) collect k)
        #'string<))

(defstruct collection
  (counts (make-hash-table :test 'equal) :type hash-table))

(defun inv-key (card-id condition)
  (format nil "~A|~A" card-id condition))

(defun collection-add (c card-id condition qty)
  (let ((k (inv-key card-id condition)))
    (setf (gethash k (collection-counts c))
          (+ (gethash k (collection-counts c) 0) qty))))

(defun collection-remove (c card-id condition qty)
  (let* ((k (inv-key card-id condition))
         (current (gethash k (collection-counts c) 0)))
    (when (< current qty)
      (error "not enough ~A — have ~D, need ~D" card-id current qty))
    (if (= current qty)
        (remhash k (collection-counts c))
        (setf (gethash k (collection-counts c)) (- current qty)))))

(defun collection-total-cards (c)
  (let ((sum 0))
    (maphash (lambda (k v) (declare (ignore k)) (incf sum v))
             (collection-counts c))
    sum))

(defun collection-unique-cards (c)
  (let ((ids (make-hash-table :test 'equal)))
    (maphash (lambda (k v) (declare (ignore v))
               (setf (gethash (subseq k 0 (position #\| k)) ids) t))
             (collection-counts c))
    (hash-table-count ids)))

(defun parse-inv-key (k)
  (let ((pipe (position #\| k)))
    (values (subseq k 0 pipe)
            (intern (string-upcase (subseq k (1+ pipe))) :keyword))))

(defun collection-total-value-cents (c cat)
  (let ((total 0))
    (maphash (lambda (k n)
               (multiple-value-bind (card-id condition) (parse-inv-key k)
                 (let ((card (catalogue-get cat card-id)))
                   (when card
                     (incf total (* (card-value-cents card condition) n))))))
             (collection-counts c))
    total))

(defun collection-missing (c cat)
  (let ((owned (make-hash-table :test 'equal)))
    (maphash (lambda (k v) (declare (ignore v))
               (setf (gethash (subseq k 0 (position #\| k)) owned) t))
             (collection-counts c))
    (remove-if (lambda (id) (gethash id owned))
               (catalogue-all-ids cat))))

(defstruct trade-bundle
  (cards nil :type list))

(defun trade-bundle-add (b card-id condition qty)
  (setf (trade-bundle-cards b)
        (append (trade-bundle-cards b) (list (list card-id condition qty)))))

(defun trade-bundle-value (b cat)
  (let ((total 0))
    (dolist (entry (trade-bundle-cards b))
      (let ((card (catalogue-get cat (first entry))))
        (when card
          (incf total (* (card-value-cents card (second entry)) (third entry))))))
    total))

(defstruct trade-evaluation
  (verdict :fair :type keyword)
  (diff-cents 0 :type integer))

(defun evaluate-trade (offered asked cat tolerance-cents)
  (let* ((o (trade-bundle-value offered cat))
         (a (trade-bundle-value asked cat))
         (diff (abs (- o a))))
    (cond
      ((<= diff tolerance-cents) (make-trade-evaluation :verdict :fair :diff-cents 0))
      ((> o a) (make-trade-evaluation :verdict :offered-more :diff-cents (- o a)))
      (t (make-trade-evaluation :verdict :asked-more :diff-cents (- a o))))))

(defun demo ()
  (let ((cat (make-catalogue)))
    (catalogue-add cat (make-card :set-code "DPN" :number 1 :name "Common One"
                                    :rarity :common :base-price-cents 25))
    (catalogue-add cat (make-card :set-code "DPN" :number 2 :name "Uncommon Two"
                                    :rarity :uncommon :base-price-cents 25))
    (catalogue-add cat (make-card :set-code "DPN" :number 3 :name "Rare Three"
                                    :rarity :rare :base-price-cents 25))
    (catalogue-add cat (make-card :set-code "DPN" :number 4 :name "Mythic Four"
                                    :rarity :mythic :base-price-cents 100))

    (let ((c (make-collection)))
      (collection-add c "DPN-001" :mint 4)
      (collection-add c "DPN-003" :near-mint 1)
      (collection-add c "DPN-004" :mint 1)
      (format t "total cards:   ~D~%" (collection-total-cards c))
      (format t "unique cards:  ~D~%" (collection-unique-cards c))
      (format t "total value:   ~D cents~%" (collection-total-value-cents c cat))
      (format t "missing cards: ~A~%" (collection-missing c cat)))

    (format t "~%=== trade evaluation ===~%")
    (let ((offered (make-trade-bundle))
          (asked (make-trade-bundle)))
      (trade-bundle-add offered "DPN-004" :mint 1)
      (trade-bundle-add asked "DPN-003" :mint 12)
      (trade-bundle-add asked "DPN-001" :mint 4)
      (format t "  offered value: ~D~%" (trade-bundle-value offered cat))
      (format t "  asked value:   ~D~%" (trade-bundle-value asked cat))
      (let ((result (evaluate-trade offered asked cat 100)))
        (format t "  verdict:       ~A (diff=~D)~%"
                (trade-evaluation-verdict result)
                (trade-evaluation-diff-cents result))))))
