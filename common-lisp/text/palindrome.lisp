;;;; Check if Palindrome — detect and find palindromic strings.

(defpackage :text/palindrome
  (:use :cl)
  (:export :palindrome-p
           :palindrome-normalized-p
           :longest-palindrome-substring))

(in-package :text/palindrome)

(defun palindrome-p (s)
  "Return T if string S reads the same forwards and backwards (exact)."
  (string= s (reverse s)))

(defun normalize (s)
  "Strip non-alphanumeric characters and downcase."
  (let ((chars (loop for ch across s
                     when (alphanumericp ch)
                     collect (char-downcase ch))))
    (coerce chars 'string)))

(defun palindrome-normalized-p (s)
  "Return T if string S is a palindrome, ignoring case, spaces, and punctuation."
  (let ((n (normalize s)))
    (string= n (reverse n))))

(defun expand-around-center (s left right)
  "Expand outward from LEFT and RIGHT while characters match.
Returns (VALUES start end) of the palindrome found."
  (let ((len (length s))
        (l left)
        (r right))
    (loop while (and (>= l 0)
                     (< r len)
                     (char= (char s l) (char s r)))
          do (decf l)
             (incf r))
    (values (1+ l) (1- r))))

(defun longest-palindrome-substring (s)
  "Find the longest palindromic substring using expand-around-center."
  (when (< (length s) 2)
    (return-from longest-palindrome-substring s))
  (let ((best-start 0)
        (best-end 0))
    (dotimes (i (length s))
      ;; Odd-length palindromes
      (multiple-value-bind (s1 e1) (expand-around-center s i i)
        (when (> (- e1 s1) (- best-end best-start))
          (setf best-start s1
                best-end e1)))
      ;; Even-length palindromes
      (multiple-value-bind (s2 e2) (expand-around-center s i (1+ i))
        (when (> (- e2 s2) (- best-end best-start))
          (setf best-start s2
                best-end e2))))
    (subseq s best-start (1+ best-end))))
