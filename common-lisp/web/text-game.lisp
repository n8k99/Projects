;;;; Text Based Game — room graph + player state + command dispatch.

(defpackage #:text-game
  (:use #:cl)
  (:export #:make-demo-world
           #:execute-command
           #:world-state #:world-log #:world-player #:world-rooms
           #:player-health #:player-gold #:player-location
           #:player-inventory #:player-equipped-damage
           #:current-room
           #:room-name #:room-description
           #:item-name #:item-kind
           #:enemy-name #:enemy-health))

(in-package #:text-game)

(defstruct item
  (id 0 :type integer) (name "" :type string)
  (kind :weapon :type keyword)
  (damage 0 :type integer) (heal 0 :type integer) (worth 0 :type integer))

(defstruct enemy
  (id 0 :type integer) (name "" :type string)
  (health 0 :type integer) (damage 0 :type integer))

(defstruct room-entry
  (id 0 :type integer) (name "" :type string) (description "" :type string)
  (exits nil :type list)             ; ((direction . room-id) ...)
  (items nil :type list) (enemies nil :type list))

(defstruct player
  (location 1 :type integer)
  (health 20 :type integer) (max-health 20 :type integer)
  (gold 0 :type integer)
  (inventory nil :type list)
  (equipped-weapon-id nil :type (or null integer))
  (equipped-damage 1 :type integer))

(defstruct world
  (rooms nil :type list)              ; alist ((id . room) ...)
  (player (make-player) :type player)
  (state :playing :type keyword)      ; :playing :dead :won
  (turn 0 :type integer)
  (win-gold 50 :type integer)
  (log nil :type list))

(defun make-demo-world ()
  (let* ((w (make-world))
         (r1 (make-room-entry :id 1 :name "Entrance Hall"
                              :description "A dusty hall with exits north and east."
                              :exits '(("north" . 2) ("east" . 3))
                              :items (list (make-item :id 100 :name "rusty sword"
                                                       :kind :weapon :damage 5)
                                            (make-item :id 101 :name "healing potion"
                                                       :kind :potion :heal 15))))
         (r2 (make-room-entry :id 2 :name "Damp Cavern"
                              :description "A cold cavern. Something shifts in the shadows."
                              :exits '(("south" . 1))
                              :enemies (list (make-enemy :id 200 :name "goblin"
                                                          :health 10 :damage 2))))
         (r3 (make-room-entry :id 3 :name "Treasure Room"
                              :description "A chalice rests on a pedestal. Exit west."
                              :exits '(("west" . 1))
                              :items (list (make-item :id 102 :name "gold chalice"
                                                       :kind :treasure :worth 50)))))
    (setf (world-rooms w) (list (cons 1 r1) (cons 2 r2) (cons 3 r3)))
    (setf (world-log w) '("You stand at the entrance of an old dungeon."))
    w))

(defun current-room (w) (cdr (assoc (player-location (world-player w)) (world-rooms w))))

(defun %log (w msg) (setf (world-log w) (append (world-log w) (list msg))))

(defparameter *provokes*
  '("go" "move" "north" "south" "east" "west" "n" "s" "e" "w"
    "take" "get" "use" "fight" "attack"))

(defparameter *dir-aliases*
  '(("n" . "north") ("s" . "south") ("e" . "east") ("w" . "west")))

(defun %split (s)
  (loop with parts = nil
        with start = nil
        for i from 0 below (length s)
        for c = (char s i)
        do (cond
             ((or (char= c #\Space) (char= c #\Tab))
              (when start (push (subseq s start i) parts) (setf start nil)))
             (t (unless start (setf start i))))
        finally (when start (push (subseq s start) parts))
                (return (nreverse parts))))

(defun execute-command (w cmd)
  (let ((before (length (world-log w))))
    (unless (eq (world-state w) :playing)
      (%log w "The game is over. Reset to play again.")
      (return-from execute-command (subseq (world-log w) before)))
    (let* ((parts (%split (string-downcase cmd))))
      (when (null parts) (return-from execute-command nil))
      (let* ((raw-verb (first parts))
             (verb raw-verb)
             (arg (format nil "~{~A~^ ~}" (rest parts))))
        (let ((alias (cdr (assoc verb *dir-aliases* :test #'string=))))
          (when alias (setf verb "go" arg alias)))
        (cond
          ((member verb '("north" "south" "east" "west") :test #'string=)
           (setf arg verb verb "go")))
        (cond
          ((string= verb "look") (%look w))
          ((string= verb "l") (%look w))
          ((or (string= verb "go") (string= verb "move")) (%go w arg))
          ((or (string= verb "take") (string= verb "get")) (%take w arg))
          ((string= verb "use") (%use w arg))
          ((or (string= verb "equip") (string= verb "wield")) (%equip w arg))
          ((or (string= verb "fight") (string= verb "attack")) (%fight w arg))
          ((or (string= verb "inventory") (string= verb "inv") (string= verb "i"))
           (%inventory w))
          ((string= verb "stats") (%stats w))
          (t (%log w (format nil "I don't understand '~A'." verb))))
        (when (member raw-verb *provokes* :test #'string=)
          (incf (world-turn w))
          (%enemies-strike w)
          (%check-end-conditions w))))
    (subseq (world-log w) before)))

(defun %look (w)
  (let ((r (current-room w)))
    (%log w (format nil "~A: ~A" (room-entry-name r) (room-entry-description r)))
    (when (room-entry-items r)
      (%log w (format nil "You see here: ~{~A~^, ~}"
                       (mapcar #'item-name (room-entry-items r)))))
    (when (room-entry-enemies r)
      (%log w (format nil "Enemies: ~{~A~^, ~}"
                       (mapcar (lambda (e) (format nil "~A (~Ahp)" (enemy-name e) (enemy-health e)))
                               (room-entry-enemies r)))))
    (%log w (format nil "Exits: ~{~A~^, ~}"
                     (sort (mapcar #'car (room-entry-exits r)) #'string<)))))

(defun %go (w direction)
  (when (zerop (length direction))
    (%log w "Go where?") (return-from %go))
  (let* ((r (current-room w))
         (dest (cdr (assoc direction (room-entry-exits r) :test #'string=))))
    (cond
      (dest (setf (player-location (world-player w)) dest)
            (%log w (format nil "You travel ~A. You are now in ~A."
                             direction (room-entry-name (current-room w)))))
      (t (%log w (format nil "You can't go ~A from here." direction))))))

(defun %find-by-name (list name-accessor query)
  (position-if (lambda (x) (search query (string-downcase (funcall name-accessor x))))
               list))

(defun %take (w name)
  (when (zerop (length name)) (%log w "Take what?") (return-from %take))
  (let* ((r (current-room w))
         (pos (%find-by-name (room-entry-items r) #'item-name name)))
    (cond
      (pos (let ((taken (nth pos (room-entry-items r))))
             (setf (room-entry-items r)
                   (append (subseq (room-entry-items r) 0 pos)
                           (subseq (room-entry-items r) (1+ pos))))
             (push taken (player-inventory (world-player w)))
             (setf (player-inventory (world-player w))
                   (reverse (player-inventory (world-player w))))
             (%log w (format nil "You take the ~A." (item-name taken)))))
      (t (%log w (format nil "There is no '~A' here." name))))))

(defun %use (w name)
  (when (zerop (length name)) (%log w "Use what?") (return-from %use))
  (let* ((p (world-player w))
         (pos (%find-by-name (player-inventory p) #'item-name name)))
    (unless pos (%log w (format nil "You don't have a '~A'." name)) (return-from %use))
    (let* ((item (nth pos (player-inventory p))))
      (case (item-kind item)
        (:potion
         (let ((before (player-health p)))
           (setf (player-health p) (min (player-max-health p) (+ (player-health p) (item-heal item))))
           (setf (player-inventory p)
                 (append (subseq (player-inventory p) 0 pos)
                         (subseq (player-inventory p) (1+ pos))))
           (%log w (format nil "You drink the ~A. Health ~A → ~A."
                            (item-name item) before (player-health p)))))
        (:treasure
         (incf (player-gold p) (item-worth item))
         (setf (player-inventory p)
               (append (subseq (player-inventory p) 0 pos)
                       (subseq (player-inventory p) (1+ pos))))
         (%log w (format nil "You pocket the ~A (+~A gold, total ~A)."
                          (item-name item) (item-worth item) (player-gold p))))
        (:weapon
         (setf (player-equipped-weapon-id p) (item-id item))
         (setf (player-equipped-damage p) (item-damage item))
         (%log w (format nil "You wield the ~A (~A damage)."
                          (item-name item) (item-damage item))))))))

(defun %equip (w name)
  (when (zerop (length name)) (%log w "Equip what?") (return-from %equip))
  (let* ((p (world-player w))
         (pos (%find-by-name (player-inventory p) #'item-name name)))
    (unless pos (%log w (format nil "You don't have a '~A'." name)) (return-from %equip))
    (let ((item (nth pos (player-inventory p))))
      (cond
        ((eq (item-kind item) :weapon)
         (setf (player-equipped-weapon-id p) (item-id item))
         (setf (player-equipped-damage p) (item-damage item))
         (%log w (format nil "You wield the ~A (~A damage)."
                          (item-name item) (item-damage item))))
        (t (%log w (format nil "The ~A is not a weapon." (item-name item))))))))

(defun %fight (w name)
  (let* ((r (current-room w))
         (p (world-player w))
         (pos (if (zerop (length name))
                  (if (room-entry-enemies r) 0 nil)
                  (%find-by-name (room-entry-enemies r) #'enemy-name name))))
    (unless pos (%log w (format nil "There's no '~A' to fight here." name)) (return-from %fight))
    (let* ((enemy (nth pos (room-entry-enemies r)))
           (dmg (player-equipped-damage p)))
      (decf (enemy-health enemy) dmg)
      (%log w (format nil "You hit the ~A for ~A damage (~Ahp remaining)."
                       (enemy-name enemy) dmg (max 0 (enemy-health enemy))))
      (when (<= (enemy-health enemy) 0)
        (setf (room-entry-enemies r)
              (append (subseq (room-entry-enemies r) 0 pos)
                      (subseq (room-entry-enemies r) (1+ pos))))
        (%log w (format nil "The ~A falls defeated." (enemy-name enemy)))))))

(defun %inventory (w)
  (let ((inv (player-inventory (world-player w))))
    (if (null inv)
        (%log w "Your pack is empty.")
        (%log w (format nil "You carry: ~{~A~^, ~}."
                         (mapcar #'item-name inv))))))

(defun %stats (w)
  (let* ((p (world-player w))
         (wpn (find (player-equipped-weapon-id p) (player-inventory p) :key #'item-id))
         (wpn-name (if wpn (item-name wpn) "bare hands")))
    (%log w (format nil "Turn ~A. Health ~A/~A. Gold ~A. Damage ~A (equipped: ~A)."
                     (world-turn w) (player-health p) (player-max-health p)
                     (player-gold p) (player-equipped-damage p) wpn-name))))

(defun %enemies-strike (w)
  (let* ((r (current-room w))
         (total (reduce #'+ (mapcar #'enemy-damage (room-entry-enemies r)))))
    (when (plusp total)
      (decf (player-health (world-player w)) total)
      (%log w (format nil "Enemies strike you for ~A damage (~A hp)!"
                       total (max 0 (player-health (world-player w))))))))

(defun %check-end-conditions (w)
  (cond
    ((<= (player-health (world-player w)) 0)
     (setf (world-state w) :dead)
     (%log w "You fall, lifeless. The dungeon is silent."))
    ((>= (player-gold (world-player w)) (world-win-gold w))
     (setf (world-state w) :won)
     (%log w (format nil "With ~A gold in hand, you've proven your fortune. You win!"
                      (player-gold (world-player w)))))))
