;;;; Text Editor — buffer model with cursor, edit operations, and undo.

(defpackage :text/text-editor
  (:use :cl)
  (:export #:text-editor
           #:make-text-editor
           #:editor-lines
           #:editor-cursor-line
           #:editor-cursor-col
           #:insert-char
           #:delete-char
           #:insert-line
           #:delete-line
           #:move-cursor
           #:goto-pos
           #:get-line
           #:get-text
           #:undo
           #:find-in-buffer
           #:replace-in-buffer))

(in-package :text/text-editor)

;;; --- Data structures ---

(defstruct cursor-pos
  (line 0 :type fixnum)
  (col 0 :type fixnum))

(defstruct edit-action
  (kind nil :type keyword)
  (cursor nil :type cursor-pos)
  (data nil :type list))

(defstruct text-editor
  (lines (vector "") :type vector)
  (cursor (make-cursor-pos) :type cursor-pos)
  (undo-stack nil :type list))

;;; --- Constructor ---

(defun make-editor (&optional (text ""))
  "Create a new text editor, optionally initialized with text."
  (let ((lines (if (string= text "")
                   (vector "")
                   (let ((parts (split-string text #\Newline)))
                     (make-array (length parts)
                                 :initial-contents parts
                                 :adjustable t
                                 :fill-pointer (length parts))))))
    ;; Ensure adjustable vector
    (unless (adjustable-array-p lines)
      (setf lines (make-array (length lines)
                              :initial-contents (coerce lines 'list)
                              :adjustable t
                              :fill-pointer (length lines))))
    (make-text-editor :lines lines
                      :cursor (make-cursor-pos :line 0 :col 0)
                      :undo-stack nil)))

(defun split-string (string delimiter)
  "Split STRING by DELIMITER character, returning a list of substrings."
  (let ((result nil)
        (start 0))
    (loop for i from 0 below (length string)
          when (char= (char string i) delimiter)
            do (push (subseq string start i) result)
               (setf start (1+ i)))
    (push (subseq string start) result)
    (nreverse result)))

;;; --- Internal helpers ---

(defun clamp-cursor (editor)
  (let* ((cur (text-editor-cursor editor))
         (lines (text-editor-lines editor))
         (max-line (1- (length lines))))
    (setf (cursor-pos-line cur) (max 0 (min (cursor-pos-line cur) max-line)))
    (let ((line-len (length (aref lines (cursor-pos-line cur)))))
      (setf (cursor-pos-col cur) (max 0 (min (cursor-pos-col cur) line-len))))))

(defun push-undo (editor kind &rest data-pairs)
  (let ((cur (text-editor-cursor editor)))
    (push (make-edit-action
           :kind kind
           :cursor (make-cursor-pos :line (cursor-pos-line cur)
                                    :col (cursor-pos-col cur))
           :data data-pairs)
          (text-editor-undo-stack editor))))

(defun vector-insert (vec index element)
  "Insert ELEMENT into adjustable vector VEC at INDEX."
  (vector-push-extend "" vec)  ; grow by one
  (loop for i from (1- (length vec)) downto (1+ index)
        do (setf (aref vec i) (aref vec (1- i))))
  (setf (aref vec index) element))

(defun vector-remove (vec index)
  "Remove element at INDEX from adjustable vector VEC."
  (let ((removed (aref vec index)))
    (loop for i from index below (1- (length vec))
          do (setf (aref vec i) (aref vec (1+ i))))
    (decf (fill-pointer vec))
    removed))

;;; --- Editor operations ---

(defun insert-char (editor ch)
  "Insert character CH at cursor position."
  (push-undo editor :insert-char :char ch)
  (let* ((cur (text-editor-cursor editor))
         (line (aref (text-editor-lines editor) (cursor-pos-line cur)))
         (col (cursor-pos-col cur)))
    (setf (aref (text-editor-lines editor) (cursor-pos-line cur))
          (concatenate 'string
                       (subseq line 0 col)
                       (string ch)
                       (subseq line col)))
    (incf (cursor-pos-col cur))))

(defun delete-char (editor)
  "Backspace: delete character before cursor."
  (let* ((cur (text-editor-cursor editor))
         (lines (text-editor-lines editor))
         (ln (cursor-pos-line cur))
         (col (cursor-pos-col cur)))
    (cond
      ((and (= col 0) (= ln 0))
       nil)  ; nothing to delete
      ((> col 0)
       (let* ((line (aref lines ln))
              (deleted (char line (1- col))))
         (push-undo editor :delete-char :char deleted)
         (setf (aref lines ln)
               (concatenate 'string
                            (subseq line 0 (1- col))
                            (subseq line col)))
         (decf (cursor-pos-col cur))))
      (t
       ;; Join with previous line
       (let* ((prev (1- ln))
              (join-col (length (aref lines prev)))
              (removed (aref lines ln)))
         (push-undo editor :delete-char-join
                    :removed-line removed :join-col join-col)
         (setf (aref lines prev)
               (concatenate 'string (aref lines prev) removed))
         (vector-remove lines ln)
         (setf (cursor-pos-line cur) prev
               (cursor-pos-col cur) join-col))))))

(defun insert-line (editor)
  "Split current line at cursor (Enter key)."
  (push-undo editor :insert-line)
  (let* ((cur (text-editor-cursor editor))
         (lines (text-editor-lines editor))
         (ln (cursor-pos-line cur))
         (col (cursor-pos-col cur))
         (line (aref lines ln))
         (before (subseq line 0 col))
         (after (subseq line col)))
    (setf (aref lines ln) before)
    (vector-insert lines (1+ ln) after)
    (setf (cursor-pos-line cur) (1+ ln)
          (cursor-pos-col cur) 0)))

(defun delete-line (editor)
  "Remove the current line."
  (let* ((cur (text-editor-cursor editor))
         (lines (text-editor-lines editor))
         (ln (cursor-pos-line cur))
         (content (aref lines ln)))
    (push-undo editor :delete-line :content content :index ln)
    (if (= (length lines) 1)
        (progn
          (setf (aref lines 0) "")
          (setf (cursor-pos-col cur) 0))
        (progn
          (vector-remove lines ln)
          (when (>= (cursor-pos-line cur) (length lines))
            (setf (cursor-pos-line cur) (1- (length lines))))
          (setf (cursor-pos-col cur) 0)))))

(defun move-cursor (editor direction)
  "Move cursor: :up, :down, :left, :right."
  (let* ((cur (text-editor-cursor editor))
         (lines (text-editor-lines editor)))
    (case direction
      (:left
       (if (> (cursor-pos-col cur) 0)
           (decf (cursor-pos-col cur))
           (when (> (cursor-pos-line cur) 0)
             (decf (cursor-pos-line cur))
             (setf (cursor-pos-col cur)
                   (length (aref lines (cursor-pos-line cur)))))))
      (:right
       (let ((line-len (length (aref lines (cursor-pos-line cur)))))
         (if (< (cursor-pos-col cur) line-len)
             (incf (cursor-pos-col cur))
             (when (< (cursor-pos-line cur) (1- (length lines)))
               (incf (cursor-pos-line cur))
               (setf (cursor-pos-col cur) 0)))))
      (:up
       (when (> (cursor-pos-line cur) 0)
         (decf (cursor-pos-line cur))
         (clamp-cursor editor)))
      (:down
       (when (< (cursor-pos-line cur) (1- (length lines)))
         (incf (cursor-pos-line cur))
         (clamp-cursor editor))))))

(defun goto-pos (editor line col)
  "Move cursor to specific position."
  (let ((cur (text-editor-cursor editor)))
    (setf (cursor-pos-line cur) line
          (cursor-pos-col cur) col)
    (clamp-cursor editor)))

(defun get-line (editor n)
  "Get content of line N."
  (let ((lines (text-editor-lines editor)))
    (if (and (>= n 0) (< n (length lines)))
        (aref lines n)
        (error "Line ~D out of range (0..~D)" n (1- (length lines))))))

(defun get-text (editor)
  "Return full buffer content."
  (let ((lines (text-editor-lines editor)))
    (with-output-to-string (s)
      (loop for i from 0 below (length lines)
            do (when (> i 0) (write-char #\Newline s))
               (write-string (aref lines i) s)))))

(defun undo (editor)
  "Revert the last edit operation."
  (when (null (text-editor-undo-stack editor))
    (return-from undo nil))
  (let* ((action (pop (text-editor-undo-stack editor)))
         (kind (edit-action-kind action))
         (prev-cur (edit-action-cursor action))
         (data (edit-action-data action))
         (lines (text-editor-lines editor))
         (cur (text-editor-cursor editor)))
    (case kind
      (:insert-char
       (let* ((ch (getf data :char))
              (ln (cursor-pos-line prev-cur))
              (col (cursor-pos-col prev-cur))
              (line (aref lines ln)))
         (setf (aref lines ln)
               (concatenate 'string (subseq line 0 col) (subseq line (1+ col))))))
      (:delete-char
       (let* ((ch (getf data :char))
              (ln (cursor-pos-line prev-cur))
              (col (cursor-pos-col prev-cur))
              (line (aref lines ln)))
         (setf (aref lines ln)
               (concatenate 'string
                            (subseq line 0 (1- col))
                            (string ch)
                            (subseq line (1- col))))))
      (:delete-char-join
       (let* ((removed (getf data :removed-line))
              (join-col (getf data :join-col))
              (prev-line (1- (cursor-pos-line prev-cur)))
              (combined (aref lines prev-line)))
         (setf (aref lines prev-line) (subseq combined 0 join-col))
         (vector-insert lines (cursor-pos-line prev-cur) removed)))
      (:insert-line
       (let ((ln (cursor-pos-line prev-cur)))
         (when (< (1+ ln) (length lines))
           (setf (aref lines ln)
                 (concatenate 'string (aref lines ln) (aref lines (1+ ln))))
           (vector-remove lines (1+ ln)))))
      (:delete-line
       (let ((content (getf data :content))
             (index (getf data :index)))
         (if (and (= (length lines) 1) (string= (aref lines 0) ""))
             (setf (aref lines 0) content)
             (vector-insert lines index content))))
      (:replace
       (let ((old-lines (getf data :old-lines)))
         (setf (text-editor-lines editor) old-lines))))
    ;; Restore cursor
    (setf (cursor-pos-line cur) (cursor-pos-line prev-cur)
          (cursor-pos-col cur) (cursor-pos-col prev-cur))))

(defun find-in-buffer (editor pattern)
  "Find all occurrences of PATTERN. Returns list of (line col) pairs."
  (let ((results nil)
        (lines (text-editor-lines editor))
        (pat-len (length pattern)))
    (loop for i from 0 below (length lines)
          do (let ((line (aref lines i))
                   (start 0))
               (loop
                 (let ((pos (search pattern line :start2 start)))
                   (unless pos (return))
                   (push (list i pos) results)
                   (setf start (1+ pos))))))
    (nreverse results)))

(defun replace-in-buffer (editor pattern replacement &optional (count -1))
  "Replace occurrences of PATTERN with REPLACEMENT. Returns number replaced."
  (let* ((lines (text-editor-lines editor))
         (old-lines (map 'vector #'identity lines))
         (total 0)
         (pat-len (length pattern))
         (rep-len (length replacement)))
    (loop for i from 0 below (length lines)
          do (when (and (>= count 0) (>= total count))
               (return))
             (let ((line (aref lines i))
                   (new-line (make-string-output-stream))
                   (start 0))
               (loop
                 (when (and (>= count 0) (>= total count))
                   (write-string (subseq line start) new-line)
                   (return))
                 (let ((pos (search pattern line :start2 start)))
                   (unless pos
                     (write-string (subseq line start) new-line)
                     (return))
                   (write-string (subseq line start pos) new-line)
                   (write-string replacement new-line)
                   (setf start (+ pos pat-len))
                   (incf total)))
               (setf (aref lines i) (get-output-stream-string new-line))))
    (when (> total 0)
      (push-undo editor :replace :old-lines old-lines))
    total))

;;; --- Demo ---

(defun demo ()
  "Demonstrate the text editor."
  (let ((ed (make-editor)))
    ;; Type "Hello, world!"
    (loop for ch across "Hello, world!"
          do (insert-char ed ch))
    (format t "After typing: '~A'~%" (get-text ed))

    ;; New line
    (insert-line ed)
    (loop for ch across "The quick fox."
          do (insert-char ed ch))
    (format t "Two lines:~%~A~%" (get-text ed))

    ;; Find
    (format t "Find 'o': ~A~%" (find-in-buffer ed "o"))

    ;; Replace
    (let ((n (replace-in-buffer ed "fox" "cat")))
      (format t "Replaced ~D: ~A~%" n (get-text ed)))

    ;; Undo
    (undo ed)
    (format t "After undo: ~A~%" (get-text ed))))
