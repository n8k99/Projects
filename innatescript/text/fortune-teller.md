---
id: G027
title: Fortune Teller
domain: text
type: rosetta-stone
status: active
depends_on: [G013, G024]
concepts:
  - constrained nondeterminism
  - stratified sampling
  - context-independent response
  - categorized pools
  - curated selection
---

# G027 — Fortune Teller

A fortune/prediction generator that produces randomized wisdom from categorized pools.

## Insight: Selection from a Curated Pool

The fortune teller extends G024's `random_quote()` with categorization. The nondeterminism from G024 returns, but now with structure: random selection constrained by category. This is **constrained nondeterminism** — the agent doesn't choose anything, it chooses from a defined space. The fortune pool is the boundary of what can be said. Agency without constraint is chaos; constraint without agency is determinism. The fortune teller lives in the productive middle: nondeterminism within bounds.

## Insight: Context-Independent Response

`ask_question()` demonstrates **context-independent response** — the question doesn't influence the answer. The fortune teller acknowledges the question but responds from its pool regardless. This is the simplest model of an agent that listens but doesn't truly respond to input — it has its own agenda (the fortune pool) independent of the query. This is the anti-pattern for good agent design, but naming it helps: agents that ignore context are fortune tellers.

## Insight: Stratified Sampling

Categorized pools are **stratified sampling** — random selection within a stratum. The resolver's nondeterminism needs to support constraints: "random, but from category X." This is different from unrestricted random (G024) and deterministic dispatch (G013). It's random within bounds. The category is the stratum, the pool is the population, the fortune is the sample. Three levels of the same concept: the space, the subset, the selection.

## Choreographic Case: Daily Inspiration System

Each morning, the temporal alarm (G011) triggers a fortune selection. The category rotates by day of week (motivation Monday, wisdom Wednesday). The fortune is injected into the daily note. This is nondeterminism + scheduling + content insertion — three Rosetta Stone concepts composing. The fortune teller doesn't know about calendars. The alarm doesn't know about fortunes. The daily note doesn't know about either. But the choreography composes them: alarm fires, category is derived from day, fortune is selected, note is updated. Each component is simple; the composition is rich.

## The Shape

```innate
(define-shape fortune-teller
  "A fortune/prediction generator with categorized pools of wisdom."

  ;; The pools: category -> list of fortune strings
  (define-state pools :type (hash-map string (list-of string))
    :default (load-default-pools))

  ;; Pre-loaded defaults
  (define-method load-default-pools ()
    (hash-map
      "wisdom"     '("The obstacle is the path."
                     "What you resist persists."
                     "Still water runs deep."
                     "The map is not the territory."
                     "Every expert was once a beginner.")
      "humor"      '("You will step on a Lego in the dark tonight."
                     "A closed mouth gathers no foot."
                     "Today is a good day to avoid making eye contact."
                     "Your socks will never match again.")
      "warning"    '("Beware the shortcut that saves no time."
                     "Not every open door is an invitation."
                     "The comfortable path leads to the boring destination."
                     "Trust your instincts — they remember what you forgot.")
      "motivation" '("You are closer than you think."
                     "The best time to start was yesterday. The second best is now."
                     "Doubt kills more dreams than failure ever will."
                     "Small steps still move you forward."
                     "Discipline is choosing what you want most over what you want now.")
      "mystery"    '("Something lost will find you when you stop looking."
                     "The answer you seek is in a room you haven't entered yet."
                     "A stranger already knows your name."
                     "The next full moon brings clarity.")))

  ;; Random from all pools — unconstrained nondeterminism
  (define-method tell-fortune ()
    (let ((all (flatten (hash-values pools))))
      (if (null all)
          "The spirits are silent."
          (nth (random (length all)) all))))

  ;; Random within a stratum — constrained nondeterminism
  (define-method tell-fortune-by-category (category)
    (let ((pool (gethash category pools)))
      (when pool
        (nth (random (length pool)) pool))))

  ;; Grow the pool
  (define-method add-fortune (text category)
    (push text (gethash category pools (list))))

  ;; Context-independent response — the anti-pattern named
  (define-method ask-question (question)
    (declare (ignore question))
    (tell-fortune))

  ;; Total count across all strata
  (define-method fortune-count ()
    (reduce #'+ (mapcar #'length (hash-values pools)))))
```

## Connections

- **G013 (Flash Cards)**: Cards dispatched by category deterministically (show me all cards in "math"). Fortunes select randomly within a category. Same structure (category -> items), different access pattern (deterministic vs. nondeterministic).
- **G024 (Quote Tracker)**: Quotes had `random_quote()` — unconstrained nondeterminism across the whole collection. Fortunes add stratification: random, but within bounds. The evolution from flat random to structured random.
- **G011 (Alarm Clock)**: The choreographic case composes temporal events with fortune selection. The alarm provides the "when," the day-of-week provides the "which category," the fortune teller provides the "what." Three independent components, one emergent behavior.
