;;;; RPG Character Stat Creator — class templates + derived stats + level progression.

(defpackage #:rpg-character
  (:use #:cl)
  (:export #:roll-rpg-char #:rpg-char-adjusted #:rpg-char-derived
           #:level-up #:spend-point #:to-text
           #:rpg-char-name #:rpg-char-class #:rpg-char-level
           #:rpg-char-base #:rpg-char-unspent-points
           #:derived-max-hp #:derived-max-mp #:derived-armour
           #:derived-carry-capacity #:derived-crit-chance))

(in-package #:rpg-character)

(defparameter *all-stats* '(:strength :dexterity :intellect :stamina :luck))

(defstruct rpg-char
  (name "" :type string)
  (class :warrior :type keyword)
  (level 1 :type integer)
  (base nil :type list)
  (unspent-points 0 :type integer))

(defstruct derived
  (max-hp 0 :type integer)
  (max-mp 0 :type integer)
  (armour 0 :type integer)
  (carry-capacity 0 :type integer)
  (crit-chance 0.0 :type single-float))

(defun multipliers (class)
  (let ((m (mapcar (lambda (s) (cons s 1.0)) *all-stats*)))
    (case class
      (:warrior
       (setf (cdr (assoc :strength m)) 1.5)
       (setf (cdr (assoc :stamina m)) 1.3))
      (:mage
       (setf (cdr (assoc :intellect m)) 1.5)
       (setf (cdr (assoc :stamina m)) 0.8))
      (:rogue
       (setf (cdr (assoc :dexterity m)) 1.4)
       (setf (cdr (assoc :luck m)) 1.3)))
    m))

(defun max-per-stat (class)
  (declare (ignore class))
  99)

(defstruct lcg
  (state 0 :type integer))

(defun make-lcg-seed (seed)
  (make-lcg :state (mod (1+ seed) (expt 2 64))))

(defun lcg-next-u32 (rng)
  (setf (lcg-state rng)
        (mod (+ (* (lcg-state rng) 6364136223846793005)
                 1442695040888963407)
             (expt 2 64)))
  (ldb (byte 32 33) (lcg-state rng)))

(defun lcg-next-in (rng lo range)
  (+ lo (mod (lcg-next-u32 rng) range)))

(defun roll-rpg-char (name class seed)
  (let ((rng (make-lcg-seed seed))
        (base nil))
    (dolist (stat *all-stats*)
      (push (cons stat (+ 3 (lcg-next-in rng 0 16))) base))
    (make-rpg-char :name name :class class :level 1
                     :base (nreverse base) :unspent-points 0)))

(defun rpg-char-adjusted (c stat)
  (let* ((b (cdr (assoc stat (rpg-char-base c))))
         (m (cdr (assoc stat (multipliers (rpg-char-class c))))))
    (floor (* b m))))

(defun rpg-char-derived (c)
  (let ((str (rpg-char-adjusted c :strength))
        (int (rpg-char-adjusted c :intellect))
        (sta (rpg-char-adjusted c :stamina))
        (dex (rpg-char-adjusted c :dexterity))
        (luk (rpg-char-adjusted c :luck))
        (lvl (rpg-char-level c)))
    (make-derived
     :max-hp (+ 10 (* sta 5) (* lvl 10))
     :max-mp (+ (* int 3) (* lvl 5))
     :armour (+ (floor str 3) (floor sta 5))
     :carry-capacity (+ 10 (* str 2))
     :crit-chance (coerce (/ (+ (* luk 0.5) (* dex 0.2)) 100.0) 'single-float))))

(defun level-up (c)
  (incf (rpg-char-level c))
  (incf (rpg-char-unspent-points c) 5))

(defun spend-point (c stat points)
  (when (> points (rpg-char-unspent-points c))
    (error "not enough points: have ~D, need ~D"
           (rpg-char-unspent-points c) points))
  (let* ((current (cdr (assoc stat (rpg-char-base c))))
         (new-val (+ current points))
         (cap (max-per-stat (rpg-char-class c))))
    (when (> new-val cap)
      (error "stat cap exceeded: ~D > ~D" new-val cap))
    (setf (cdr (assoc stat (rpg-char-base c))) new-val)
    (decf (rpg-char-unspent-points c) points)))

(defun stat-name (stat)
  (string-downcase (symbol-name stat)))

(defun rpg-class-name (class)
  (string-downcase (symbol-name class)))

(defun to-text (c)
  (with-output-to-string (out)
    (format out "name: ~A~%" (rpg-char-name c))
    (format out "class: ~A~%" (rpg-class-name (rpg-char-class c)))
    (format out "level: ~D~%" (rpg-char-level c))
    (format out "unspent: ~D~%" (rpg-char-unspent-points c))
    (dolist (stat *all-stats*)
      (let ((val (cdr (assoc stat (rpg-char-base c)))))
        (when val
          (format out "stat:~A=~D~%" (stat-name stat) val))))))

(defun demo ()
  (let ((c (roll-rpg-char "Rook" :warrior 42)))
    (format t "=== ~A the ~A ===~%"
            (rpg-char-name c) (rpg-class-name (rpg-char-class c)))
    (dolist (s *all-stats*)
      (format t "  ~10A base=~2D adj=~2D~%"
              (stat-name s)
              (cdr (assoc s (rpg-char-base c)))
              (rpg-char-adjusted c s)))
    (let ((d (rpg-char-derived c)))
      (format t "~%  HP: ~D MP: ~D Armour: ~D~%"
              (derived-max-hp d) (derived-max-mp d) (derived-armour d))
      (format t "  Carry: ~D Crit: ~,4F~%"
              (derived-carry-capacity d) (derived-crit-chance d)))
    (level-up c) (level-up c)
    (spend-point c :strength 5)
    (spend-point c :stamina 3)
    (let ((d (rpg-char-derived c)))
      (format t "~%=== After 2 levels + spend ===~%")
      (format t "  level=~D unspent=~D~%"
              (rpg-char-level c) (rpg-char-unspent-points c))
      (format t "  STR=~D STA=~D~%"
              (cdr (assoc :strength (rpg-char-base c)))
              (cdr (assoc :stamina (rpg-char-base c))))
      (format t "  HP: ~D Armour: ~D~%" (derived-max-hp d) (derived-armour d)))))
