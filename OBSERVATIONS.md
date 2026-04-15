# Observations

Running notes on what the Rosetta Stone reveals about language design, paradigm boundaries, and what InnateScript is and isn't.

---

## G001 — Find PI to the Nth Digit

**Not everything is a choreography.**

PI computation is pure math. It doesn't coordinate agents. It doesn't verify output against reality. It doesn't need concurrent execution or fulfillment fallbacks. A single function in any language computes it faster and more correctly than any choreography could.

InnateScript expresses this as `@pi{digits: N}` — a resolver-provided native. The evaluator's host language (Common Lisp) provides the implementation. The choreographic language doesn't compute PI. It knows how to *use* PI in a coordination context.

This is the first boundary: InnateScript is a coordination language. Problems that require no coordination resolve natively through the evaluator.

**The evaluator's host language is the resolver's native library.** If the evaluator is Common Lisp, `@pi{digits: N}` bottoms out in Lisp. If the evaluator were Lean, the same reference would bottom out in a proven-correct function. Same choreography, different guarantees. The language is agnostic to its own substrate.

---

## G002 — Fibonacci Sequence

**InnateScript coordinates interpretation, not computation.**

Computation already has languages. Every language in this repo can compute Fibonacci. What no language before InnateScript can express is: "Kathryn maps this sequence to revenue targets, Sanjay validates it against historical data, Eliana checks they're both grounded, and the `where` scores whether the combined interpretation actually reflects growth trajectory rather than wishful scaling."

The agents don't compute. They *interpret*. The choreography coordinates interpretation. That's the gap in every existing language — not "how do I calculate this" but "how do I get three perspectives on a calculation to converge into something I can trust."

Pure computation resolves natively: `@fib{n: 20}`. The choreography emerges when agents need to coordinate *around* the result. The sequence is a function call. The interpretation is a dance.

---

## G003 — Prime Factorization

**Math and choreography share vocabulary.**

Factorization decomposes a whole into its irreducible parts. Choreographic projection decomposes a global intention into each agent's irreducible local slice. The structure is the same — break something into pieces that can't be broken further, then work with the pieces independently.

The `where` for a factorization-based choreography echoes the fundamental theorem of arithmetic: the decomposition is unique and the product of the factors reconstructs the original. Applied to coordination: did the agents' assignments cover every irreducible piece, and do the pieces multiply back into the whole?

Three projects in. Pure computation still resolves natively. But this is the first time the *shape* of a computation mirrors a coordination concept. Decomposition into irreducibles is what both prime factorization and choreographic projection do. The Rosetta Stone is starting to show where math and orchestration share structure.

---

## G004 — Next Prime Number

**Streams don't finish.**

Every project so far produces a result and stops. A prime generator produces *endlessly* — values on demand, no termination. That's a different shape, and InnateScript's coordination primitives assume things finish. `concurrent` runs, `join` waits, `where` scores the outcome. But a generator has no outcome. It has a *flow*.

`until` might handle this: `@primes until 100 found`. The spec says "time-bounded or condition-bounded waiting." A count is a condition. But the syntax hasn't been tested against a stream — it's been used for bounding an agent's obligation or a choreographic context, not for bounding an infinite producer.

This is the first time the Rosetta Stone asks a question the spec doesn't clearly answer. Four projects in and the language is already being stress-tested by a basic programming concept: the infinite iterator.

**Postscript — the infinite iterator as heartbeat.** The nightly summary in the daily note template — `(@LenaMorris){nightly_summary -> @vault_notes:weekly}` — is an infinite iterator. It fires every night. Forever. It never terminates. And because Lena has to summarize *what happened today*, every other ghost is under pressure to have done something worth summarizing. The infinite iterator isn't a gap in InnateScript. It's the foundation. The tick. The heartbeat. The temporal chain — daily, weekly, monthly, quarterly, yearly, decade, century, millennium — is a nested stack of infinite iterators, each consuming the output of the one below. InnateScript doesn't need `while true`. The vault has `Temporal/`.

