;;;; CAPTCHA Maker — challenge-response with expiration and attempt limits.

(defpackage #:captcha
  (:use #:cl)
  (:export #:make-service #:issue #:verify #:cleanup
           #:challenge-id #:challenge-prompt #:challenge-expected
           #:challenge-kind #:challenge-solved
           #:result-kind #:result-attempts-remaining
           #:active-count))

(in-package #:captcha)

(defstruct challenge
  (id 0 :type integer)
  (kind :math :type keyword)
  (prompt "" :type string)
  (expected "" :type string)
  (created-ms 0 :type integer)
  (expires-ms 0 :type integer)
  (attempts-used 0 :type integer)
  (max-attempts 3 :type integer)
  (solved nil :type boolean))

(defstruct result
  (kind :unknown :type keyword)
  (attempts-remaining nil :type (or null integer)))

(defstruct service
  (challenges nil :type list)
  (next-id 1 :type integer)
  (default-ttl-ms 60000 :type integer)
  (max-attempts 3 :type integer)
  (rng-state 42 :type integer))

(defun make-service-with (ttl max-att seed)
  (make-service :default-ttl-ms ttl :max-attempts max-att :rng-state seed))

(defun %next-random (s)
  (setf (service-rng-state s)
        (logand (+ (* (service-rng-state s) 6364136223846793005)
                   1442695040888963407)
                #xFFFFFFFFFFFFFFFF))
  (ash (service-rng-state s) -32))

(defun %generate (s kind)
  (case kind
    (:math
     (let ((a (1+ (mod (%next-random s) 20)))
           (b (1+ (mod (%next-random s) 20)))
           (op (mod (%next-random s) 3)))
       (case op
         (0 (values (format nil "What is ~A + ~A?" a b) (format nil "~A" (+ a b))))
         (1 (let ((hi (max a b)) (lo (min a b)))
              (values (format nil "What is ~A - ~A?" hi lo) (format nil "~A" (- hi lo)))))
         (2 (values (format nil "What is ~A × ~A?" a b) (format nil "~A" (* a b)))))))
    (:word-recall
     (let* ((words #("butterfly" "chronicle" "lighthouse" "symphony" "mountain"))
            (w (aref words (mod (%next-random s) (length words)))))
       (values (format nil "Type this word: ~A" w) w)))
    (:reverse-string
     (let* ((inputs #("hello" "rosetta" "captcha" "innate"))
            (w (aref inputs (mod (%next-random s) (length inputs))))
            (rev (reverse w)))
       (values (format nil "Reverse this: ~A" w) rev)))
    (:sequence-count
     (let ((length (+ 5 (mod (%next-random s) 6)))
           (seq "") (digits 0))
       (loop repeat length do
         (let ((r (mod (%next-random s) 36)))
           (if (< r 10)
               (progn (setf seq (format nil "~A~A" seq r)) (incf digits))
               (setf seq (format nil "~A~A" seq (code-char (+ (char-code #\A) (- r 10))))))))
       (values (format nil "How many digits in: ~A?" seq) (format nil "~A" digits))))))

(defun issue (s kind now-ms)
  (let ((id (service-next-id s)))
    (incf (service-next-id s))
    (multiple-value-bind (prompt expected) (%generate s kind)
      (let ((c (make-challenge :id id :kind kind :prompt prompt :expected expected
                                :created-ms now-ms
                                :expires-ms (+ now-ms (service-default-ttl-ms s))
                                :max-attempts (service-max-attempts s))))
        (setf (service-challenges s) (append (service-challenges s) (list c)))
        c))))

(defun %answers-match (expected user)
  (string-equal (string-trim '(#\Space #\Tab #\Newline #\Return) expected)
                (string-trim '(#\Space #\Tab #\Newline #\Return) user)))

(defun verify (s id user-answer now-ms)
  (let ((c (find id (service-challenges s) :key #'challenge-id)))
    (cond
      ((null c) (make-result :kind :unknown))
      ((challenge-solved c) (make-result :kind :already-solved))
      ((> now-ms (challenge-expires-ms c)) (make-result :kind :expired))
      ((>= (challenge-attempts-used c) (challenge-max-attempts c))
       (make-result :kind :too-many-attempts))
      (t
       (incf (challenge-attempts-used c))
       (cond
         ((%answers-match (challenge-expected c) user-answer)
          (setf (challenge-solved c) t)
          (make-result :kind :success))
         (t
          (make-result :kind :wrong-answer
                       :attempts-remaining (- (challenge-max-attempts c)
                                              (challenge-attempts-used c)))))))))

(defun cleanup (s now-ms)
  (let ((before (length (service-challenges s))))
    (setf (service-challenges s)
          (remove-if (lambda (c)
                       (or (challenge-solved c)
                           (> now-ms (challenge-expires-ms c))))
                     (service-challenges s)))
    (- before (length (service-challenges s)))))

(defun active-count (s) (length (service-challenges s)))
