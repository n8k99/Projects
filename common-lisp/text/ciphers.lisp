;;;; Vigenere / Vernam / Caesar Ciphers — classical text encryption.

(defpackage #:text/ciphers
  (:use #:cl)
  (:export #:caesar-encrypt #:caesar-decrypt
           #:vigenere-encrypt #:vigenere-decrypt
           #:vernam-encrypt #:vernam-decrypt))

(in-package #:text/ciphers)

;;; Caesar cipher

(defun caesar-shift-char (ch shift)
  "Shift a single character by SHIFT positions, preserving case. Non-alpha pass through."
  (cond
    ((upper-case-p ch)
     (code-char (+ (mod (+ (- (char-code ch) (char-code #\A)) shift) 26)
                   (char-code #\A))))
    ((lower-case-p ch)
     (code-char (+ (mod (+ (- (char-code ch) (char-code #\a)) shift) 26)
                   (char-code #\a))))
    (t ch)))

(defun caesar-encrypt (text shift)
  "Encrypt TEXT using Caesar cipher with given SHIFT."
  (let ((s (mod shift 26)))
    (map 'string (lambda (ch) (caesar-shift-char ch s)) text)))

(defun caesar-decrypt (text shift)
  "Decrypt Caesar cipher by shifting in the opposite direction."
  (caesar-encrypt text (- shift)))

;;; Vigenere cipher

(defun alpha-p (ch)
  "Return T if CH is an ASCII letter."
  (or (upper-case-p ch) (lower-case-p ch)))

(defun vigenere-encrypt (text key)
  "Encrypt TEXT using Vigenere cipher with given KEY (alphabetic string)."
  (assert (and (> (length key) 0)
               (every #'alpha-p key))
          (key) "Key must be a non-empty alphabetic string")
  (let ((key-upper (string-upcase key))
        (ki 0))
    (map 'string
         (lambda (ch)
           (if (alpha-p ch)
               (let* ((shift (- (char-code (char key-upper (mod ki (length key-upper))))
                                (char-code #\A)))
                      (base (if (upper-case-p ch) (char-code #\A) (char-code #\a)))
                      (result (code-char (+ (mod (+ (- (char-code ch) base) shift) 26) base))))
                 (incf ki)
                 result)
               ch))
         text)))

(defun vigenere-decrypt (text key)
  "Decrypt Vigenere cipher by inverting each key letter's shift."
  (assert (and (> (length key) 0)
               (every #'alpha-p key))
          (key) "Key must be a non-empty alphabetic string")
  (let ((inverted (map 'string
                       (lambda (ch)
                         (code-char (+ (mod (- 26 (- (char-code (char-upcase ch))
                                                     (char-code #\A)))
                                            26)
                                       (char-code #\A))))
                       key)))
    (vigenere-encrypt text inverted)))

;;; Vernam (one-time pad) cipher

(defun vernam-encrypt (data key)
  "Encrypt DATA (list of bytes) using Vernam XOR cipher. KEY must be >= length of DATA."
  (assert (>= (length key) (length data))
          (key) "Key must be at least as long as the data")
  (mapcar #'logxor data (subseq key 0 (length data))))

(defun vernam-decrypt (data key)
  "Decrypt Vernam cipher (XOR is its own inverse)."
  (vernam-encrypt data key))
