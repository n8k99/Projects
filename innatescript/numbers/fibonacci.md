# Fibonacci Sequence

Another pure computation. No coordination needed. Same boundary as PI.

## Resolver native

```dpn
@fib{n: @n} -> @result
@fibonacci{count: @n} -> @sequence
@fibonacci{up_to: @limit} -> @sequence
```

## Where it becomes a choreography

Fibonacci is interesting when it's a *tool inside a coordination problem*. Growth modeling, resource projection, sequence analysis:

```dpn
[growth_projection @project
    @fibonacci{count: 12} -> @fib_sequence
    concurrent [
        @KathrynLyonne{map @fib_sequence to monthly revenue targets}
        @SanjayPatel{validate growth curve against historical data}
    ]
    join
    <- @ElianaRiviera{verify projections are grounded}
] where [projections reflect actual growth trajectory, not wishful scaling]
```

The Fibonacci sequence itself is a function call. The choreography is what you do with it — three agents interpreting a mathematical sequence through three lenses, verified, joined, scored against reality.

## Design note

Two projects in. The pattern holds: pure computation resolves natively, choreographies emerge when agents need to coordinate around the results. InnateScript doesn't compute — it orchestrates computation.
