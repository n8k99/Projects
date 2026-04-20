;;;; Code Snippet Manager — templated code fragments with tab-stop expansion.

(defpackage #:snippet-manager
  (:use #:cl)
  (:export #:make-snippet #:expand-snippet
           #:make-snippet-library #:library-add #:library-find-by-trigger
           #:library-find-by-language #:library-find-by-tag
           #:make-tab-stop #:tab-stop-number #:tab-stop-offset #:tab-stop-length
           #:expansion-text #:expansion-stops))

(in-package #:snippet-manager)

(defstruct tab-stop
  (number 0 :type integer)
  (offset 0 :type integer)
  (length 0 :type integer))

(defstruct expansion
  (text "" :type string)
  (stops nil :type list))

(defstruct snippet
  (trigger "" :type string)
  (language "" :type string)
  (tags nil :type list)
  (body "" :type string)
  (description "" :type string))

(defstruct snippet-library
  (snippets nil :type list))

(defun library-add (lib snippet)
  (setf (snippet-library-snippets lib)
        (append (snippet-library-snippets lib) (list snippet))))

(defun library-find-by-trigger (lib trigger language)
  (find-if (lambda (s)
             (and (string= (snippet-trigger s) trigger)
                  (string= (snippet-language s) language)))
           (snippet-library-snippets lib)))

(defun library-find-by-language (lib language)
  (sort (remove-if-not (lambda (s) (string= (snippet-language s) language))
                        (snippet-library-snippets lib))
        (lambda (a b) (string< (snippet-trigger a) (snippet-trigger b)))))

(defun library-find-by-tag (lib tag)
  (sort (remove-if-not (lambda (s) (member tag (snippet-tags s) :test #'string=))
                        (snippet-library-snippets lib))
        (lambda (a b) (string< (snippet-trigger a) (snippet-trigger b)))))

(defun find-matching (body open)
  (let ((depth 1)
        (i (1+ open))
        (n (length body)))
    (loop while (< i n)
          for c = (char body i)
          do (cond ((char= c #\{) (incf depth))
                   ((char= c #\}) (decf depth)
                                  (when (zerop depth) (return-from find-matching i))))
             (incf i))
    nil))

(defun parse-number (str default)
  (let ((n (ignore-errors (parse-integer str))))
    (or n default)))

(defun expand-snippet (snippet)
  (let* ((body (snippet-body snippet))
         (n (length body))
         (out (make-array 0 :element-type 'character :adjustable t :fill-pointer 0))
         (stops nil)
         (i 0))
    (loop while (< i n) do
          (let ((c (char body i)))
            (cond
              ((and (char= c #\$) (< (1+ i) n) (char= (char body (1+ i)) #\{))
               (let ((close (find-matching body (1+ i))))
                 (if close
                     (let* ((inside (subseq body (+ i 2) close))
                            (colon (position #\: inside))
                            (num-str (if colon (subseq inside 0 colon) inside))
                            (default (if colon (subseq inside (1+ colon)) ""))
                            (number (parse-number num-str 0)))
                       (push (make-tab-stop :number number
                                             :offset (length out)
                                             :length (length default)) stops)
                       (loop for ch across default do (vector-push-extend ch out))
                       (setf i (1+ close)))
                     (progn (vector-push-extend c out) (incf i)))))
              ((and (char= c #\$) (< (1+ i) n)
                    (digit-char-p (char body (1+ i))))
               (let ((j (1+ i)))
                 (loop while (and (< j n) (digit-char-p (char body j))) do (incf j))
                 (let ((number (parse-number (subseq body (1+ i) j) 0)))
                   (push (make-tab-stop :number number
                                         :offset (length out)
                                         :length 0) stops)
                   (setf i j))))
              (t (vector-push-extend c out)
                 (incf i)))))
    (make-expansion
     :text (coerce out 'string)
     :stops (sort (nreverse stops)
                  (lambda (a b)
                    (cond
                      ((and (= (tab-stop-number a) 0) (= (tab-stop-number b) 0)) nil)
                      ((= (tab-stop-number a) 0) nil)
                      ((= (tab-stop-number b) 0) t)
                      (t (< (tab-stop-number a) (tab-stop-number b)))))))))

(defun demo ()
  (let ((lib (make-snippet-library)))
    (library-add lib (make-snippet
                       :trigger "for" :language "python" :tags '("loop")
                       :body "for ${1:item} in ${2:collection}:
    ${3:pass}
$0"
                       :description "for loop"))
    (library-add lib (make-snippet
                       :trigger "fn" :language "rust" :tags '("function")
                       :body "fn ${1:name}(${2}) -> ${3:()} {
    $0
}"
                       :description "function"))
    (dolist (trig '(("for" "python") ("fn" "rust")))
      (let* ((s (library-find-by-trigger lib (first trig) (second trig)))
             (e (expand-snippet s)))
        (format t "=== ~A (~A) ===~%" (first trig) (second trig))
        (format t "expanded:~%~A~%" (expansion-text e))
        (format t "stops:~%")
        (dolist (st (expansion-stops e))
          (format t "  #~D @offset=~D length=~D~%"
                  (tab-stop-number st) (tab-stop-offset st) (tab-stop-length st)))
        (format t "~%")))))
