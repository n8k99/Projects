---
id: G029
title: Random Gift Suggestions
domain: text
type: rosetta-stone
status: active
depends_on: [G015, G023, G024, G026]
concepts:
  - attribute-based matching
  - relevance scoring
  - profile matching
  - constrained ranking
  - fuzzy selection
  - agent discovery
---

# G029 — Random Gift Suggestions

A recommendation engine that suggests gifts based on recipient attributes. Unlike exact search (G023) or exact category filter (G026), matching here is fuzzy: a gift matching 3 of 4 traits ranks higher than one matching 1 of 4.

## Insight: Attribute-Based Matching Is the First Recommendation Primitive

Gift suggestion is ATTRIBUTE-BASED MATCHING — the first recommendation engine in the Rosetta Stone. Unlike search (G023, exact substring) or filter (G026, exact category), matching is FUZZY: a gift that matches 3/4 traits is better than one matching 1/4. Relevance scoring — count matching attributes, sort by score — is the primitive that powers every recommendation system. The `where` clause selects; the `sort` clause ranks. Selection is boolean; ranking is continuous. The leap from "does it match?" to "how well does it match?" is the leap from search to recommendation.

## Insight: Traits Are a Profile — Profile Matching Is Agent Discovery

Traits are a PROFILE — a set of attributes that describe an entity. The recipient has traits. The gift has compatible traits. Matching profiles is how agents find each other in the noosphere: an agent with capability X matches a task requiring capability X. Gift suggestion is agent-task matching in disguise. Sarah needs a tool for scheduling — she describes her needs (traits), the agent catalog is searched by profile matching, the best-fit agent is suggested. The gift catalog is the agent registry. The recipient's traits are the task requirements. `suggest()` is `discover-agent()`.

## Insight: Budget Constraint Is a Filter on Top of Ranking

Budget constraint is a FILTER ON TOP OF RANKING — first score by relevance, then filter by budget. This is the pattern from G015 (Dijkstra with constraints): optimize within bounds. The `where` scores the match; the budget is a `<-` gate that filters before presentation. Two operations compose: score, then gate. The ranking doesn't know about the budget. The budget doesn't know about the ranking. But the composition — rank then filter — is the universal pattern for "best within constraints."

## Choreographic Case: Agent Discovery via Capability Profiles

Agents recommending resources to each other based on capability profiles. Sarah needs a tool for scheduling — she describes her needs (traits), the agent catalog is searched by profile matching, the best-fit agent is suggested. Gift suggestion IS agent discovery. The recipient is the requester. The gift catalog is the agent registry. The traits are the capability requirements. The relevance score is the fit metric. The budget is the resource constraint. The suggestion is the dispatch decision.

## The Shape

