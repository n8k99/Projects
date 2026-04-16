;;;; Reverse a String — reverse character order.
;;;; CL's REVERSE works on sequences, but we implement explicitly for the exercise.

(defpackage :text/reverse-string
  (:use :cl)
  (:export :reverse-string-chars :reverse-words :palindrome-p))

(in-package :text/reverse-string)

(defun reverse-string-chars (s)
  "Reverse the characters in string S.
Implemented explicitly rather than calling CL:REVERSE."
  (let* ((len (length s))
         (result (make-string len)))
    (loop for i from 0 below len
          for j from (1- len) downto 0
          do (setf (char result j) (char s i)))
    result))

(defun split-words (s)
  "Split string S on whitespace, returning a list of words."
  (let ((words nil)
        (start nil))
    (loop for i from 0 to (length s)
          for ch = (if (< i (length s)) (char s i) #\Space)
          do (cond
               ((and (not start) (graphic-char-p ch) (char/= ch #\Space))
                (setf start i))
               ((and start (or (char= ch #\Space) (= i (length s))))
                (push (subseq s start i) words)
                (setf start nil))))
    (nreverse words)))

(defun reverse-words (s)
  "Reverse the order of words in string S, keeping each word intact."
  (format nil "~{~A~^ ~}" (nreverse (split-words s))))

(defun palindrome-p (s)
  "Return T if string S reads the same forwards and backwards."
  (string= s (reverse-string-chars s)))
