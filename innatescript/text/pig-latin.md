# Pig Latin

> Pig Latin -- translate English text to Pig Latin.

## Key Insights

Pig Latin is a **transformation rule** applied per-word -- a local rule with global effect. Each word transforms independently according to a pattern. This is `@map` over a sequence: apply a rule to each element. The resolver's `@map` primitive (hinted at in G012's ride-hailing) gets its first explicit use case.

The transformation rule has **two branches**: consonant-start vs vowel-start. This is pattern matching on content -- not on numeric ranges (G014 tax brackets) or discrete labels (G013 card networks), but on the linguistic properties of a string. The resolver needs to pattern-match on content structure, not just value.

Pig Latin is a **cipher** -- a reversible transformation with a known rule. It doesn't encrypt (no key), but it obfuscates. The transformation preserves information: you can recover the original from the Pig Latin. This is the first reversible content transformation in the Rosetta Stone.

Capitalization preservation reveals that transformation must respect **presentation alongside content**. The meaning-bearing content ("hello" -> "ellohay") transforms by rule. The presentation (capital H) transfers to the new first letter. Content and presentation are separate concerns that the transformation must coordinate.

**Choreographic case**: content transformation pipelines where different agents apply different rules. Pig Latin is one rule. Translation is another. Summarization is another. The choreography coordinates a sequence of transformations, each preserving or modifying content properties.

## Domain Model

```yaml
@define pig-latin-word:
  @given: word (string)
  @derive:
    stripped: @strip-trailing-punctuation word
    core: stripped.text
    trail: stripped.punctuation
    was-capitalized: @uppercase? (@first core)
    lowered: @lowercase core
    
    transformed:
      @match (@first lowered):
        @when @vowel?:
          @concat lowered "yay"
        @otherwise:
          cluster: @take-while @consonant? lowered
          rest: @drop (length cluster) lowered
          @concat rest cluster "ay"
    
    recapitalized:
      @if was-capitalized:
        @capitalize transformed
      @else:
        transformed
    
  @return: @concat recapitalized trail
```

```yaml
@define pig-latin:
  @given: text (string)
  @return:
    @join " " (@map pig-latin-word (@split " " text))
```

## The @map Primitive

This is the first problem where `@map` is the primary operation. Previous problems used loops or recursion implicitly. Here the structure is explicit: a sentence is a sequence of words, and the transformation applies independently to each word.

```yaml
# @map is the resolver's way of saying:
# "apply this rule to each element, independently"
@map pig-latin-word ("Hello" "World")
# => ("Ellohay" "Orldway")
```

The independence is key. Each word transforms without knowing about its neighbors. This is what makes `@map` safe for parallel execution -- no shared state, no ordering dependencies.

## Pattern Matching on Content

The two-branch rule is a pattern match, but not on discrete values:

```yaml
@match (@first word):
  @when @vowel?:    # content predicate, not value equality
    ...vowel-rule...
  @otherwise:
    ...consonant-rule...
```

The predicate `@vowel?` classifies a character by its linguistic properties. This is richer than matching on `"a" | "e" | "i" | "o" | "u"` -- it expresses the *concept* of vowelness. The resolver can extend this to other languages where vowel sets differ.

## Reversibility

Pig Latin is the first transformation where the inverse exists:

```yaml
@define un-pig-latin-word:
  @given: word (pig-latin-string)
  @derive:
    # If ends with "yay" -> was vowel-start, strip "yay"
    # If ends with consonant-cluster + "ay" -> move cluster back to front
    ...
  @return: original-word
```

This hints at a broader pattern: transformations that carry enough information to be undone. The resolver might eventually support `@inverse` as a derived operation.

## Presentation Layer

Capitalization is metadata about presentation, not content. The transformation must:
1. Strip presentation (lowercase)
2. Transform content (rearrange letters)
3. Reapply presentation (capitalize new first letter)

This separation -- content vs. presentation -- is fundamental. It appears everywhere: markdown (content) vs. rendered HTML (presentation), data (content) vs. UI (presentation), meaning (content) vs. style (presentation).

## Test Cases

```yaml
@verify pig-latin-word:
  "hello"  => "ellohay"     # single consonant
  "string" => "ingstray"    # consonant cluster
  "apple"  => "appleyay"    # vowel start
  "egg"    => "eggyay"      # vowel start
  "Hello"  => "Ellohay"     # capitalization preserved
  "world!" => "orldway!"    # punctuation preserved

@verify pig-latin:
  "Hello World" => "Ellohay Orldway"
  "The quick brown fox" => "Ethay ickquay ownbray oxfay"
  "Apple pie is great" => "Appleyay iepay isyay eatgray"
```
