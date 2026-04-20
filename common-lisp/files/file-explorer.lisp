;;;; File Explorer — navigable virtual filesystem with path resolution.

(defpackage #:file-explorer
  (:use #:cl)
  (:export #:make-explorer #:explorer-cwd-string #:cd #:ls #:mkdir #:touch
           #:stat #:total-size #:resolve-path
           #:listing-name #:listing-kind #:listing-size #:listing-mtime))

(in-package #:file-explorer)

(defstruct node
  (kind :dir :type keyword)         ; :dir :file
  (size 0 :type integer)
  (mtime 0 :type integer)
  (children (make-hash-table :test 'equal) :type hash-table))

(defun make-dir-node ()
  (make-node :kind :dir))

(defun make-file-node (size mtime)
  (make-node :kind :file :size size :mtime mtime))

(defstruct explorer
  (root (make-dir-node) :type node)
  (cwd nil :type list))

(defstruct listing
  (name "" :type string)
  (kind :file :type keyword)
  (size 0 :type integer)
  (mtime 0 :type integer))

(defun explorer-cwd-string (e)
  (concatenate 'string "/" (format nil "~{~A~^/~}" (explorer-cwd e))))

(defun split-by (s ch)
  (loop for start = 0 then (1+ end)
        for end = (position ch s :start start)
        collect (subseq s start end)
        while end))

(defun resolve-path (e path)
  (when (string= path "") (return-from resolve-path nil))
  (let ((segs (if (and (> (length path) 0) (char= (char path 0) #\/))
                  '()
                  (copy-list (explorer-cwd e)))))
    (dolist (seg (split-by path #\/))
      (cond
        ((or (string= seg "") (string= seg ".")) nil)
        ((string= seg "..") (when segs (setf segs (butlast segs))))
        (t (setf segs (append segs (list seg))))))
    (or segs :root)))

(defun segments-of (resolved)
  (if (eq resolved :root) '() resolved))

(defun node-at (e segs)
  (let ((cur (explorer-root e)))
    (dolist (s segs cur)
      (unless (eq (node-kind cur) :dir) (return nil))
      (let ((next (gethash s (node-children cur))))
        (unless next (return nil))
        (setf cur next)))))

(defun cd (e path)
  (let* ((resolved (resolve-path e path))
         (segs (when resolved (segments-of resolved))))
    (unless resolved (error "not found: ~A" path))
    (let ((node (node-at e segs)))
      (unless node (error "not found: ~A" path))
      (unless (eq (node-kind node) :dir) (error "not a dir: ~A" path))
      (setf (explorer-cwd e) segs)
      t)))

(defun ls (e &optional (path ""))
  (let* ((resolved (if (string= path "")
                       (or (explorer-cwd e) :root)
                       (resolve-path e path)))
         (segs (when resolved (segments-of resolved))))
    (unless resolved (error "not found: ~A" path))
    (let ((node (node-at e segs)))
      (unless node (error "not found: ~A" path))
      (unless (eq (node-kind node) :dir) (error "not a dir: ~A" path))
      (let ((entries nil))
        (maphash (lambda (name child)
                   (push (make-listing
                          :name name
                          :kind (node-kind child)
                          :size (if (eq (node-kind child) :file)
                                    (node-size child) 0)
                          :mtime (if (eq (node-kind child) :file)
                                     (node-mtime child) 0))
                         entries))
                 (node-children node))
        ;; Dirs first, then alphabetical.
        (sort entries
              (lambda (a b)
                (cond
                  ((and (eq (listing-kind a) :dir) (eq (listing-kind b) :file)) t)
                  ((and (eq (listing-kind a) :file) (eq (listing-kind b) :dir)) nil)
                  (t (string< (listing-name a) (listing-name b))))))))))

(defun mkdir (e path)
  (let* ((resolved (resolve-path e path))
         (segs (when resolved (segments-of resolved))))
    (unless (and resolved segs) (error "already exists: ~A" path))
    (let* ((name (car (last segs)))
           (parent-segs (butlast segs))
           (parent (node-at e parent-segs)))
      (unless parent (error "not found: ~A" path))
      (unless (eq (node-kind parent) :dir) (error "not a dir: ~A" path))
      (when (gethash name (node-children parent))
        (error "already exists: ~A" path))
      (setf (gethash name (node-children parent)) (make-dir-node))
      t)))

(defun touch (e path size mtime)
  (let* ((resolved (resolve-path e path))
         (segs (when resolved (segments-of resolved))))
    (unless (and resolved segs) (error "already exists: ~A" path))
    (let* ((name (car (last segs)))
           (parent-segs (butlast segs))
           (parent (node-at e parent-segs)))
      (unless parent (error "not found: ~A" path))
      (unless (eq (node-kind parent) :dir) (error "not a dir: ~A" path))
      (setf (gethash name (node-children parent)) (make-file-node size mtime))
      t)))

(defun stat (e path)
  (let* ((resolved (resolve-path e path))
         (segs (when resolved (segments-of resolved))))
    (unless resolved (error "not found: ~A" path))
    (let ((node (node-at e segs)))
      (unless node (error "not found: ~A" path))
      (let ((name (if segs (car (last segs)) "/")))
        (if (eq (node-kind node) :dir)
            (make-listing :name name :kind :dir :size 0 :mtime 0)
            (make-listing :name name :kind :file
                          :size (node-size node)
                          :mtime (node-mtime node)))))))

(defun size-of (node)
  (if (eq (node-kind node) :file)
      (node-size node)
      (let ((sum 0))
        (maphash (lambda (k v) (declare (ignore k))
                   (incf sum (size-of v)))
                 (node-children node))
        sum)))

(defun total-size (e path)
  (let* ((resolved (resolve-path e path))
         (segs (when resolved (segments-of resolved))))
    (unless resolved (error "not found: ~A" path))
    (let ((node (node-at e segs)))
      (unless node (error "not found: ~A" path))
      (size-of node))))

(defun demo ()
  (let ((e (make-explorer)))
    (mkdir e "/home")
    (mkdir e "/home/nathan")
    (mkdir e "/home/nathan/docs")
    (touch e "/home/nathan/docs/a.txt" 100 1000)
    (touch e "/home/nathan/docs/b.txt" 200 2000)
    (touch e "/home/nathan/notes.md" 50 500)
    (mkdir e "/tmp")
    (format t "=== cwd: ~A ===~%" (explorer-cwd-string e))
    (cd e "/home/nathan")
    (format t "cd /home/nathan → cwd: ~A~%" (explorer-cwd-string e))
    (cd e "docs")
    (format t "cd docs → cwd: ~A~%" (explorer-cwd-string e))
    (cd e "../..")
    (format t "cd ../.. → cwd: ~A~%" (explorer-cwd-string e))
    (format t "~%=== ls /home/nathan ===~%")
    (dolist (l (ls e "/home/nathan"))
      (format t "  ~A ~A  size=~A  mtime=~A~%"
              (if (eq (listing-kind l) :dir) "d" "f")
              (listing-name l) (listing-size l) (listing-mtime l)))
    (format t "~%total size /home = ~A~%" (total-size e "/home"))
    (format t "resolve /a/./b/../c = ~A~%" (resolve-path e "/a/./b/../c"))))
