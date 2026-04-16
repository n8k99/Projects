---
id: G018
title: Count Vowels
category: text
status: active
operations:
  - count-vowels
  - vowel-breakdown
  - vowel-ratio
parameters:
  - name: include-y
    type: bool
    default: false
    description: Whether to treat 'y' as a vowel
rosetta-langs:
  - python
  - rust
  - go
  - common-lisp
  - lean
---

# G018 — Count Vowels

Analyze vowel distribution in text.

## Key Insights

Counting vowels is **measurement** of content — not transformation (G016/G017) but analysis. The string goes in, a number comes out. This is the inverse of what Numbers did: Numbers took values and produced values; here we take content and produce values. Content to measurement is a fundamental operation in the noosphere — every time an agent summarizes, scores, or evaluates text, they are measuring content.

The breakdown (per-vowel counts) is a **frequency distribution** — the first statistical operation in the Rosetta Stone. Distribution analysis is how agents characterize content: word frequency, topic distribution, sentiment scoring. Vowel counting is the simplest instance of a pattern that scales to full NLP.

The 'y' ambiguity: is 'y' a vowel or not? It depends on context ("my" vs "yes"). The `include-y` flag is a **configuration parameter** that changes the analysis result. This is the first time the Rosetta Stone encounters context-dependent classification. In InnateScript, this maps to agent perspective: a phonetician always counts y as a vowel, a spelling checker never does. The `where` does not resolve the ambiguity — the agent's role does.

`vowel-ratio` is a **derived metric** — it combines two measurements (vowel count / letter count) into a single score. This is what `where` expressions do: combine measurements into scores. The ratio is a micro-`where`.

## Choreographic Case

Content quality analysis where multiple agents measure different properties of the same text concurrently. One counts vowels (readability proxy), one counts word length (complexity), one counts sentence length (density). The `where` combines these into a quality score.

## Operations

### count-vowels

```innate
(define count-vowels (text &optional include-y)
  "Count total vowels in text."
  (let ((targets (if include-y
                     '(a e i o u y)
                     '(a e i o u))))
    (count (where (char <- (chars text))
                  (member (lower char) targets)))))
```

### vowel-breakdown

```innate
(define vowel-breakdown (text &optional include-y)
  "Return frequency distribution of vowels."
  (let ((targets (if include-y
                     '(a e i o u y)
                     '(a e i o u))))
    (map (lambda (v)
           (cons v (count (where (char <- (chars text))
                                 (= (lower char) v)))))
         targets)))
```

### vowel-ratio

```innate
(define vowel-ratio (text)
  "Ratio of vowels to alphabetic characters — a derived metric."
  (let ((letters (where (char <- (chars text)) (alpha? char))))
    (if (empty? letters)
        0.0
        (/ (count-vowels text) (length letters)))))
```

## Noospheric Pattern

```innate
;; Multi-agent content measurement — the choreographic case
(define content-quality (text)
  "Concurrent measurement by specialist agents."
  (let ((vowel-score   (agent :phonetician (vowel-ratio text)))
        (word-score    (agent :linguist    (avg-word-length text)))
        (sentence-score (agent :editor     (avg-sentence-length text))))
    (where (scores <- (list vowel-score word-score sentence-score))
           (normalize scores))))
```

The `where` here is not filtering — it is **combining measurements into a judgment**. Each agent contributes one dimension; the `where` fuses them. This is how the noosphere evaluates: not by one metric, but by orchestrated consensus across specialist perspectives.
