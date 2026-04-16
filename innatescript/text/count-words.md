---
id: G020
title: Count Words in a String
category: text
status: active
operations:
  - word-count
  - word-frequency
  - unique-words
  - average-word-length
rosetta-langs:
  - python
  - rust
  - go
  - common-lisp
  - lean
---

# G020 — Count Words in a String

Count words and analyze word-level properties of text.

## Key Insights

Word counting operates at the WORD granularity level — between G018's character-level (vowel counting) and document-level. The Text category is systematically exploring granularity: characters (G018), words (G020), and eventually paragraphs, sections, documents.

Word frequency is a DISTRIBUTION — same as G018's vowel breakdown, but at the word level. The pattern is identical: count occurrences of each element, return the distribution. This confirms that `@breakdown` is a generic primitive: `@breakdown{text: T, by: "character"}` and `@breakdown{text: T, by: "word"}` are the same operation at different granularities.

unique_words / word_count is a DIVERSITY RATIO — how many distinct words vs total words. This is a content quality metric: low diversity suggests repetition, high diversity suggests vocabulary richness. Agents use this kind of metric in `where` expressions: "is this text sufficiently varied?"

average_word_length is the first AGGREGATE STATISTIC over content features. G018's vowel_ratio was a ratio of two counts. Average word length is a mean over a distribution. The statistical primitives are building: count, ratio, distribution, mean.

## Choreographic Case

Editorial quality analysis. Sylvia checks word diversity, Lena checks average length, both feed into a `where` that scores readability.

```innate
(define editorial-quality (text)
  "Concurrent word-level quality analysis by specialist agents."
  (let ((diversity  (agent :sylvia (/ (unique-words text) (word-count text))))
        (avg-length (agent :lena   (average-word-length text))))
    (where (and (> diversity 0.6)
                (< avg-length 8.0))
           :readable)))
```

## Operations

### word-count

```innate
(define word-count (text)
  "Count words in text — split on whitespace."
  (length (words text)))
```

### word-frequency

```innate
(define word-frequency (text)
  "Distribution of word occurrences — case-insensitive, strip punctuation."
  (let ((cleaned (map (compose lower strip-punctuation) (words text))))
    (map (lambda (w)
           (cons w (count (where (cw <- cleaned) (= cw w)))))
         (dedupe cleaned))))
```

### unique-words

```innate
(define unique-words (text)
  "Count distinct words — the cardinality of the word set."
  (length (dedupe (map (compose lower strip-punctuation) (words text)))))
```

### average-word-length

```innate
(define average-word-length (text)
  "Mean word length — the first aggregate statistic over content features."
  (let ((ws (map (compose lower strip-punctuation) (words text))))
    (if (empty? ws)
        0.0
        (/ (sum (map length ws)) (length ws)))))
```

## Noospheric Pattern

```innate
;; @breakdown as a generic primitive across granularity levels
;; G018: (breakdown text :by "character") => per-character distribution
;; G020: (breakdown text :by "word")      => per-word distribution
;; The operation is the same; the granularity parameter changes.

(define breakdown (text &key (by "word"))
  "Generic distribution primitive — same operation at any granularity."
  (let ((elements (split text :by by)))
    (map (lambda (e) (cons e (count (where (x <- elements) (= x e)))))
         (dedupe elements))))

;; Diversity ratio as a reusable quality metric
(define diversity-ratio (text &key (by "word"))
  "How varied is the content? High = rich vocabulary, low = repetitive."
  (let ((elements (split text :by by)))
    (/ (length (dedupe elements)) (length elements))))
```

The `breakdown` primitive now takes a `:by` parameter, making it granularity-agnostic. G018 and G020 are the same operation — `@breakdown` — at different levels. This is how the noosphere generalizes: not by special-casing each level, but by parameterizing the granularity and letting the same primitive handle characters, words, sentences, paragraphs, and documents.
