;;;; Telnet Application — line-oriented protocol with state-gated commands.

(defpackage #:telnet
  (:use #:cl)
  (:export #:make-server
           #:connect
           #:handle-line
           #:get-session
           #:authenticated-users
           #:session-id #:session-state #:session-username #:session-history
           #:outcome-kind #:outcome-text))

(in-package #:telnet)

(defstruct session
  (id 0 :type integer)
  (state :unauthenticated :type keyword)     ; :unauthenticated :authenticated :disconnected
  (username nil :type (or null string))
  (history nil :type list))                  ; list of (line . reply)

(defstruct server
  (sessions (make-hash-table :test 'eql))
  (next-id 1 :type integer))

(defstruct outcome
  (kind :reply :type keyword)                ; :reply :disconnect :error
  (text "" :type string))

(defun connect (server)
  (let ((id (server-next-id server)))
    (incf (server-next-id server))
    (setf (gethash id (server-sessions server))
          (make-session :id id))
    id))

(defun get-session (server id) (gethash id (server-sessions server)))

(defun authenticated-users (server)
  (let ((names nil))
    (maphash (lambda (id sess)
               (declare (ignore id))
               (when (eq (session-state sess) :authenticated)
                 (push (session-username sess) names)))
             (server-sessions server))
    (sort names #'string<)))

(defun %split-cmd (line)
  (let ((pos (position-if (lambda (c) (or (char= c #\Space) (char= c #\Tab))) line)))
    (if pos
        (values (subseq line 0 pos)
                (string-trim '(#\Space #\Tab) (subseq line (1+ pos))))
        (values line ""))))

(defun handle-line (server id line)
  (let* ((trimmed (string-trim '(#\Space #\Tab #\Newline #\Return) line))
         (out (%dispatch server id trimmed))
         (sess (get-session server id)))
    (when sess
      (setf (session-history sess)
            (append (session-history sess)
                    (list (cons trimmed (outcome-text out)))))
      (when (eq (outcome-kind out) :disconnect)
        (setf (session-state sess) :disconnected)))
    out))

(defun %dispatch (server id line)
  (let ((sess (get-session server id)))
    (unless sess
      (return-from %dispatch (make-outcome :kind :error :text (format nil "unknown session ~A" id))))
    (when (eq (session-state sess) :disconnected)
      (return-from %dispatch (make-outcome :kind :error :text "session is disconnected")))
    (multiple-value-bind (cmd rest) (%split-cmd line)
      (cond
        ((zerop (length cmd))
         (make-outcome :kind :reply :text ""))
        ((string= cmd "help")
         (make-outcome :kind :reply
                       :text "commands: help, login <name>, whoami, who, echo <text>, quit"))
        ((string= cmd "login")
         (cond
           ((zerop (length rest))
            (make-outcome :kind :error :text "usage: login <name>"))
           ((eq (session-state sess) :authenticated)
            (make-outcome :kind :error
                          :text (format nil "already logged in as ~A" (session-username sess))))
           (t
            (setf (session-state sess) :authenticated)
            (setf (session-username sess) rest)
            (make-outcome :kind :reply :text (format nil "welcome, ~A" rest)))))
        ((string= cmd "whoami")
         (if (eq (session-state sess) :authenticated)
             (make-outcome :kind :reply :text (session-username sess))
             (make-outcome :kind :error :text "not logged in")))
        ((string= cmd "who")
         (let ((users (authenticated-users server)))
           (make-outcome :kind :reply
                         :text (if users
                                   (format nil "~{~A~^, ~}" users)
                                   "no one is logged in"))))
        ((string= cmd "echo")
         (make-outcome :kind :reply :text rest))
        ((string= cmd "quit")
         (make-outcome :kind :disconnect :text "goodbye"))
        (t
         (make-outcome :kind :error :text (format nil "unknown command: ~A" cmd)))))))
