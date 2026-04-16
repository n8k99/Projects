;;;; Chat Application (Networking) — multi-room chat with users and history.

(defpackage #:chat-app
  (:use #:cl)
  (:export #:make-chat-server
           #:register-user
           #:create-room
           #:join-room
           #:leave-room
           #:send-message
           #:get-messages
           #:list-rooms
           #:list-users
           #:message-sender
           #:message-room
           #:message-text
           #:message-timestamp))

(in-package #:chat-app)

;;; --- Message ---

(defstruct message
  (sender "" :type string)
  (room "" :type string)
  (text "" :type string)
  (timestamp (get-universal-time) :type integer))

;;; --- Chat Room ---

(defstruct chat-room
  (name "" :type string)
  (members nil :type list)     ; list of user-name strings
  (messages nil :type list))   ; list of message structs

;;; --- Chat User ---

(defstruct chat-user
  (name "" :type string)
  (rooms nil :type list))      ; list of room-name strings

;;; --- Chat Server ---

(defstruct chat-server
  (rooms (make-hash-table :test #'equal) :type hash-table)
  (users (make-hash-table :test #'equal) :type hash-table))

(defun register-user (server name)
  "Register a new user on the server. Signals an error if the name is taken."
  (when (gethash name (chat-server-users server))
    (error "User '~A' already exists" name))
  (let ((user (make-chat-user :name name)))
    (setf (gethash name (chat-server-users server)) user)
    user))

(defun create-room (server room-name)
  "Create a new chat room. Signals an error if the room already exists."
  (when (gethash room-name (chat-server-rooms server))
    (error "Room '~A' already exists" room-name))
  (let ((room (make-chat-room :name room-name)))
    (setf (gethash room-name (chat-server-rooms server)) room)
    room))

(defun %get-user (server name)
  (or (gethash name (chat-server-users server))
      (error "No such user: '~A'" name)))

(defun %get-room (server room-name)
  (or (gethash room-name (chat-server-rooms server))
      (error "No such room: '~A'" room-name)))

(defun join-room (server user-name room-name)
  "Add a user to a room."
  (let ((user (%get-user server user-name))
        (room (%get-room server room-name)))
    (pushnew user-name (chat-room-members room) :test #'equal)
    (pushnew room-name (chat-user-rooms user) :test #'equal)
    t))

(defun leave-room (server user-name room-name)
  "Remove a user from a room."
  (let ((user (%get-user server user-name))
        (room (%get-room server room-name)))
    (setf (chat-room-members room)
          (remove user-name (chat-room-members room) :test #'equal))
    (setf (chat-user-rooms user)
          (remove room-name (chat-user-rooms user) :test #'equal))
    t))

(defun send-message (server user-name room-name text)
  "Send a message from a user to a room. User must be a member."
  (%get-user server user-name)
  (let ((room (%get-room server room-name)))
    (unless (member user-name (chat-room-members room) :test #'equal)
      (error "User '~A' is not a member of room '~A'" user-name room-name))
    (let ((msg (make-message :sender user-name :room room-name :text text)))
      (setf (chat-room-messages room)
            (append (chat-room-messages room) (list msg)))
      msg)))

(defun get-messages (server room-name &optional since)
  "Return messages from a room. If SINCE (universal-time) is given, only
messages at or after that timestamp are returned."
  (let ((room (%get-room server room-name)))
    (if since
        (remove-if (lambda (m) (< (message-timestamp m) since))
                   (chat-room-messages room))
        (copy-list (chat-room-messages room)))))

(defun list-rooms (server)
  "Return a sorted list of room names."
  (let ((names nil))
    (maphash (lambda (k v) (declare (ignore v)) (push k names))
             (chat-server-rooms server))
    (sort names #'string<)))

(defun list-users (server room-name)
  "Return a sorted list of members in a room."
  (let ((room (%get-room server room-name)))
    (sort (copy-list (chat-room-members room)) #'string<)))

;;; --- Demo ---

(defun demo ()
  (let ((server (make-chat-server)))
    ;; Register users
    (register-user server "alice")
    (register-user server "bob")
    (register-user server "carol")

    ;; Create rooms
    (create-room server "#general")
    (create-room server "#engineering")

    ;; Join rooms
    (join-room server "alice" "#general")
    (join-room server "bob" "#general")
    (join-room server "carol" "#general")
    (join-room server "alice" "#engineering")
    (join-room server "bob" "#engineering")

    ;; Send messages
    (send-message server "alice" "#general" "Hello everyone!")
    (send-message server "bob" "#general" "Hey Alice!")
    (send-message server "carol" "#general" "Good morning!")
    (send-message server "alice" "#engineering" "Sprint planning at 2pm")
    (send-message server "bob" "#engineering" "Sounds good, I'll be there")

    ;; Show rooms
    (format t "Rooms: ~{~A~^, ~}~%" (list-rooms server))
    (terpri)

    ;; Show members
    (dolist (room-name (list-rooms server))
      (format t "Members of ~A: ~{~A~^, ~}~%" room-name (list-users server room-name)))
    (terpri)

    ;; Show messages
    (dolist (room-name (list-rooms server))
      (format t "--- ~A ---~%" room-name)
      (dolist (msg (get-messages server room-name))
        (format t "  [~A] ~A: ~A~%"
                (message-timestamp msg)
                (message-sender msg)
                (message-text msg)))
      (terpri))

    ;; Carol leaves
    (leave-room server "carol" "#general")
    (format t "After carol leaves #general:~%")
    (format t "  Members: ~{~A~^, ~}~%" (list-users server "#general"))))
