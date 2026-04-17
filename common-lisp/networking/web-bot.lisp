;;;; Web Bot — automated web interaction: crawling, scraping, and task automation.

(defpackage #:web-bot
  (:use #:cl)
  (:export #:make-web-bot
           #:bot-add-page
           #:bot-fetch
           #:bot-navigate
           #:bot-crawl
           #:bot-execute-script
           #:make-bot-script
           #:script-navigate
           #:script-assert
           #:script-extract
           #:script-fill
           #:page-title
           #:page-extract-links
           #:page-extract-text
           #:crawl-result-summary
           #:crawl-result-page-count
           #:bot-page-url
           #:bot-page-body
           #:bot-page-status-code
           #:bot-action-success))

(in-package #:web-bot)

(defstruct bot-page
  (url "" :type string)
  (status-code 200 :type integer)
  (body "" :type string))

(defun page-title (page)
  "Extract the <title> content from a page."
  (let* ((body (bot-page-body page))
         (start (search "<title>" body :test #'char-equal)))
    (when start
      (let ((end (search "</title>" body :start2 (+ start 7) :test #'char-equal)))
        (when end
          (string-trim '(#\Space #\Tab #\Newline #\Return)
                       (subseq body (+ start 7) end)))))))

(defun page-extract-links (page)
  "Extract all href values from a page."
  (let ((body (bot-page-body page))
        (links nil)
        (pos 0))
    (loop
      (let ((href-pos (search "href=\"" body :start2 pos)))
        (unless href-pos (return (nreverse links)))
        (let* ((start (+ href-pos 6))
               (end (position #\" body :start start)))
          (when end
            (push (subseq body start end) links))
          (setf pos (1+ start)))))))

(defun page-extract-text (page)
  "Strip HTML tags and return plain text."
  (let ((body (bot-page-body page))
        (result (make-string-output-stream))
        (in-tag nil))
    (loop for ch across body do
      (cond ((char= ch #\<) (setf in-tag t))
            ((char= ch #\>) (setf in-tag nil) (write-char #\Space result))
            ((not in-tag) (write-char ch result))))
    (string-trim '(#\Space) (get-output-stream-string result))))

;;; --- Crawl ---

(defstruct crawl-result
  (pages (make-hash-table :test #'equal) :type hash-table)
  (errors (make-hash-table :test #'equal) :type hash-table)
  (link-graph (make-hash-table :test #'equal) :type hash-table))

(defun crawl-result-page-count (r) (hash-table-count (crawl-result-pages r)))
(defun crawl-result-error-count (r) (hash-table-count (crawl-result-errors r)))

(defun crawl-result-summary (r)
  (let ((all-links (make-hash-table :test #'equal)))
    (maphash (lambda (url targets)
               (declare (ignore url))
               (dolist (t targets) (setf (gethash t all-links) t)))
             (crawl-result-link-graph r))
    (format nil "Crawl: ~D pages, ~D errors, ~D unique links"
            (crawl-result-page-count r) (crawl-result-error-count r)
            (hash-table-count all-links))))

;;; --- Bot Actions ---

(defstruct bot-action
  (action-type "" :type string)
  (target "" :type string)
  (value "" :type string)
  (result "" :type string)
  (success t)
  (error "" :type string))

(defstruct bot-script
  (name "" :type string)
  (actions nil :type list))

(defun script-navigate (script url)
  (push (make-bot-action :action-type "navigate" :target url) (bot-script-actions script))
  script)
(defun script-assert (script condition)
  (push (make-bot-action :action-type "assert" :target condition) (bot-script-actions script))
  script)
(defun script-extract (script pattern &optional var-name)
  (push (make-bot-action :action-type "extract" :target pattern :value (or var-name ""))
        (bot-script-actions script))
  script)
(defun script-fill (script field value)
  (push (make-bot-action :action-type "fill" :target field :value value)
        (bot-script-actions script))
  script)

;;; --- Web Bot ---

(defstruct web-bot
  (page-store (make-hash-table :test #'equal) :type hash-table)
  (visited (make-hash-table :test #'equal) :type hash-table)
  (history nil :type list)
  (current-page nil)
  (form-data (make-hash-table :test #'equal) :type hash-table))

(defun bot-add-page (bot url body &optional (status 200))
  (setf (gethash url (web-bot-page-store bot))
        (make-bot-page :url url :body body :status-code status)))

(defun bot-fetch (bot url)
  (setf (gethash url (web-bot-visited bot)) t)
  (push url (web-bot-history bot))
  (or (gethash url (web-bot-page-store bot))
      (make-bot-page :url url :status-code 404 :body "Not Found")))

(defun bot-navigate (bot url)
  (let ((page (bot-fetch bot url)))
    (setf (web-bot-current-page bot) page)
    page))

(defun bot-crawl (bot start-url &optional (max-pages 100))
  (let ((result (make-crawl-result))
        (queue (list start-url))
        (domain (let ((parts (uiop:split-string start-url :separator '(#\/))))
                  (when (>= (length parts) 3) (third parts)))))
    (loop while (and queue (< (crawl-result-page-count result) max-pages))
          for url = (pop queue)
          unless (or (gethash url (crawl-result-pages result))
                     (gethash url (crawl-result-errors result)))
          do (let ((page (bot-fetch bot url)))
               (if (/= (bot-page-status-code page) 200)
                   (setf (gethash url (crawl-result-errors result))
                         (format nil "HTTP ~D" (bot-page-status-code page)))
                   (let ((links (page-extract-links page)))
                     (setf (gethash url (crawl-result-pages result)) page)
                     (setf (gethash url (crawl-result-link-graph result)) links)
                     (dolist (link links)
                       (unless (or (gethash link (crawl-result-pages result))
                                   (gethash link (crawl-result-errors result)))
                         (when (or (null domain)
                                   (let ((parts (uiop:split-string link :separator '(#\/))))
                                     (and (>= (length parts) 3)
                                          (string= (third parts) domain))))
                           (push link queue))))))))
    result))
