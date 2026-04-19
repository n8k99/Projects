;;;; Chat Application — bidirectional broadcast across N concurrent actors.
;;;;
;;;; Every actor is both a producer (sending messages) and a consumer (reading
;;;; its inbox). One send fans out to every joined user's inbox. A monotonic
;;;; sequence counter assigned under the chat-room lock gives every message a total
;;;; order consistent across all recipients — the standard trick for getting
;;;; deterministic ordering out of a concurrent system.
;;;;
;;;; SBCL is assumed for threading.

(defpackage #:chat-app
  (:use #:cl)
  (:export #:make-chat-room
           #:join
           #:leave
           #:send-message
           #:read-messages
           #:history
           #:user-count
           #:message-seq
           #:message-from-id
           #:message-from-name
           #:message-text))

(in-package #:chat-app)

(defstruct message
  (seq 0 :type integer)
  (from-id 0 :type integer)
  (from-name "" :type string)
  (text "" :type string))

(defstruct inbox
  (name "" :type string)
  (messages nil :type list))                    ; oldest first

(defstruct chat-room
  (lock (sb-thread:make-mutex))
  (users (make-hash-table :test 'eql))          ; id -> inbox
  (log nil :type list)
  (next-id 1 :type integer)
  (next-seq 1 :type integer))

(defun join (chat-room name)
  (sb-thread:with-mutex ((chat-room-lock chat-room))
    (let ((id (chat-room-next-id chat-room)))
      (incf (chat-room-next-id chat-room))
      (setf (gethash id (chat-room-users chat-room))
            (make-inbox :name name))
      id)))

(defun leave (chat-room id)
  (sb-thread:with-mutex ((chat-room-lock chat-room))
    (remhash id (chat-room-users chat-room))))

(defun send-message (chat-room sender-id text)
  (sb-thread:with-mutex ((chat-room-lock chat-room))
    (let ((sender (gethash sender-id (chat-room-users chat-room))))
      (unless sender (return-from send-message nil))
      (let* ((seq (chat-room-next-seq chat-room))
             (msg (make-message :seq seq
                                :from-id sender-id
                                :from-name (inbox-name sender)
                                :text text)))
        (incf (chat-room-next-seq chat-room))
        (push msg (chat-room-log chat-room))
        (maphash (lambda (id inbox)
                   (declare (ignore id))
                   (setf (inbox-messages inbox)
                         (append (inbox-messages inbox) (list msg))))
                 (chat-room-users chat-room))
        t))))

(defun read-messages (chat-room id)
  (sb-thread:with-mutex ((chat-room-lock chat-room))
    (let ((inbox (gethash id (chat-room-users chat-room))))
      (unless inbox (return-from read-messages nil))
      (let ((out (inbox-messages inbox)))
        (setf (inbox-messages inbox) nil)
        out))))

(defun history (chat-room)
  (sb-thread:with-mutex ((chat-room-lock chat-room))
    (reverse (copy-list (chat-room-log chat-room)))))

(defun user-count (chat-room)
  (sb-thread:with-mutex ((chat-room-lock chat-room))
    (hash-table-count (chat-room-users chat-room))))
