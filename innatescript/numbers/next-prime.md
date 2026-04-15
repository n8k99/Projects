# Next Prime Number

Pure computation. But this one has an infinite generator — primes on demand, no stopping point. That's a different shape.

## Resolver native

```dpn
@is_prime{n: @n} -> @bool
@next_prime{after: @n} -> @prime
@nth_prime{n: @n} -> @prime
@primes{count: @n} -> @prime_list
```

## The generator question

The previous projects return a value. This one can return *endlessly*. An infinite prime generator is a stream — it produces values as long as something is consuming them.

InnateScript has nothing for streams yet. `concurrent` runs things in parallel and `join` waits for all of them. But a stream doesn't join — it keeps producing. `until` could bound it: "generate primes `until` 100 found" or "generate primes `until` 30 seconds." But an unbounded stream that a choreography consumes as needed — that's a pattern the language doesn't express.

```dpn
[@primes until 10 found -> @prime_list]
```

Does `until` work with a count, not just a duration? The spec says "time-bounded or condition-bounded waiting." A count is a condition. But the syntax hasn't been tested against this case.

## Design note

Four projects in. First encounter with streams and generators. InnateScript's coordination primitives assume things *finish* — concurrent work, joined, scored. A generator doesn't finish. It produces until told to stop. That's a gap, or it's what `until` already handles. The Rosetta Stone is asking the question.
