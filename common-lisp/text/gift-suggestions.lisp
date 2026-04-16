;;;; Random Gift Suggestions — attribute-based gift recommendation.

(defpackage :text/gift-suggestions
  (:use :cl)
  (:export #:gift
           #:make-gift
           #:gift-name
           #:gift-category
           #:gift-price-range
           #:gift-suitable-for
           #:gift-suggester
           #:make-gift-suggester
           #:make-default-suggester
           #:add-gift
           #:suggest
           #:random-suggestion
           #:suggest-by-category
           #:gift-count))

(in-package :text/gift-suggestions)

;;; --- Data structures ---

(defstruct gift
  (name "" :type string)
  (category "" :type string)
  (price-range "" :type string)   ; "budget", "mid", "premium"
  (suitable-for nil :type list))  ; list of trait strings

(defstruct gift-suggester
  (gifts nil :type list))

;;; --- Default catalog ---

(defun load-default-gifts ()
  "Return a list of 24 pre-loaded gifts across categories."
  (list
   ;; Tech
   (make-gift :name "Mechanical Keyboard" :category "tech" :price-range "mid"
              :suitable-for '("techie" "creative"))
   (make-gift :name "Raspberry Pi Kit" :category "tech" :price-range "budget"
              :suitable-for '("techie" "creative"))
   (make-gift :name "Noise-Cancelling Headphones" :category "tech" :price-range "premium"
              :suitable-for '("techie" "music-lover"))
   (make-gift :name "Smart Home Starter Kit" :category "tech" :price-range "mid"
              :suitable-for '("techie"))
   ;; Books
   (make-gift :name "Leather-Bound Journal" :category "books" :price-range "mid"
              :suitable-for '("bookworm" "creative"))
   (make-gift :name "Complete Tolkien Collection" :category "books" :price-range "premium"
              :suitable-for '("bookworm" "adventurer"))
   (make-gift :name "Pocket Poetry Anthology" :category "books" :price-range "budget"
              :suitable-for '("bookworm" "creative"))
   (make-gift :name "Cookbook: World Cuisines" :category "books" :price-range "mid"
              :suitable-for '("bookworm" "foodie"))
   ;; Outdoor
   (make-gift :name "Hammock" :category "outdoor" :price-range "budget"
              :suitable-for '("adventurer"))
   (make-gift :name "Hiking Backpack" :category "outdoor" :price-range "mid"
              :suitable-for '("adventurer"))
   (make-gift :name "Camping Cookset" :category "outdoor" :price-range "mid"
              :suitable-for '("adventurer" "foodie"))
   (make-gift :name "Trail Running Shoes" :category "outdoor" :price-range "premium"
              :suitable-for '("adventurer"))
   ;; Cooking
   (make-gift :name "Cast Iron Skillet" :category "cooking" :price-range "budget"
              :suitable-for '("foodie"))
   (make-gift :name "Spice Collection Box" :category "cooking" :price-range "mid"
              :suitable-for '("foodie" "adventurer"))
   (make-gift :name "Chef's Knife Set" :category "cooking" :price-range "premium"
              :suitable-for '("foodie"))
   (make-gift :name "Pasta Maker" :category "cooking" :price-range "mid"
              :suitable-for '("foodie" "creative"))
   ;; Music
   (make-gift :name "Vinyl Record Starter Pack" :category "music" :price-range "budget"
              :suitable-for '("music-lover"))
   (make-gift :name "Concert Tickets" :category "music" :price-range "mid"
              :suitable-for '("music-lover" "adventurer"))
   (make-gift :name "MIDI Controller" :category "music" :price-range "mid"
              :suitable-for '("music-lover" "techie" "creative"))
   (make-gift :name "Turntable" :category "music" :price-range "premium"
              :suitable-for '("music-lover"))
   ;; Art
   (make-gift :name "Watercolor Set" :category "art" :price-range "budget"
              :suitable-for '("creative"))
   (make-gift :name "Drawing Tablet" :category "art" :price-range "mid"
              :suitable-for '("creative" "techie"))
   (make-gift :name "Museum Membership" :category "art" :price-range "mid"
              :suitable-for '("creative" "bookworm"))
   (make-gift :name "Oil Paint Master Set" :category "art" :price-range "premium"
              :suitable-for '("creative"))))

;;; --- Operations ---

(defun make-default-suggester ()
  "Create a gift suggester pre-loaded with the default catalog."
  (make-gift-suggester :gifts (load-default-gifts)))

(defun add-gift (suggester name category price-range &key (suitable-for nil))
  "Add a gift to the suggester. Returns the new gift."
  (let ((g (make-gift :name name :category category
                      :price-range price-range :suitable-for suitable-for)))
    (push g (gift-suggester-gifts suggester))
    g))

(defun gift-count (suggester)
  "Return the number of gifts in the catalog."
  (length (gift-suggester-gifts suggester)))

(defun relevance-score (traits gift)
  "Count how many of TRAITS match the gift's suitable-for list."
  (let ((trait-set (mapcar #'string-downcase traits)))
    (count-if (lambda (s) (member (string-downcase s) trait-set :test #'string=))
              (gift-suitable-for gift))))

(defun suggest (suggester traits &key budget)
  "Suggest gifts matching TRAITS, sorted by relevance (descending).
   If BUDGET is given, only gifts in that price range are included."
  (let* ((gifts (gift-suggester-gifts suggester))
         (filtered (if budget
                      (remove-if-not (lambda (g) (string-equal (gift-price-range g) budget))
                                     gifts)
                      gifts))
         (scored (remove-if (lambda (pair) (zerop (car pair)))
                            (mapcar (lambda (g) (cons (relevance-score traits g) g))
                                    filtered))))
    (mapcar #'cdr
            (sort scored #'> :key #'car))))

(defun random-suggestion (suggester)
  "Return a random gift, or NIL if the catalog is empty."
  (let ((gifts (gift-suggester-gifts suggester)))
    (when gifts
      (nth (random (length gifts)) gifts))))

(defun suggest-by-category (suggester category)
  "Return all gifts in CATEGORY (case-insensitive)."
  (let ((cat (string-downcase category)))
    (remove-if-not (lambda (g) (string-equal (string-downcase (gift-category g)) cat))
                   (gift-suggester-gifts suggester))))
