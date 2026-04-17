;;;; P2P File Sharing App — decentralized file exchange between peers.

(defpackage #:p2p-file-sharing
  (:use #:cl)
  (:export #:make-peer-node
           #:add-file
           #:remove-file
           #:get-chunk
           #:discover-peer
           #:remove-peer
           #:prune-stale-peers
           #:heartbeat
           #:get-catalog
           #:receive-catalog
           #:search-network
           #:request-file
           #:receive-chunk
           #:get-missing-chunks
           #:simulate-transfer
           #:make-file-chunk
           #:file-chunk-verify
           #:peer-node-peer-id
           #:peer-node-local-files
           #:transfer-request-status
           #:transfer-request-progress))

(in-package #:p2p-file-sharing)

;;; --- Simple SHA-256 hash (hex string of sxhash for portability) ---
;;; A real implementation would use ironclad or sb-md5. For the Rosetta Stone,
;;; we use a content fingerprint via sxhash.

(defun %content-hash (data)
  "Return a hex string fingerprint for DATA (a string or byte vector)."
  (format nil "~16,'0X" (sxhash data)))

;;; --- Data Structures ---

(defstruct file-chunk
  (index 0 :type integer)
  (data "" :type string)
  (sha256 "" :type string))

(defun make-file-chunk-from-data (index data)
  "Create a file chunk with computed hash."
  (make-file-chunk :index index :data data :sha256 (%content-hash data)))

(defun file-chunk-verify (chunk)
  "Verify a chunk's data matches its hash."
  (string= (%content-hash (file-chunk-data chunk))
            (file-chunk-sha256 chunk)))

(defstruct shared-file
  (name "" :type string)
  (size 0 :type integer)
  (sha256 "" :type string)
  (chunk-size 64 :type integer)
  (chunk-count 0 :type integer))

(defun %make-shared-file (name size sha256 chunk-size)
  (make-shared-file :name name :size size :sha256 sha256
                    :chunk-size chunk-size
                    :chunk-count (if (zerop size) 0
                                     (ceiling size chunk-size))))

(defstruct peer-info
  (peer-id "" :type string)
  (address "" :type string)
  (port 0 :type integer)
  (last-seen (get-universal-time) :type integer)
  (shared-files nil :type list))

(defun peer-info-stale-p (peer &optional (timeout 300))
  (> (- (get-universal-time) (peer-info-last-seen peer)) timeout))

(defstruct transfer-request
  (request-id "" :type string)
  (filename "" :type string)
  (requester-id "" :type string)
  (provider-id "" :type string)
  (status :active)   ; :pending :active :completed :failed
  (chunks-received 0 :type integer)
  (total-chunks 0 :type integer))

(defun transfer-request-progress (xfer)
  (if (zerop (transfer-request-total-chunks xfer)) 0.0
      (/ (float (transfer-request-chunks-received xfer))
         (float (transfer-request-total-chunks xfer)))))

(defstruct peer-node
  (peer-id "" :type string)
  (address "" :type string)
  (port 0 :type integer)
  (peers (make-hash-table :test #'equal) :type hash-table)
  (local-files (make-hash-table :test #'equal) :type hash-table)
  (shared-catalog (make-hash-table :test #'equal) :type hash-table)
  (transfers (make-hash-table :test #'equal) :type hash-table)
  (chunks-store (make-hash-table :test #'equal) :type hash-table)
  (next-request-id 0 :type integer))

;;; --- Local File Management ---

(defun add-file (node name data &optional (chunk-size 64))
  "Add a file to the local store and share it."
  (let* ((sha (%content-hash data))
         (sf (%make-shared-file name (length data) sha chunk-size)))
    (setf (gethash name (peer-node-local-files node)) data)
    (setf (gethash name (peer-node-shared-catalog node)) sf)
    sf))

(defun remove-file (node name)
  "Stop sharing a file."
  (remhash name (peer-node-local-files node))
  (remhash name (peer-node-shared-catalog node)))

(defun get-chunk (node filename chunk-index)
  "Get a specific chunk of a local file."
  (let ((data (gethash filename (peer-node-local-files node)))
        (sf (gethash filename (peer-node-shared-catalog node))))
    (when (and data sf)
      (let* ((start (* chunk-index (shared-file-chunk-size sf)))
             (end (min (+ start (shared-file-chunk-size sf)) (length data))))
        (when (< start (length data))
          (make-file-chunk-from-data chunk-index (subseq data start end)))))))

;;; --- Peer Discovery ---

(defun discover-peer (node peer-id address port)
  "Register a known peer."
  (let ((info (make-peer-info :peer-id peer-id :address address :port port)))
    (setf (gethash peer-id (peer-node-peers node)) info)
    info))

(defun remove-peer (node peer-id)
  (remhash peer-id (peer-node-peers node)))

(defun prune-stale-peers (node &optional (timeout 300))
  "Remove stale peers. Returns list of removed IDs."
  (let (stale)
    (maphash (lambda (id peer)
               (when (peer-info-stale-p peer timeout)
                 (push id stale)))
             (peer-node-peers node))
    (dolist (id stale stale)
      (remhash id (peer-node-peers node)))))

(defun heartbeat (node peer-id)
  (let ((p (gethash peer-id (peer-node-peers node))))
    (when p (setf (peer-info-last-seen p) (get-universal-time)))))

;;; --- Catalog Exchange ---

(defun get-catalog (node)
  "Return this peer's shared catalog as a list."
  (let (catalog)
    (maphash (lambda (k v) (declare (ignore k)) (push v catalog))
             (peer-node-shared-catalog node))
    catalog))

(defun receive-catalog (node peer-id catalog)
  "Store another peer's catalog."
  (let ((p (gethash peer-id (peer-node-peers node))))
    (when p
      (setf (peer-info-shared-files p) catalog)
      (setf (peer-info-last-seen p) (get-universal-time)))))

(defun search-network (node query)
  "Search known peers for files matching QUERY. Returns ((peer-id . shared-file) ...)."
  (let ((q (string-downcase query))
        results)
    (maphash (lambda (pid peer)
               (dolist (sf (peer-info-shared-files peer))
                 (when (search q (string-downcase (shared-file-name sf)))
                   (push (cons pid sf) results))))
             (peer-node-peers node))
    (nreverse results)))

;;; --- File Transfer ---

(defun request-file (node provider-id filename)
  "Initiate a transfer request. Returns (values request-id transfer-request) or NIL."
  (let ((peer (gethash provider-id (peer-node-peers node))))
    (unless peer (return-from request-file nil))
    (let ((sf (find filename (peer-info-shared-files peer)
                    :key #'shared-file-name :test #'string=)))
      (unless sf (return-from request-file nil))
      (incf (peer-node-next-request-id node))
      (let* ((req-id (format nil "xfer-~D" (peer-node-next-request-id node)))
             (total (shared-file-chunk-count sf))
             (xfer (make-transfer-request
                    :request-id req-id
                    :filename filename
                    :requester-id (peer-node-peer-id node)
                    :provider-id provider-id
                    :status :active
                    :total-chunks total)))
        (setf (gethash req-id (peer-node-transfers node)) xfer)
        (setf (gethash req-id (peer-node-chunks-store node))
              (make-array total :initial-element nil))
        (values req-id xfer)))))

(defun receive-chunk (node request-id chunk)
  "Receive a chunk for an active transfer. Returns T on success."
  (let ((xfer (gethash request-id (peer-node-transfers node))))
    (unless (and xfer (eq (transfer-request-status xfer) :active))
      (return-from receive-chunk nil))
    (unless (file-chunk-verify chunk)
      (return-from receive-chunk nil))
    (let ((chunks (gethash request-id (peer-node-chunks-store node))))
      (unless (and chunks (< (file-chunk-index chunk) (length chunks)))
        (return-from receive-chunk nil))
      (setf (aref chunks (file-chunk-index chunk)) chunk)
      (let ((received (count-if #'identity chunks)))
        (setf (transfer-request-chunks-received xfer) received)
        (when (= received (transfer-request-total-chunks xfer))
          (setf (transfer-request-status xfer) :completed)
          ;; Reassemble
          (let ((assembled (apply #'concatenate 'string
                                  (loop for i below (length chunks)
                                        collect (file-chunk-data (aref chunks i))))))
            (setf (gethash (transfer-request-filename xfer)
                           (peer-node-local-files node))
                  assembled)))
        t))))

(defun get-missing-chunks (node request-id)
  "Return list of chunk indices not yet received."
  (let ((chunks (gethash request-id (peer-node-chunks-store node))))
    (when chunks
      (loop for i below (length chunks)
            when (null (aref chunks i))
            collect i))))

;;; --- Simulation ---

(defun simulate-transfer (provider requester filename)
  "Simulate a complete transfer between two peers. Returns the request-id."
  (multiple-value-bind (req-id xfer)
      (request-file requester (peer-node-peer-id provider) filename)
    (when req-id
      (dotimes (i (transfer-request-total-chunks xfer))
        (let ((chunk (get-chunk provider filename i)))
          (when chunk
            (receive-chunk requester req-id chunk))))
      req-id)))
