;;;; Remote Login — authenticated session management with credentials and tokens.

(defpackage #:remote-login
  (:use #:cl)
  (:export #:make-auth-server
           #:auth-register
           #:auth-login
           #:validate-session
           #:auth-logout
           #:revoke-all-sessions
           #:active-sessions
           #:login-result-success
           #:login-result-token
           #:login-result-error
           #:session-username
           #:session-role
           #:session-valid-p
           #:auth-server-audit-log))

(in-package #:remote-login)

;;; --- Hashing (portable via sxhash) ---

(defun %hash-password (salt password)
  (format nil "~16,'0X" (sxhash (concatenate 'string salt password))))

(defun %random-hex (n)
  (format nil "~(~{~2,'0X~}~)"
          (loop repeat n collect (random 256))))

;;; --- Data Structures ---

(defstruct credential
  (username "" :type string)
  (password-hash "" :type string)
  (salt "" :type string)
  (role "user" :type string)
  (locked nil)
  (failed-attempts 0 :type integer)
  (last-login nil))

(defun create-credential (username password &optional (role "user"))
  (let ((salt (%random-hex 16)))
    (make-credential :username username
                     :password-hash (%hash-password salt password)
                     :salt salt :role role)))

(defun verify-credential (cred password)
  (string= (%hash-password (credential-salt cred) password)
            (credential-password-hash cred)))

(defstruct session
  (token "" :type string)
  (username "" :type string)
  (role "" :type string)
  (created (get-universal-time) :type integer)
  (expires 0 :type integer)
  (ip-address "" :type string)
  (is-active t))

(defun session-valid-p (s)
  (and (session-is-active s)
       (> (session-expires s) (get-universal-time))))

(defstruct login-attempt
  (username "" :type string)
  (ip-address "" :type string)
  (timestamp (get-universal-time) :type integer)
  (success nil)
  (reason "" :type string))

(defstruct login-result
  (success nil)
  (token nil)
  (error "" :type string)
  (attempts-remaining nil))

;;; --- Auth Server ---

(defstruct auth-server
  (session-ttl 1800 :type integer)  ; seconds
  (max-failed 5 :type integer)
  (lockout-duration 900 :type integer)
  (credentials (make-hash-table :test #'equal) :type hash-table)
  (sessions (make-hash-table :test #'equal) :type hash-table)
  (audit-log nil :type list)
  (lockout-until (make-hash-table :test #'equal) :type hash-table))

(defun auth-register (server username password &optional (role "user"))
  (when (gethash username (auth-server-credentials server))
    (return-from auth-register nil))
  (setf (gethash username (auth-server-credentials server))
        (create-credential username password role))
  t)

(defun %log-attempt (server username ip success reason)
  (push (make-login-attempt :username username :ip-address ip
                            :success success :reason reason)
        (auth-server-audit-log server)))

(defun auth-login (server username password &optional (ip ""))
  (let ((now (get-universal-time)))
    ;; Check lockout
    (let ((until (gethash username (auth-server-lockout-until server))))
      (when (and until (< now until))
        (%log-attempt server username ip nil "account locked")
        (return-from auth-login
          (make-login-result :error "Account locked")))
      (when until
        (remhash username (auth-server-lockout-until server))
        (let ((c (gethash username (auth-server-credentials server))))
          (when c (setf (credential-failed-attempts c) 0)))))
    ;; Find credential
    (let ((cred (gethash username (auth-server-credentials server))))
      (unless cred
        (%log-attempt server username ip nil "unknown user")
        (return-from auth-login
          (make-login-result :error "Invalid credentials")))
      (when (credential-locked cred)
        (%log-attempt server username ip nil "account disabled")
        (return-from auth-login
          (make-login-result :error "Account disabled")))
      ;; Verify
      (unless (verify-credential cred password)
        (incf (credential-failed-attempts cred))
        (let ((remaining (- (auth-server-max-failed server)
                            (credential-failed-attempts cred))))
          (when (<= remaining 0)
            (setf (gethash username (auth-server-lockout-until server))
                  (+ now (auth-server-lockout-duration server)))
            (setf (credential-failed-attempts cred) 0)
            (%log-attempt server username ip nil "locked after max attempts")
            (return-from auth-login
              (make-login-result :error "Account locked" :attempts-remaining 0)))
          (%log-attempt server username ip nil "wrong password")
          (return-from auth-login
            (make-login-result :error "Invalid credentials" :attempts-remaining remaining))))
      ;; Success
      (setf (credential-failed-attempts cred) 0)
      (setf (credential-last-login cred) now)
      (let* ((token (%random-hex 32))
             (session (make-session :token token :username username
                                   :role (credential-role cred)
                                   :created now
                                   :expires (+ now (auth-server-session-ttl server))
                                   :ip-address ip)))
        (setf (gethash token (auth-server-sessions server)) session)
        (%log-attempt server username ip t "login successful")
        (make-login-result :success t :token token)))))

(defun validate-session (server token)
  (let ((s (gethash token (auth-server-sessions server))))
    (when (and s (session-valid-p s)) s)))

(defun auth-logout (server token)
  (let ((s (gethash token (auth-server-sessions server))))
    (when s (setf (session-is-active s) nil) t)))

(defun revoke-all-sessions (server username)
  (let ((count 0))
    (maphash (lambda (tok s)
               (declare (ignore tok))
               (when (and (string= (session-username s) username)
                          (session-is-active s))
                 (setf (session-is-active s) nil)
                 (incf count)))
             (auth-server-sessions server))
    count))

(defun active-sessions (server &optional username)
  (let (result)
    (maphash (lambda (tok s)
               (declare (ignore tok))
               (when (and (session-valid-p s)
                          (or (null username) (string= (session-username s) username)))
                 (push s result)))
             (auth-server-sessions server))
    result))
