;;;; Quiz Maker — polymorphic questions with file round-trip serialisation.

(defpackage #:quiz-maker
  (:use #:cl)
  (:export #:make-quiz #:add-mc-question #:add-tf-question #:add-sa-question
           #:to-text #:from-text #:score-attempt
           #:quiz-id #:quiz-title #:quiz-questions))

(in-package #:quiz-maker)

(defstruct question
  (kind :mc :type keyword)        ; :mc :tf :sa
  (prompt "" :type string)
  (options nil :type list)
  (correct-index 0 :type integer)
  (correct-bool nil :type boolean)
  (expected "" :type string))

(defstruct quiz
  (id 0 :type integer)
  (title "" :type string)
  (questions nil :type list))

(defun add-mc-question (q prompt options correct-index)
  (setf (quiz-questions q)
        (append (quiz-questions q)
                (list (make-question :kind :mc :prompt prompt
                                      :options options :correct-index correct-index)))))

(defun add-tf-question (q prompt correct)
  (setf (quiz-questions q)
        (append (quiz-questions q)
                (list (make-question :kind :tf :prompt prompt :correct-bool correct)))))

(defun add-sa-question (q prompt expected)
  (setf (quiz-questions q)
        (append (quiz-questions q)
                (list (make-question :kind :sa :prompt prompt :expected expected)))))

(defun to-text (q)
  (with-output-to-string (s)
    (format s "QUIZ~%id: ~A~%title: ~A~%" (quiz-id q) (quiz-title q))
    (dolist (qn (quiz-questions q))
      (format s "---~%")
      (case (question-kind qn)
        (:mc
         (format s "kind: mc~%prompt: ~A~%" (question-prompt qn))
         (dolist (opt (question-options qn)) (format s "option: ~A~%" opt))
         (format s "correct: ~A~%" (question-correct-index qn)))
        (:tf
         (format s "kind: tf~%prompt: ~A~%correct: ~A~%"
                 (question-prompt qn)
                 (if (question-correct-bool qn) "true" "false")))
        (:sa
         (format s "kind: sa~%prompt: ~A~%expected: ~A~%"
                 (question-prompt qn) (question-expected qn)))))))

(defun %split-lines (text)
  (let ((lines nil) (start 0))
    (loop for i from 0 below (length text) do
      (when (char= (char text i) #\Newline)
        (push (subseq text start i) lines)
        (setf start (1+ i))))
    (when (< start (length text))
      (push (subseq text start) lines))
    (nreverse lines)))

(defun %field-value (line prefix)
  (when (and (>= (length line) (length prefix))
             (string= (subseq line 0 (length prefix)) prefix))
    (subseq line (length prefix))))

(defun from-text (text)
  (let ((lines (%split-lines text)))
    (unless (string= (first lines) "QUIZ") (error "missing QUIZ header"))
    (let* ((id-str (%field-value (second lines) "id: "))
           (id (parse-integer id-str))
           (title (%field-value (third lines) "title: "))
           (rest-lines (cdddr lines))
           (q (make-quiz :id id :title title)))
      (loop while rest-lines do
        (unless (string= (first rest-lines) "---")
          (error "expected --- got ~A" (first rest-lines)))
        (setf rest-lines (rest rest-lines))
        (let ((block-lines nil))
          (loop while (and rest-lines (not (string= (first rest-lines) "---")))
                do (push (first rest-lines) block-lines)
                   (setf rest-lines (rest rest-lines)))
          (add-question-from-block q (nreverse block-lines))))
      q)))

(defun add-question-from-block (q block)
  (let ((fields (make-hash-table :test 'equal))
        (options nil))
    (dolist (line block)
      (let ((opt (%field-value line "option: ")))
        (if opt
            (push opt options)
            (let ((colon-pos (search ": " line)))
              (when colon-pos
                (setf (gethash (subseq line 0 colon-pos) fields)
                      (subseq line (+ colon-pos 2))))))))
    (let ((kind (gethash "kind" fields)))
      (cond
        ((string= kind "mc")
         (add-mc-question q (gethash "prompt" fields)
                           (nreverse options)
                           (parse-integer (gethash "correct" fields))))
        ((string= kind "tf")
         (add-tf-question q (gethash "prompt" fields)
                           (string= (gethash "correct" fields) "true")))
        ((string= kind "sa")
         (add-sa-question q (gethash "prompt" fields)
                           (gethash "expected" fields)))
        (t (error "unknown kind: ~A" kind))))))

(defun check-answer (q ans)
  (and ans (eq (first ans) (question-kind q))
       (case (question-kind q)
         (:mc (= (second ans) (question-correct-index q)))
         (:tf (eq (second ans) (question-correct-bool q)))
         (:sa (string-equal
               (string-trim '(#\Space) (second ans))
               (string-trim '(#\Space) (question-expected q)))))))

(defun score-attempt (q answers)
  "ANSWERS is a list of answer-tuples (kind value) or NIL for skipped."
  (let ((total (length (quiz-questions q)))
        (correct 0) (skipped 0) (wrong 0))
    (loop for qn in (quiz-questions q)
          for ans = (pop answers) do
      (cond
        ((null ans) (incf skipped))
        ((check-answer qn ans) (incf correct))
        (t (incf wrong))))
    (list :total total :correct correct :skipped skipped :wrong wrong)))
