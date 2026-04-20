;;;; Quick Launcher — ranked launchable items with frecency and file round-trip.
;;;;
;;;; Persistent launcher state (what you've run, count, when) round-trips via
;;;; to-text/from-text. Matching has two layers — text match
;;;; (exact > prefix > substring > keyword) and frecency
;;;; (log(1+count)/(1+age-hours)). Text match dominates; frecency breaks ties.

(defpackage #:quick-launcher
  (:use #:cl)
  (:export #:make-launcher #:add-entry #:get-entry #:query #:record-launch
           #:to-text #:from-text #:launcher-size
           #:make-entry #:entry-id #:entry-name #:entry-action
           #:entry-keywords #:entry-count #:entry-last
           #:match-entry-id #:match-score))

(in-package #:quick-launcher)

(defstruct entry
  (id 0 :type integer)
  (name "" :type string)
  (action "" :type string)
  (keywords nil :type list)
  (count 0 :type integer)
  (last 0 :type integer))

(defstruct launcher
  (entries (make-hash-table) :type hash-table)
  (frecency-weight 0.1 :type single-float))

(defstruct match
  (entry-id 0 :type integer)
  (score 0.0 :type single-float))

(defun add-entry (launcher entry)
  (setf (gethash (entry-id entry) (launcher-entries launcher)) entry))

(defun get-entry (launcher id)
  (gethash id (launcher-entries launcher)))

(defun launcher-size (launcher)
  (hash-table-count (launcher-entries launcher)))

(defun lower (s) (string-downcase s))

(defun starts-with (s prefix)
  (and (>= (length s) (length prefix))
       (string= s prefix :end1 (length prefix))))

(defun contains (s needle)
  (search needle s))

(defun text-match (needle entry)
  (if (zerop (length needle))
      0.5
      (let ((name-lc (lower (entry-name entry))))
        (cond
          ((string= name-lc needle) 1.0)
          ((starts-with name-lc needle) 0.8)
          ((contains name-lc needle) 0.6)
          (t
           (loop for k in (entry-keywords entry)
                 for k-lc = (lower k)
                 when (string= k-lc needle) return 0.5
                 when (contains k-lc needle) return 0.4
                 finally (return 0.0)))))))

(defun frecency (entry now-ms)
  (if (zerop (entry-count entry))
      0.0
      (let* ((age-ms (max 0 (- now-ms (entry-last entry))))
             (age-hours (/ age-ms 3600000.0)))
        (/ (log (+ 1.0 (entry-count entry))) (+ 1.0 age-hours)))))

(defun query (launcher text now-ms)
  (let ((needle (string-downcase (string-trim " " text)))
        (results nil))
    (maphash (lambda (id entry)
               (declare (ignore id))
               (let ((tm (text-match needle entry)))
                 (when (> tm 0.0)
                   (let ((score (+ tm (* (launcher-frecency-weight launcher)
                                          (frecency entry now-ms)))))
                     (push (make-match :entry-id (entry-id entry)
                                       :score (coerce score 'single-float))
                           results)))))
             (launcher-entries launcher))
    (sort results
          (lambda (a b)
            (cond
              ((> (match-score a) (match-score b)) t)
              ((< (match-score a) (match-score b)) nil)
              (t (< (match-entry-id a) (match-entry-id b))))))))

(defun record-launch (launcher id now-ms)
  (let ((e (gethash id (launcher-entries launcher))))
    (unless e (error "unknown entry ~A" id))
    (incf (entry-count e))
    (setf (entry-last e) now-ms)
    t))

(defun to-text (launcher)
  (with-output-to-string (s)
    (format s "LAUNCHER~%")
    (let ((ids (sort (loop for id being the hash-keys of (launcher-entries launcher)
                           collect id)
                     #'<)))
      (dolist (id ids)
        (let ((e (gethash id (launcher-entries launcher))))
          (format s "---~%")
          (format s "id: ~D~%" (entry-id e))
          (format s "name: ~A~%" (entry-name e))
          (format s "action: ~A~%" (entry-action e))
          (dolist (k (entry-keywords e))
            (format s "keyword: ~A~%" k))
          (format s "count: ~D~%" (entry-count e))
          (format s "last: ~D~%" (entry-last e)))))))

(defun split-lines (text)
  (loop for start = 0 then (1+ end)
        for end = (position #\Newline text :start start)
        collect (subseq text start end)
        while end))

(defun from-text (text)
  (let* ((all-lines (split-lines text))
         (lines (if (and all-lines (string= (car (last all-lines)) ""))
                    (butlast all-lines)
                    all-lines)))
    (unless (and lines (string= (first lines) "LAUNCHER"))
      (error "missing LAUNCHER header"))
    (let ((launcher (make-launcher))
          (rest-lines (rest lines)))
      (loop while rest-lines do
            (unless (string= (first rest-lines) "---")
              (error "expected --- separator"))
            (setf rest-lines (rest rest-lines))
            (multiple-value-bind (entry remaining)
                (parse-entry rest-lines)
              (add-entry launcher entry)
              (setf rest-lines remaining)))
      launcher)))

(defun parse-entry (lines)
  (let ((id nil) (name nil) (action nil)
        (keywords nil) (count 0) (last 0)
        (remaining lines))
    (loop while (and remaining (not (string= (first remaining) "---")))
          do (let* ((line (first remaining))
                    (colon (search ": " line)))
               (unless colon (error "no `: ` in ~A" line))
               (let ((key (subseq line 0 colon))
                     (val (subseq line (+ colon 2))))
                 (cond
                   ((string= key "id") (setf id (parse-integer val)))
                   ((string= key "name") (setf name val))
                   ((string= key "action") (setf action val))
                   ((string= key "keyword") (push val keywords))
                   ((string= key "count") (setf count (parse-integer val)))
                   ((string= key "last") (setf last (parse-integer val))))))
             (setf remaining (rest remaining)))
    (unless (and id name action) (error "missing id/name/action"))
    (values (make-entry :id id :name name :action action
                         :keywords (nreverse keywords)
                         :count count :last last)
            remaining)))

(defun demo ()
  (let ((l (make-launcher)))
    (add-entry l (make-entry :id 1 :name "Firefox" :action "firefox"
                              :keywords (list "browser" "web")))
    (add-entry l (make-entry :id 2 :name "Files" :action "nautilus"
                              :keywords (list "folder")))
    (add-entry l (make-entry :id 3 :name "Terminal" :action "kitty"
                              :keywords (list "shell" "cli")))
    (add-entry l (make-entry :id 4 :name "File Manager" :action "thunar"
                              :keywords (list "folder")))
    (format t "=== Query 'file' (fresh) ===~%")
    (dolist (m (query l "file" 0))
      (format t "  ~A score=~,3F~%"
              (entry-name (get-entry l (match-entry-id m)))
              (match-score m)))
    (format t "~%=== After 5x launch id=2 at t=1M, query 'file' at t=2M ===~%")
    (dotimes (i 5) (record-launch l 2 1000000))
    (dolist (m (query l "file" 2000000))
      (format t "  ~A score=~,3F~%"
              (entry-name (get-entry l (match-entry-id m)))
              (match-score m)))
    (format t "~%=== Round-trip check ===~%")
    (let* ((text (to-text l))
           (parsed (from-text text)))
      (format t "  original: ~D, parsed: ~D~%" (launcher-size l) (launcher-size parsed))
      (format t "  id=2 count restored: ~D~%" (entry-count (get-entry parsed 2)))
      (format t "  id=2 last restored: ~D~%" (entry-last (get-entry parsed 2))))))
