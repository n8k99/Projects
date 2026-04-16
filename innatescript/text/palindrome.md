# Check if Palindrome

Symmetry detection. Content that is its own reverse.

## Resolver native

```dpn
@palindrome?{s: @text} -> @bool
@palindrome_normalized?{s: @text} -> @bool
@longest_palindrome{s: @text} -> @text
```

## Key insight: palindrome as composition

A palindrome is a SYMMETRY — content that is its own reverse. This connects G019 back to G016 (reverse). `is_palindrome(s) = (s == reverse(s))`. The palindrome check composes two previously defined operations: reverse + equality. This is the first time the Rosetta Stone builds on its own earlier work.

```dpn
@palindrome?{s: "racecar"} -> true
@palindrome?{s: "hello"} -> false
```

The simplest test: does content equal its own reversal? But the interesting part is that this operation didn't need a new native — it's a composition of `@reverse_string` and `==`. The resolver could derive `@palindrome?` from existing primitives. It chooses to name it because the concept is load-bearing: symmetry detection is a pattern that recurs across domains.

## Normalization as projection

Normalization (ignore case, spaces, punctuation) is PROJECTION — reducing content to its essential structure before comparison. The same text at different normalization levels produces different palindrome answers.

```dpn
@palindrome?{s: "Race car"} -> false
@palindrome_normalized?{s: "Race car"} -> true
@palindrome_normalized?{s: "A man, a plan, a canal: Panama"} -> true
```

"Race car" is not a palindrome. "racecar" is. The difference is what you choose to ignore. This is the `where` in disguise: the normalization defines the frame of comparison. Every `where` clause in a choreography is a normalization — it says which details matter and which don't. The resolver must carry normalization context alongside the operation.

## Search within content

The longest palindrome substring is a SEARCH problem over content — finding structure within text. This is the first search operation in the Rosetta Stone (Dijkstra searched graphs, not content). The expand-around-center algorithm treats each character as a potential center of symmetry and expands outward — a local-to-global search pattern.

```dpn
@longest_palindrome{s: "forgeeksskeegfor"} -> "geeksskeeg"
@longest_palindrome{s: "babad"} -> "bab"  ;; or "aba" — both valid
```

Each position is a hypothesis: "is this the center of a palindrome?" The algorithm tests every hypothesis and keeps the best. This is a pattern the resolver will see again: enumerate local candidates, expand each, keep the winner. Substring search is the text-domain version of graph search.

## Where it becomes a choreography

Quality verification where agents check for structural properties in content. A palindrome checker is a specialized `<-` gate that verifies a structural property. More generally: agents that scan content for patterns (grammar checkers, format validators, plagiarism detectors) are all palindrome checkers generalized.

```dpn
[content_verification @document
    @Vincent{scan for structural patterns}
    <- @palindrome_gate{
        verify: @palindrome_normalized?{s: @document.title}
        verify: @longest_palindrome{s: @document.body}.length > threshold
    }
    -> @Sylvia{edit if structural properties not met}
    <- @ElianaRiviera{final approval}
] where [structural properties define quality gates, normalization defines what counts]
```

The choreography generalizes: any agent that validates content structure is performing a palindrome check at some level of abstraction. Is the document well-formed? Is the argument symmetrical? Does the narrative mirror its opening at the close? These are all "is it a palindrome?" asked at different granularities.

## Design note

First composition in the Rosetta Stone. G019 builds on G016 rather than introducing an entirely new primitive. This is significant: the resolver's vocabulary is growing, and new operations emerge from combining existing ones. The palindrome check is `reverse + equality`. The normalized version adds `projection` before comparison. The longest substring adds `search` over positions. Three operations, each layering one new concept onto the composition.
