;;;; CD Burning App — multi-session disc model + FFD packing + ISO names.

(defpackage #:cd-burner
  (:use #:cl)
  (:export #:make-burn-file #:burn-file-name #:burn-file-size-bytes
           #:make-disc #:disc-capacity-bytes #:disc-sessions #:disc-state
           #:disc-used-bytes #:disc-remaining-bytes
           #:disc-open-session #:disc-add-file #:disc-close-session
           #:disc-finalize
           #:pack-files-ffd #:canonicalize-iso-name
           #:+leadin-bytes+ #:+leadout-bytes+))

(in-package #:cd-burner)

(defconstant +leadin-bytes+ 13000000)
(defconstant +leadout-bytes+ 22000000)

(defstruct burn-file
  (name "" :type string)
  (size-bytes 0 :type integer))

(defstruct session
  (tracks nil :type list)
  (state :open :type keyword))

(defun session-overhead-bytes (_s)
  (declare (ignore _s))
  (+ +leadin-bytes+ +leadout-bytes+))

(defun session-total-track-bytes (s)
  (reduce #'+ (session-tracks s) :key #'burn-file-size-bytes :initial-value 0))

(defun session-total-bytes (s)
  (+ (session-total-track-bytes s) (session-overhead-bytes s)))

(defstruct disc
  (capacity-bytes 0 :type integer)
  (sessions nil :type list)
  (state :blank :type keyword))

(defun disc-used-bytes (d)
  (reduce #'+ (disc-sessions d) :key #'session-total-bytes :initial-value 0))

(defun disc-remaining-bytes (d)
  (max 0 (- (disc-capacity-bytes d) (disc-used-bytes d))))

(define-condition burn-error (error)
  ((reason :initarg :reason :reader burn-error-reason)))

(defun disc-open-session (d)
  (case (disc-state d)
    (:finalized (error 'burn-error :reason :disc-finalized))
    (:session-open (error 'burn-error :reason :session-already-open))
    (otherwise
     (setf (disc-sessions d)
           (append (disc-sessions d) (list (make-session))))
     (setf (disc-state d) :session-open))))

(defun disc-add-file (d file)
  (when (eq (disc-state d) :finalized)
    (error 'burn-error :reason :disc-finalized))
  (unless (eq (disc-state d) :session-open)
    (error 'burn-error :reason :no-open-session))
  (let ((available (disc-remaining-bytes d)))
    (when (> (burn-file-size-bytes file) available)
      (error 'burn-error :reason :not-enough-space)))
  (let ((last-session (car (last (disc-sessions d)))))
    (setf (session-tracks last-session)
          (append (session-tracks last-session) (list file)))))

(defun disc-close-session (d)
  (when (eq (disc-state d) :finalized)
    (error 'burn-error :reason :disc-finalized))
  (unless (eq (disc-state d) :session-open)
    (error 'burn-error :reason :no-open-session))
  (setf (session-state (car (last (disc-sessions d)))) :closed)
  (setf (disc-state d) :session-closed))

(defun disc-finalize (d)
  (when (eq (disc-state d) :finalized)
    (error 'burn-error :reason :disc-finalized))
  (when (eq (disc-state d) :session-open)
    (disc-close-session d))
  (setf (disc-state d) :finalized))

(defun pack-files-ffd (files disc-capacity num-discs)
  "Returns (:discs <list-of-disc-file-lists> :overflow <list>)."
  (let* ((sorted (sort (copy-list files) #'> :key #'burn-file-size-bytes))
         (usable (max 0 (- disc-capacity (+ +leadin-bytes+ +leadout-bytes+))))
         (discs (make-array num-discs :initial-element nil))
         (remaining (make-array num-discs :initial-element usable))
         (overflow nil))
    (dolist (f sorted)
      (let ((placed nil))
        (loop for i from 0 below num-discs
              unless placed do
              (when (<= (burn-file-size-bytes f) (aref remaining i))
                (decf (aref remaining i) (burn-file-size-bytes f))
                (setf (aref discs i) (append (aref discs i) (list f)))
                (setf placed t)))
        (unless placed (push f overflow))))
    (list :discs (coerce discs 'list)
          :overflow (nreverse overflow))))

(defun clean-chars (s)
  (with-output-to-string (out)
    (loop for c across s do
          (let ((uc (char-upcase c)))
            (when (or (alphanumericp uc) (char= uc #\_))
              (write-char uc out))))))

(defun split-ext (name)
  (let ((i (position #\. name :from-end t)))
    (if (and i (> i 0))
        (values (subseq name 0 i) (subseq name (1+ i)))
        (values name ""))))

(defun canonicalize-iso-name (name existing-set)
  "existing-set is a hash-table: name -> t."
  (multiple-value-bind (stem ext) (split-ext name)
    (let* ((stem-clean (clean-chars stem))
           (ext-clean (clean-chars ext))
           (ext-trunc (subseq ext-clean 0 (min 3 (length ext-clean))))
           (stem-base (subseq stem-clean 0 (min 8 (length stem-clean))))
           (base (if (zerop (length ext-trunc))
                     stem-base
                     (concatenate 'string stem-base "." ext-trunc))))
      (if (null (gethash base existing-set))
          base
          (loop for n from 1 to 999
                for suffix = (format nil "~~~D" n)
                for room = (max 0 (- 8 (length suffix)))
                for short = (subseq stem-clean 0 (min room (length stem-clean)))
                for candidate = (if (zerop (length ext-trunc))
                                    (concatenate 'string short suffix)
                                    (concatenate 'string short suffix "." ext-trunc))
                unless (gethash candidate existing-set)
                return candidate
                finally (return base))))))

(defun demo ()
  (let ((d (make-disc :capacity-bytes 700000000)))
    (disc-open-session d)
    (disc-add-file d (make-burn-file :name "album1.flac" :size-bytes 100000000))
    (disc-add-file d (make-burn-file :name "album2.flac" :size-bytes 150000000))
    (format t "after 2 files: used=~D remaining=~D~%"
            (disc-used-bytes d) (disc-remaining-bytes d))
    (disc-close-session d)
    (format t "state: ~A~%" (disc-state d))
    (disc-open-session d)
    (disc-add-file d (make-burn-file :name "bonus.mp3" :size-bytes 50000000))
    (format t "after session 2: sessions=~D used=~D~%"
            (length (disc-sessions d)) (disc-used-bytes d))
    (disc-finalize d)
    (format t "final state: ~A~%" (disc-state d)))

  (let* ((files (list (make-burn-file :name "big.iso" :size-bytes 500000000)
                       (make-burn-file :name "mid.mp4" :size-bytes 200000000)
                       (make-burn-file :name "small.txt" :size-bytes 1000000)))
         (result (pack-files-ffd files 700000000 2))
         (discs (getf result :discs)))
    (loop for i from 0 below (length discs) do
          (format t "disc ~D: (~{~A~^ ~})~%" i
                  (mapcar #'burn-file-name (nth i discs))))
    (format t "overflow: (~{~A~^ ~})~%"
            (mapcar #'burn-file-name (getf result :overflow))))

  (let ((existing (make-hash-table :test 'equal)))
    (dolist (name '("Report.Document.Txt" "report.txt" "report.txt" "my file@2026!.txt"))
      (let ((c (canonicalize-iso-name name existing)))
        (setf (gethash c existing) t)
        (format t "  ~A -> ~A~%" name c)))))
