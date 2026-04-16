;;;; Count Words in a String — word-level text analysis.

(defpackage :text/count-words
  (:use :cl)
  (:export #:word-count
           #:word-frequency
           #:unique-words
           #:average-word-length))

(in-package :text/count-words)

(defun split-words (text)
  "Split TEXT on whitespace, returning a list of non-empty strings."
  (let ((words '())
        (start nil))
    (loop for i from 0 below (length text)
          for ch = (char text i)
          do (cond
               ((member ch '(#\Space #\Tab #\Newline #\Return))
                (when start
                  (push (subseq text start i) words)
                  (setf start nil)))
               ((null start)
                (setf start i)))
          finally (when start
                    (push (subseq text start) words)))
    (nreverse words)))

(defun strip-punctuation (word)
  "Strip leading and trailing punctuation from WORD."
  (let ((start 0)
        (end (length word)))
    (loop while (and (< start end)
                     (not (alphanumericp (char word start))))
          do (incf start))
    (loop while (and (> end start)
                     (not (alphanumericp (char word (1- end)))))
          do (decf end))
    (if (>= start end)
        ""
        (subseq word start end))))

(defun clean-word (word)
  "Strip punctuation and downcase WORD."
  (string-downcase (strip-punctuation word)))

(defun word-count (text)
  "Count words in TEXT (split on whitespace)."
  (length (split-words text)))

(defun word-frequency (text)
  "Count occurrences of each word (case-insensitive, strip punctuation).
   Returns an alist mapping each lowercase word to its count."
  (let ((counts '()))
    (dolist (word (split-words text))
      (let ((cleaned (clean-word word)))
        (when (plusp (length cleaned))
          (let ((entry (assoc cleaned counts :test #'string=)))
            (if entry
                (incf (cdr entry))
                (push (cons cleaned 1) counts))))))
    (nreverse counts)))

(defun unique-words (text)
  "Count distinct words (case-insensitive, strip punctuation)."
  (length (word-frequency text)))

(defun average-word-length (text)
  "Return the mean word length (punctuation stripped).
   Returns 0.0 if there are no words."
  (let ((words (remove-if (lambda (w) (zerop (length w)))
                          (mapcar #'clean-word (split-words text)))))
    (if (null words)
        0.0
        (/ (reduce #'+ words :key #'length)
           (length words)))))

;;; Demo
(defun demo ()
  "Run demonstration with sample strings."
  (dolist (phrase '("The quick brown fox jumps over the lazy dog"
                    "To be or not to be, that is the question."))
    (format t "~%--- ~S ---~%" phrase)
    (format t "  word count:        ~D~%" (word-count phrase))
    (format t "  word frequency:    ~S~%" (word-frequency phrase))
    (format t "  unique words:      ~D~%" (unique-words phrase))
    (format t "  avg word length:   ~,2F~%" (average-word-length phrase))))
