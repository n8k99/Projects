# G026 — News Ticker and Game Scores

> A real-time prioritized message stream. Items arrive with priority and optional expiry. The ticker sorts by importance, then recency, and renders as a scrolling feed.

## Insight Map

### Priority Stream
A news ticker is the first Rosetta Stone encounter with **priority ordering**. G025's journal was chronological only. The ticker adds a second axis: importance. Priority + time = how agents triage incoming information. This is how the noosphere decides what matters *right now*.

### Temporal Scope (TTL)
Expiry means items have a **temporal window** — they're valid for a duration, then gone. This connects to G011's alarm clock (temporal triggers) but in reverse: instead of "do X when time arrives," it's "stop showing X when time expires." Temporal scope is how the noosphere manages attention — not everything is permanent. Some information is only relevant right now.

### Interrupts (Breaking News)
Breaking news is an **interrupt** — it preempts normal flow. In InnateScript, this maps to the `<-` gate operating in reverse: instead of filtering input, it forces output. An alert that demands immediate attention regardless of what else is happening. The choreography must handle interrupts without losing the rest of the stream.

### Content/Presentation Split
The ticker display format (`>>> item1 | item2 >>>`) is a **rendering concern** — the same data displayed differently depending on context. This is G017's content/presentation split applied to a stream: the data model (priority, category, timestamp) is separate from the display format (scrolling text). One ticker, many possible views.

## Schema

```innate
(ticker-item
  :text      String
  :category  (enum :news :sports :finance :alert)
  :priority  (range 1 5)            ; 5 = highest
  :timestamp Instant
  :expires-at (option Instant))     ; nil = never expires

(news-ticker
  :items (stream ticker-item))      ; ordered collection with temporal semantics
```

## Protocol

```innate
;; Post an item to the ticker
(defgate post (text category priority &optional ttl-seconds)
  -> ticker-item
  :pre  (and (member category '(:news :sports :finance :alert))
             (<= 1 priority 5))
  :effect (push! item (ticker :items)))

;; Breaking news — max priority alert, immediate
(defgate breaking (text)
  -> ticker-item
  :sugar (post text :alert 5)
  :interrupt true)                  ; preempts normal stream ordering

;; Get active items, sorted by priority desc then timestamp asc
(defgate get-current ()
  -> (list ticker-item)
  :filter (not (expired? item))
  :order  (desc :priority) (asc :timestamp))

;; Filter by category
(defgate get-by-category (category)
  -> (list ticker-item)
  :from (get-current)
  :filter (= (item :category) category))

;; Purge expired items
(defgate expire-old ()
  -> Nat                            ; count removed
  :effect (remove-if! expired? (ticker :items)))

;; Render as scrolling display
(defgate display ()
  -> String
  :render ">>> ~{text ' | '} >>>")
```

## Choreography — Morning Briefing

```innate
;; Nathan's morning briefing as a prioritized ticker.
;; Each ghost posts their most important update.
;; The ticker sorts by priority. Breaking alerts from
;; Kathryn (market crash) preempt Eliana's infrastructure report.

(defchoreography morning-briefing
  :roles (nathan kathryn eliana desmond)
  :stream (news-ticker)

  ;; Each ghost posts their update
  (par
    (eliana   -> stream : (post "API latency p99 at 45ms, all green" :news 3))
    (desmond  -> stream : (post "3 PRs ready for review" :news 2))
    (kathryn  -> stream : (post "Q1 revenue up 12%" :finance 3)))

  ;; Stream automatically sorts by priority
  (nathan <- stream : (get-current))
  ;; => Q1 revenue (3), API latency (3), PRs ready (2)

  ;; Kathryn detects a market crash — breaking interrupt
  (kathryn -> stream : (breaking "MARKET CRASH: S&P down 7%, trading halted"))

  ;; The breaking alert preempts everything
  (nathan <- stream : (get-current))
  ;; => MARKET CRASH (5), Q1 revenue (3), API latency (3), PRs ready (2)

  ;; After the crisis window passes, the alert expires
  (stream : (expire-old))
  ;; The ticker returns to normal priority ordering
  )
```

## Connections

| Gate | Connects To | Insight |
|------|------------|---------|
| `post` | G025 journal `add-entry` | Both append to a stream, but ticker adds priority axis |
| `breaking` | G011 alarm `set-alarm` | Both are temporal triggers — alarm fires forward, breaking fires *now* |
| `expire-old` | G011 alarm expiry | Temporal scope: information valid for a window then gone |
| `display` | G017 content/presentation | Same data, different rendering — model vs. view separation |
| `get-by-category` | G019 RSS `by-feed` | Category filtering is the same pattern as feed filtering |
| `:interrupt` | `<-` gate (reversed) | Instead of filtering input, forces output — attention interrupt |
