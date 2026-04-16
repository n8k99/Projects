# Reverse a String

The first text operation. Content in, content out.

## Resolver native

```dpn
@reverse_string{s: @text} -> @result
@reverse_words{s: @text} -> @result
@palindrome?{s: @text} -> @bool
```

## Key insight: content, not values

Text is the first category where the Rosetta Stone operates on CONTENT rather than VALUES. Numbers are abstract quantities. Strings are concrete sequences with meaning. Reversing "Hello" is a structural transformation; reversing a sentence's word order changes meaning. The resolver's Text natives operate on meaning-bearing content, not abstract values.

String reversal is the simplest TRANSFORMATION — it takes content and produces content. Every Numbers native took values and produced values. Text natives take content and produce content. The distinction matters because agents communicate in text. Chat messages, reports, summaries — the noosphere runs on strings. Text manipulation isn't a utility; it's infrastructure.

## Two granularities

`reverse_words` vs `reverse_string` — two operations on the same data at different granularity levels. Characters vs words. This is the first encounter with granularity in the Rosetta Stone. The resolver needs to operate on content at multiple levels: characters, words, sentences, paragraphs, documents.

```dpn
@reverse_string{s: "Hello, World!"} -> "!dlroW ,olleH"
@reverse_words{s: "Hello, World!"} -> "World! Hello,"
```

Same input. Different structural level. Different result. The resolver must know which level you mean.

## Where it becomes a choreography

Content processing pipelines where agents transform text at different granularity levels. Sylvia edits at the sentence level. Lena summarizes at the paragraph level. Vincent formats at the character level. The choreography coordinates across granularities:

```dpn
[content_pipeline @document
    concurrent [
        @Vincent{@reverse_string where needed for stylistic effect}
        @Sylvia{restructure sentences for clarity}
        @Lena{summarize each section at paragraph level}
    ]
    join
    <- @ElianaRiviera{verify transformations preserve meaning}
] where [each agent operates at its own granularity, the choreography ensures coherence across levels]
```

## Design note

First text project. The pattern shifts: pure computation still resolves natively, but now the data itself carries meaning. When an agent reverses a string, it's not just rearranging bytes — it's transforming content. The choreographies that emerge from text operations will be about meaning preservation, not just correctness.