```innate
(define-shape gift-suggester
  "Attribute-based gift recommendation engine."

  ;; A gift: name, category, price range, and compatible traits
  (define-record gift
    (name         :type string)
    (category     :type string)     ; tech, books, outdoor, cooking, music, art
    (price-range  :type string)     ; budget, mid, premium
    (suitable-for :type (list-of string)))  ; traits like "techie", "bookworm", etc.

  ;; The catalog
  (define-state catalog :type (list-of gift)
    :default (load-default-catalog))

  ;; Pre-loaded defaults: 24 gifts across 6 categories
  (define-method load-default-catalog ()
    (list
      ;; Tech
      (gift "Mechanical Keyboard" "tech" "mid" '("techie" "creative"))
      (gift "Raspberry Pi Kit" "tech" "budget" '("techie" "creative"))
      (gift "Noise-Cancelling Headphones" "tech" "premium" '("techie" "music-lover"))
      (gift "Smart Home Starter Kit" "tech" "mid" '("techie"))
      ;; Books
      (gift "Leather-Bound Journal" "books" "mid" '("bookworm" "creative"))
      (gift "Complete Tolkien Collection" "books" "premium" '("bookworm" "adventurer"))
      (gift "Pocket Poetry Anthology" "books" "budget" '("bookworm" "creative"))
      (gift "Cookbook: World Cuisines" "books" "mid" '("bookworm" "foodie"))
      ;; Outdoor
      (gift "Hammock" "outdoor" "budget" '("adventurer"))
      (gift "Hiking Backpack" "outdoor" "mid" '("adventurer"))
      (gift "Camping Cookset" "outdoor" "mid" '("adventurer" "foodie"))
      (gift "Trail Running Shoes" "outdoor" "premium" '("adventurer"))
      ;; Cooking
      (gift "Cast Iron Skillet" "cooking" "budget" '("foodie"))
      (gift "Spice Collection Box" "cooking" "mid" '("foodie" "adventurer"))
      (gift "Chef's Knife Set" "cooking" "premium" '("foodie"))
      (gift "Pasta Maker" "cooking" "mid" '("foodie" "creative"))
      ;; Music
      (gift "Vinyl Record Starter Pack" "music" "budget" '("music-lover"))
      (gift "Concert Tickets" "music" "mid" '("music-lover" "adventurer"))
      (gift "MIDI Controller" "music" "mid" '("music-lover" "techie" "creative"))
      (gift "Turntable" "music" "premium" '("music-lover"))
      ;; Art
      (gift "Watercolor Set" "art" "budget" '("creative"))
      (gift "Drawing Tablet" "art" "mid" '("creative" "techie"))
      (gift "Museum Membership" "art" "mid" '("creative" "bookworm"))
      (gift "Oil Paint Master Set" "art" "premium" '("creative"))))

  ;; Relevance scoring: count matching traits
  (define-method relevance-score (traits gift)
    "Count how many of TRAITS appear in the gift's suitable-for list."
    (let ((trait-set (map downcase traits)))
      (count (lambda (s) (member? (downcase s) trait-set))
             (suitable-for gift))))

  ;; Core operation: suggest by traits with optional budget constraint
  (define-method suggest (traits &optional budget)
    "Suggest gifts matching TRAITS, sorted by relevance (descending).
     If BUDGET is given, filter to that price range first."
    (let* ((filtered (if budget
                        (where (lambda (g) (equal? (price-range g) budget))
                               catalog)
                        catalog))
           (scored (map (lambda (g) (cons (relevance-score traits g) g))
                        filtered))
           (nonzero (where (lambda (pair) (> (car pair) 0)) scored))
           (sorted (sort nonzero > :key car)))
      (map cdr sorted)))

  ;; Random suggestion — unconstrained nondeterminism from G024
  (define-method random-suggestion ()
    "Return a random gift from the catalog."
    (if (null catalog)
        nil
        (nth (random (length catalog)) catalog)))

  ;; Category filter — exact match from G026
  (define-method suggest-by-category (category)
    "Return all gifts in CATEGORY."
    (where (lambda (g) (equal? (downcase (category g))
                               (downcase category)))
           catalog))

  ;; Add a new gift to the catalog
  (define-method add-gift (name category price-range suitable-for)
    "Add a gift to the catalog."
    (let ((g (gift name category price-range suitable-for)))
      (push! g catalog)
      g)))
```

## Resolution Protocol

```innate
;; Gift suggestion resolves through profile matching:
;; 1. Build trait set from recipient description
;; 2. Score each gift by trait intersection count
;; 3. Apply budget gate (filter, not rank)
;; 4. Sort by score descending
;; 5. Present top matches

(resolve gift-suggester
  (where (suggest '("techie" "bookworm") :budget "mid"))
  ;; => Mechanical Keyboard (2 matches), Leather-Bound Journal (1), ...

  ;; The same pattern generalizes to agent discovery:
  (where (discover-agent '("scheduling" "calendar" "reminder"))
         :constraint (< cost budget))
  ;; => CalendarAgent (3 matches), ReminderBot (1), ...
)
```
