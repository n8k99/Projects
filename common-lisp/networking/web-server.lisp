;;;; Small Web Server — HTTP server with routing and static file serving.

(defpackage #:web-server
  (:use #:cl)
  (:export #:make-web-server
           #:add-route
           #:serve-static
           #:handle-request
           #:process-raw
           #:response-200
           #:response-404
           #:response-json
           #:parse-http-request
           #:http-request-method
           #:http-request-path
           #:http-request-body
           #:http-request-headers
           #:http-request-query-params
           #:http-response-status-code
           #:http-response-body
           #:web-server-access-log))

(in-package #:web-server)

(defstruct http-request
  (method :get)
  (path "/" :type string)
  (headers nil :type list)
  (query-params nil :type list)
  (body "" :type string)
  (remote-addr "" :type string))

(defstruct http-response
  (status-code 200 :type integer)
  (status-text "OK" :type string)
  (headers nil :type list)
  (body "" :type string))

(defun response-200 (body &optional (content-type "text/html; charset=utf-8"))
  (make-http-response :status-code 200 :status-text "OK" :body body
                      :headers (list (cons "Content-Type" content-type))))

(defun response-404 (&optional (path ""))
  (make-http-response :status-code 404 :status-text "Not Found"
                      :body (format nil "404 Not Found: ~A" path)))

(defun response-json (data)
  (make-http-response :status-code 200 :status-text "OK" :body data
                      :headers (list (cons "Content-Type" "application/json"))))

(defun parse-http-request (raw)
  "Parse a raw HTTP request string."
  (let* ((lines (uiop:split-string raw :separator '(#\Newline)))
         (lines (mapcar (lambda (l) (string-right-trim '(#\Return) l)) lines))
         (first-line (or (first lines) "GET / HTTP/1.1"))
         (parts (uiop:split-string first-line :separator '(#\Space)))
         (method-str (or (first parts) "GET"))
         (full-path (or (second parts) "/"))
         (qmark (position #\? full-path))
         (path (if qmark (subseq full-path 0 qmark) full-path))
         (query-params (when qmark
                         (let ((qs (subseq full-path (1+ qmark))))
                           (loop for pair in (uiop:split-string qs :separator '(#\&))
                                 for eq = (position #\= pair)
                                 when eq collect (cons (subseq pair 0 eq) (subseq pair (1+ eq)))))))
         (headers nil)
         (body-start nil))
    (loop for i from 1 below (length lines)
          for line = (nth i lines)
          do (if (string= line "")
                 (progn (setf body-start (1+ i)) (return))
                 (let ((colon (position #\: line)))
                   (when colon
                     (push (cons (string-downcase (string-trim '(#\Space) (subseq line 0 colon)))
                                 (string-trim '(#\Space) (subseq line (1+ colon))))
                           headers)))))
    (make-http-request
     :method (intern (string-upcase method-str) :keyword)
     :path path :headers (nreverse headers)
     :query-params query-params
     :body (if (and body-start (< body-start (length lines)))
               (format nil "~{~A~^~%~}" (subseq lines body-start))
               ""))))

(defun serialize-response (resp)
  (let ((headers (copy-list (http-response-headers resp))))
    (unless (assoc "Content-Length" headers :test #'string-equal)
      (push (cons "Content-Length" (format nil "~D" (length (http-response-body resp)))) headers))
    (unless (assoc "Content-Type" headers :test #'string-equal)
      (push (cons "Content-Type" "text/html; charset=utf-8") headers))
    (format nil "HTTP/1.1 ~D ~A~c~c~{~A: ~A~c~c~}~c~c~A"
            (http-response-status-code resp) (http-response-status-text resp) #\Return #\Newline
            (loop for (k . v) in headers append (list k v #\Return #\Newline))
            #\Return #\Newline (http-response-body resp))))

;;; --- Server ---

(defstruct web-server
  (name "rosetta-server" :type string)
  (routes nil :type list)      ; ((method path handler) ...)
  (static-files (make-hash-table :test #'equal) :type hash-table)
  (access-log nil :type list))

(defun add-route (server method path handler)
  (push (list method path handler) (web-server-routes server)))

(defun serve-static (server path content &optional (content-type "text/html"))
  (setf (gethash path (web-server-static-files server)) (cons content content-type)))

(defun handle-request (server request)
  (let ((method (http-request-method request))
        (path (http-request-path request)))
    ;; Static files
    (when (eq method :get)
      (let ((sf (gethash path (web-server-static-files server))))
        (when sf
          (let ((resp (response-200 (car sf) (cdr sf))))
            (%log-access server method path (http-response-status-code resp))
            (return-from handle-request resp)))))
    ;; Routes
    (dolist (route (web-server-routes server))
      (when (and (eq (first route) method) (string= (second route) path))
        (let ((resp (funcall (third route) request)))
          (%log-access server method path (http-response-status-code resp))
          (return-from handle-request resp))))
    (let ((resp (response-404 path)))
      (%log-access server method path 404)
      resp)))

(defun process-raw (server raw)
  (let* ((request (parse-http-request raw))
         (response (handle-request server request)))
    (serialize-response response)))

(defun %log-access (server method path status)
  (push (list :method method :path path :status status
              :timestamp (get-universal-time))
        (web-server-access-log server)))
