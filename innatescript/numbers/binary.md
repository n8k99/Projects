# Binary to Decimal and Back

A choreography that converts between number systems.

## Design feedback

Base conversion is pure computation — shift bits, multiply, accumulate. Like PI, it requires no coordination. But unlike PI (which is a single opaque function), base conversion is a *family* of transforms parameterized by base. This is the first project where InnateScript's resolver exposes a parameterized native:

```dpn
@binary{value: @n} -> @bits
@decimal{value: @bits} -> @n
@base{value: @n, radix: 16} -> @hex
```

The interesting choreographic case is format negotiation — when two agents need to agree on a representation:

```dpn
concurrent [
    @sensor{read_value} -> @raw
    @display{format: "hex"}
]
join
@base{value: @raw, radix: 16} -> @formatted
@display{show: @formatted}
```

The conversion itself is a resolver native. The choreography is the coordination around it: who produces the value, who consumes it, what format they agree on.

## What this means

Parameterized natives (`@base{value: N, radix: R}`) are the resolver's standard library. Each language in the Rosetta Stone implements the algorithm; InnateScript names it and places it in a coordination context. The resolver maps `@binary`, `@decimal`, `@base` to whichever host implementation is available.

## Native implementation

The resolver provides `@binary{value: N}`, `@decimal{value: S}`, and `@base{value: N, radix: R}` as built-ins. Conversion algorithms live in the host language.
