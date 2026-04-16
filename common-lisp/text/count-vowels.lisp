;;;; Count Vowels — analyze vowel distribution in text.

(defpackage :text/count-vowels
  (:use :cl)
  (:export #:count-vowels
           #:vowel-breakdown
           #:vowel-ratio))

(in-package :text/count-vowels)

(defparameter *vowels* '(#\a #\e #\i #\o #\u)
  "Standard English vowels.")

(defun vowelp (ch &optional include-y)
  "Return T if CH is a vowel. When INCLUDE-Y is true, treat Y as a vowel."
  (let ((lower (char-downcase ch)))
    (or (member lower *vowels* :test #'char=)
        (and include-y (char= lower #\y)))))

(defun count-vowels (text &optional include-y)
  "Count total vowels in TEXT. When INCLUDE-Y is true, count Y as a vowel."
  (loop for ch across text
        count (vowelp ch include-y)))

(defun vowel-breakdown (text &optional include-y)
  "Return an alist mapping each vowel to its count in TEXT.
   E.g. ((#\\a . 3) (#\\e . 5) (#\\i . 2) (#\\o . 1) (#\\u . 0))."
  (let ((keys (if include-y
                  '(#\a #\e #\i #\o #\u #\y)
                  '(#\a #\e #\i #\o #\u))))
    (mapcar (lambda (vowel)
              (cons vowel
                    (loop for ch across text
                          count (char= (char-downcase ch) vowel))))
            keys)))

(defun vowel-ratio (text)
  "Return the ratio of vowels to total alphabetic characters in TEXT.
   Returns 0.0 if TEXT contains no alphabetic characters."
  (let ((alpha-count (loop for ch across text count (alpha-char-p ch))))
    (if (zerop alpha-count)
        0.0
        (/ (count-vowels text) alpha-count))))

;;; Demo
(defun demo ()
  "Run demonstration with sample strings."
  (dolist (phrase '("Hello World"
                    "The quick brown fox"
                    "Why try fly by my dry cry?"))
    (format t "~%--- ~S ---~%" phrase)
    (format t "  vowels:      ~D~%" (count-vowels phrase))
    (format t "  vowels (+y): ~D~%" (count-vowels phrase t))
    (format t "  breakdown:   ~S~%" (vowel-breakdown phrase))
    (format t "  breakdown (+y): ~S~%" (vowel-breakdown phrase t))
    (format t "  ratio:       ~,3F~%" (vowel-ratio phrase))))
