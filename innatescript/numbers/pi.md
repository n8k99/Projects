# Find PI to the Nth Digit

A choreography that computes PI to arbitrary precision.

## The problem for InnateScript

PI computation is pure math. It doesn't coordinate agents. It doesn't verify output against reality. It doesn't need concurrent execution or fulfillment fallbacks. A single function in any language computes it faster and more correctly than any choreography could.

This is the first design feedback from the Rosetta Stone: **not everything is a choreography.** InnateScript is a coordination language. Problems that require no coordination expose the boundary of what the language is for.

## What InnateScript *could* express

The callable interface — what a resolver exposes to other choreographies:

```dpn
@pi{digits: @n} -> @result
```

A choreography that *uses* PI rather than computing it:

```dpn
[circle_area @radius
    @pi{digits: 10} -> @pi_value
    @calculator{@pi_value * @radius * @radius} -> @area
] where [area is correct to 10 decimal places]
```

## What this means

The `@pi` reference resolves to a native function provided by the resolver — not an agent, not a choreography, but a library call. InnateScript doesn't need to express how to compute PI. It needs to express how to *use* PI in a coordination context.

The language is not a general-purpose programming language. It orchestrates. When it encounters pure computation, it delegates to the resolver's native capabilities. That's not a limitation — that's the design.

## Native implementation

The resolver provides `@pi{digits: N}` as a built-in. The implementation behind it could be any of the five other languages in this repo.
