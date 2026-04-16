;;;; Pig Latin — translate English text to Pig Latin.

(defpackage :text/pig-latin
  (:use :cl)
  (:export :pig-latin-word :pig-latin))

(in-package :text/pig-latin)

(defun vowelp (ch)
  "Return T if CH is a vowel."
  (member (char-downcase ch) '(#\a #\e #\i #\o #\u)))

(defun pig-latin-word (word)
  "Convert a single English word to Pig Latin.
Consonant cluster start: move cluster to end, add \"ay\".
Vowel start: add \"yay\".
Preserves capitalization of original first letter and trailing punctuation."
  (when (zerop (length word))
    (return-from pig-latin-word word))
  ;; Split trailing punctuation
  (let* ((alpha-end (position-if #'alpha-char-p word :from-end t))
         (alpha-end (if alpha-end (1+ alpha-end) 0))
         (core (subseq word 0 alpha-end))
         (trail (subseq word alpha-end)))
    (when (zerop (length core))
      (return-from pig-latin-word word))
    (let* ((was-cap (upper-case-p (char core 0)))
           (lower (string-downcase core))
           (result
             (if (vowelp (char lower 0))
                 (concatenate 'string lower "yay")
                 ;; Find consonant cluster
                 (let ((cluster-end (or (position-if #'vowelp lower) (length lower))))
                   (concatenate 'string
                                (subseq lower cluster-end)
                                (subseq lower 0 cluster-end)
                                "ay")))))
      ;; Restore capitalization
      (when was-cap
        (setf result (concatenate 'string
                                  (string (char-upcase (char result 0)))
                                  (subseq result 1))))
      (concatenate 'string result trail))))

(defun pig-latin (text)
  "Convert an English sentence to Pig Latin."
  (let ((words (uiop:split-string text :separator " ")))
    (format nil "~{~A~^ ~}" (mapcar #'pig-latin-word words))))
