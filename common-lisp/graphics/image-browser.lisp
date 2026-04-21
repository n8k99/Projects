;;;; Image Browser — 2D grid navigation + LRU cache + filter/sort query.

(defpackage #:image-browser
  (:use #:cl)
  (:export #:make-image-entry #:image-entry-id #:image-entry-name
           #:image-entry-size-bytes #:image-entry-tags
           #:make-filter #:apply-query
           #:make-grid #:grid-move #:grid-total #:grid-selected
           #:make-lru-cache #:lru-get #:lru-put #:lru-len #:lru-contains
           #:make-browser #:browser-set-entries #:browser-set-query
           #:browser-filtered-entries #:browser-selected-entry
           #:browser-move-cursor #:browser-navigate-into #:browser-navigate-up
           #:browser-current-path #:browser-get-or-load-thumbnail
           #:browser-grid))

(in-package #:image-browser)

(defstruct image-entry
  (id 0 :type integer)
  (name "" :type string)
  (path "" :type string)
  (width 0 :type integer)
  (height 0 :type integer)
  (size-bytes 0 :type integer)
  (tags nil :type list)
  (created-ms 0 :type integer))

;; Filter as plist.
(defun make-filter-name-contains (s) (list :kind :name-contains :s s))
(defun make-filter-tag-has (s) (list :kind :tag-has :s s))
(defun make-filter-size-over (n) (list :kind :size-over :n n))
(defun make-filter-size-under (n) (list :kind :size-under :n n))
(defun make-filter-width-at-least (n) (list :kind :width-at-least :n n))
(defun make-filter-created-after (n) (list :kind :created-after :n n))

(defun filter-matches-p (pred entry)
  (case (getf pred :kind)
    (:name-contains (search (getf pred :s) (image-entry-name entry)))
    (:tag-has (member (getf pred :s) (image-entry-tags entry) :test #'string=))
    (:size-over (> (image-entry-size-bytes entry) (getf pred :n)))
    (:size-under (< (image-entry-size-bytes entry) (getf pred :n)))
    (:width-at-least (>= (image-entry-width entry) (getf pred :n)))
    (:created-after (> (image-entry-created-ms entry) (getf pred :n)))))

;; Query as plist: (:filters (...) :sort-by :name :order :asc)
(defun make-query (&key filters (sort-by :name) (order :asc))
  (list :filters filters :sort-by sort-by :order order))

(defun sort-key-for (sort-by)
  (case sort-by
    (:name #'image-entry-name)
    (:size #'image-entry-size-bytes)
    (:created #'image-entry-created-ms)
    (:width #'image-entry-width)))

(defun cmp-for (sort-by order)
  (let ((key (sort-key-for sort-by)))
    (lambda (a b)
      (let ((av (funcall key a))
            (bv (funcall key b)))
        (let ((ord (cond
                     ((stringp av) (string< av bv))
                     (t (< av bv)))))
          (if (eq order :desc) (not ord) ord))))))

(defun apply-query (entries query)
  (let* ((filters (getf query :filters))
         (filtered (remove-if-not (lambda (e)
                                     (every (lambda (p) (filter-matches-p p e))
                                             filters))
                                   entries)))
    (sort (copy-list filtered)
          (cmp-for (getf query :sort-by) (getf query :order)))))

(defstruct grid
  (cols 0 :type integer)
  (total 0 :type integer)
  (selected 0 :type integer))

(defun grid-rows (g)
  (if (or (zerop (grid-cols g)) (zerop (grid-total g))) 0
      (ceiling (grid-total g) (grid-cols g))))

(defun grid-row-col (g)
  (multiple-value-bind (row col) (floor (grid-selected g) (grid-cols g))
    (values row col)))

(defun grid-move (g mv)
  (when (zerop (grid-total g)) (return-from grid-move))
  (multiple-value-bind (row col) (grid-row-col g)
    (case mv
      (:left (when (> (grid-selected g) 0) (decf (grid-selected g))))
      (:right (when (< (1+ (grid-selected g)) (grid-total g))
                (incf (grid-selected g))))
      (:up (when (> row 0)
              (setf (grid-selected g) (+ (* (1- row) (grid-cols g)) col))))
      (:down
       (let ((next (+ (* (1+ row) (grid-cols g)) col)))
         (cond
           ((< next (grid-total g)) (setf (grid-selected g) next))
           ((< (1+ row) (grid-rows g))
            (setf (grid-selected g) (1- (grid-total g)))))))
      (:home (setf (grid-selected g) 0))
      (:end (when (> (grid-total g) 0)
              (setf (grid-selected g) (1- (grid-total g))))))))

;; LRU cache: capacity + ordered alist (front = LRU, back = MRU)
(defstruct lru-cache
  (capacity 0 :type integer)
  (entries nil :type list)) ; alist: ((key . val) ...); head = LRU

(defun lru-contains (c k)
  (not (null (assoc k (lru-cache-entries c)))))

(defun lru-len (c) (length (lru-cache-entries c)))

(defun lru-get (c k)
  (let ((pair (assoc k (lru-cache-entries c))))
    (when pair
      (setf (lru-cache-entries c)
            (append (remove pair (lru-cache-entries c) :test #'eq)
                    (list pair)))
      (cdr pair))))

(defun lru-put (c k v)
  "Returns evicted key or NIL."
  (let ((existing (assoc k (lru-cache-entries c))))
    (cond
      (existing
       (setf (cdr existing) v)
       (setf (lru-cache-entries c)
             (append (remove existing (lru-cache-entries c) :test #'eq)
                     (list existing)))
       nil)
      ((>= (length (lru-cache-entries c)) (lru-cache-capacity c))
       (let* ((evicted-pair (first (lru-cache-entries c)))
              (evicted-key (car evicted-pair)))
         (setf (lru-cache-entries c)
               (append (rest (lru-cache-entries c))
                        (list (cons k v))))
         evicted-key))
      (t
       (setf (lru-cache-entries c)
             (append (lru-cache-entries c) (list (cons k v))))
       nil))))

(defstruct browser
  (grid-cols 3 :type integer)
  (breadcrumbs nil :type list)
  (all-entries nil :type list)
  (query nil :type list)
  (grid nil :type (or null grid))
  (thumbnail-cache nil :type (or null lru-cache)))

(defun make-new-browser (&key (grid-cols 3) (thumbnail-cache-capacity 10))
  (let ((br (make-browser :grid-cols grid-cols
                            :query (make-query))))
    (setf (browser-grid br) (make-grid :cols grid-cols))
    (setf (browser-thumbnail-cache br)
          (make-lru-cache :capacity thumbnail-cache-capacity))
    br))

(defun browser-set-entries (br entries)
  (setf (browser-all-entries br) entries)
  (let ((filtered (apply-query entries (browser-query br))))
    (setf (browser-grid br)
          (make-grid :cols (browser-grid-cols br)
                      :total (length filtered)))))

(defun browser-set-query (br query)
  (setf (browser-query br) query)
  (let ((filtered (apply-query (browser-all-entries br) query)))
    (setf (browser-grid br)
          (make-grid :cols (browser-grid-cols br)
                      :total (length filtered)))))

(defun browser-filtered-entries (br)
  (apply-query (browser-all-entries br) (browser-query br)))

(defun browser-selected-entry (br)
  (let ((filtered (browser-filtered-entries br))
        (idx (grid-selected (browser-grid br))))
    (when (and (>= idx 0) (< idx (length filtered)))
      (nth idx filtered))))

(defun browser-navigate-into (br subdir)
  (setf (browser-breadcrumbs br)
        (append (browser-breadcrumbs br) (list subdir))))

(defun browser-navigate-up (br)
  (if (browser-breadcrumbs br)
      (progn
        (setf (browser-breadcrumbs br)
              (butlast (browser-breadcrumbs br)))
        t)
      nil))

(defun browser-current-path (br)
  (concatenate 'string "/"
                (format nil "~{~A~^/~}" (browser-breadcrumbs br))))

(defun browser-move-cursor (br mv)
  (grid-move (browser-grid br) mv))

(defun browser-get-or-load-thumbnail (br id producer)
  "Returns (values v miss-p)."
  (let ((cached (lru-get (browser-thumbnail-cache br) id)))
    (if cached
        (values cached nil)
        (let ((v (funcall producer)))
          (lru-put (browser-thumbnail-cache br) id v)
          (values v t)))))

(defun demo ()
  (let ((br (make-new-browser :grid-cols 3 :thumbnail-cache-capacity 3))
        (entries (list
                   (make-image-entry :id 1 :name "alpha.jpg" :path "/img/alpha.jpg"
                                      :width 1920 :height 1080 :size-bytes 200000
                                      :tags '("nature") :created-ms 1000)
                   (make-image-entry :id 2 :name "beta.jpg" :path "/img/beta.jpg"
                                      :width 800 :height 600 :size-bytes 50000
                                      :tags '("abstract") :created-ms 2000)
                   (make-image-entry :id 3 :name "gamma.png" :path "/img/gamma.png"
                                      :width 1024 :height 768 :size-bytes 150000
                                      :tags '("nature") :created-ms 3000)
                   (make-image-entry :id 4 :name "delta.jpg" :path "/img/delta.jpg"
                                      :width 2560 :height 1440 :size-bytes 500000
                                      :tags '("nature") :created-ms 4000)
                   (make-image-entry :id 5 :name "epsilon.png" :path "/img/epsilon.png"
                                      :width 1920 :height 1080 :size-bytes 100000
                                      :tags '("abstract") :created-ms 5000))))
    (browser-set-entries br entries)
    (format t "all: ~D; grid total: ~D~%"
            (length (browser-filtered-entries br))
            (grid-total (browser-grid br)))

    (browser-set-query br (make-query
                             :filters (list (make-filter-name-contains ".jpg")
                                             (make-filter-tag-has "nature"))
                             :sort-by :size
                             :order :asc))
    (format t "nature jpg by size asc: (~{~A~^ ~})~%"
            (mapcar #'image-entry-name (browser-filtered-entries br)))
    (format t "grid total: ~D~%" (grid-total (browser-grid br)))

    (format t "selected: ~A~%" (image-entry-name (browser-selected-entry br)))
    (browser-move-cursor br :right)
    (format t "after right: ~A~%" (image-entry-name (browser-selected-entry br)))
    (browser-move-cursor br :right)
    (format t "after right at end: ~A~%" (image-entry-name (browser-selected-entry br)))

    (browser-navigate-into br "photos")
    (browser-navigate-into br "2026")
    (format t "path: ~A~%" (browser-current-path br))
    (browser-navigate-up br)
    (format t "after up: ~A~%" (browser-current-path br))

    (let ((calls 0))
      (let ((producer (lambda () (incf calls) #(0 1 2))))
        (multiple-value-bind (_ miss1) (browser-get-or-load-thumbnail br 1 producer)
          (declare (ignore _))
          (multiple-value-bind (_ miss2) (browser-get-or-load-thumbnail br 1 producer)
            (declare (ignore _))
            (multiple-value-bind (_ miss3) (browser-get-or-load-thumbnail br 2 producer)
              (declare (ignore _))
              (format t "cache miss: ~A hit: ~A miss2: ~A~%" miss1 (not miss2) miss3)))))
      (format t "producer calls: ~D~%" calls))

    (let ((lru (make-lru-cache :capacity 2)))
      (lru-put lru 1 'a)
      (lru-put lru 2 'b)
      (lru-get lru 1)
      (let ((evicted (lru-put lru 3 'c)))
        (format t "LRU evicted key: ~A~%" evicted)))))
