;;;; FTP Program — file transfer protocol client/server model.

(defpackage #:ftp-program
  (:use #:cl)
  (:export #:make-ftp-server
           #:server-add-file
           #:server-handle
           #:make-ftp-client
           #:client-connect
           #:client-send
           #:client-list
           #:client-get
           #:client-put
           #:client-delete
           #:client-size
           #:client-quit))

(in-package #:ftp-program)

;;; --- Server ---

(defstruct ftp-server
  "A simplified FTP server with a virtual filesystem."
  (files (make-hash-table :test #'equal) :type hash-table))

(defun server-add-file (server name content)
  "Seed the virtual filesystem with a file."
  (setf (gethash name (ftp-server-files server)) content))

(defun split-command (command)
  "Split a command string into up to 3 parts: cmd, arg1, rest."
  (let* ((trimmed (string-trim '(#\Space #\Tab #\Newline) command))
         (pos1 (position #\Space trimmed)))
    (if (null pos1)
        (list trimmed)
        (let* ((cmd (subseq trimmed 0 pos1))
               (rest1 (string-left-trim '(#\Space) (subseq trimmed (1+ pos1))))
               (pos2 (position #\Space rest1)))
          (if (null pos2)
              (list cmd rest1)
              (list cmd
                    (subseq rest1 0 pos2)
                    (subseq rest1 (1+ pos2))))))))

(defun sorted-keys (hash-table)
  "Return a sorted list of hash-table keys."
  (sort (loop for k being the hash-keys of hash-table collect k) #'string<))

(defun server-handle (server command)
  "Process a command string and return a response string."
  (let* ((parts (split-command command))
         (cmd (string-upcase (first parts)))
         (files (ftp-server-files server)))
    (cond
      ((string= cmd "LIST")
       (if (zerop (hash-table-count files))
           "226 (empty)"
           (format nil "226 File list:~%~{~a~^~%~}" (sorted-keys files))))

      ((string= cmd "RETR")
       (if (< (length parts) 2)
           "501 Syntax error in parameters"
           (let* ((filename (second parts))
                  (content (gethash filename files)))
             (if content
                 (format nil "226 Transfer complete:~%~a" content)
                 (format nil "550 ~a: No such file" filename)))))

      ((string= cmd "STOR")
       (if (< (length parts) 3)
           "501 Syntax error in parameters"
           (let ((filename (second parts))
                 (content (third parts)))
             (setf (gethash filename files) content)
             (format nil "226 ~a stored (~a bytes)" filename (length content)))))

      ((string= cmd "DELE")
       (if (< (length parts) 2)
           "501 Syntax error in parameters"
           (let ((filename (second parts)))
             (if (gethash filename files)
                 (progn
                   (remhash filename files)
                   (format nil "250 ~a deleted" filename))
                 (format nil "550 ~a: No such file" filename)))))

      ((string= cmd "SIZE")
       (if (< (length parts) 2)
           "501 Syntax error in parameters"
           (let* ((filename (second parts))
                  (content (gethash filename files)))
             (if content
                 (format nil "213 ~a" (length content))
                 (format nil "550 ~a: No such file" filename)))))

      ((string= cmd "QUIT")
       "221 Goodbye")

      (t
       (format nil "502 Command not implemented: ~a" cmd)))))

;;; --- Client ---

(defstruct ftp-client
  "A simplified FTP client that sends commands to an FtpServer."
  (server nil :type (or null ftp-server))
  (connected nil :type boolean))

(defun client-connect (client server)
  "Connect the client to a server instance."
  (setf (ftp-client-server client) server
        (ftp-client-connected client) t)
  "220 Service ready")

(defun client-send (client command)
  "Send a command to the connected server and return the response."
  (if (or (not (ftp-client-connected client))
          (null (ftp-client-server client)))
      "421 Not connected"
      (let ((response (server-handle (ftp-client-server client) command)))
        (when (string= (string-upcase (string-trim '(#\Space) command)) "QUIT")
          (setf (ftp-client-connected client) nil
                (ftp-client-server client) nil))
        response)))

(defun client-list (client)
  (client-send client "LIST"))

(defun client-get (client filename)
  (client-send client (format nil "RETR ~a" filename)))

(defun client-put (client filename content)
  (client-send client (format nil "STOR ~a ~a" filename content)))

(defun client-delete (client filename)
  (client-send client (format nil "DELE ~a" filename)))

(defun client-size (client filename)
  (client-send client (format nil "SIZE ~a" filename)))

(defun client-quit (client)
  (client-send client "QUIT"))

;;; --- Demo ---

(defun demo ()
  "Demonstrate the FTP client/server interaction."
  (let ((server (make-ftp-server))
        (client (make-ftp-client)))
    (server-add-file server "readme.txt" "Welcome to the FTP server.")
    (server-add-file server "data.csv" "name,value
alpha,1
beta,2")

    (format t "~a~%" (client-connect client server))

    (format t "~%--- LIST ---~%")
    (format t "~a~%" (client-list client))

    (format t "~%--- RETR readme.txt ---~%")
    (format t "~a~%" (client-get client "readme.txt"))

    (format t "~%--- STOR notes.txt ---~%")
    (format t "~a~%" (client-put client "notes.txt" "These are my notes."))

    (format t "~%--- LIST ---~%")
    (format t "~a~%" (client-list client))

    (format t "~%--- SIZE data.csv ---~%")
    (format t "~a~%" (client-size client "data.csv"))

    (format t "~%--- DELE data.csv ---~%")
    (format t "~a~%" (client-delete client "data.csv"))

    (format t "~%--- QUIT ---~%")
    (format t "~a~%" (client-quit client))

    (format t "~%--- After disconnect ---~%")
    (format t "~a~%" (client-list client))))
