;;;; News Ticker and Game Scores — real-time prioritized message stream.

(defpackage #:news-ticker
  (:use #:cl)
  (:export #:make-ticker
           #:post-item
           #:breaking-news
           #:get-current
           #:get-by-category
           #:expire-old
           #:display-ticker
           #:ticker-item-text
           #:ticker-item-category
           #:ticker-item-priority
           #:ticker-item-timestamp
           #:ticker-item-expires-at))

(in-package #:news-ticker)

(defparameter *valid-categories* '(:news :sports :finance :alert))

(defstruct ticker-item
  (text "" :type string)
  (category :news :type keyword)
  (priority 1 :type (integer 1 5))
  (timestamp (get-universal-time) :type integer)
  (expires-at nil :type (or null integer)))

(defstruct news-ticker
  (items nil :type list))

(defun make-ticker ()
  "Create a new empty news ticker."
  (make-news-ticker))

(defun post-item (ticker text category priority &optional ttl-seconds)
  "Post an item to the ticker. TTL-SECONDS sets optional expiry."
  (assert (member category *valid-categories*)
          (category)
          "Category must be one of ~A, got ~A" *valid-categories* category)
  (assert (<= 1 priority 5)
          (priority)
          "Priority must be 1-5, got ~A" priority)
  (let ((item (make-ticker-item
               :text text
               :category category
               :priority priority
               :timestamp (get-universal-time)
               :expires-at (when ttl-seconds
                             (+ (get-universal-time) ttl-seconds)))))
    (push item (news-ticker-items ticker))
    item))

(defun breaking-news (ticker text)
  "Post a breaking news item with maximum priority."
  (post-item ticker text :alert 5))

(defun item-expired-p (item)
  "Return T if item has expired."
  (let ((expires (ticker-item-expires-at item)))
    (and expires (>= (get-universal-time) expires))))

(defun get-current (ticker)
  "Return active non-expired items sorted by priority (desc) then time (asc)."
  (let ((active (remove-if #'item-expired-p (news-ticker-items ticker))))
    (sort (copy-list active)
          (lambda (a b)
            (if (/= (ticker-item-priority a) (ticker-item-priority b))
                (> (ticker-item-priority a) (ticker-item-priority b))
                (< (ticker-item-timestamp a) (ticker-item-timestamp b)))))))

(defun get-by-category (ticker category)
  "Return current items filtered by category."
  (remove-if-not (lambda (item)
                   (eq (ticker-item-category item) category))
                 (get-current ticker)))

(defun expire-old (ticker)
  "Remove expired items. Return count of items removed."
  (let ((before (length (news-ticker-items ticker))))
    (setf (news-ticker-items ticker)
          (remove-if #'item-expired-p (news-ticker-items ticker)))
    (- before (length (news-ticker-items ticker)))))

(defun display-ticker (ticker)
  "Format the ticker as a scrolling display string."
  (let ((current (get-current ticker)))
    (if (null current)
        ">>> No items >>>"
        (format nil ">>> ~{~A~^ | ~} >>>"
                (mapcar #'ticker-item-text current)))))

;;; Demo

(defun demo ()
  (let ((ticker (make-ticker)))
    (post-item ticker "Markets rally on strong earnings" :finance 3)
    (post-item ticker "Lakers defeat Celtics 112-108" :sports 2)
    (post-item ticker "New climate accord signed by 40 nations" :news 4)
    (post-item ticker "Flash sale: tech stocks surge 5%" :finance 2 1)

    (format t "=== Full Ticker ===~%")
    (format t "~A~%" (display-ticker ticker))

    (format t "~%=== Sports Only ===~%")
    (dolist (item (get-by-category ticker :sports))
      (format t "  [~A] ~A~%" (ticker-item-priority item) (ticker-item-text item)))

    (format t "~%=== Finance Only ===~%")
    (dolist (item (get-by-category ticker :finance))
      (format t "  [~A] ~A~%" (ticker-item-priority item) (ticker-item-text item)))

    (sleep 2)

    (let ((removed (expire-old ticker)))
      (format t "~%=== After expiry (~A removed) ===~%" removed)
      (format t "~A~%" (display-ticker ticker)))

    (breaking-news ticker "EARTHQUAKE: 7.2 magnitude off Pacific coast")
    (format t "~%=== BREAKING NEWS ===~%")
    (format t "~A~%" (display-ticker ticker))))