---

## G005 — Tile Cost Calculator

**The computation is trivial. The decision context isn't.**

Anyone can multiply tiles by price. The reason a tile cost calculator exists is that a person is standing in a hardware store making a purchasing decision. The computation serves a context.

InnateScript doesn't make math easier. It makes the *conversation around math* structured. The `where` for a renovation choreography isn't "did the math work" — the math always works. The `where` is "can we afford this." That's judgment. That's an agent. The calculator doesn't know about budgets. The choreography does.

Five projects in. First encounter with a computation that exists because of a human decision context. The pattern: trivial computation + non-trivial judgment = choreography.

---

## G006 — Mortgage Calculator

**Every temporal `where` is an amortized obligation.**

A mortgage breaks a large obligation into periodic payments. Kathryn's monthly aspiration — forex covers infrastructure costs — works the same way. The monthly target is the principal. Each day's P&L is a payment. The `pace_check` function turns a distant target into a daily score: are we ahead or behind?

This pattern scales across the entire temporal chain. The daily `where` is a pace check against the weekly aspiration. The weekly against the monthly. The monthly against the quarterly. Each level amortizes the one above it. The mortgage calculator isn't about houses — it's the math underneath every temporal `where` in the noosphere.

Six projects in. First direct mapping between a Rosetta Stone project and InnateScript's core design need. The `pace_check` function is what Kathryn calls every night, and it's the same function the temporal compression chain uses to evaluate whether any period is tracking toward its parent's aspiration.

---

## G007 — Change Return Program

**The Rosetta Stone is accidentally building Kathryn's toolkit.**

Making change: given $0.87, allocate quarters first, then dimes, then nickels. Cover the largest denomination, carry the remainder.

Covering monthly bills: given cumulative P&L, cover Captivate ($19, day 1), then Anthropic ($100, day 10), then DigitalOcean ($24, day 15), then ElevenLabs ($22, day 22). Same algorithm. Different denominations.

G005 (tile cost) → what does it cost? G006 (mortgage) → are we on pace? G007 (change) → can we cover the bills, and which ones fall short? The Numbers category is building a financial stack — not because anyone designed it that way, but because the project ordering mirrors what a trading ghost actually needs.

The `cover_obligations` function is the first thing in this repo that takes a schedule of uneven deadlines and reports what's covered and what isn't. That's the stepped `where` — not linear pace against month-end, but coverage against specific due dates. The daily `where` becomes: are we tracking toward the *next* bill, not just the monthly total?

---

## Meta-observation: G001–G007 as shared infrastructure

**The same seven functions serve every executive ghost.**

The Numbers category isn't Kathryn's toolkit. It's *everyone's* toolkit. Same `pace_check`, same `cover_obligations`, different denominations, different `where`:

- **Kathryn**: trading covers bills by due dates
- **Eliana**: infrastructure capacity covers workload before deadlines
- **JMax**: compliance obligations met before filing dates
- **Sylvia**: publication rubrics met before deadlines
- **Vincent**: image specifications *exceeded* by due date

All five run concurrently inside a monthly operations choreography. The `join` waits for all five. The `where` at the top: is EM operational?

But each executive's `<-` gate is a *named choreography* — not a simple check but a full interior dance. Sylvia's publication has drafting, copy editing, fact checking, final polish, each with its own `<-` verification gates and its own `where` (rubric scores above threshold). Vincent's deliverable has creation, resolution verification, brand alignment checks, with a `where` that says *exceeds* spec, not just meets it.

The monthly operations choreography doesn't see any of this interior complexity. It calls `@sylvia_publication` and gets back a score. The three-bracket limit forces the extraction. The result: a library of named choreographies that compose into the top-level `where`, all built on the same seven functions from the Rosetta Stone's Numbers category.

Seven projects into a 130-project exercise, and the computation layer for an entire multi-department autonomous organization has started to emerge from basic programming challenges. The functions are the foundation. The choreographies are the architecture. The `where` is the judgment.

---
