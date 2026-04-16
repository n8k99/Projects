;;;; Credit Card Validator — Luhn algorithm and network identification.

(defpackage :numbers/credit-card
  (:use :cl)
  (:export :luhn-check :identify-network :validate-card))

(in-package :numbers/credit-card)

(defun digits-only (number)
  "Extract only digit characters from NUMBER string, return as list of integers."
  (loop for c across number
        when (digit-char-p c)
        collect (digit-char-p c)))

(defun luhn-check (number)
  "Validate a credit card number using the Luhn algorithm."
  (let ((digits (reverse (digits-only number))))
    (when (< (length digits) 2)
      (return-from luhn-check nil))
    (let ((total 0))
      (loop for i from 0
            for d in digits
            do (let ((v (if (oddp i)
                           (let ((doubled (* d 2)))
                             (if (> doubled 9) (- doubled 9) doubled))
                           d)))
                 (incf total v)))
      (zerop (mod total 10)))))

(defun digits-string (number)
  "Return only the digit characters of NUMBER as a string."
  (remove-if-not #'digit-char-p number))

(defun prefix-int (s len)
  "Parse the first LEN characters of string S as an integer."
  (parse-integer (subseq s 0 (min len (length s)))))

(defun identify-network (number)
  "Identify the card network by prefix and length.
Returns \"Visa\", \"Mastercard\", \"Amex\", \"Discover\", or NIL."
  (let* ((digits (digits-string number))
         (n (length digits)))
    (when (< n 2)
      (return-from identify-network nil))
    ;; Amex: 34xx or 37xx, length 15
    (when (and (= n 15)
               (let ((p2 (subseq digits 0 2)))
                 (or (string= p2 "34") (string= p2 "37"))))
      (return-from identify-network "Amex"))
    ;; Visa: starts with 4, length 13 or 16
    (when (and (char= (char digits 0) #\4)
               (or (= n 13) (= n 16)))
      (return-from identify-network "Visa"))
    ;; Length 16 networks
    (when (= n 16)
      (let ((p2 (prefix-int digits 2))
            (p4 (prefix-int digits 4)))
        ;; Mastercard: 51-55 or 2221-2720
        (when (or (<= 51 p2 55) (<= 2221 p4 2720))
          (return-from identify-network "Mastercard"))
        ;; Discover: 6011, 644-649, 65xx
        (when (string= (subseq digits 0 4) "6011")
          (return-from identify-network "Discover"))
        (let ((p3 (prefix-int digits 3)))
          (when (or (<= 644 p3 649) (string= (subseq digits 0 2) "65"))
            (return-from identify-network "Discover")))))
    nil))

(defun validate-card (number)
  "Validate a credit card number. Returns a plist with :valid, :network, :display."
  (let* ((digits (digits-string number))
         (n (length digits))
         (last4 (if (>= n 4) (subseq digits (- n 4)) digits))
         (display (concatenate 'string "****" last4))
         (network (identify-network digits))
         (valid (and (luhn-check digits) (not (null network)))))
    (list :valid valid :network network :display display)))
