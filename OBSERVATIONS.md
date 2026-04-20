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

## G008 — Binary to Decimal and Back

**The first parameterized native family.**

Every resolver native so far has been a single function: `@pi{digits: N}`, `@fib{n: 20}`, `@prime_factors{n: 100}`. Binary conversion introduces a *family* of transforms parameterized by base. `@base{value: N, radix: R}` is one function that covers binary, octal, hex, base36 — any radix from 2 to 36. The resolver doesn't need `@binary` and `@hex` and `@octal` as separate entries. It needs one parameterized native that handles the whole space.

This is a design pattern for the resolver's standard library: don't enumerate cases, parameterize. `@base{value: 42, radix: 2}` and `@base{value: 42, radix: 16}` are the same call with different arguments. The resolver exposes capabilities, not a catalog.

**Format negotiation is the choreographic case.**

The computation is trivial. The interesting question is coordination: two agents that need to agree on a representation. A sensor produces decimal values. A display expects hex. The choreography isn't the conversion — it's the agreement about format, the concurrent readiness of producer and consumer, and the `join` that wires them together. The `@base` native sits in the middle as a format bridge.

This is the second shape of choreographic utility (after G005's "trivial computation + non-trivial judgment"): trivial computation + non-trivial agreement. The agents don't disagree about math. They disagree about representation. The choreography resolves the disagreement.

Eight projects in. The resolver's standard library is starting to have structure: single-value natives (`@pi`, `@fib`), decomposition natives (`@prime_factors`), and now parameterized family natives (`@base`). Three patterns for how computation enters a coordination language.

---

## G009 — Calculator

**The language recognizes itself.**

A calculator is an expression evaluator. Tokenize, parse with precedence, walk the tree, reduce to a value. Every language in the Rosetta Stone implements this from scratch — recursive descent, the same algorithm since the 1960s. But InnateScript's resolver already IS a recursive descent evaluator. Every `@reference{with: @nested{references}}` is an expression tree being walked. The resolver parses structure, respects binding depth, evaluates subexpressions before outer expressions, and returns a value. That's the calculator algorithm.

`@calc{expr: "2 + 3 * 4"}` is a resolver native. But unlike `@pi` or `@base`, this native doesn't perform a computation that the resolver can't — it performs a computation that the resolver *already does*. The calculator is the resolver seen from the outside.

**Expressions with agent-resolved terms are inherently concurrent.**

The choreographic insight: when the terms of an expression come from different agents, the expression becomes coordination. `@calc{expr: "@revenue - @infra_cost - @staff_cost"}` requires three agents to produce values before the expression can reduce. The `concurrent`/`join` pattern from G008's format negotiation reappears, but now the synchronization point isn't format agreement — it's arithmetic reduction. The agents compute in parallel. The expression waits for all terms. The result is a single number that none of them could produce alone.

This is the third shape of choreographic utility: trivial computation + concurrent term resolution. The math is elementary. The coordination is the collection of terms from independent sources into a single evaluable expression.

**The recursive descent is the generic protocol.**

Nine projects in. The resolver's generic protocol — parse a reference, resolve its dependencies, return a value — is formally the same algorithm as recursive descent parsing. This isn't a metaphor. The resolver walks `@a{x: @b{y: @c}}` exactly the way a calculator walks `(a + (b * c))`: innermost first, propagate results upward, reduce at each level. The Rosetta Stone's calculator column doesn't show InnateScript solving a problem. It shows InnateScript recognizing that its core evaluation model is the problem, already solved.

Four patterns for resolver natives now: single-value (`@pi`), decomposition (`@prime_factors`), parameterized family (`@base`), and identity native (`@calc`) — where the native's algorithm is the resolver's own algorithm.

---

## G010 — Unit Converter

**Every agent has a native frame.**

Unit conversion is frame translation. The same physical quantity expressed in different contexts: kilometers for Tokyo, miles for a US carrier. Celsius for a French chef, Fahrenheit for a Boston kitchen. The computation is trivial — multiply by a ratio or apply a formula. The insight is that *agents think in frames*, and choreographies are what happen at the boundaries between frames.

This isn't metaphor. Kathryn thinks in dollars. Eliana thinks in server-hours. Vincent thinks in pixels-per-inch. When they coordinate, every exchanged quantity has an implicit unit negotiation. InnateScript makes that negotiation explicit: `@convert{value: @dist_km, from: "km", to: "mi"}` at the boundary, not buried inside each agent's implementation.

**Not all frame translations are proportional.**

Temperature breaks the ratio pattern. Celsius to Fahrenheit involves an offset — `F = C * 9/5 + 32`. You can't just multiply. You can't chain it with other conversions by composing factors. The resolver must know which *kind* of translation it's performing: scaling (distance, weight, volume) or transformation (temperature).

This distinction matters beyond units. Translating between Kathryn's financial frame and Eliana's infrastructure frame isn't scaling. "How many server-hours does $100 buy?" depends on which cloud, which instance type, which region, whether spot pricing is available. The conversion function is contextual, not a fixed ratio. Unit conversion is the simplest case of a problem that gets arbitrarily complex in real coordination: frame translation with non-trivial relationships.

**Normalize-to-base is the resolver's canonical pattern.**

Every conversion normalizes to a base unit (meters, grams, liters), then converts from base to target. This is the same thing the resolver does everywhere: take input in context, reduce to a canonical internal form, produce output in the requested context. `@base` did it with number representations. `@convert` does it with measurement units. The resolver's generic protocol is: context in, canonical form, context out.

Ten projects in. The resolver's standard library has a clear shape: natives for pure computation (`@pi`, `@fib`, `@prime_factors`), for representation translation (`@base`), for self-referential evaluation (`@calc`), and now for frame translation (`@convert`). Each new native reveals another facet of what the resolver already is — not a growing catalog of functions, but a growing understanding of the resolver's own generic protocol applied to different domains.

---

## G011 — Alarm Clock

**The resolver meets time.**

G001–G010 are all synchronous. You call a function, it returns a value. The alarm clock breaks this: "do nothing now, but when 07:00 arrives, do THIS." This is the first project that introduces a *temporal obligation* — a computation that can't be evaluated on demand because it depends on a condition that hasn't happened yet. The resolver evaluates expressions. The alarm clock asks: who decides WHEN to evaluate?

**The vault already answered this question.**

The temporal chain — Daily Notes, Weekly, Monthly, Quarterly, Yearly — is a nested stack of alarm clocks. `(@LenaMorris){nightly_summary}` fires every night. The weekly review fires every Friday. The quarterly assessment fires on the first of every third month. Each level of the temporal hierarchy is an alarm with a different resolution. InnateScript didn't need to invent a scheduler. The vault's `Temporal/` directory IS the scheduler, implemented as a filesystem hierarchy where each directory level is a tick rate.

**Two execution modes crystallize.**

InnateScript now has two distinct modes:
1. **On-demand**: the resolver evaluates when asked (every project G001–G010)
2. **Scheduled**: the temporal chain fires an alarm, which invokes the resolver on a choreography

The `@alarm` native bridges them. It lets choreographies register obligations that the scheduler fulfills. The resolver doesn't need to know about time. The scheduler doesn't need to know about evaluation. The alarm is the interface — it says WHEN to the scheduler and WHAT to the resolver.

**Alarms don't produce values. They produce events.**

G004's infinite iterator produced an endless stream of values. An alarm produces exactly one event at a specific time. The `until` keyword means something different here: not "keep generating until a condition" but "suspend until a moment arrives." Stream termination vs. temporal suspension. These look similar in syntax but are fundamentally different operations. The language may need both, and G011 is where the ambiguity surfaces.

**Alarms trigger choreographies, not functions.**

The morning-ops alarm at 07:00 doesn't just beep — it kicks off a `concurrent` fan-out to Kathryn, Eliana, and Sarah. Their results `join` into a briefing. The alarm is the starting gun; the choreography is the race. This pattern — alarm triggers choreography — is how the vault actually works. The daily note template isn't a reminder. It's a script with a temporal trigger and a multi-agent payload.

Eleven projects in. The Numbers category has traversed from pure math (`@pi`) through evaluation (`@calc`) and frame translation (`@convert`) to temporal scheduling (`@alarm`). The resolver's world has expanded from "evaluate this expression" to "evaluate this expression at this time." A new architectural layer — the scheduler — has emerged, and it was already built into the vault's directory structure. The Rosetta Stone didn't discover something new. It named something that was already there.

---

## G012 — Distance Between Two Cities

**The resolver reaches outside itself.**

Every project before G012 was self-contained. The inputs were numbers, the outputs were numbers, everything was computable from first principles. But Tokyo's latitude isn't computable — it's a fact about the world. `@city{name: "Tokyo"}` doesn't calculate coordinates. It looks them up. This introduces a fourth resolver primitive after computation, scheduling, and evaluation: the **fact lookup**.

The distinction matters architecturally. Computed values are deterministic and timeless — `sin(0.5)` never changes. Facts are contingent and mutable — a city's coordinates could be refined, a new city could be added, a warehouse could relocate. The resolver must treat these differently. Computed values cache forever. Fact lookups must respect the freshness of their source.

**The fact store is the database.**

In Python/Rust/Go/Lisp/Lean, the city table is a hardcoded dict. In InnateScript, it's the Postgres DB on the droplet. `@city{name: "Tokyo"} -> @coords` bottoms out in a database query. This is the first time the Rosetta Stone has distinguished between "the resolver computes a value" and "the resolver retrieves a value from shared state." The dependency graph doesn't care — a value is a value whether computed or looked up — but the source matters for consistency, caching, and mutation.

**Shared mutable state enters the picture.**

The fact store isn't read-only. Agents could update it — add new cities, correct coordinates, mark warehouses as closed. Two agents querying at different times might get different answers. This is the beginning of the transaction story. Within a single choreography, the resolver needs snapshot consistency — all fact lookups see the same version of the world. Across choreographies, eventual consistency may be acceptable. The Rosetta Stone's hardcoded lookup tables hide this problem entirely. InnateScript can't.

**Proximity-based coordination is the choreographic pattern.**

The computation (Haversine) is pure math. The coordination is: gather locations from multiple agents concurrently, compute distances in parallel, select the nearest. This pattern — concurrent fact gathering, parallel computation, aggregation — appears in logistics, ride-hailing, emergency response, resource allocation. The `@haversine` native does the math. The choreography does the coordination. The fact store provides the world.

Twelve projects in. The resolver now has four kinds of primitives: computation (pure math), evaluation (recursive descent), scheduling (temporal obligations), and fact lookup (world-state from the DB). The first three are self-contained — the resolver can do them alone. The fourth requires an external data source. The resolver has crossed the boundary from closed system to open system. Everything from here on can potentially depend on the state of the world.

---

## G013 — Credit Card Validator

**Structural validity is not truth.**

The Luhn algorithm proves a card number is well-formed — no transposed digits, checksum passes. It says nothing about whether the card is real, funded, or authorized. This is the first project about the distinction between structural verification and semantic verification. A number can pass Luhn and still be fake. An agent's output can be correctly formatted and still be wrong.

This maps directly to InnateScript's `<-` gates. A `<-` gate is a cheap structural check that runs before an expensive operation. The Luhn check is the canonical `<-` gate: free to compute, rejects obviously invalid inputs, doesn't guarantee anything about what passes. The payment processor authorization is the expensive semantic check that follows. The architecture is: cheap filters first, expensive verification last.

**Progressive trust via ascending-cost gates.**

The full payment flow is a pipeline of `<-` gates, each more expensive than the last:
1. Luhn check (free, local arithmetic)
2. Fraud detection (cheap, local ML model)
3. Network authorization (expensive, remote API call)

Each gate prevents the next expensive operation from running on bad input. This is a general pattern for InnateScript: verification costs increase as you move through a choreography, and each `<-` gate is a circuit breaker that prevents wasted work downstream. The `where` at the top of the choreography doesn't need to repeat these checks — the gates already filtered.

**Network identification is dispatch.**

Visa starts with 4. Mastercard starts with 51-55. Amex starts with 34 or 37. The prefix determines the handler. This is pattern-matching dispatch — routing to the correct processor based on input shape. InnateScript's resolver does the same thing: `@reference{...}` is inspected, and the resolver dispatches to the appropriate native or agent based on the reference's shape. The card network IS a dispatcher. The `match` keyword in the InnateScript spec makes this explicit.

**Masking is the first encounter with information hiding.**

`****1234` — showing the last four digits while hiding the rest. Not every agent in a choreography needs the full card number. The `ChargeAgent` needs it. The `AuditLogger` and `CustomerNotifier` don't. The choreography controls who sees what by passing the masked `display` to agents that don't need the real number. This is access control at the data level, enforced by the choreography's structure rather than by runtime permissions.

Thirteen projects in. The Numbers category has shifted from pure computation to trust and verification. The Luhn algorithm is trivial math. The insight is architectural: validation is gating, gating is progressive trust, and InnateScript's `<-` gates implement the same ascending-cost verification pipeline that real payment systems use. The resolver isn't just computing values anymore — it's deciding what to trust.

---

## G014 — Tax Calculator

**Rules are a fifth kind of primitive.**

Tax brackets are neither computation (derivable from math) nor facts (observable from the world) nor validation (structural checks) nor scheduling (temporal triggers). They're *policy* — human-created conditional logic that a legislature defined. `@tax{income: 50000}` applies a rule set. The resolver needs a new category: the rule engine.

The distinction from facts (G012) is subtle but load-bearing. City coordinates change because the world changes (tectonics, surveying refinement). Tax brackets change because *humans decide* to change them. Facts have freshness. Rules have *versions*. `@rules:tax_brackets{year: 2024}` and `@rules:tax_brackets{year: 2025}` are different rule sets, not stale vs. fresh copies of the same data. The resolver needs both: mutable facts AND versioned rules.

**Range dispatch extends the `match` keyword.**

G013's credit card validation dispatched on discrete prefixes — Visa starts with 4, Amex with 34/37. Tax brackets dispatch on continuous ranges — 10% on the first $11,600, 12% on the next $35,550. InnateScript's `match` needs to handle both discrete and range patterns. But tax brackets aren't simple dispatch — they're *cumulative*. Each dollar passes through every bracket up to its level. This isn't "which ONE handler" but "which handlers, and how much in each." The operation is a fold over ranges, not a single dispatch. `@fold_brackets` is a new pattern: cumulative range application.

**Same data, different agent perspectives.**

The calculation produces one number: total tax. But effective rate and marginal rate are different extractions from the same result. Kathryn reports effective rate to Nathan — "you're paying 18%," the big picture. JMax analyzes marginal rate for tax planning — "your next dollar is taxed at 24%," the decision-relevant edge. The choreography doesn't privilege either view. Both are valid projections of the same computation. This is why InnateScript separates computation from reporting: the resolver computes, agents interpret.

**Tax planning is cross-version coordination.**

The full choreographic case requires reasoning across rule versions: current-year brackets vs. proposed next-year brackets. JMax compares outcomes under both rule sets and identifies optimal timing for income recognition. This is a new coordination pattern — not concurrent agents working on different data (G012's warehouse proximity), but the same agent working on the same data under *different rules*. The `concurrent` isn't parallelizing work; it's parallelizing *policy scenarios*.

Fourteen projects in. Five resolver primitives now: computation (pure math), evaluation (recursive descent), scheduling (temporal obligations), fact lookup (world-state), and rule application (versioned policy). The Numbers category started with calculating digits of PI and ends — one project from now — with graph algorithms. The trajectory: from numbers you can derive to numbers that depend on the world, on time, on trust, and now on human decisions.

---

## G015 — Dijkstra's Algorithm

**The graph underneath everything.**

A dependency graph IS a choreography. The resolver already walks graphs when it resolves nested `@references` — `@a{x: @b{y: @c}}` is a DAG with edges a→b→c. Dijkstra formalizes what the resolver does informally: find the optimal path through a graph of dependencies.

But the resolver walks the *entire* dependency graph — it resolves everything reachable. Dijkstra finds the *shortest* path — it skips expensive alternatives. This is the optimization the resolver could make: when multiple resolution paths exist for a reference, choose the cheapest one. The resolver becomes a shortest-path finder through the space of possible resolutions.

**Weighted edges are costs.**

Time, money, computational resources, trust. A choreography where agent A can produce a result in 2 seconds or agent B can produce it in 10 seconds has a weighted choice. Dijkstra over the agent graph finds the cheapest choreography. This connects back to G013's ascending-cost `<-` gates: the gate pipeline IS a shortest-path problem — find the verification sequence that rejects bad inputs at minimum cost.

**The noosphere is a weighted graph.**

Agents are nodes. Communication channels are edges. Weights are latency, cost, trust. Finding the shortest path from intention to fulfillment is what InnateScript does. Dijkstra is the algorithm underneath.

Dynamic rerouting extends this: when an agent in the path fails, rebuild the graph without that agent and re-run Dijkstra from the current position. The noosphere heals itself by finding the next-best path. The cost of resilience is the difference between the original shortest path and the rerouted one.

---

## Meta-observation: The Numbers Category (G001–G015)

**Fifteen algorithms. One architecture.**

The Numbers category didn't just implement 15 projects in 6 languages. It discovered the resolver's architecture, one lens at a time:

| Project | What it revealed |
|---------|-----------------|
| G001 PI, G002 Fibonacci, G003 Prime Factors | **Computation engine.** Pure functions, resolver natives, the boundary between coordination and calculation. |
| G004 Next Prime | **Infinite iterators.** Streams, the `until` keyword, the temporal chain as heartbeat. |
| G005 Tile Cost, G006 Mortgage, G007 Change Return | **The `where` system.** Judgment, pace checks, obligation coverage — shared infrastructure for every executive ghost. |
| G008 Binary/Decimal | **Parameterized natives.** Representation translation, format negotiation, capabilities not catalogs. |
| G009 Calculator | **Identity native.** The resolver IS recursive descent. Expressions with agent-resolved terms are inherently concurrent. |
| G010 Unit Converter | **Frame translation.** Agents think in frames. Choreographies negotiate at boundaries. Not all translations are proportional. |
| G011 Alarm Clock | **Temporal obligations.** Two execution modes: on-demand and scheduled. The vault's Temporal/ directory is the scheduler. |
| G012 Distance Cities | **Fact store.** World-state from the DB. Shared mutable state. Snapshot consistency within choreographies. |
| G013 Credit Card | **Trust gates.** Structural vs semantic validity. Progressive trust via ascending-cost `<-` gates. Information hiding. |
| G014 Tax Calculator | **Rule engine.** Versioned policy. Range dispatch. Cumulative fold. Same data, different agent perspectives. |
| G015 Dijkstra | **Graph optimizer.** The noosphere is a weighted graph. The resolver is Dijkstra finding the cheapest path from intention to fulfillment. |

The resolver's blueprint: computation engine + evaluation engine + scheduler + fact store + rule engine + trust pipeline + graph optimizer. Each project was a facet. Together they describe the shape of InnateScript's evaluator before a single line of interpreter code has been written.

The Numbers category is complete.

---

## G016 — Reverse a String

**Content enters the Rosetta Stone.**

The Text category shifts the Rosetta Stone from values to content. Numbers are abstract quantities — they mean nothing on their own. Strings carry meaning. Reversing "Hello" is a structural transformation. Reversing a sentence's word order changes what it says. The resolver's Text natives operate on meaning-bearing content, not abstract values.

This is infrastructure, not utility. Agents communicate in text — chat messages, reports, summaries, daily notes. The noosphere *runs* on strings. Text manipulation is the substrate of agent communication.

**Granularity is a new axis.**

`reverse_string` operates on characters. `reverse_words` operates on words. Same input, different structural level, different result. This is the first encounter with granularity in the Rosetta Stone. The resolver needs to operate on content at multiple levels: characters, words, sentences, paragraphs, documents. The level you choose determines what the operation means.

This connects to the vault's own structure. A daily note is characters at the bottom, words above that, sections, then the whole document. Different agents operate at different levels — Vincent at the character/formatting level, Sylvia at the sentence level, Lena at the paragraph/summary level. The choreography coordinates across granularities.

Sixteen projects in. The category boundary is real: Numbers discovered the resolver's computational architecture. Text will discover the resolver's content architecture.

---

## G017 — Pig Latin

**`@map` gets its first explicit use.**

Pig Latin applies a transformation rule independently to each word in a sentence. This is `@map` — apply a function to each element of a sequence. The resolver's `@map` primitive was hinted at in G012's ride-hailing (`@map{over: @drivers, apply: @haversine{...}}`), but Pig Latin is the first project where `@map` is the primary operation, not a supporting one.

The independence is what makes `@map` powerful. Each word transforms without knowing about its neighbors. No shared state, no ordering dependencies. This means `@map` is safe for parallel execution — the resolver can distribute the words across agents and collect results. The sentence is a sequence. The transformation is local. The coordination is `@map`.

**Pattern matching on content properties.**

The two-branch rule (consonant-start vs vowel-start) is pattern matching, but not on discrete labels (Visa/MC) or numeric ranges ($0–$11,600). It's matching on *linguistic properties* of the content. `@vowel?` classifies a character by what it IS, not what it equals. This is richer than `match "a" | "e" | "i" | "o" | "u"` — it expresses the *concept* of vowelness, which could extend to other languages where vowel sets differ.

**Content and presentation separate.**

Capitalization is metadata about presentation, not content. The transformation must: strip presentation (lowercase), transform content (rearrange letters), reapply presentation (capitalize new first letter). This separation — content vs. presentation — is fundamental. Markdown is content; rendered HTML is presentation. Data is content; UI is presentation. The resolver needs to track both and coordinate transformations that respect the distinction.

**Reversibility: transformations that can be undone.**

Pig Latin is a cipher — reversible, with a known rule. You can recover "hello" from "ellohay." This is the first reversible content transformation in the Rosetta Stone. The resolver might eventually support `@inverse` as a derived operation for any reversible transformation.

---

## G018 — Count Vowels

**Measurement: content in, values out.**

G016 and G017 were transformations — content in, content out. Vowel counting is *measurement* — content in, values out. This is the inverse of what Numbers did. Every time an agent summarizes, scores, or evaluates text, they're measuring content. Content → measurement is a fundamental noospheric operation.

**Frequency distribution is the first statistical primitive.**

The per-vowel breakdown (`{a: 3, e: 5, i: 2, o: 1, u: 0}`) is a frequency distribution. This is the simplest instance of a pattern that scales to full NLP: word frequency, topic distribution, sentiment scoring, readability indices. The resolver's `@breakdown` pattern — measure a property across elements, return the distribution — appears here for the first time.

**Context-dependent classification: the 'y' problem.**

Is 'y' a vowel? Depends who you ask. A phonetician counts it. A spelling checker doesn't. The `include_y` parameter isn't resolving ambiguity — it's expressing *perspective*. In InnateScript, this maps to agent role: the same analysis function produces different results depending on which agent is asking. The `where` doesn't decide if 'y' is a vowel. The agent's role does.

"Why try fly by my dry cry?" — zero vowels without y, seven with it. Same text, different measurement, depending on perspective. This is the clearest demonstration yet that the resolver needs to support parameterized analysis where the parameter reflects the agent's frame, not the data's properties.

**Derived metrics are micro-`where` expressions.**

`vowel_ratio` = vowel count / letter count. It combines two measurements into a single score. This is exactly what `where` expressions do at the choreography level: combine measurements from multiple agents into a composite judgment. The vowel ratio is a `where` in miniature.

Eighteen projects in. Three into Text, and three content operations have emerged: transformation (reverse), rule-based transformation (pig latin), and measurement (vowel count). The next question: what happens when you combine them?

---

## G019 — Check if Palindrome

**The first composition.**

`is_palindrome(s) = (s == reverse(s))`. The palindrome check didn't need a new primitive — it's reverse (G016) + equality. This is the first time the Rosetta Stone builds on its own earlier work. The resolver's vocabulary is growing, and new operations emerge from combining existing ones rather than inventing them.

**Normalization is projection.**

"Race car" is not a palindrome. "racecar" is. The difference is what you choose to ignore — case, spaces, punctuation. Normalization is projection: reducing content to its essential structure before comparison. This is the `where` in disguise. Every `where` clause is a normalization — it defines which details matter and which don't. The normalization level determines the answer.

**Substring search is local-to-global.**

The longest palindrome substring uses expand-around-center: treat each character as a potential center of symmetry, expand outward, keep the best. This is a search pattern — enumerate local candidates, test each, keep the winner. It's the text-domain version of Dijkstra's graph search: explore from a starting point, expand, optimize.

Nineteen projects in. The composition insight is significant: the resolver's operations are starting to layer. Reverse is a primitive. Palindrome is reverse + equality. Normalized palindrome is projection + reverse + equality. The layers build.

---

## G020 — Count Words in a String

**`@breakdown` is a generic primitive.**

G018 counted vowels (character-level breakdown). G020 counts words (word-level breakdown). The operation is identical: count occurrences of each element, return the distribution. `@breakdown{text: T, by: "character"}` and `@breakdown{text: T, by: "word"}` are the same primitive at different granularities. The Text category is confirming that granularity is a parameter, not a separate operation.

**The statistical primitives are accumulating.**

G018 gave us count and ratio. G020 adds distribution (word frequency), cardinality (unique words), diversity ratio (unique/total), and mean (average word length). The statistical vocabulary: count → ratio → distribution → cardinality → diversity → mean. Each new project adds a statistical operation that `where` expressions can use.

**Diversity ratio is a content quality metric.**

unique_words / word_count: low means repetitive, high means varied. Agents use this in `where` expressions: "is this text sufficiently rich?" This is a micro-`where` — a single measurement that captures a quality dimension. The editorial `where` will combine multiple micro-`where` metrics (diversity, readability, density) into a composite score.

Twenty projects in. The granularity axis is clear: characters (G018), words (G020), and the Text category will eventually reach sentences, paragraphs, documents. Same operations at every level, parameterized by `:by`.

---

## G021 — Text Editor

**The first stateful object.**

Everything before G021 was stateless — functions that take input and produce output. The text editor has explicit, mutable state: a buffer (content), a cursor (position), an undo stack (history). Each operation *transforms* the state. This is the Rosetta Stone's first confrontation with state management.

**The undo stack is event sourcing.**

Every edit is recorded as an event. The current buffer is the result of applying all events from the beginning. Replay forward: you get the document. Replay backward: you undo. The undo stack IS the document's history. In InnateScript, every choreography could carry an event log — a history of agent actions that can be replayed to reconstruct or revert state.

**The buffer is a document. The vault is a collection of buffers.**

Each vault file is a TextEditor buffer. The text editor isn't a utility — it's the *primitive the vault is made of*. Lines, positions, navigation, search, replace — these are the operations that agents perform on vault content. The text editor model is the vault's API.

**Cursor is attention.**

Where in the document are you focused? The cursor is the simplest model of agent attention — a point of focus in structured content. When Sylvia reads a document, she has a cursor. When Lena summarizes, she scans. When Vincent formats, he moves character by character. Different agents, different cursor patterns, same buffer.

**Editor operations are a command language.**

Insert, delete, move, undo — a DSL for manipulating text. This is eerily close to what InnateScript itself is: a command language for manipulating the noosphere. The text editor's command set is a micro-InnateScript. The `define-shape` in the InnateScript spec shows state + commands + history as a unified construct — a pattern that could generalize to any stateful entity in the noosphere.

**Collaborative editing is the choreographic frontier.**

When multiple agents share a buffer, you get the OT/CRDT problem: concurrent edits, conflict resolution, attributed undo. InnateScript's choreography model handles this naturally — roles (who edits), phases (when they edit), and the undo stack attributes each edit to an agent so conflicts can be resolved by agent priority.

Twenty-one projects in. G021 is the most conceptually dense project so far. It introduces state, history, attention, command languages, and collaborative editing in a single problem. The text editor is where data (buffer) meets behavior (commands) meets history (undo) meets attention (cursor). It's the most complete model of agent-document interaction in the Rosetta Stone.

---

## G022 — RSS Feed Creator

**From string to document.**

RSS is the first structured content in the Rosetta Stone. An RSS feed isn't text you manipulate character by character — it's a document with typed fields in a hierarchy: feed > channel > items, each with title, link, description, pubDate. Structure isn't just formatting — it's meaning. The jump from G021's line-based buffer to G022's schema-defined document is the jump from "string" to "structured content."

**Serialization is how agents communicate across boundaries.**

`to_xml()` / `from_xml()` is the first roundtrip format in the Rosetta Stone. Content goes to wire format and back without loss. This is serialization — the mechanism agents use to communicate across process boundaries, machine boundaries, time boundaries. When an agent persists state to the DB and another agent reads it, serialization is the bridge. InnateScript's resolver needs serialization primitives because agents may not share memory.

**RSS is a publication channel: one producer, many consumers.**

This maps directly to InnateScript's broadcast pattern. An agent publishes to a channel, subscribers consume. The dpn-reader on Nathan's desktop IS an RSS consumer. The vault's temporal chain could be exposed as an RSS feed — each daily note is an RSS item with a title (date), content (the day's work), and a publication date. The Rosetta Stone keeps building infrastructure that already exists in the vault.

---

## G023 — Post it Notes Program

**The first data store.**

G012's city table was read-only lookup. G021's text editor was a single document with cursor operations. The note board is a *collection* of documents with CRUD operations — create, read, update, delete. This is the database pattern at its simplest. Every vault, every table, every collection is a note board with different field names.

**Identity makes content addressable.**

Each note has an ID — the first time the Rosetta Stone assigns identity to content. Strings and numbers are values; notes are *entities*. Two notes with the same content are still different notes because they have different IDs. Identity is what makes content mutable: without an ID, you can't say "update *that* note" — you can only say "update all notes that look like this." Identity is the difference between a value and an entity.

**Search is the simplest query.**

`search_notes(query)` filters a collection by a predicate. This is `SELECT * FROM notes WHERE content LIKE '%query%'` — the most basic database query. The resolver needs query primitives because agents need to find things in collections. G020's word frequency was measurement; G023's search is retrieval. Different operations, both driven by content analysis.

The note board IS the vault in miniature. Notes = vault_notes table. Add = create file. Search = grep. Delete = archive. Twenty-three projects in and the Rosetta Stone keeps building things that already exist in Nathan's infrastructure.

---

## G024 — Quote Tracker

**Curation adds metadata that transforms a list into a knowledge structure.**

G023 was flat CRUD. G024 adds a curation layer: tags, author attribution, source references. These metadata fields transform a pile of notes into a navigable library. The difference between a collection and a knowledge base is curation — the metadata that makes content findable, relatable, and meaningful.

**Multi-label classification via tags.**

A quote can belong to multiple categories simultaneously. G018's `include_y` was binary. G013's card network was single-dispatch. Tags are multi-label classification — each item lives in multiple overlapping sets at once. The resolver needs set-valued properties and multi-label queries. This is how the noosphere works: every entity has multiple facets, multiple roles, multiple memberships. A daily note is both a Temporal artifact AND a project log AND a personal journal.

**Nondeterminism enters the Rosetta Stone.**

`random_quote()` — the first function that doesn't return the same result for the same input. Every previous function was deterministic: same input, same output. Random selection introduces nondeterminism as a primitive. This connects to agent autonomy: when an agent chooses which approach to try, it's making a nondeterministic selection from its option space.

**Attribution is provenance.**

The author field tracks where content came from. In the noosphere, every piece of content has provenance: which agent produced it, when, in what context. The quote tracker's `author` field is the simplest model of provenance — and the vault already tracks this via frontmatter fields like `CEO`, `Executive Ghost`, and the ghost-tagged action items in daily notes.

Twenty-four projects in. The Text category has traversed from string manipulation (G016–G020) through state machines (G021) to structured content (G022) and data stores (G023–G024). The pattern: raw text → structured documents → collections of documents → curated knowledge bases. Each step adds a layer of meaning.

---

## G025 — Guestbook / Journal

**The first immutable data structure.**

G023's notes could be edited and deleted. Journal entries cannot. Once written, they're permanent. This is the event log pattern from G021's undo stack, but without the undo. The log IS the truth. No revision, no deletion — only append.

This is how the vault's daily notes work. You append to today. You don't edit yesterday. The daily note's "What I Did Today" section is a journal — chronological, authored, immutable once the day passes. The Rosetta Stone keeps building primitives that the vault already embodies.

**Time is the primary index.**

Entries are ordered by when they were written, not by content or priority. G025 is chronological-only. This connects to G011's temporal chain — the journal's timeline IS a temporal chain at the entry level. Date-range queries (`get_entries_in_range`) are the first temporal queries in the Rosetta Stone: "what happened between Monday and Wednesday?"

**Append-only + chronological + attributed = audit trail.**

Author on every entry. Timestamp on every entry. No edits. This is the accountability pattern: you can always answer "who wrote what, when." The vault's git history is an audit trail. The DB's conversations table is an audit trail. The journal is the simplest expression of this pattern.

---

## G026 — News Ticker and Game Scores

**Priority enters the picture.**

G025's journal was chronological only. The ticker adds a second ordering axis: importance. Priority + time = how agents triage incoming information. Items sorted by priority first, then recency within the same priority. This is the first multi-key sort in the Rosetta Stone, and it's the one that matters most for agent attention management.

**Temporal scope: items expire.**

TTL (time-to-live) means items are valid for a window, then gone. This is G011's alarm clock in reverse: instead of "do X when time arrives," it's "stop showing X when time expires." Not everything is permanent. Some information is only relevant right now. The noosphere needs both permanent records (G025's journal) and ephemeral signals (G026's ticker). Temporal scope is how agents manage attention — the ticker auto-prunes so agents aren't drowning in stale data.

**Breaking news is an interrupt.**

Max-priority insertion that preempts normal flow. In InnateScript, this maps to forced output — an alert that demands immediate attention regardless of what else is happening. The choreography must handle interrupts: a breaking alert from Kathryn (market crash) preempts Eliana's scheduled infrastructure report. Not all information waits its turn.

**The ticker is the dpnbar.**

Nathan's Quickshell bar already IS a ticker — widgets displaying real-time status (clock, battery, bluetooth, network) sorted by spatial priority (left to right). The Rosetta Stone is building the data model behind the bar.

---

## G027 — Fortune Teller

**Constrained nondeterminism.**

G024 introduced raw nondeterminism (random from entire pool). G027 adds constraints: random, but *from category X*. This is stratified sampling — nondeterministic selection within defined bounds. The resolver's randomness needs to support constraints. Unrestricted random (G024), constrained random (G027), and deterministic dispatch (G013) are three points on a spectrum from full autonomy to full control.

**Context-independent response: the anti-pattern.**

`ask_question(question)` acknowledges the question but responds from its pool regardless. The question doesn't influence the answer. This is the simplest model of an agent that listens but doesn't truly respond to input — it has its own agenda independent of the query. Naming this anti-pattern is useful: agents that ignore context are fortune tellers. Good agents are the opposite — their responses are shaped by the query.

**Nondeterminism + scheduling + content = composition.**

The daily inspiration case: the temporal alarm (G011) fires each morning, triggers a fortune selection (G027) from a category that rotates by day of week, and injects the result into the daily note (G025). Three Rosetta Stone concepts composing naturally. The projects aren't just accumulating — they're starting to *combine*.

Twenty-seven projects in. The Text category's data model arc: flat collections (G023) → curated collections (G024) → immutable logs (G025) → priority streams with expiry (G026) → constrained random pools (G027). Each step adds a different access pattern to the same basic idea of "a collection of text items."

---

## G028 — Vigenere / Vernam / Caesar Ciphers

**Reversible transformation with a secret.**

G017's Pig Latin was reversible with a public rule. Ciphers are reversible with a *private* rule — the key. The key introduces shared secret: two parties must agree on it before communication. In InnateScript, this maps to private agent-to-agent channels. The choreography may be public, but the content is opaque to agents without the key.

**The security spectrum: fixed < repeating < random.**

Caesar (26 possible keys, trivially broken by enumeration), Vigenere (repeating key, breakable by frequency analysis), Vernam (one-time pad, theoretically unbreakable). The progression shows that security scales with key entropy relative to message entropy. Perfect security (Vernam) requires a key as long as the message, used only once — the cost of perfect privacy equals the cost of the thing it protects.

This is a deep constraint. In the noosphere, maintaining a truly private channel between two agents requires as much coordination overhead as the communication itself. Most agents settle for good-enough encryption because perfect security doesn't scale. The trust model is a cost-benefit trade.

**Encryption is content-level information hiding.**

G013 masked part of a value (showing last 4 digits). Ciphers hide ALL the content. Only agents with the key can read it. This is access control via cryptographic means, enforced by mathematics rather than by the choreography's structure. The choreography says who *should* see the data. The cipher ensures only they *can*.

---

## G029 — Random Gift Suggestions

**Attribute-based matching: the first recommendation engine.**

Unlike exact search (G023) or category filter (G026), gift suggestion scores by *relevance* — count how many of the recipient's traits match the gift's compatible traits, sort by score. A gift matching 3/4 traits ranks higher than one matching 1/4. This is fuzzy matching, and it's the primitive behind every recommendation system.

**Profiles are how agents find each other.**

The recipient has traits. The gift has compatible traits. Matching profiles is how agents discover each other in the noosphere: an agent with capability X matches a task requiring capability X. Gift suggestion is agent-task matching in disguise. The `suggest()` function is `@find_agent{needs: ["scheduling", "calendar"]}` generalized.

**Filter on top of ranking: optimize within bounds.**

Budget constraint applies after relevance scoring — first rank by match quality, then filter by what you can afford. This is G015's Dijkstra with constraints: optimize (shortest path / best match), then prune (budget / capacity). The pattern: score → filter → select. The `where` scores the match; the budget is a `<-` gate.

---

## G030 — Text to HTML Generator

**Format translation: same content, different representation.**

Text-to-HTML is G008's base conversion (binary↔decimal) and G010's unit conversion (km↔miles) applied to documents. The content is the same; the representation changes. The resolver's normalize-to-canonical pattern applies: parse markup → internal structure → emit target format.

**This is how the vault renders.**

Every vault note is markdown. Every rendered view is HTML (or will be). The text-to-html converter is the rendering pipeline that sits between vault content and user-visible output. The Rosetta Stone is building the vault's renderer, one formatting rule at a time.

**Multi-granularity parsing.**

The converter operates at two levels simultaneously: block structure (paragraphs, headers, lists — line-level) and inline formatting (bold, italic, code, links — character-level). Block parsing happens first, then inline formatting runs within each block. This is the first multi-granularity parser in the Rosetta Stone — and it mirrors how agents process documents: understand the structure first, then the details within each section.

**Pattern-based transformation at scale.**

The inline rules (`**bold**` → `<strong>`, `*italic*` → `<em>`, `` `code` `` → `<code>`, `[text](url)` → `<a>`) are the same class of operation as G017's Pig Latin: scan for patterns, transform matches, preserve non-matching content. But where Pig Latin had one rule applied per-word, the HTML converter has multiple rules applied within each block. The transformation pipeline is: escape entities → apply patterns in order → wrap in block tags.

Thirty projects in. The Text category has completed its arc from raw strings to structured content: manipulation (G016–G020) → state machines (G021) → formats and serialization (G022, G030) → data stores (G023–G027) → security (G028) → recommendation (G029). Two projects remain.

---

## G031 — CD Key Generator

**Self-verifying tokens: proof embedded in the artifact.**

A CD key carries its own verification. The last group is a checksum of the first four groups. Anyone can validate a key by recomputing the checksum — no database lookup, no network call, no external authority. The key IS its own proof.

This is the opposite of G012's fact lookup, which required external state. Self-verifying tokens are *offline verification* — they scale infinitely because each validation is independent. In InnateScript, this maps to agent credentials: a token that proves authorization without calling back to a central authority. The issuing agent and the validating agent don't need to communicate. The token is the communication.

**The checksum is a simplified digital signature.**

The issuer computes the checksum (signs). The validator recomputes (verifies). The security is weak (the algorithm is public), but the pattern is real. Real digital signatures use asymmetric cryptography — the signing key is private, the verification key is public. CD keys use symmetric verification — anyone who knows the algorithm can both generate and verify. The difference is the trust model.

**Batch generation is `@map` over nondeterminism.**

`generate_batch(n)` applies a nondeterministic function n times independently. Each key is independently random. This is G017's `@map` + G024's nondeterminism combined. The independence is key: batch generation is embarrassingly parallel.

---

## G032 — Regex Query Tool

**Regex is the substrate of text manipulation.**

Every text operation in the Rosetta Stone — reverse, palindrome, cipher, HTML conversion — could be expressed as a sequence of regex operations. Regex is the primitive underneath text manipulation, the way Dijkstra was the primitive underneath graph operations. G030's `**bold** → <strong>` is just `s/\*\*(.*?)\*\*/<strong>\1<\/strong>/g`. The converters and transformers of the Text category are all special cases of regex replace.

**Pre-built patterns are domain schemas expressed as strings.**

EMAIL, URL, PHONE, DATE, IP_ADDRESS — each pattern encodes a domain's structural rules as a string. An email has `local@domain.tld`. A date has `YYYY-MM-DD`. The regex is the schema. This is the text equivalent of G022's RSS schema — structured content described by its pattern, verifiable by matching.

**`extract_groups()` is ETL for the noosphere.**

Capture groups pull typed fields out of free text. A log line becomes `(timestamp, level, message)`. An email becomes `(local, domain)`. This is how agents parse natural language into structured data: the regex is the schema, the text is the input, the groups are the fields. Structured extraction from unstructured content is the bridge between human-readable and machine-processable.

---

## Meta-observation: The Text Category (G016–G032)

**Seventeen projects. One content architecture.**

The Text category discovered InnateScript's content architecture, the way Numbers discovered the computational architecture:

| Project | What it revealed |
|---------|-----------------|
| G016 Reverse | **Content vs values.** Text carries meaning. Granularity axis (chars vs words). |
| G017 Pig Latin | **`@map` primitive.** Per-element transformation. Content/presentation split. Reversibility. |
| G018 Count Vowels | **Measurement.** Content → values. `@breakdown` primitive. Agent perspective via parameters. |
| G019 Palindrome | **Composition.** Building on prior operations. Normalization as projection. Content search. |
| G020 Count Words | **Generic `@breakdown`.** Same operation at different granularities. Statistical vocabulary grows. |
| G021 Text Editor | **State machines.** Event sourcing. Cursor as attention. Editor ops as command language. |
| G022 RSS Feed | **Structured content.** String → document. Serialization. Publication channels. |
| G023 Post-it Notes | **Data stores.** CRUD. Identity makes content addressable. Search as query. |
| G024 Quote Tracker | **Curated collections.** Multi-label classification. Nondeterminism. Provenance. |
| G025 Guestbook | **Immutable logs.** Append-only. Time as primary index. Audit trails. |
| G026 News Ticker | **Priority streams.** Multi-key sort. Temporal scope (TTL). Interrupts. |
| G027 Fortune Teller | **Constrained nondeterminism.** Stratified sampling. Context-independent response (anti-pattern). |
| G028 Ciphers | **Encryption.** Shared secrets. Security spectrum. Content-level information hiding. |
| G029 Gift Suggestions | **Recommendation.** Attribute matching. Profiles. Filter on top of ranking. |
| G030 Text to HTML | **Format translation.** Multi-granularity parsing. Vault rendering pipeline. |
| G031 CD Key Generator | **Self-verifying tokens.** Offline verification. Embedded proof. Batch nondeterminism. |
| G032 Regex Query | **Universal pattern matching.** Domain schemas as patterns. Structured extraction from free text. |

The content architecture: `@map` for per-element transformation + `@breakdown` for measurement + state machines with event sourcing + serialization for cross-boundary communication + CRUD for collections + regex as the universal substrate. Each project was a facet. Together they describe InnateScript's content layer — how agents create, transform, store, query, protect, and render text.

Numbers gave us the resolver's computational core. Text gives us the resolver's content layer. Together: a computational engine that operates on meaning-bearing content.

The Text category is complete.

---

## G033 — FTP Program

**The first protocol.**

Every project before G033 was self-contained — functions calling functions within a single process. FTP introduces a *boundary*. The client and the server are distinct entities communicating through a defined command language: LIST, RETR, STOR, DELE, SIZE, QUIT. The client doesn't access the filesystem directly. It can only ask, through the protocol, and accept what comes back.

This is what `@reference` resolution IS. Every `@agent/operation` in InnateScript is a request-response across a boundary. The resolver doesn't reach into agents — it speaks the protocol. FTP makes this pattern explicit and named. The command string is the request. The status-code-prefixed response is the reply. The protocol is the contract.

**The command language IS a micro-InnateScript.**

LIST is `@server/list-files`. RETR is `@server/get`. STOR is `@server/put`. The FTP command vocabulary maps one-to-one onto named operations in a `define-role`. The text-based command protocol and InnateScript's `@reference` syntax are the same thing at different levels of formality — a string that names an operation and supplies arguments, sent across a boundary to an entity that executes it and returns a result.

**The virtual filesystem is a remote resource.**

G023's post-it notes were a local collection — direct access, no intermediary. The FTP server's filesystem is a *remote* collection — the same CRUD operations, but gated by the protocol. The client can't `files.insert()`. It must `STOR filename content` and accept the response code. This indirection is the difference between a local variable and a database table. The vault's notes aren't local to any agent — they're behind `dpn-ipc`, which IS the FTP protocol wearing different clothes.

**Client-server is the simplest choreography.**

Two roles, strict request-response, no concurrency. The client initiates. The server responds. One message at a time. This is the degenerate case of choreography — a dance with exactly two participants and a turn-taking rule so simple it doesn't need `concurrent` or `join`. Everything more complex builds on this.

Thirty-three projects in. The Networking category begins where the Rosetta Stone crosses from self-contained computation to distributed communication. The first thing it discovers: every `@reference` was already an FTP command.

---

## G034 — Get Atomic Time from Internet Clock

**Consensus on "now."**

FTP exchanged *content* across a boundary. Atomic time exchanges *truth* — an authoritative answer to "what time is it?" The server doesn't store files. It holds a reference clock. The client doesn't want data. It wants to correct its own clock. This is the first *calibration* protocol in the Rosetta Stone: a local state (my clock) is measured against an authoritative state (the server's clock), and the difference is used to correct the local state.

**Every distributed system needs a shared "now."**

The temporal chain — Daily Notes, Weekly, Monthly — assumes everyone agrees when midnight is. When agents run on different machines with different clocks, they don't. The daily note triggers at midnight *whose* midnight? NTP answers this: everyone syncs to the same authority, bounding the disagreement to milliseconds. Without time consensus, the temporal chain is a fiction — agents write to yesterday's note while others write to today's.

**Latency introduces fundamental uncertainty.**

The server sends its timestamp at time T. The message takes D seconds to arrive. The client receives it at T+D but thinks it's receiving T. The round-trip estimation (measure the total trip, divide by two, assume symmetric latency) is a *best guess*. Perfect synchronization is impossible over a network. The noosphere doesn't need perfect sync — it needs bounded error. As long as agents agree on which *day* it is, the temporal chain works. NTP's millisecond precision is overkill for daily notes, but essential for transaction ordering.

**Calibration is a general pattern.**

Time sync is calibration applied to clocks. But the pattern — sample local vs. remote, compute offset, average samples to reduce noise, apply correction — generalizes to any quantity that drifts. Vault replica reconciliation: how far has my local copy diverged from the droplet? Agent belief calibration: how far has my model of the world diverged from observed reality? The calibration protocol is a reusable pattern, and G034 is its simplest instance.

**Multiple samples reduce noise.**

A single query gives one offset measurement, contaminated by variable latency. Three queries give three measurements that can be averaged. The more samples, the better the estimate. This is the first statistical protocol in the Rosetta Stone — not statistical analysis of data (G018-G020 did that) but statistical improvement of a *measurement process*. The `where` for time sync isn't "is the offset zero?" — it's "is the bounded error acceptable?"

Thirty-four projects in. G033 gave us the request-response protocol. G034 gives us the calibration protocol — and with it, the first consensus primitive. Agents that agree on time can coordinate. Agents that don't are just shouting into the void.

---

## G035 — Chat Application (Networking)

**The first multi-party choreography.**

G033's FTP was bilateral — one client, one server, strict turn-taking. Chat shatters this: N users, M rooms, messages fan out many-to-many. Every member of a room is simultaneously publisher and subscriber. The room IS a topic. The membership list IS the subscription registry. `send_message` IS `publish` scoped to a channel. This is pub-sub evolved from G022's one-to-many RSS into a fully symmetric many-to-many topology.

**Dynamic membership introduces lifecycle.**

FTP's client connected once and stayed connected. Chat users join and leave rooms mid-conversation. This is new — agents entering and exiting a choreography while it's running. Every prior pattern assumed fixed participants. Chat forces the questions: does a new member see backlog? Does a departed member's messages persist? Does an empty room dissolve? These are agent lifecycle questions, and the noosphere needs answers for all of them.

**History is a scoped journal.**

`get_messages(room, since)` is exactly G025's `get_entries(since)` with a namespace. The room is the journal, the message is the entry, the timestamp is the cursor. Message history is journaling with a scope parameter — proving that the journal pattern from Text generalizes across contexts. The daily note is a room called "today." The conversations table is a set of rooms. Same pattern, different scope.

**Rooms are the unit of isolation.**

Messages in `#general` don't leak into `#engineering`. Each room is its own world. This is the first encounter with *namespace isolation* in the Rosetta Stone — the guarantee that operations in one scope don't affect another. In InnateScript, choreographic scopes provide this isolation: agents inside one choreography don't see the state of another unless explicitly bridged.

**The chathud is the living instance.**

Nathan's Quickshell chathud reads and writes the `conversations` table via `dpn-ipc`. It has rooms (channels), users (nathan, agents), messages with timestamps, membership. The six implementations in the Rosetta Stone are this same architecture expressed in six syntaxes. The Rosetta Stone keeps building things that already exist — and this time, the thing it built is the primary communication channel between human and agents.

**Membership enforcement is a `<-` gate.**

`send_message` checks `room.members.contains(user)` before accepting the message. This is G013's structural validation reappearing as an access control gate: you must be a member to speak. The gate is cheap (set lookup), the operation it protects is meaningful (persisting a message to shared state). Progressive trust: first you register, then you join, then you speak. Each step is a gate that enables the next.

Thirty-five projects in. Three into Networking, and the progression is clear: bilateral protocol (G033) → consensus protocol (G034) → multi-party messaging (G035). The communication layer is building from point-to-point to broadcast to fully connected mesh. Each project adds a dimension of coordination complexity: boundaries, then shared reference frames, then dynamic group membership.

---

## G036 — Fetch Current Weather

**The resolver reaches the internet.**

G012 introduced the fact store — world-state from a local database. G036 extends this to a *live external authority*: an HTTP API that returns a snapshot of the atmosphere, valid for minutes at best. The city table in G012 was static. Weather is the first **ephemeral fact** — a value that's true when you receive it and false by the time you act on it.

This is the resolver's first encounter with **fact freshness as a first-class concern**. `@weather{city: "Jacksonville"}` doesn't return a cached constant. It returns a snapshot with an implicit TTL. The resolver must treat the result as time-bounded: useful now, stale soon, gone by tomorrow. G026's temporal scope (TTL on ticker items) reappears as a data-level concern — not "stop showing this item" but "stop trusting this value."

**HTTP is FTP generalized.**

G033's bilateral request-response protocol returns with a universal addressing scheme. `GET /data/2.5/weather?q=Jacksonville` is `RETR weather-jacksonville` with headers, status codes, and structured responses. But the key difference: the FTP response was a raw file. The HTTP response is **structured data** (JSON) that must be parsed into typed fields. This is G022's serialization pattern (structured content across a boundary) applied to an API response. The resolver needs HTTP + JSON parsing as a combined primitive: fetch and structure in one operation.

**API keys are shared secrets applied to trust.**

G028 introduced shared secrets between two parties. The API key is the same pattern applied to service trust: the key proves identity, gates access, and tracks usage. Without it, the server returns 401. This is G013's progressive trust model applied at the network boundary: the key is the cheapest `<-` gate (does this client have permission?), and the weather data is the expensive operation it protects.

**Frame translation returns, live.**

G010 discovered that agents think in frames and choreographies negotiate at boundaries. Weather data arrives in Kelvin — the API's canonical frame. Nathan thinks in Fahrenheit. European agents think in Celsius. The `@convert` pattern from G010 is embedded in every weather report: temperature stored in the canonical frame (Kelvin) and projected into the consumer's frame on demand. Same data, different views. The canonical form is what the API returns. The projections are what agents consume.

**Comparison is concurrent fact gathering.**

`compare_weather([Jacksonville, London, Tokyo])` requires three independent HTTP requests. Each returns a snapshot. The comparison aggregates snapshots into a composite view: warmest, coldest, most humid, windiest. This is G012's proximity-based coordination pattern: gather facts from multiple sources concurrently, compute derived metrics in parallel, aggregate into a decision-relevant structure.

The `where` for a weather-aware choreography isn't "is the weather good?" — it's "are the facts fresh?" Stale weather is worse than bad weather. At least bad weather is true.

Thirty-six projects in. Four into Networking. The category's arc: bilateral protocol (G033) → consensus on shared reference (G034) → multi-party messaging (G035) → live external data from a remote authority (G036). Each project extends the resolver's reach: from across a socket, to across a clock, to across a room, to across the internet. The resolver is no longer a closed system. It consumes the world.

---

## G037 — P2P File Sharing App

**The first symmetric topology.**

Every network pattern before G037 had an asymmetry. FTP: client asks, server answers. NTP: client calibrates against authority. Chat: server routes between clients. Weather: client consumes from API provider. One side was always structurally different from the other.

P2P shatters this. Every peer is both client and server simultaneously. Alice serves files to Bob while requesting files from Carol. The role distinction collapses. This is the noosphere's natural topology — agents are symmetric. Any agent can request from any other agent. Any agent can serve to any other agent. The hub-and-spoke model was training wheels. The real architecture is a mesh.

**Discovery is the bootstrap problem.**

How does a peer find other peers when there's no center? You need at least one known peer to start — the bootstrap node. From there, peers exchange peer lists, and the network grows organically through gossip. In the noosphere, agent discovery works identically: a new ghost starts with one known agent (the orchestrator), learns about others through interaction, and builds its own peer table. The bootstrap is the first `@reference` that resolves.

**Catalogs are distributed indexes.**

No single peer has a global view. Each peer maintains its own index of what its neighbors share. Search is local — you query your known peers' catalogs, not a global database. The network's total knowledge is the union of all local catalogs, but no one node sees the union.

The vault works this way. Lena knows about daily notes. Kathryn knows about financial positions. Sylvia knows about publications. No single agent indexes everything. The orchestrator coordinates, but the knowledge is distributed. There is no SELECT * FROM everything. There is only "ask the agents who know."

**Chunked transfer is progressive verification.**

A file splits into chunks. Each chunk carries its own hash. The receiver verifies each chunk independently — G013's progressive trust model applied to data transfer. A corrupted chunk is rejected without invalidating the entire transfer. Re-request chunk 7, not the whole file. Partial progress is preserved.

This is how large choreographies should fail: if step 7 of 10 fails, re-execute step 7. The dance doesn't restart from the beginning. The chunk is the atomic unit of both trust and retry. The insight from G013's `<-` gates extends to error recovery: cheap verification gates expensive re-execution.

**Content addressing makes identity location-independent.**

A file's SHA-256 identifies it regardless of which peer hosts it. The same file on Alice's machine and Bob's machine has the same hash. You can request "file with hash X" from any peer that has it. This decouples identity from location.

G023's post-it notes used sequential IDs — assigned by a central server, meaningless without it. Content addressing is the opposite: identity derived from content itself, requiring no authority. This is the difference between a database primary key and a content hash. The primary key says "row 47 in this table." The hash says "this exact content, wherever it lives." The noosphere should use content addressing for knowledge artifacts — a note's identity is what it says, not where it's stored.

**Staleness and liveness enter the peer model.**

Peers go offline. Peers' catalogs become stale. The `prune_stale_peers` function removes peers not heard from within a timeout — G026's TTL pattern applied to network membership. A peer that was alive five minutes ago might be gone now. The network's topology is dynamic, not static. Every connection is provisional.

This is the first time the Rosetta Stone explicitly models liveness — the ongoing question "is this entity still there?" NTP (G034) assumed the server was always available. Chat (G035) tracked join/leave but within a stable server. P2P has no stable anything. Everything is provisional, everything expires, and the network survives because it doesn't depend on any single node.

Thirty-seven projects in. Five into Networking. The topology progression: point-to-point (G033) → client-authority (G034) → client-server-clients (G035) → client-API (G036) → **peer-to-peer mesh (G037)**. The asymmetry has dissolved. From here, every networking project builds on a foundation where nodes are equal, discovery is organic, and trust is verified per-chunk.

---

## G038 — Port Scanner

**Systematic exploration of a boundary.**

Every networking project before G038 *knew* what it was connecting to. FTP connected to a file server. NTP to a time server. Chat to a chat server. Weather to an API. The destination was given. The port scanner doesn't know — it **discovers** what's listening by probing systematically. Each port is a question. The aggregate of answers is a map of the host's surface.

This is the Rosetta Stone's first reconnaissance tool. Not "connect to service X" but "what services exist?" The shift from known-target to discovery is architecturally significant. In the noosphere, this is agent capability discovery: given a new ghost, what can it do? The resolver could probe `@agent/list-capabilities` — the port scan of the noosphere. Each capability is an open port. The scan builds a profile before the choreography begins.

The ghost index in the vault — the catalog of agents and their roles — is the bootstrap node for this discovery. You don't scan every possible capability space. You start with the index and verify: is this ghost alive? Does it still support these operations? The index is the well-known port table. The scan confirms which entries have live implementations.

**Three states, not two.**

Open, closed, filtered. A port isn't binary. "Filtered" means something is silently dropping packets — a firewall, a rate limiter, network partition. The probe doesn't fail; it gets no answer at all. This is richer than success/failure. The resolver's `@reference` resolution needs the same three states: resolved (open), explicitly failed (closed), timed out with no response (filtered). G013's structural validation was binary. G038 adds silence as a distinct outcome.

The distinction matters for error handling. "Closed" means "the agent understood the request and refused." "Filtered" means "the request never reached the agent, or the agent is not responding." The retry strategy differs: retry filtered (the path may clear), don't retry closed (the answer won't change).

**Concurrent probing is the natural model.**

Scanning 1024 ports sequentially takes 1024 timeouts. Scanning concurrently takes roughly one timeout (with enough threads). Each probe is independent — no shared state, no ordering dependency. This is G017's `@map` applied to network exploration: same operation applied to each element, results collected, no inter-element coordination.

The scanner's thread pool is the resolver's `concurrent` block. Each probe runs independently. The results `join` into a report. The `where` evaluates the aggregate: is the surface larger than expected? Did something appear that shouldn't be there?

**Well-known ports are a service registry.**

Port 22 is SSH. Port 5432 is PostgreSQL. The mapping from numbers to names is a global convention — a namespace. The scanner doesn't just find open ports; it translates them into meaning using the registry. The resolver's namespace serves the same function: `@weather` maps to the weather service. Port scanning discovers which entries in the registry have live implementations.

**Scan comparison is change detection.**

`compare_scans(before, after)` produces a diff: newly opened, newly closed, still open. This is the first **temporal diff** in the Networking category — not comparing two hosts, but comparing the *same host at two different times*. A sequence of diffs is a changelog. The port scanner, run on G011's alarm schedule, becomes a service monitoring system that journals its findings (G025) and alerts on unexpected changes (G026's breaking-news interrupt).

The `where` for infrastructure security isn't "are the right ports open?" — it's "did the surface change unexpectedly?" Expected changes are benign. Unexpected changes are the signal.

Thirty-eight projects in. Six into Networking. The category adds its first exploration tool: not connecting to a known service, but discovering what services exist. The pattern — probe, classify, aggregate, compare over time — is the foundation of monitoring, and it's built from primitives the Rosetta Stone already established: `@map` for parallel probing, three-state classification, temporal comparison, and service registries as namespaces.

---

## G039 — Mail Checker

**Polling is the alarm clock applied to external state.**

G011 gave us the alarm — a temporal trigger. G036 gave us live external data. G039 combines both: poll an external source on a schedule, report what changed since the last check. The daily note template IS a mail check: `(@SarahLin){task_audit}` polls the task table, filters for changes, reports what's new. Every recurring check in the vault is a mail checker with a different inbox.

The mail check is the universal monitoring primitive. Any time an agent asks "what's new since I last looked?" — that's a mail check. The inbox is just the most recognizable instance. The pattern: remember what you've seen, compare against what exists now, report the delta.

**The inbox is a filtered priority stream.**

G026's ticker was a priority stream with TTL. The inbox extends it: messages arrive unsolicited (push), accumulate until checked (queue), and the consumer controls when they look and what they look at. The filter pipeline is G013's progressive gates applied to content: sender filter → subject filter → body filter → since filter. Each gate reduces the result set. The `where` for an inbox check isn't "did mail arrive?" — it's "did mail arrive *that matters*?"

This is attentional economy. The agent doesn't process every message. It processes the messages that pass its filters. The filter IS the agent's attention policy. Different agents with different filters reading the same inbox see different realities — G018's vowel-count perspective parameter, scaled to messaging.

**Read-state is agent attention.**

Three states: unread (unprocessed), read (acknowledged), flagged (actionable). This is the agent's triage protocol. The unread count is attention debt — how much input hasn't been processed. The flagged count is action debt — how much acknowledged input still requires response.

The daily note has the same three states: sections that haven't been filled yet (unread), sections that have content (read), sections with action items (flagged). The daily note IS an inbox. The morning pages section is a message from Nathan to the vault. The `(@SarahLin){task_audit}` section is a message from Sarah. Reading the daily note is checking the inbox.

**Notification callbacks invert the temporal model.**

`on_new_mail(callback)` pushes to the agent when something arrives — event-driven rather than poll-driven. The vault uses both: the daily note template is polling (check once per section). The chathud is event-driven (new messages appear immediately). Same data, different temporal models. The agent chooses which model based on urgency: poll for batch review, subscribe for real-time.

This is the first time the Rosetta Stone explicitly models both push and pull in the same system. FTP (G033) was pull. Chat (G035) was push. The mail checker supports both — `check()` for pull, `on_new_mail()` for push — unified in one inbox model.

**The mail server is a message router.**

`send()` routes a message to all recipients with local mailboxes. This is G035's chat server stripped to its essence: accept a message, deliver to matching mailboxes. No rooms, no membership — just routing by address. The simplification reveals the core: a server is a function from recipient addresses to mailboxes. Everything else is policy layered on top.

Thirty-nine projects in. Seven into Networking. The category's arc adds its first monitoring primitive: the poll-filter-report pattern that underlies every recurring check in the vault. The mail checker isn't just email — it's the abstract shape of "what changed since I last looked?" applied to any source of incoming data.

---

## G040 — Packet Sniffer

**Passive observation — watching without acting.**

Every networking project before G040 was active — the agent initiated contact, sent requests, probed ports. The packet sniffer is the first passive tool. It doesn't connect. It doesn't send. It listens. It watches traffic flowing between others without generating any of its own.

This is a fundamentally different mode of interaction. Active tools ask questions. The sniffer observes answers flowing between others. In the noosphere, this is the monitoring agent — an entity that watches choreographies execute without participating. Lena's `{nightly_summary}` is a packet sniffer: she doesn't produce the day's work, she observes what the other agents produced and reports on the aggregate shape. Not every agent in a choreography needs to act. Some exist to observe, measure, and report. The sniffer is the prototype for the observer role.

**`@breakdown` returns at the network level.**

G018 introduced `@breakdown` — measure a property across elements, return the distribution. The packet sniffer applies the same operation to network traffic: protocol distribution, bytes per IP, connections per source, bandwidth per second. The measurement primitive is truly generic — it doesn't care whether it's counting vowels in a string or TCP segments in a capture. Partition by property, count per partition, return the distribution. Same algorithm, different domain, same insight.

**Connection tracking is bidirectional identity.**

A flow from A:1234 → B:80 and the return from B:80 → A:1234 are the same conversation. The `ConnectionKey` normalizes direction by sorting endpoints — recognizing that two flows are one entity viewed from different sides. In InnateScript, agent communication is inherently bidirectional. A request from Kathryn to Eliana and Eliana's response are the same conversation. The choreography needs connection tracking to correlate the outbound request with the inbound response.

**Anomaly detection is statistical `where`.**

`detect_syn_flood` looks for too many SYN packets without matching ACKs. `detect_port_scan` looks for too many distinct destination ports from one source. These aren't examining individual packets — they evaluate the aggregate shape of the traffic. Statistical signatures that indicate hostile behavior.

This is the `where` pattern applied to surveillance. The `where` doesn't judge individual actions. It judges the pattern of actions over time. The same pattern applies to the noosphere: an agent that sends too many requests without processing responses is exhibiting SYN-flood behavior. An agent that probes every capability without using any is port scanning. Network security patterns map directly to agent behavior monitoring.

**The capture filter is a lens.**

Different filters on the same traffic stream produce different views. Filter by protocol: see only TCP. Filter by source IP: see only one host's traffic. Filter by port: see only web traffic. The capture filter is G010's frame translation applied to observation — same raw data, different projected views depending on the observer's needs.

Forty projects in. Eight into Networking. The category has now covered both sides of the interaction spectrum: active (send, request, probe) and passive (observe, measure, detect). The sniffer adds the observer role — the first agent type in the Rosetta Stone that exists to watch, not to act. This completes the toolset: connect (G033), synchronize (G034), communicate (G035), consume (G036), share (G037), discover (G038), monitor (G039), and now observe (G040).

---

## G041 — Country from IP Lookup

**Identity-to-context resolution.**

G012 mapped city names to coordinates — a human-readable identifier enriched with geographic data. G041 inverts this: a machine identifier (IP address) mapped to geographic context. The IP itself is opaque — four numbers separated by dots. The database supplies meaning: country, city, coordinates, timezone.

This is a new resolver pattern. Given an opaque identifier, enrich it with contextual information from a lookup table. The resolver already does this with every `@reference` — `@kathryn` is an opaque name that resolves to a full agent profile. IP geolocation is the same pattern applied to network addresses. The Ghost Registry built earlier tonight is the GeoIP database of the noosphere: given a ghost name, resolve to team, role, and capabilities.

**Private addresses are unresolvable outside their scope.**

`192.168.1.1` has no country. It exists only within its local network. The lookup must recognize this and return "private" instead of guessing. In InnateScript, some `@references` are similarly scope-limited — a local variable inside a choreography has no meaning outside that choreography. The private IP check is the resolver's scope boundary: this identifier is valid, but only within a context I can't see from here.

**Binary search over sorted ranges is the resolver's dispatch table.**

The database is a sorted list of IP ranges. Lookup is O(log n) binary search. This is the same dispatch mechanism the resolver needs for `@references`: given an identifier, binary search the namespace for the matching handler. G013's card prefixes were discrete dispatch. G014's tax brackets were sequential ranges. G041 formalizes both into a single pattern: sorted ranges + binary search = fast dispatch. The resolver's namespace IS a GeoIP database where the "IP" is the reference name and the "location" is the handler.

**Proximity routing connects geography to choreography.**

`nearest_server(client_ip, server_ips)` applies G012's Haversine distance to find the closest server. This is G015's Dijkstra simplified to a single-hop case: no graph, just direct distances to all candidates, pick the smallest. In the noosphere, agent selection could use the same pattern: when multiple agents can fulfill a request, choose the nearest — whether "nearest" means geographic proximity, organizational proximity (same department), or capability proximity (best skill match).

**Country breakdown is `@breakdown` applied to identity.**

`country_breakdown(ips)` partitions IPs by country and counts per partition. This is G018's vowel frequency and G020's word frequency applied to network identities. The measurement primitive continues to generalize across every domain the Rosetta Stone touches: characters → words → protocols → countries. Same algorithm. Different partition key. Same insight.

**The droplet resolves.**

`144.126.251.126` → United States, New York. The DigitalOcean droplet that hosts the DPN database has a geographic identity. The SSH tunnel from localhost:5433 to the droplet crosses 900 miles. The IP lookup quantifies what was already known: the data lives in New York, the client sits in Jacksonville.

Forty-one projects in. Nine into Networking. The category adds context resolution: enriching opaque identifiers with meaning from a lookup table. The pattern connects back to the Ghost Registry — the bootstrap node is the noosphere's GeoIP database, and every `@reference` resolution is a lookup in that table.

---

## G042 — Whois Search Tool

**Ownership — who controls this name?**

G041 answered "where is this address?" Whois answers "who owns this name?" The domain is the identity. The whois record is its provenance: who registered it, when, through whom, and when it expires. This is the first time the Rosetta Stone queries for ownership of a network identity — not what it is or where it is, but who it belongs to.

In the noosphere, every `@reference` has an owner. The Ghost Registry is the whois database of the noosphere. A query on `@finance_positions` returns: registrant is Kathryn Lyonne, registered to the Success Department, capabilities include `@cover_obligations` and `@pace_check`, status active. The parallel is exact.

**Registration is temporal — names expire.**

Domains have creation dates, update dates, and expiry dates. A domain that exists today might expire tomorrow. The name is a renewable lease, not a permanent grant. This connects to G037's peer staleness and G026's TTL: identities in the network are time-bounded.

In InnateScript, agent assignments could follow the same model. Kathryn's delegation of `@finance_positions` was granted at a point in time. The `check_expiry` function — warn about domains expiring within N days — maps to proactive monitoring: whose capability delegation needs renewal? Which agent assignments are about to lapse?

**Semi-structured text parsing is G032 returned.**

Whois responses are nearly-but-not-quite structured. Each registrar formats differently. `Registrar:` vs `Registrar Name:`. `Creation Date:` vs `Created Date:`. The parser must handle multiple surface forms for the same concept. This is G032's regex toolkit applied to a real-world protocol where normalization precedes extraction.

The challenge scales: the resolver will face the same problem with agent communication. Different agents express the same concept differently. The parser that handles "Creation Date" and "Created Date" is the same parser that handles an agent saying "task complete" vs "finished" vs "done." Normalization of surface forms into canonical concepts.

**Record comparison is domain-level change detection.**

`compare_records(before, after)` is G038's `compare_scans` applied to registration data. Registrar change → possible domain transfer. Name server change → possible DNS hijack. Expiry date change → renewal or lapse. The temporal diff pattern is now fully general across the Rosetta Stone: port scans, inbox deltas, and now registration records. Always the same shape: snapshot, snapshot, delta. The domain determines what the delta means.

**The registrar is the trust anchor.**

The registrar is the entity that vouches for the domain's ownership. This is a trust chain: you trust the whois record because you trust the registrar, and you trust the registrar because ICANN accredited it. G013's progressive trust gates at the domain registration level. The chain: ICANN → registrar → registrant → domain → content. Trust flows downward through delegation.

Forty-two projects in. Ten into Networking. The category now covers identity at three levels: where it is (G041 geolocation), who owns it (G042 whois), and what it does (G038 port scan). Together they form a complete identity profile for any network entity — location, ownership, and capability. The same three dimensions the Ghost Registry captures for each agent.

---

## G043 — Zip / Postal Code Lookup

**The third resolution layer.**

G012 resolved city names to coordinates. G041 resolved IP addresses to countries. G043 resolves postal codes to locations. Three layers of identity-to-context resolution, each mapping a different identifier to geographic data. The resolver's pattern is consistent across all three: take an opaque identifier, look it up, return enriched context. The identifier type varies. The operation doesn't.

**Postal codes are hierarchical identifiers.**

US zip codes encode hierarchy in their digits: `3` = southeast, `32` = northeast Florida, `322` = Jacksonville area. The first digit routes to a region. The first three narrow to a sectional center. All five specify the delivery area. This is namespace hierarchy — the same structure as `@success.strategic.kathryn` or `www.example.com`. The prefix routes, the suffix specifies.

The resolver's namespace could follow postal code logic: the first segment routes to the department, the next to the team, the last to the agent. Hierarchical dispatch where each level narrows the resolution space. This is cheaper than flat lookup when the namespace is large — you don't search 64 ghosts, you search 7 departments then 10 team members.

**Radius search is proximity discovery.**

`codes_within_radius(center, km)` finds all entries near a geographic point. This is G029's gift suggestion (attribute-based matching with scored results) applied to physical space. G038's port scan enumerated ports on a host. G043's radius search enumerates locations near a point. The pattern — enumerate candidates, measure distance, filter by threshold, sort by proximity — is the same across domains.

In the noosphere, capability search works identically: "find all agents within 2 skill-hops of this capability" is a radius search in capability space. The metric changes from kilometers to skill similarity, but the algorithm doesn't.

**Multiple indexes serve different query patterns.**

The database has three indexes: by-code (exact lookup), by-city (reverse lookup), by-state (regional aggregation). Same data, different entry points. The resolver needs the same flexibility: look up an agent by name, search by capability, filter by department. Each access pattern requires its own index into the same underlying registry.

**Jacksonville to St. Augustine: ~50 km.**

The Haversine distance between 32202 (Jacksonville) and 32084 (St. Augustine) is approximately 50 kilometers. The same road Nathan drives to [[The Amp]], quantified by the postal code database. The Rosetta Stone keeps building tools that describe the world the user inhabits.

Forty-three projects in. Eleven into Networking. The identity resolution trilogy is complete: IP → country (G041), domain → owner (G042), postal code → city (G043). Three different identifier types, three different enrichment databases, one consistent pattern. The resolver's generic protocol — identifier in, context out — applies unchanged across all three. Four projects remain in Networking.

---

## G044 — Remote Login

**Identity verification across a boundary.**

Every networking project so far assumed trust — the client connected, and the server answered. Remote login introduces distrust. The server doesn't know who the client is until they prove it. The credential is the proof. The session token is the ongoing assertion of that proof. This is the Rosetta Stone's first trust *establishment* protocol — not checking whether data is valid (G013), but checking whether an entity is who they claim to be.

**Salted hashing is one-way trust.**

The server never stores the password. It stores a salted hash — a one-way transformation that can verify but never recover the original. G028's cipher was reversible (given the key, decrypt). The hash is irreversible. The server confirms "you know the password" without itself knowing the password. In the noosphere, agent authentication works the same way: the agent proves its identity by demonstrating knowledge without transmitting the secret.

**Sessions are time-bounded identity assertions.**

The token is G031's self-verifying artifact combined with G026's TTL. It asserts: "this client authenticated as this user, and the assertion expires at this time." Every temporal pattern in the Rosetta Stone converges here: G011's alarm (session timeout is a deferred obligation), G026's TTL (the session expires like a ticker item), G042's domain expiry (the session is a renewable lease). Time bounds trust.

**Brute-force protection is rate-limited `<-` gates.**

Lock the account after N failed attempts — G013's progressive trust applied to authentication. Each failure is a failed gate. After too many failures, the gate closes for a cooldown. The ascending cost: first attempt is free, subsequent attempts accumulate risk, final attempt triggers lockout.

The audit log is G025's immutable journal applied to security events. Every attempt recorded: who, from where, when, success or failure, reason. This is the accountability pattern from G025 deployed at the authentication boundary.

**Revoke-all is retroactive trust invalidation.**

`revoke_all(username)` invalidates every session for a user — retroactive cancellation of previously-granted trust. The sessions were valid when created. They're invalid now because the trust basis changed. This is new: the Rosetta Stone's first retroactive operation. Everything before this was append-only (journals, tickers, audit logs). Revocation goes backward, invalidating artifacts that were valid at creation time.

In the noosphere, if an agent's credentials are compromised, every choreography it's participating in must be halted and re-authenticated. The revocation propagates through all active sessions — a cascade that the resolver must handle atomically.

Forty-four projects in. Twelve into Networking. The category adds its first trust establishment protocol. The progression: the Networking category started with connecting (G033), moved through identity resolution (G041–G043), and now reaches identity *verification*. You can't just say who you are. You have to prove it. Three projects remain.

---

## G045 — Site Checker with Time Scheduling

**The convergence project.**

G045 is where five prior patterns compose into a single operational tool:

| Source | Pattern | How it appears |
|--------|---------|---------------|
| G011 Alarm Clock | Scheduled trigger | Check every N seconds |
| G026 News Ticker | Priority dashboard with TTL | Status board with freshness |
| G036 Weather | HTTP request to external endpoint | The health check itself |
| G038 Port Scanner | Three-state classification | Up/down/degraded ≈ open/closed/filtered |
| G039 Mail Checker | Poll-filter-report with change detection | Check, compare to previous, alert on change |

The site checker doesn't introduce a new concept. It demonstrates that the concepts already built compose naturally. This is the Rosetta Stone doing what G019 (Palindrome) first showed: operations layer. Reverse + equality = palindrome. Alarm + HTTP + three-state + polling + alerting = site checker. The vocabulary is rich enough to describe complex operational tools without new primitives.

**Four-state health is the agent liveness model.**

Up, down, degraded, unknown. G038 had three states. The site checker adds **degraded** — the entity responds, but not well enough. This matters for choreographies: a degraded agent might complete the task but miss the deadline or produce suboptimal output. The `where` needs to score not just completion but quality. An agent that returns a result in 30 seconds when the SLA is 5 seconds is degraded, not down.

**Consecutive failures are confidence decay.**

One failure is noise. Five consecutive failures is a pattern. The `consecutive_failures` counter is a confidence metric — how sure are we that the diagnosis is real? This prevents false alerts from transient errors while catching genuine outages. G034 (NTP) used multiple samples to reduce measurement noise. G045 uses consecutive checks to reduce diagnostic noise. Same principle: repeated observation increases confidence.

**Status change alerts are event-sourced state transitions.**

The alert fires on *transitions*, not states. "Down" isn't an alert — "up → down" is. The system notifies when something changes, not when something is. This is G021's event-sourcing pattern applied to monitoring: the check history is the event log, the current status is the derived state, the alert is the transition event.

The daily note uses the same model. Lena's `{nightly_summary}` doesn't report the vault's state — it reports what *changed*. The transition is the news. The state is the context.

Forty-five projects in. Thirteen into Networking. The convergence project proves the Rosetta Stone's vocabulary is sufficient to compose complex tools from existing primitives. Two projects remain in Networking.

---

## G046 — Small Web Server

**The other side of the boundary.**

Every networking project before G046 was a client — sending requests, consuming responses. The web server receives requests and produces responses. This completes the bilateral protocol from G033 (FTP): we built the client side then; now we build the server side.

This is architecturally significant for the noosphere. Every ghost has been a requester. The web server says: a ghost can also serve. `@kathryn/finance_positions` isn't Kathryn requesting data — it's Kathryn's service endpoint that other agents call. Every capability in the Ghost Registry is a route in Kathryn's web server. The route table IS the capability list.

**Routing is `match` applied to URLs.**

The router matches method + path to a handler. `GET /health` → `health_handler`. This is G013's prefix dispatch applied to URL paths. The route table is the resolver's namespace: each entry maps an identifier (the URL) to a handler (the function). URL routing and `@reference` resolution are the same algorithm with different syntax.

**Request-response is the universal protocol shape.**

Client sends structured request (method, path, headers, body). Server returns structured response (status, headers, body). Every interaction in the Rosetta Stone follows this shape: FTP commands, NTP queries, chat messages, weather API calls. HTTP formalizes what the resolver has been doing implicitly since G001.

**Static file serving is cached resolution.**

A static file maps a path to a fixed response — no computation, just lookup and return. This is the resolver's cache: if the value hasn't changed, return the stored result. Static files are the degenerate case where the answer never changes.

**The dpn-api-client IS this web server.**

Nathan's `dpn-api-client` at `144.126.251.126:8080` receives IPC requests, routes to handlers (read, write, refresh), returns JSON. The Rosetta Stone's web server is the simplified version of what's already running on the droplet.

---

## G047 — Web Bot

**The agent that navigates.**

The server sat still. The bot moves — it navigates, follows links, reads pages, extracts content, fills forms, and makes assertions. It's the first autonomous actor in the Networking category: an entity that decides where to go based on what it finds.

This is the ghost. Every executive ghost is a web bot: navigate the vault, read documents, extract data, fill in daily note sections, assert conditions via `where`. Lena's `{nightly_summary}` is a crawl: start at today's daily note, follow links to project pages, extract accomplishments, compose a summary. The bot is the abstract shape of agent behavior.

**Crawling is BFS over a link graph.**

Start at a URL, extract links, add to queue, visit each. Breadth-first search over the web's directed graph. The vault's `[[wiki-links]]` form the same graph. A vault crawler would follow `[[links]]` the way the bot follows `href` attributes. Same algorithm, different link syntax. The `em-org-wallpaper` on Nathan's desktop renders this graph.

**Scripted automation is a choreography.**

The `BotScript` is ordered steps with preconditions. Navigate, assert, extract, fill. Each depends on the previous. Failure halts the sequence. The `assert` is the `<-` gate. The `extract` stores variables. The script IS a choreography — a web bot script expressed in InnateScript would be indistinguishable from a regular choreography.

**Same-domain filtering is scope restriction.**

`same_domain_only: true` keeps the crawler within one domain. The agent stays within its authorized namespace. In the noosphere, a ghost assigned to The Forge shouldn't crawl The Markets unless authorized. The domain filter is the role boundary.

---

## Meta-observation: The Networking Category (G033–G047)

**Fifteen projects. One distributed architecture.**

The Networking category discovered InnateScript's distributed architecture, the way Numbers discovered the computational core and Text discovered the content layer:

| Project | What it revealed |
|---------|-----------------|
| G033 FTP | **Bilateral protocol.** Request-response across a boundary. Commands as a micro-InnateScript. |
| G034 Atomic Time | **Consensus.** Shared reference frames. Calibration via multiple samples. |
| G035 Chat | **Multi-party messaging.** Pub-sub, dynamic membership, scoped history. |
| G036 Weather | **Live external data.** Fact freshness. API keys as shared secrets. Frame translation. |
| G037 P2P Sharing | **Symmetric topology.** No center. Chunked transfer. Content addressing. |
| G038 Port Scanner | **Boundary exploration.** Three-state probing. Service discovery. Change detection. |
| G039 Mail Checker | **Polling pattern.** Inbox model. Read-state as attention. Push and pull unified. |
| G040 Packet Sniffer | **Passive observation.** Traffic analysis. `@breakdown` at network level. Anomaly detection. |
| G041 IP Lookup | **Identity-to-context.** Geographic resolution. Proximity routing. Binary search dispatch. |
| G042 Whois Search | **Ownership resolution.** Domain provenance. Temporal registration. Trust chains. |
| G043 Zip Lookup | **Hierarchical identifiers.** Namespace hierarchy. Radius search. Multiple indexes. |
| G044 Remote Login | **Trust establishment.** Salted hashing. Sessions as time-bounded trust. Brute-force protection. |
| G045 Site Checker | **Convergence.** Five patterns composed. Four-state health. Consecutive failure confidence. |
| G046 Web Server | **Server side.** Routing as dispatch. Request-response formalized. Static serving as cache. |
| G047 Web Bot | **Autonomous navigation.** Crawling as BFS. Scripts as choreographies. Domain as scope. |

The distributed architecture: bilateral protocols + consensus + messaging + external data + peer mesh + discovery + monitoring + observation + identity resolution + ownership + trust + health monitoring + serving + autonomous agents. Each project was a facet. Together they describe the noosphere's network layer — how agents discover, authenticate, communicate, monitor, serve, and navigate across boundaries.

Numbers gave us the resolver's computational core. Text gave us the content layer. Networking gives us the distributed communication layer. Together: a computational engine that operates on meaning-bearing content and communicates across network boundaries.

The Networking category is complete.

---

## G048 — Product Inventory

**The first persistent entity.**

Everything before G048 either computed a value and returned it, or processed a stream and moved on. Nothing *stayed*. A product is the first thing in the Rosetta Stone with **identity that persists across calls**. The SKU is not a parameter — it's a name. `SKU-001` on Monday is the same `SKU-001` on Friday, with different quantities, same identity.

This is the shape of every vault entity. A project has a slug, a task has an id, a conversation has a thread. They persist. Their fields mutate. The category is called `classes`, but what it's really modeling is `entities`.

**Quantity is an aggregate, not a field.**

`quantity_on_hand(sku)` is not stored. It is summed from the movement ledger. Current state is a projection over event history. This is event sourcing, and it is how the noosphere already works: `current_context` on a project is the latest aggregate over a stream of conversations; a daily note's `accomplishments` section is a projection over T.A.S.K.S. completions. The inventory makes the pattern explicit — the ledger is canonical, the field is a view.

The implication for InnateScript: `@inventory/quantity{sku}` is not a field access. It is a resolver fold over a stream. Swap the stream for `@conversations{project: X}` and the same fold computes a project's message count. The resolver's primary operation over persistent entities is `fold-over-history`, not `read-field`.

**Reorder points are the `where` threshold.**

A reorder point is a condition: when on-hand drops to or below this number, fire. The `restock_needed` list is the set of products whose `where` currently fails. This is Kathryn's `pace_check` applied to inventory — not "are we on track for month-end revenue," but "are we on track for next week's stockout." Same shape, different units.

Every `where` clause in InnateScript can be re-read as a reorder point on an aggregate. The inventory is the canonical example: the clause `quantity_on_hand > reorder_point` is what a thousand `where` blocks in the noosphere will eventually look like.

**Receive and sell are preconditioned transitions.**

`sell` fails if `on_hand < requested`. This is the first transition in the Rosetta Stone with a precondition that depends on aggregated state. The choreographic reading: `@inventory/sell{sku, qty}` is a step with a `<-` gate. The gate tests the aggregate. If it fails, the choreography halts. This is not error handling — it is structural. The transition is only legal when the precondition holds. The distinction between "illegal" and "failed" matters: an overdrawn sell is not a runtime error to recover from, it's an invalid state transition that shouldn't occur.

Forty-seven projects in, the Rosetta Stone ignored identity and persistence. G048 is where the noosphere's entity layer begins: the shape of a vault record, expressed six ways.

---

## G049 — Movie Store

**The join table is the conversation.**

G048 modeled a single entity. G049 introduces two entities connected by a third. Movies and Customers are independent — neither references the other. The rental is the join record: when a customer borrows a movie, the interaction is captured as a new entity with its own identity and lifecycle.

This is how the noosphere works. `conversations` joins user and project. `tasks` joins agent and goal. `annotations` joins ghost and article. Every time two entities interact, a third record captures the interaction — and that third record is where the temporal dimension lives. Movies don't have due dates. Customers don't have due dates. The *rental* has a due date, because obligations live on interactions, not on the things being interacted with.

**Availability is capacity minus active obligations.**

`copies_total` is stored. `copies_available` is `total − count(active_rentals)`. Same aggregate pattern as G048, now with a filter on an auxiliary entity. This formula generalizes to every availability question in the noosphere: an agent's availability = allocated tokens − active choreographies; a project's bandwidth = weekly hours − active commitments; a timeline slot's freedom = calendar capacity − scheduled events. The movie store is the minimum viable capacity model.

**Due dates are the first temporal obligation.**

A rental with `due_at` creates a pending obligation until the movie is returned. Between rented and due, healthy. After due with no return, overdue. This is the first temporal `where` in the Rosetta Stone: a condition that flips from pass to fail as time passes, even though no one touched the system. InnateScript's `until` now has a concrete model — `@rental until @returned or @due_at` — an obligation bounded by *either* fulfillment or deadline. This is the shape of every scheduled check: nightly summary (due at midnight), forex pace check (due at month-end), Rosetta Stone goal (due at milestone completion). Pass or overdue, scored automatically, no one calls the check.

**The overdue query is a passive observer.**

`store.overdue(now)` doesn't send emails, charge fees, or lock accounts. It reports the set of obligations that have crossed their deadlines. The reaction — late fees, notifications — is a separate choreography that consumes the overdue list. The data layer answers "what is true right now"; the choreography layer decides "what to do about it." `@store/overdue` is a pure query; a cron-triggered choreography reads it and scores a `where`. The store is oblivious to consequences. That separation is the correct shape of a state model — data is observational, reaction is dispatched.

The Classes category is two projects in and has already described the full entity grammar: things with identity (G048), things that interact (G049). The rest of the category will elaborate these two primitives across seventeen domains.

---

## G050 — Reservation System

**Specific resources, not fungible copies.**

G049 modeled movies as fungible — any of 2 copies of Casablanca satisfies a rental. G050 modifies the pattern: seat 12A is specifically 12A. Room 304 has a specific view. If you reserve 12A, you can't be handed 12B. This is the first model with identity at the resource level, not just at the title level. The resource table is the primary key space; fungibility becomes a special case (multiple resources of the same kind with equivalent attributes).

The noosphere has both. The conversations table: thread `1a2b3c` is specifically that thread. The tasks table: task #247 is a specific task. The temporal calendar: `[[2026-04-19]]` is specifically that day — not a fungible "some day in April." G050 describes how the vault models things that can't be substituted.

**Intervals conflict, points don't.**

A movie rental is active-or-not at a point in time. A reservation is a *span*. Two spans either overlap or they don't, under the clean definition `[a,b) overlaps [c,d) ≡ a < d ∧ c < b`. Half-open intervals matter: a reservation ending at 3:00 and another starting at 3:00 do not conflict. This is the same convention the temporal calendar uses — `[[2026-04-19]]` is `[April 19 00:00, April 20 00:00)`. Adjacent days don't overlap. Half-open semantics is how discrete time composes without ambiguity.

InnateScript choreographies that coordinate over intervals — "Sarah handles the morning, Kathryn the afternoon" — should use the same half-open semantics by default. Two agents with touching intervals don't conflict; the handoff is clean.

**Availability is a negative search.**

For G049, available = `on_hand > 0`. For G050, available-for-[start, end) = resources with *no reservation* overlapping that range. The query is structured as negation over conflicts: for each resource, check the conflict set is empty.

This generalizes. A free agent for a task = agents with no active choreography during the needed window. A free meeting slot = times with no existing event in any participant's calendar. A quiet period for deployment = time ranges with no open incident, no change freeze, no release. Every scheduling question in the noosphere is "empty conflict set" over intervals.

**Cancel is the first non-atomic retraction.**

Rent/return happened at discrete moments: rent now, return later. A reservation is different — it is a *future claim*. The interval `[start, end)` may not have arrived yet when you cancel. Canceling retracts an intention before it takes full effect; returning releases an effect that already started.

This is the shape of scheduled choreographies. A `@nightly-summary` scheduled for midnight has a future claim on Lena's attention. If it's cancelled at 22:00, it retracts. If it runs and Lena partially executes, "cancel" wouldn't make sense — you'd need a "return partial" or `rollback` semantics. InnateScript needs both: a `cancel` primitive for future choreographies and a `retract` primitive for in-flight ones. G050 surfaces the distinction that G049 didn't need.

G048 → identity. G049 → interaction. G050 → interaction across a future interval with exclusivity constraints. Classes is showing its shape as a progressive elaboration of the same entity grammar.

---

## G051 — Student Grade Book

**The join carries a value, not just a status.**

G049's rental had a status: active or returned. G050's reservation had a status: booked, cancelled, completed. G051's grade has a *number*. The relationship between a student and an assignment is not just "did it happen" — it's "how well did it happen, expressed as a measurement."

The valued join is more important than either endpoint. A student without grades is unscored. An assignment without grades is ungraded. The edge is the primary object; the nodes are just its endpoints. This is the shape of every performance record in the noosphere: the conversation carries the tone, the task completion carries the quality, the forex trade carries the P&L. The *relationship* is where the performance data lives.

**Aggregation lives on the edge.**

Because the join is valued, you can aggregate over it. Student's course average is the weighted mean of their grades in that course. Class average is the mean grade on one assignment across students. Distribution is the histogram of course averages across students. All three aggregations are folds over the grade table, grouped differently. G048 aggregated a single entity's events; G051 aggregates the *edges* between two entity sets. This is the primary analytical operation over a relational model.

**Nested weighted aggregation is the temporal compression chain.**

Course average = `Σ(weight × grade) / Σ(weight)` over assignments. GPA = `Σ(credit × course_gpa_points) / Σ(credit)` over courses. Two levels of weighted mean — the first project where aggregation composes. And it generalizes cleanly:

- Daily pace = weighted average of today's task scores.
- Weekly pace = credit-weighted average of the seven daily paces.
- Monthly pace = day-weighted average of the four weekly paces.
- Quarterly, yearly — same structure, more levels.

G006 (mortgage) introduced the idea of amortized obligation; G051 gives the precise arithmetic. The temporal compression chain is a GPA calculator that runs forever. Every `pace_check` in the noosphere is a course average; every wider-horizon summary is a nested weighted mean over those pace checks.

**Missing is distinct from zero.**

A student who hasn't submitted assignment #3 doesn't have a zero. They have *unknown*. Two policies both matter: `current_average` folds only graded assignments (how are they doing on the work they've done?); `projected_average` treats missing as zero (if nothing else gets submitted, where do they land?). Both queries are valid — they answer different questions. The grade book supports both because absence itself is informational.

This applies throughout the noosphere. If Kathryn made no forex trades today, is today missing data or a zero-P&L day? Depends on expectations. The `missing_as_zero` parameter is not an implementation detail — it is the shape of a question about expectation. When you ask a dashboard for an average, the answer depends on what you meant by "no entry."

G048 identity → G049 interaction (status-valued) → G050 scheduled interaction → G051 measured interaction. Classes is elaborating from "things exist" to "things interact with measurable quality" — each project adds a layer to the noosphere's performance model.

---

## G052 — Bank Account Manager

**The first multi-sided event.**

Every prior project had single-sided events. Inventory movements touched one product; grades touched one student-assignment pair. G052 introduces the double-entry transaction: one operation, *multiple* entries, all landing atomically. A transfer is two entries — `(A, -100)` and `(B, +100)` — that MUST be written together or not at all. Deposits and withdrawals are the degenerate single-entry case where money crosses the bank's boundary.

This is the shape of every coordinated state change. A git commit is a multi-entry transaction: several files modified, all or none. A choreography that updates three vault records atomically is a multi-entry transaction. A transaction in PostgreSQL is literally this. G052 is the Rosetta Stone naming the pattern the noosphere has been using implicitly.

**Conservation as a global invariant.**

Transfer entries sum to zero. Money is conserved *within* the bank; deposits and withdrawals break conservation locally because they cross the system boundary, but internally every redistribution is zero-sum:

```
∀ tx where tx.kind = transfer: Σ(entries.amount) = 0
```

This is a ledger-wide invariant, checkable at any time (`ledger_is_balanced` / `ledgerBalanced` / `LedgerIsBalanced` in every implementation). If it ever fails, the ledger is corrupted — a reconstruction bug, a race, a missing entry. The invariant is a detection mechanism, not just a rule.

Generalizes throughout the noosphere: whenever state redistributes within a closed system, the redistribution must sum to zero. Reassigning work between agents conserves total work. Moving a task from backlog to active conserves total tasks. The Bank is canonical because money makes conservation explicit, but the pattern is everywhere a state transition touches more than one entity.

**Balance is still an aggregate — just with multi-entry transactions.**

Same event-sourcing pattern as G048/G049/G050/G051: the transaction log is canonical; balance is the projection onto one account. What makes G052 stronger: a single transaction contributes to *multiple* accounts simultaneously. A transfer adds a `-100` entry to A's projection and a `+100` entry to B's projection. The same log powers both queries. This is how database views work — one source of truth, many derived tables.

**Closure has a numeric precondition; frozen is a meta-policy.**

G052 introduces two kinds of transition gate:

- **Numeric precondition**: `close(account)` requires `balance == 0`. An equality, not an inequality — and aggregates are the predicate subject. "Zero balance" is the general name for "no unresolved obligations." It shows up everywhere: can't archive a project with open tasks, can't close a conversation with unresolved references.

- **Meta-policy gate**: a frozen account refuses new transactions regardless of amount or balance. `status == open` is a *policy condition*, not a *data condition*. Both can fail a `where`; they fail for different reasons and warrant different reactions. Read-only modes, maintenance windows, compliance holds — all meta-policy gates on otherwise-valid data.

InnateScript needs the distinction in its `where` vocabulary: `where balance > 0` is data; `where status == open` is policy. Rejecting a transaction because you can't afford it is different from rejecting it because the account is suspended. The noosphere will encounter both constantly.

G048 identity → G049 status-valued interaction → G050 scheduled interaction → G051 measured interaction → G052 *atomic multi-entity state change with conservation*. Five projects in, the Classes category has laid down the full vocabulary the noosphere uses to model coordinated state.

---

## G053 — Library Catalog

**Three layers of identity.**

G048 had one entity type. G049 and G050 had two. G053 has three — title, copy, loan. G049's movie copies were fungible; no one cared which of two Casablanca copies you got. G053's copies have identity because they can be lost, damaged, moved, or withdrawn *individually*. A title with three copies, one lost, two available, is a different catalog record from a title with three available copies.

This three-layer pattern appears throughout the noosphere. A *project* (abstract work), its *phases* (specific instances with their own state), and the *tasks* within each phase (temporal claims on agent attention). A *conversation thread* (abstract channel), its *messages* (specific posts with their own state), and *reactions* (temporal claims on messages). Every rich vault entity is a Title/Copy/Loan triple wearing different labels.

**Subjects are a tree, queried by prefix.**

Dewey Decimal isn't a flat category list — it's a *prefix-closed namespace*. `"500"` is natural sciences; `"512"` is algebra; `"512.5"` is a subfield. A title at `"500.512.5"` sits three levels deep. "Everything in 500" is a prefix query, not an equality lookup.

Same algebra as vault wiki-links. `[[The Forge]]` is a namespace; `[[The Forge/Temporal]]` is a subspace; `[[The Forge/Temporal/Daily Notes]]` is a leaf. The `.` separator in a subject path and the `/` separator in a vault path are informationally identical: both are tree delimiters. Prefix queries over either are the same operation. The temporal calendar itself is a tree (year/quarter/month/week/day) — every temporal query is Dewey with different punctuation.

**Holds are a FIFO queue with a wakeup trigger — and the wakeup is atomic with the trigger event.**

When all copies are out, patrons queue. First-come-first-served. When a copy is returned, the front of the queue wakes up: hold becomes `fulfilled`, attached to the returned copy. This is the first *passive waiting primitive with automatic fulfillment* in the Rosetta Stone. G050's reservations were explicit — someone called `book`. G053's holds sit until state elsewhere (a return) fires the wakeup.

Critically, `return_copy` marks the copy available AND fulfills the next hold in a single atomic step. If those weren't coupled, a race would open a gap where the copy is available but no one has been notified. G052 taught atomic multi-entity updates; G053 uses the pattern to keep queue state consistent with shelf state. Every task queue, email inbox, interrupt handler in the noosphere needs the same coupling.

**Renewal is in-flight mutation of a deadline, and the coordination precondition is the queue.**

G050 had two obligation transitions: cancel (retract future claim) and complete (interval ends). G053 adds a third — renew — where the claim continues but the deadline moves forward. Not rescheduling (both ends move), not cancellation (claim ends). Mid-flight extension.

And the precondition is structural: renewal fails if other patrons have waiting holds on the title. The check isn't on the loan's own invariants — it's on the *state of a separate entity set* (the hold queue). "You can't extend your claim at the expense of a queue behind you" is the general shape of yield-to-waiters, and it prevents the familiar anti-pattern where the current holder keeps renewing indefinitely while newcomers wait.

In the noosphere: a choreography running past its scheduled end can renew if nothing downstream is waiting. If a downstream choreography has a hold on the same agent or resource, renewal blocks. G050 introduced scheduled obligations; G053 introduces the first policy that makes scheduled obligations yield to queues behind them.

G048 identity → G049 interaction (one-off, status-valued, fungible copies) → G050 scheduled interaction (specific resources, intervals) → G051 measured interaction (valued join, aggregation) → G052 atomic multi-entity change (conservation) → G053 *three-layer identity with tree-structured taxonomy, waiting queues, and in-flight deadline mutation gated by queue state*. Six projects, and the Classes category has enumerated most of the primitives the noosphere will ever need.

---

## G054 — Patient / Doctor Scheduler

**Dual-resource availability is a conjunction, not a sum.**

G050's reservations checked one calendar per booking. G054 requires two at once: an appointment occupies the *doctor's* calendar AND the *room's* calendar simultaneously. Availability is the conjunction:

```
schedule(p, d, r, [s, e))  ⇔  doctor_free(d, [s, e))  ∧  room_free(r, [s, e))
```

The two conflict queries are independent (different tables, no shared state) but the commit is joint (both must pass before either side is marked busy). This is a small atomic transaction in the G052 sense — reads against two resources, then a single write if both reads approve.

Generalizes instantly throughout the noosphere: a build needs both a runner and a cache lock; a meeting needs every attendee's calendar AND a conference room; a deploy needs the CI pipeline AND the production window. Most real-world choreographies involve multiple resources in alignment, not just one. G054 is the Rosetta Stone's first conjunctive availability constraint; expect variations at every turn.

**Reschedule requires self-exclusion.**

Moving an existing appointment is not "cancel + rebook" — that pattern loses the slot between cancel and rebook, racing other bookings for it. It is a single mutation: the appointment's start and end move in place. But the conflict check must exclude the appointment's *own current state*, or it conflicts with itself and no move is ever legal.

```
reschedule(x, [s', e'))  ⇔  doctor_free(x.doctor, [s', e'), except x)
                          ∧  room_free(x.room,    [s', e'), except x)
```

The `except x` clause is new. Previous conflict checks in G050 and G053 were global ("anything overlapping"). Reschedule introduces the *exclusion predicate* on the entity being updated. This generalizes to every in-place update of a temporal obligation: a deploy window being shifted excludes itself, a shift being moved excludes itself, an event being rescheduled excludes itself. "Ignore yourself when checking if you'd collide with someone" is the shape of move-in-place in any system with temporal uniqueness.

**`find_slot` turns availability into a search problem.**

G050's `available(start, end)` asked: given this window, what's free? G054's `find_slot(doctor, duration, within)` asks: within this window, where is *something* free? The second is a search, not a check. The scheduler walks time forward at some step granularity, evaluating the dual-resource constraint at each step, and returns the first point where both calendars agree.

This is the first project where the answer is **computed by scanning** rather than retrieved by lookup. Every "when can we meet" dialog, every agent scheduler looking for a free window is running this algorithm. The step granularity is an explicit policy parameter — a 15-minute step won't find 10-minute slots starting on the 7's; a 5-second step is wasteful for hour-long appointments. The scheduler exposes the knob rather than hardcoding it; this is the Rosetta Stone's first explicit cost/precision tradeoff on a resolver native.

**find_slot is advisory; schedule is authoritative.**

Between `find_slot` returning a candidate and `schedule` committing it, other choreographies might book that slot. `find_slot` cannot hold a lock — it would stall every other scheduler. The right pattern is: `find_slot` hands you a candidate, `schedule` tries to commit, and the `where` catches the race. This is optimistic concurrency expressed as two separate resolver calls, and it generalizes to every coordination layer that exposes both "look" and "commit" primitives. The look is cheap and hint-only; the commit is authoritative and atomic.

**One record, three projections.**

An appointment is one record. But it appears in three schedules: doctor's, room's, patient's. One source, three views — the database-view pattern from G052 elevated from "two accounts per transaction" to "three indexes per appointment." Every vault record that participates in multiple indexes has this shape. A task lives in the Tasks index, the Project index, and the Agent-assigned index simultaneously; a conversation in the Thread index, Participants index, and Topic index. G054 is the first project where multi-index membership is explicit and ergonomic — a primary list with filtered projections.

G048 identity → G049 status-valued interaction → G050 scheduled interaction → G051 measured interaction → G052 atomic multi-entity change → G053 three-layer identity + queue + in-flight renewal → G054 *conjunctive availability + self-exclusion on move + search-based slot finding*. Seven projects, and the Classes category has gathered the full palette the noosphere's coordination layer will be built out of.

---

## G055 — Recipe Creator and Manager

**The entity is its inner structure.**

Every prior entity in the Rosetta Stone had scalar fields. Products had prices, accounts had balances, appointments had start/end. G055's recipe has a **list of line items**, each itself a small structure (ingredient + quantity + unit). The recipe's identity is its name; its *substance* is the ordered multiset inside.

Operations on scalar-field entities are trivially typed — add, set. Operations on structure-field entities are *maps and folds over the inner collection*. Scale: map every line. Can_make: fold lines against the pantry. Shopping list: filter-map lines. The operation is not a single write — it is a traversal.

This is how every rich vault entity works. A project has a list of goals. An agent has a capability set. A conversation has a message stream. The `Projects`, `Agents`, `Conversations` indexes each hold compositional entities. G055 is the first Rosetta Stone project to formally model one.

**Scaling is a pure linear map — and exact arithmetic matters.**

`scale(recipe, factor)` multiplies every line's quantity by factor and returns a new recipe with the same shape. No mutation, linear over the structure. In categorical terms, scaling is a morphism in the category of recipes-at-different-quantities: all scales share line identities; only quantities change.

Exact arithmetic is load-bearing here. Scaling 100g by 1/3 should give 100/3 g, not 33.333333. The implementations use rationals everywhere quantities live: Python's `Fraction`, a Rust `Qty { num, den }`, Common Lisp's native rationals, a Lean `Qty` struct, a Go `Qty` type. Floats would be wrong — the same reason G052 used integer cents. The demo's output includes `275/2 g` and `3/2 tsp` after making half a recipe, proving the rationals survive the full pipeline.

The noosphere will have many such morphisms over compositional entities: "compress this project's timeline by 30%", "halve the agent's workload", "double this conversation's priority weights." G055 introduces the pattern cleanly; the rest of Classes will use it.

**`can_make` is an N-ary conjunction over a dynamic width.**

G054 introduced binary conjunction (doctor AND room). G055 generalizes to **N-ary**: can_make is a conjunction over *every line*, and the width is dynamic — some recipes have 3 ingredients, some have 30. The conjunction adapts.

```
can_make(r)  ⇔  ∀ line ∈ r.lines: pantry_covers(line)
```

This is the shape of every "requirements met" check in the noosphere. All dependencies installed? Conjunction over a dynamic dependency list. All tests pass? Conjunction over a test list. All agents ready? Conjunction over the roster. Width-dynamic conjunction is how reality works; G055 is the first project where it appears cleanly.

When the conjunction fails, the report is **structured** — which lines failed, and why. Three reason codes: `not_stocked`, `unit_mismatch`, `insufficient`. The failure mode isn't a single boolean or a single error string; it's a list of per-line diagnoses. Good `where` clauses in InnateScript should always return this shape when they fail. "where failed" with no detail is almost useless; "where failed because A (reason x), B (reason y), C (reason z)" is actionable.

**Shopping list is a computed delta — not a stored record.**

```
shopping_list = required − available    (per line, clipped at zero)
```

The list doesn't exist as a record anywhere. It is derived on demand from the gap between desire and reality. First project where the main output of a query is a *gap report*.

Gap-based reporting is its own primitive. Every noosphere planning operation produces such a report: a project's roadmap gap is "required phases minus completed phases"; a context gap is "what the choreography needs minus what it has access to"; a capability gap is "what the task demands minus what the agent has." These are computed deltas over structured fields. Recipes make the pattern concrete with the clearest possible example.

**`make` is atomic multi-entity consumption — conservation's other face.**

G052 introduced atomic multi-entity updates via conservation (transfer entries sum to zero). G055 uses the same atomicity for a different semantic: **consumption** rather than transfer. `make(recipe)` subtracts from N pantry items; either all subtractions land or none do. The distinction matters:

- G052's transfer preserved total (conservation): what left one account arrived at another.
- G055's make does NOT preserve — ingredients are destroyed. The pantry's total diminishes.

Two faces of multi-entity atomic updates: conservation (money, labels) and consumption (materials, attention, compute). Both need atomicity; they differ in whether the total is invariant. InnateScript's choreography engine will need the distinction when scoring `where` clauses — "did the transfer balance?" is a conservation check; "did we have enough?" is a consumption check.

G048 identity → G049 status interaction → G050 scheduled → G051 measured → G052 atomic (conservation) → G053 three-layer + queue + renewal → G054 conjunctive + self-exclusion + search → G055 *structure-valued entity + exact scaling + N-ary conjunction + gap reporting + atomic consumption*. Eight projects, and Classes has now shown how the noosphere's compositional entities — projects, agents, conversations — get their scale, their requirements checks, their shopping lists, and their atomic consumption semantics.

---

## G056 — Image Gallery

**The entity as a metadata envelope around external state.**

Every prior entity was self-contained: the product had a price, the recipe had lines, those values *were* the data. G056 introduces **external references**: the image's bytes live at a path, a URL, an S3 key. The `Image` record is a description — filename, dimensions, mime type, a content hash, and a `BlobRef` pointer.

The database doesn't own the bytes. It doesn't know if the file still exists at that path, if it's been modified, if the bytes match the hash. The entity is a pointer, not the content. `BlobRef.exists` is a last-known-availability flag, not a live property. This is the first stale-reference primitive in the repo.

This is how every vault references the outside world. A daily note references a screenshot at some filesystem path. A conversation references an article by URL. A project references a codebase directory. An annotation references a Figma design. Whenever authoritative state lives elsewhere, the entity becomes a metadata envelope, and stale-reference semantics become unavoidable.

**Derived artifacts are cache entries, not canonical state.**

A thumbnail is a function of the source image. A preview is a function of the source image. If the source changes, they become stale and need regeneration. The source is canonical; the derivatives are cache views. G055 had pure-function derivation (`scale(recipe, 2)` returned a value); G056 has **materialized** derivation (the thumbnail persists as a separate artifact). This introduces the first implicit cache in the Rosetta Stone and therefore the first implicit staleness question.

In the noosphere: every "rendered" artifact is a cache of its source. Rendered project summaries cache YAML frontmatter. Compiled choreography definitions cache their source. Export PDFs cache their notes. G056 doesn't solve cache invalidation — it names the pattern: source identity (content hash) is the invalidation key.

**Identity by content, not by assignment.**

A `content_hash` field turns the image into a content-addressed entity. Two uploads with the same bytes have the same hash and, for dedup purposes, are the same content. This is a different identity model from everything before:

- G048–G055: identity by assignment — `"SKU-001"` is whatever was first assigned that SKU. IDs are external and arbitrary.
- G056: identity by content — the hash is computed from the bytes. Same bytes, same id.

Git blobs work this way. The vault's conversations could dedup quote-replies by content hash. Any time you want "is this the same stuff?" without comparing all the bytes, you want content addressing. `duplicates()` is the query this enables: group by hash, return hashes with more than one image. The demo found `IMG-001` and `IMG-002` share hash `a1b2c3` — different IDs, same content, flagged automatically.

**Tag queries are set algebra.**

G053's library subjects formed a tree; queries were prefix matches. G056's tags form a flat set; queries are set operations:

```
with_all_tags({T, B})         = T ∩ B       (intersection)
with_any_tag({T, B})          = T ∪ B       (union)
without_tags({T}, {P})        = T \ P       (difference)
```

Tree queries navigate hierarchy; set queries combine orthogonal axes. Both are needed, and both already exist in the vault. A conversation at `[[The Forge/Temporal/Daily]]` lives in a tree AND has tags `{daily, music-related, urgent}` in a flat set. The two query shapes compose: "all urgent daily notes under The Forge" is a subject-prefix filter followed by an all-tags filter.

InnateScript needs both primitives: `@scope{path: ...}` for tree navigation, `@filter{tags: ...}` for set algebra. Together they form a richer query language than either alone. The demo shows all three set operations working correctly: `travel AND beach → [IMG-001]`, `travel OR forest → [IMG-001, IMG-003, IMG-004]`, `travel BUT NOT private → [IMG-001, IMG-003]`.

**Tags are intrinsic; albums are curated. This distinction is structural.**

An image *is* tagged "sunset" because its content depicts one — the tag is a property of the image itself, auto-inferrable. An image *belongs to* "Summer 2026 Trip" because a human grouped it there — the album membership is not inherent, it is an act of curation.

| | Tags (intrinsic) | Albums (curated) |
|---|---|---|
| Origin | Property of the content | Decision by a curator |
| Auto-inferrable? | Often yes | No — requires intent |
| Ordering | Set (unordered) | Sequence (curator order) |
| Removal | Statement about content | Statement about the collection |

Every tag-able entity in the noosphere has this distinction. A conversation's topic tags (intrinsic) vs its appearance in the "Urgent Follow-Ups" queue (curated). A project's domain tags (intrinsic) vs its inclusion in Q3 planning (curated). Muddling the two is a category error: you can't auto-infer curated membership, and you shouldn't require human intent for intrinsic classification. G056 makes the distinction structural — tags live on the image record; albums are their own entity with explicit curator and order. The demo's "Summer Trip" album preserved curator order `[IMG-003, IMG-001]` rather than falling back to alphabetical — the order is part of the curation.

G048 identity → G049 status interaction → G050 scheduled → G051 measured → G052 atomic (conservation) → G053 three-layer + queue + renewal → G054 conjunctive + self-exclusion + search → G055 structure-valued + exact scaling + N-ary conjunction + gap + atomic consumption → G056 *external refs + content-addressed identity + cache views + set-algebra tags + intrinsic-vs-curated distinction*. Nine projects; Classes now models every flavor of entity the noosphere will need to represent reality.

---

## G057 — Handle Large Numbers

**The class IS a value, not a container.**

Every prior Classes project modeled a *thing in the world*: a product, a reservation, a grade, a recipe, an image. The entity was a container with domain-specific fields. G057 is qualitatively different — a BigInt doesn't contain anything external. The BigInt *is* its own value. `parse("12345")` is the number twelve thousand three hundred forty-five, not a record about it.

This is the distinction between:

- **Domain entities** — Product, Appointment, Recipe. Shaped by the world they model. Fields are domain vocabulary.
- **Value classes** — BigInt, Date, Vec2, Color. Shaped by algebra. Fields are representation, not meaning.

Every language has both. The noosphere will have both: a project is a domain entity; a pace score is a value class. A conversation is a domain entity; a token count is a value class. G057 is the Rosetta Stone's first value class and establishes the pattern future numeric/symbolic types will follow.

**Operations are laws, not just methods.**

A BigInt's `add` is not "the function that adds." It must satisfy commutativity:

```
∀ x, y : add(x, y) == add(y, x)
```

associativity:

```
∀ x, y, z : add(add(x, y), z) == add(x, add(y, z))
```

and `mul` must distribute over `add`:

```
∀ x, y, z : mul(x, add(y, z)) == add(mul(x, y), mul(x, z))
```

These are not suggestions. They are the *specification* of what makes the class a correct number system. An implementation with a working `add` method but failing commutativity would be wrong regardless of whether individual cases passed.

Property-based tests are the right shape. `addition_is_commutative` samples two numbers and checks `a + b == b + a`; one such test with a good generator replaces hundreds of example-based tests. Every value class in the noosphere going forward should ship with property tests alongside example tests. This is the first Classes project where the distinction between example and property testing becomes load-bearing.

**Canonical form is a class invariant.**

Multiple representations can encode the same value:

```
{negative: false, digits: []}        == 0  (canonical)
{negative: false, digits: [0]}       == 0  (leading zero)
{negative: true,  digits: []}        == 0  ("negative zero")
{negative: false, digits: [5, 0, 0]} == 5  (with trailing zeros in storage)
```

Only one is canonical. All constructors — `from_int`, `parse`, `add`, `sub`, `mul` — MUST produce canonical form. Once canonical form is enforced, structural equality is value equality: two BigInts represent the same number iff their stored representations are identical. Without canonical form, equality requires normalization on every comparison, and the type becomes a bug farm.

Every noosphere value class will have invariants of this shape. A Date must normalize February 30 to March 2. A Duration must not overflow. A Color must clamp channels to [0, 1]. Canonical form is how a value class enforces that it actually represents what it claims.

**Schoolbook algorithms — the first non-trivial algorithm in Classes.**

G048–G056 had essentially no algorithms, only data-shape exercises (sum, filter, group-by, map). G057 is the first Classes project where correctness requires implementing a real algorithm:

- **Addition**: digit + digit + carry, loop over the longer operand.
- **Subtraction**: digit − digit − borrow, assuming the minuend is larger.
- **Multiplication**: each digit of one operand times every digit of the other, shifted and summed.

These are the algorithms you learned in elementary school. Writing them clearly in six languages is the point. The Rust `999 * 999 = 998001` runs by literal carry propagation through three digits; the carry chain is visible in the loop. `30! = 265252859812191058636308480000000` requires 30 multiplications, each propagating carries through up to 33 digits.

Performance isn't the point — base-10 limbs are pedagogically clear because every intermediate value matches what a human would write on paper. In production, BigInt uses base 2^32/2^64 limbs and Karatsuba or FFT multiplication. The Rosetta Stone prefers clarity over speed by explicit choice.

**Signed arithmetic is two cases.**

Same-sign addition is magnitude addition. Different-sign addition is magnitude subtraction with the sign of the larger-magnitude operand. Subtraction is `x - y == x + (-y)`. Multiplication XORs the signs and multiplies magnitudes. The sign logic is tiny; the magnitude arithmetic is the real work.

This "same-sign add / different-sign sub with sign-of-larger" is the template for every signed numeric type. G052's bank account already did this implicitly in its transfer logic. G057 makes it explicit and primitive. Every future signed value class in the noosphere (debit/credit, gain/loss, inflow/outflow) uses this case analysis.

**Why it matters: the noosphere needs unboundedness.**

Token counts across a year can exceed 64-bit range. Cumulative P&L across trades can exceed native integer limits. Cryptographic hashes live far outside any machine integer. Any aggregation at scale requires a type that *cannot* silently overflow. BigInt's entire point — the single property that earns it its place — is that addition can always succeed. When the domain assumes unboundedness but the language doesn't provide it, you need a value class to enforce the invariant.

G048 identity → G049 status interaction → G050 scheduled → G051 measured → G052 atomic (conservation) → G053 three-layer + queue + renewal → G054 conjunctive + self-exclusion + search → G055 structured field + scaling + gap + atomic consumption → G056 external refs + content-addressed + set-algebra + intrinsic-vs-curated → G057 *value class + laws + canonical form + schoolbook algorithm + signed case-analysis*. Ten projects. Classes now contains both the noosphere's domain entities (G048–G056) and its first value class (G057); every entity in the vault or the resolver will be one of these two shapes.

---

## G058 — Chart Making Class / API

**The chart is a function, not a container.**

Traditional OO charts have a `Chart` class with `add_series`, `set_color`, `draw` — state accumulates, mutation everywhere. G058 takes the grammar-of-graphics approach: the chart is a spec + data; rendering is a pure function `render(spec, data) → output`. No hidden state. Same spec + data always produces the same output. This is how Vega, Vega-Lite, D3, and ggplot2 work.

Every prior Classes project had a container with state. G058 inverts the pattern: the spec is static, data flows through, output is computed. The class is a **pipeline definition**, not a state machine. Composability follows naturally — combine two specs into a layered chart; parameterize a spec over datasets for small multiples; swap one renderer for another to change output format.

**Scales are functions, made explicit.**

The core primitive of data visualization is the scale — a function from data domain to visual range. `LinearScale(0, 100, 0, 40).at(50) = 20`. The class exists to name the function and carry its parameters. Three scale variants cover most charts: linear (continuous → continuous), ordinal (categories → positions), log (orders of magnitude).

Every visualization reduces to "which scales, on which fields?" This is the first project in the Rosetta Stone where the domain model is explicitly **higher-order**: the spec contains functions, not just values. The noosphere will use scales everywhere — project progress 0–100% onto a visual bar; a pace_check score [-1, +1] onto a color channel. Every dashboard widget is a chart spec in miniature.

**Each stage is independently testable.**

The pipeline decomposes into five pure functions: data, scale, encode, layout, render. Every prior Classes project had a monolithic "do the thing" method — schedule the appointment, record the grade, check out the book. G058 is the first project where the operation decomposes into stages you can test in isolation. If the chart looks wrong, you can pinpoint the bad stage: bad data, bad scale, bad encoding, bad layout, or bad renderer.

This generalizes to every pipeline in the noosphere. A choreography has stages (parse, validate, project, execute, report). A build has stages (fetch, compile, test, package, deploy). Each stage is independently testable if it's a pure function. G058 is the Rosetta Stone's first *composed-pipeline* class, and the pattern scales up without modification.

**Declarative spec vs imperative drawing.**

The chart spec doesn't say "draw a rectangle at (100, 200) with width 50, height 80." It says "bar chart with `x = category`, `y = value`". The renderer figures out the rectangles.

This is the first separation in the Rosetta Stone between **what to show** and **how to draw it**. The noosphere needs this separation everywhere. A daily-note template says "list today's completed tasks" (declarative); the renderer decides checkbox style. A project summary says "show blockers, goals, current context" (declarative); the renderer decides card-vs-table. Declarative specs are interchangeable — swap the renderer, same spec, different output.

**The renderer is a pluggable backend.**

We render to ASCII here because every language has strings and terminals. The same spec could render to SVG, canvas, PDF, TikZ, or a truecolor terminal. Only the final stage changes. **Backend-as-strategy** is the pattern, and it's the only reason grammar-of-graphics libraries support ten output formats without rewriting the user-facing API.

The demo proves the separation: the same temperature dataset renders as a bar chart AND as a line chart, with the same `DataPoint` type feeding both. Backend is pluggable; spec is declarative; pipeline is pure. In the vault, one dashboard spec could drive terminal widgets, emailed weekly reports, and web UI without any change to the spec layer.

**The line chart reveals the Rosetta Stone's first algorithmic *drawing*.**

Unlike the bar chart (draw a bar per row), the line chart interpolates. Between two plotted points, empty cells along the diagonal get filled with dot-marks (`·`) via a step-along-the-longer-axis walk. This is a tiny Bresenham-ish algorithm — the first time a Classes project renders **continuous visuals from discrete marks**. It's still schoolbook-clear; but it's no longer just iteration over rows. The grid-of-chars + step-along-diagonal technique is what every terminal UI library uses underneath.

G048 identity → G049 status interaction → G050 scheduled → G051 measured → G052 atomic → G053 three-layer+queue+renewal → G054 conjunctive+search → G055 structured+scaling+gap → G056 external-refs+set-algebra → G057 value class + laws → G058 *pipeline class + scale-as-function + declarative spec + pluggable backend + grid rendering*. Eleven projects. Classes has now modeled: static entities (G048), two-way interactions (G049), intervals (G050), measurements (G051), atomic multi-entity (G052), three-layer taxonomies (G053), dual-calendar scheduling (G054), structure-valued containers (G055), external references (G056), value types (G057), and composed pipelines (G058). One-third of the category left.

---

## G059 — Shape Area and Perimeter

**One interface, many implementations — and six different dispatch mechanisms.**

The simplest possible shape-polymorphism problem exists because it is the cleanest demonstration of the big OO idea: **different types of things obey the same protocol**. Circle, Rectangle, Triangle, Regular Polygon share no data; they share a behavioral contract — each answers `area()` and `perimeter()` using its own geometry.

The six languages express the same contract with fundamentally different machinery. Studying them side by side is its own lesson in what polymorphism means:

| Language | Mechanism | Open? |
|---|---|---|
| Python | ABC + duck typing | Open |
| Rust | Traits, `Box<dyn Shape>` | Open |
| Go | Structural interfaces (implicit satisfaction) | Open, structural |
| Common Lisp | Generic functions (defgeneric/defmethod) | Open, multimethod |
| Lean | Inductive sum type, exhaustive pattern match | **Closed** |
| InnateScript | Resolver dispatch on kind tag | Configurable |

Five are open — adding a new shape means writing a new module, nowhere else. Lean's inductive-type approach is closed — the compiler enforces exhaustivity, refusing to compile if you add a case and forget a function's pattern match. Both are correct; they optimize for different things.

**Open polymorphism enables extension without modification.**

In Rust, Go, Python, and Common Lisp, adding a Pentagon is one new file — implement the protocol, done. The `Shape` interface is not modified. The `total_area` function still works because it depends only on the protocol, not on the enumeration of implementers.

Open polymorphism is the norm in most runtime-dispatched systems. The noosphere needs it: new agent types, new document formats, new choreography kinds should be addable as new modules without touching protocol definitions. This is why the vault uses string-tagged kinds (`kind: agent`, `kind: project`, `kind: goal`) — anyone can define a new kind and its resolvers.

**Closed polymorphism enables exhaustivity checking.**

Lean's inductive `Shape` says: there are EXACTLY four shapes. Add a fifth and every pattern-matching function fails to compile until it handles the new case. The compiler has enumerated the cases and confirms none are missing.

This is a different kind of power. Open polymorphism lets you add without touching the core. Closed polymorphism refuses to compile incomplete code. Every choreography in InnateScript faces the same choice: when the set of cases is stable, closed polymorphism catches forgotten cases at compile time; when the set is evolving, open polymorphism lets ecosystems grow without coordination.

For real software: stable taxonomies (`payment_method: credit | debit | cash`) benefit from closed polymorphism; evolving ecosystems (plugins, formats, agent kinds) need open polymorphism. The noosphere will use both — and the tension is productive.

**Heterogeneous collections are the payoff.**

All six languages can hold a list of mixed shapes and compute the total area:

```
total_area([Circle(5), Rectangle(4, 6), Triangle(3, 4, 5)])
```

Python's version is cleanest because Python boxes everything by default. Rust's is noisiest because Rust forces explicit `Box<dyn Shape>`. Go lands between them — no syntax for boxing, but a pointer indirection under the hood. The *semantic* operation is identical across all six: fold over a mixed list, calling `.area()` on each. If you can write `shapes.map(&.area).sum`, you have polymorphism — whether the dispatch is a vtable lookup, a class-method lookup, a generic-function lookup, or a pattern match is implementation detail.

**Smart constructors enforce validity at the type boundary.**

Triangle has a non-trivial precondition: the triangle inequality. If violated, no valid triangle exists. Every implementation handles this at construction:

- Python: `__post_init__` raises `ValueError`.
- Rust: `Triangle::new` returns `Result<Triangle, String>`.
- Go: `NewTriangle` returns `(Triangle, error)`.
- Common Lisp: `make-triangle` signals via `error`.
- Lean: `Shape.mkTriangle` returns `Except String Shape`.

The pattern is **smart constructor**: the type allows only valid values because the constructor refuses to build invalid ones. Once you hold a `Triangle`, its sides are valid by construction — `area()` doesn't re-check. This generalizes: every domain-constrained value class uses smart constructors. A `Date` refuses February 30; a `Percentage` refuses -5. The smart-constructor pattern, combined with the value-class pattern from G057, is how the noosphere will enforce domain invariants without runtime checks throughout the codebase.

**Collection operations belong to the interface, not to any implementer.**

`total_area`, `largest_by_area`, and `sort_by_perimeter` are defined on `list[Shape]`, never on any concrete shape. They are the first Rosetta Stone functions that operate *purely through the interface* — the implementations are invisible to them. This is the signature of well-designed polymorphic code: the caller can't (and shouldn't) know which concrete types are present.

G048 identity → G049 status → G050 scheduled → G051 measured → G052 atomic → G053 three-layer+queue+renewal → G054 conjunctive+search → G055 structured+gap → G056 external-refs+set-algebra → G057 value class → G058 pipeline class → G059 *interface-dispatch + open-vs-closed polymorphism + smart constructors + interface-only collection ops*. Twelve projects; Classes has now delivered every shape of entity (domain entities, value classes, pipelines) and every shape of dispatch (enum match, trait/interface, generic function, resolver) the noosphere will ever need.

---

## G060 — Matrix Class

**Shape is part of the value.**

A Matrix is not just "a grid of floats." It is a **shape + contents**. A 2×3 matrix and a 3×2 matrix have the same six entries but are different values: one represents a linear map from ℝ³ to ℝ², the other from ℝ² to ℝ³. You cannot substitute one for the other.

G057's BigInt had invariants over its representation (canonical form). G060's Matrix has invariants over a *structural property* — `(rows, cols)` — that gates which operations are legal. Two matrices of incompatible shape can't be added. Runtime check in Rust/Go/Python/CL; type error in dependent-typed Lean if taken further. Either way, the *kind of illegality* is structural — the values could be anything and the operation would still be illegal. Shape is first-class.

**Multiplication is non-commutative.**

Every arithmetic operation so far has been commutative: integer add, scalar multiply, set union. G060 introduces an operation where `A × B ≠ B × A` — not just different values, but often different shapes. With A=2×3 and B=3×2: A×B is 2×2, B×A is 3×3. They are not merely unequal; they are objects of different types.

This is qualitatively new. A large fraction of programmers have internalized commutativity as "obvious" because scalar arithmetic taught it. G060 re-teaches: in the real world, order matters for most operations. Function composition is non-commutative. Choreography sequencing is non-commutative. Writing a log entry before acquiring a lock is not the same as the reverse. G060 is the Rosetta Stone's first explicit non-commutative algebra; every pipeline that sequences operations inherits the lesson.

Associativity saves the day. `(AB)C == A(BC)` even though neither equals `BAC`. Associativity is what lets chains of matrices be written without parentheses — the same property that makes function composition chainable. When designing the resolver's `then` combinator in InnateScript, associativity is the right law to preserve, not commutativity.

**Two-sided identity, parameterized by shape — a taste of dependent types.**

Every commutative operation has a unique identity: 0 for addition, 1 for multiplication, ∅ for union. G060 has **two-sided identities parametrized by shape**:

```
I_n × A == A  (when A is n×anything)
A × I_m == A  (when A is anything×m)
```

A different identity for every square size. `I_2` is not a universal neutral element; it's neutral specifically for 2-row or 2-column partners. This is the first taste of dependent types: the type of a value (`I_n`) depends on a runtime parameter (`n`). Lean could encode this at compile time as `identity : (n : Nat) → Matrix n n`. The other languages cannot — the shape parameter lives at runtime.

**Determinant is a partially-defined operation.**

`det(A)` is defined only for square matrices. On a 2×3 matrix, no determinant exists — not "returns zero" or "returns null." It has no meaning.

Every prior operation in the Rosetta Stone was total — defined on every input. G060 introduces the first partial operation: valid on a subset of the domain. Partial operations are load-bearing in real systems. `first()` of an empty list has no meaning. `divide` by zero has no meaning. `max` of an empty set has no meaning. Ignoring partiality produces a large fraction of runtime crashes. G060 is the first project where partiality is *principled* — not an oversight, but a fundamental property of the math — and the error-handling approach (returning `Result<f64>` or raising a typed error) sets the template for every future partial operation in the noosphere.

**Shape inference for the result.**

When you multiply `m×n` by `n×p`, the result is `m×p`. Inner dimensions cancel; outer dimensions survive. The caller doesn't specify the output shape; it's derived from the operands. This is the first project where a value's shape is *computed* rather than *specified*. Every tensor library, every type-inferring compiler does this — G060 is the minimal teaching example.

**Cofactor expansion is O(n!) and we don't care.**

The determinant algorithm here — Laplace/cofactor expansion on the first row — is O(n!). Real libraries use LU decomposition (O(n³)). The Rosetta Stone chooses cofactor for the same reason G057 chose base-10 digits: the algorithm matches the math you learned on paper. Clarity over speed; a teaching corpus earns readability by explicit trade.

G048 identity → ... → G057 value class → G058 pipeline class → G059 polymorphic-interface → G060 *shape-typed value + non-commutative arithmetic + partial operations + shape-parameterized identity + shape inference*. Thirteen projects. Classes has now enumerated every fundamental kind of class the noosphere will need.

---

## G061 — Flower Shop Ordering

**The cart is the first intentionally-mutable primary entity.**

Every prior entity was either immutable (value classes like BigInt, Recipe) or mutated only through atomic commits (inventory movements, appointment reschedules). G061's Cart is *designed* to evolve: add flowers, remove, change quantities, apply discounts. A single cart may be modified dozens of times before anyone commits to an order.

This is the first project where the class's design point is *ongoing mutation as the primary operation*. Previous classes treated mutation as the rare operation; G061 treats it as the default. Carts are conversations with yourself: "I want roses — actually, make it six — plus two tulips — hmm, scratch the tulips, add lilies." In the noosphere, this maps directly onto *drafts*: a daily note being written, a project being scoped, a choreography being designed before being executed.

**Build-up/commit separates editing from committing.**

The lifecycle has three distinct stages: Draft (Cart, mutable) → Placed (Order, committed) → Cancelled/Fulfilled (terminal). Only the transition from draft to placed is atomic; everything within draft is freely mutable; nothing within placed or terminal can be modified.

This three-phase lifecycle (edit → commit → finalize) recurs throughout the vault. The Daily Note is edited freely all day (draft), the nightly summary is the commit (placed), the following day's rollup archives it (fulfilled). A conversation has the same shape: messages accumulate, the reply commits, and it's either answered or retracted. G061 makes the pattern explicit.

**Prices are frozen at add-time.**

```
add_to_cart(cart, "ROSE", 6)   # captures current price: $4.50
# ... time passes ...
flower["ROSE"].unit_price = 6.00   # catalog price changes
checkout(cart)                  # still charges $4.50
```

The cart line carries the price it had when added, not whatever the flower costs at checkout. The demo verifies this: ROSE's catalog price changes to $6.00 after the cart is built, but the cart's line remains at $4.50. The receipt reflects the frozen price.

This invariant is load-bearing. If price tracked the catalog live, carts would become unpredictable. The freeze is a promise: "you agreed to this price; we'll honor it." Generalizes: the vault will make analogous choices everywhere — some references are live (`@projects.get_current`), some are frozen (`@conversation.original_message`). G061 introduces the explicit snapshot pattern, distinct from live reference.

**Discount rules compose via sum of independent applications.**

A cart may have multiple discount rules, each with its own applicability check. 10% off orders over $30. Free shipping over $40. All can apply simultaneously. The total discount is `Σ rule.discount_cents(subtotal, shipping)` across rules. Some are conditional on subtotal, some are scoped to a specific cost, but all share the interface `(subtotal, shipping) → discount_cents`.

This is the closed-polymorphism version of G059's open interface — a fixed small set of rule types, each with its own dispatch logic. The noosphere composes rules this way constantly: pace-check strategies (daily + weekly + monthly), agent routing filters (priority + capability + timezone), permission checks (user + role + project + resource). G061 is the canonical teaching example.

**Cancellation is reversal — an inverse operation.**

Cancelling a placed order is not "mark it cancelled." It *restores* the inventory that was decremented at checkout. For each line, `stock += line.quantity`. This is the first explicit inverse operation in the Rosetta Stone.

Compare to prior cancellations:

- G050's `cancel` on a reservation just marked status — the interval was future, nothing needed unwinding.
- G049's `return` on a rental marked the loan complete — a separate lifecycle event, not a reversal.
- G061's `cancel_order` applies the inverse of checkout's commit — inventory state is explicitly reversed.

G050 cancelled a *promise*; G049 ended a *duration*; G061 reverses a *fact*. When state has been committed, cancellation is inverse-operation, not status-flip.

**Not every commit has a clean inverse.** Sending an email has no true inverse. Publishing a post has no true inverse. Notifying a user has no true inverse. When a system advertises "cancel/undo," it's making a claim about which operations are invertible. G061's inventory model is cleanly reversible because stock decrements commute with increments. Many real operations aren't, and the lifecycle quietly restricts cancellation to `PLACED` states to avoid pretending it can reverse a shipped order — a design lesson that applies to every choreography with side effects.

G048 ... → G060 shape-typed value → G061 *build-up/commit mutable draft + price snapshot + discount composition + inverse-operation cancellation*. Fourteen projects. Classes has gone through the full spectrum from pure value classes (G057) through polymorphic hierarchies (G059), algebraic operations (G060), and now transactional mutable drafts with explicit reversibility.

---

## G062 — Vending Machine

**Operations gated by state, not (only) by data.**

Every prior Classes project had operations gated by data invariants: sufficient stock, valid triangle, correct shape. G062 introduces a qualitatively new gate — the machine's **state**. `select` in Idle state isn't "insufficient funds"; it's the WRONG OPERATION for the current state. The machine has no coins; asking it to dispense is a category error.

The distinction: data gates say "the numbers don't add up"; state gates say "the system isn't in a posture to do that at all." This is the noosphere's model for any multi-turn interaction. A choreography in `planning` accepts `commit_plan`; in `executing` it accepts `abort` but not `commit_plan`; in `completed` it accepts nothing mutating. A conversation in `awaiting_reply` cannot accept another prompt. Every stateful protocol in the vault is an FSM, and G062 introduces the primitive.

**Transitions are first-class events — and failure doesn't always transition.**

Insert a coin: `Idle → Accepting`. Successful select: `Accepting → Idle`. Refund: `Accepting → Idle`. But **failed** select (insufficient funds, sold out, can't make change) preserves state — the user keeps their coins and can retry, add more, or refund.

That choice is load-bearing. Data-shortage failures leave the user in Accepting with their coins intact. They failed to pick the right thing, but the machine doesn't punish them by dropping coins. The only ways out of Accepting are a valid purchase or an explicit refund. This is a deliberate FSM design choice, and it has noosphere-wide implications: failures preserve state when the caller might recover, transition when the failure is terminal. Being explicit about which is which, for every failure, is the discipline good FSM design demands.

**Change-making is G007 with a new constraint: finite coin pool.**

G007 Change Return assumed unlimited coins. G062 constrains supply to the machine's current inventory (plus the coins just inserted). If the machine needs 15¢ but holds only quarters, no solution exists. The algorithm tries high-denom first, capped at available counts; if a denomination runs out mid-computation, it continues with smaller denoms; if the amount can't be reached at all, the purchase fails and coins refund intact.

This is the Rosetta Stone's first **capacity-constrained** algorithm. The demo's second machine illustrates: no small change loaded, user inserts $1.00 for 30¢ gum, owed 70¢ but only a single dollar coin in the pool. No valid change. Purchase fails. Refund returns the $1.00 exactly.

Capacity-constrained greedy appears throughout the noosphere — token budget for an LLM call (use the fewest, capped at the limit), agent assignment under load (prefer least-loaded, capped at capacity), disk allocation (largest free block first, capped at available). G062 is the minimal teaching example.

**Purchase is multi-entity atomic mutation with bidirectional flow.**

A successful select touches four pieces of state simultaneously: slot inventory decrements, coin inventory loses change coins but gains inserted coins, inserted-coins reset, state transitions. Four coordinated updates — all land or none do.

The new twist versus G052 (transfer) and G061 (checkout): **bidirectional flow within the same entity**. The coin inventory simultaneously *loses* change and *gains* payment. The inventory's composition changes, not just its totals. This generalizes to any exchange system — a forex trade exchanges one currency for another, a work-swap decrements one agent's queue and increments another's. G062 shows the primitive with the clearest possible domain.

**Two-phase settlement: inserted coins are held separate until commit.**

A subtle but important design choice — inserted coins are NOT part of the machine's change-making pool until the purchase commits. During Accepting, they sit in a separate `inserted_coins` table. This matters for the failure case: if a purchase fails and coins refund, the refund returns the exact coins that were inserted (quarters for quarters, dollars for dollars). If inserted coins were merged into inventory immediately, the refund could only return coins from the general pool, which might not match.

Keeping them separate until commit is the first instance of **two-phase settlement** in the Rosetta Stone. Phase 1: insert coins into a holding area. Phase 2: commit (merge into main pool) or refund (return from holding). This is how real payment systems work — holds on credit cards, escrow accounts, pending transactions. G062 makes the pattern concrete.

G048 ... → G060 shape-typed → G061 build-up/commit → G062 *finite state machine + state-gated ops + capacity-constrained greedy + bidirectional atomic flow + two-phase settlement*. Fifteen projects. Only three Classes projects remain (Josephus, Family Tree, and the wildcard choice at the end). Classes has delivered its full vocabulary; the remaining projects are applications, not new primitives.

---

## G063 — Josephus Problem

**Modular arithmetic replaces structure.**

N people in a ring; count k; remove; repeat. The whole of circular traversal fits in `(pos + k - 1) mod |ring|`. No linked list. No cycle detection. No wrap-around bookkeeping. Modulo *is* the ring.

This is the first Rosetta Stone project where **arithmetic replaces structure**. A circular linked list works — Common Lisp's classic `(setf (cdr tail) head)` trick — but the modular index is shorter, faster, and portable across every language with `%`. The design lesson is the choice: arithmetic over structure, wherever the structure exists only to be walked cyclically.

Modular position is everywhere in the noosphere: day-of-week (`date mod 7`), hour-of-day, beat-of-a-measure, ring-buffer writeback in the IPC layer. G063 isolates the primitive at minimal scale.

**The recurrence obsoletes the simulation.**

```
J(1, k) = 0
J(n, k) = (J(n-1, k) + k) mod n
```

O(n) versus the simulation's O(n²). For n=1,000,000 the recurrence runs in milliseconds; the simulation takes a trillion operations. The closed form is not an optimisation — it is a **qualitative change in what the problem is**.

G063 is the first Rosetta Stone project where a one-line recurrence obliterates a visible simulation. The simulation stays in the code as a **verification oracle** — property-tested against the recurrence for every (n, k) up to a bound — but in production nobody runs it.

This generalises. Fibonacci has the golden-ratio formula. BigInt multiplication has Karatsuba and FFT. Many problems with obvious quadratic simulations have sub-quadratic closed forms that nobody would guess from the problem statement. The lesson: when the simulation is easy and slow, ask whether there's a shortcut. Sometimes there isn't. Sometimes there's a one-liner hiding in the math.

**Two queries on the same process, two optimal algorithms.**

"Who survives?" — recurrence, O(n). "In what order are they eliminated?" — only the simulation answers this, O(n²). The two questions have different optimal algorithms on the same process.

This pattern recurs everywhere:
- Git **blame** (who last touched this line?) vs git **log** (the whole history).
- Agent dispatch — "who gets this job?" (one decision) vs "how did we assign yesterday's 10,000 jobs?" (the full ledger).
- Final balance of an account (one number) vs the movement log (every entry).

One answer is a reduction over a process. The other answer *is* the process. **Choose the algorithm to match the query, not the query to match the algorithm.** Systems that store only the closed form can't answer narrative questions; systems that store only the log compute summaries slowly. Most real systems carry both, by design.

**The answer is the fixpoint of iterated reduction.**

Simulation is a fold that keeps deleting until one element remains. The survivor is not a computed property of the input; the survivor IS the final state of the iteration. This is the first Rosetta Stone project where the output is the terminal point of a transformation rather than a derived scalar.

Consensus algorithms iterate until no message changes state; physics simulations iterate until energy stops decreasing; optimisation iterates until the gradient vanishes. Every "keep going until you can't" algorithm has the same shape as Josephus simulation. G063 is the minimal discrete instance.

**Closed forms hide the process.**

There is a philosophical edge here: the recurrence tells you *who* survives but gives no account of *how*. You cannot point at `J(n,k) = (J(n-1,k) + k) mod n` and say "she survived because the eighth elimination removed the person three seats to her left." The recurrence is statement-of-fact without narrative. The simulation has narrative but no shortcut.

The noosphere carries both. Audit logs, movement ledgers, and choreography traces record the narrative. Computed projections drop it and keep the summary. **Neither is more correct.** They answer different questions, and any system that commits to only one will be unable to answer the questions the other handles. Be explicit about which you store — and accept that storing both is the cost of answering both kinds of question.

G048 ... → G062 finite state machine → G063 *circular indexing + closed-form recurrence + two-queries/two-algorithms + fixpoint answer + narrative-vs-summary*. Sixteen projects. Josephus is the smallest project in Classes by code volume — the whole recurrence is three lines — and one of the largest in conceptual yield. Two projects remain in the category.

---

## G064 — Family Tree Creator

**The class references itself.**

Every prior Classes project had fields of primitives or references to *other* types. Person is the first type whose most important field is a list of references back into its own collection — `Person.parents: List<PersonId>`. The type is self-referential; the graph is in the data, not beside it. Conversations reference conversations. Documents reference documents. Tasks reference tasks. Every recursive structure in the vault has this shape. G064 is the primitive.

Two representations available, one chosen: **reference fields on the node** (direct, duplicative if both directions stored, risk of drift) versus **external edge list** (normalised, SQL-shaped, costlier queries, no duplication). G064 takes the first and *derives* the reverse index (children) as a maintained cache, reconciled on every edit. Every implementation in every language rebuilds this the same way — the pattern is portable.

**"Family tree" is a misnomer — it's a DAG.**

A tree has one root and one path per node. A family pedigree has multiple roots and multiple paths to shared ancestors (cousins married each other at some point in most old families). The correct data structure is a directed acyclic graph, and traversal must track a seen-set to avoid re-visiting the same ancestor via multiple paths.

This is the same lesson as the vault's wiki-link graph: tools that assume tree structure (breadcrumbs, "parent folder" for tags) fail silently on the real shape of the data. Any structure that permits joining of branches — references, citations, block-level transclusions, friendship graphs — is not a tree and should not be rendered as one.

**Cycle prevention is the first transitive-closure invariant.**

Adding P as a parent of C requires checking that P is not already a descendant of C — walking the *entire reachable* structure, not just the proposed edge's neighbours, and refusing if the edge would close a cycle. This is the first Rosetta Stone invariant that spans the transitive closure of the structure rather than a local neighbourhood.

Transitive invariants are expensive. G064's check is O(n) per `set_parents`. Real genealogy databases maintain precomputed ancestor/descendant sets, invalidated on edit — cheap reads at the cost of expensive writes. The choice between "compute on read" and "materialise and maintain" is one every graph-backed system makes.

Same shape elsewhere in the vault: task blockers (adding "A blocks B" when B transitively blocks A would deadlock the project), citation graphs (self-citation through intermediaries is broken), choreography dependencies (A requires B requires A is a logical error). All three demand the same transitive closure walk G064 performs.

**Traversal is the primary interface.**

Family trees aren't queried by field lookup. They're queried by traversal: `ancestors(me)`, `descendants(me)`, `common_ancestors(a, b)`, `siblings(me)`, `cousins(me, n)`. Each is a BFS with a particular expansion rule — and G064's BFS is parameterised on a successor function and a depth bound, so the same search engine handles every query.

This is the first Classes project where **search, not indexed lookup, is the API**. Wiki-link walks, conversation reply-chain traversals, and task-blocker resolution all have the same shape. The BFS code in G064 can be copied verbatim into any graph-shaped domain.

**Bounded traversal: cost budget built into the query.**

`ancestors(me, max_depth=3)` asks not "who are all my ancestors?" but "who are my ancestors, but only back three generations." The search terminates at depth 3 even if more ancestors exist beyond — the cost budget cuts the BFS at the frontier, not post-filters the result.

Bounded traversal is the default for any unbounded-graph query: web crawls with max-depth, federated feed walks with hop limits, git log with max-count, LLM context-window budgeting as a depth bound on conversation history. G064 introduces the primitive at minimal scale, and the `None`/`-1`/`nil` "unbounded" sentinel remains available for cases where the graph is known to be small.

**Set algebra over computed sets.**

`common_ancestors(a, b) = ancestors(a) ∩ ancestors(b)` — set algebra composed on top of algorithmic sets rather than pre-materialised ones. G056 did set algebra on tag *fields* (the data existed); G064 does set algebra on *traversal results* (the data is computed by algorithm, doesn't exist until asked for). Most real graph queries work this way: "authors I've co-written with AND who cited Paper X" is one BFS on the co-authorship graph ∩ one BFS on the citation graph, neither set stored anywhere. G064 makes the compositionality explicit.

G048 ... → G063 closed-form shortcut → G064 *self-referential graph + transitive-closure invariant + traversal-as-API + bounded BFS + set algebra over computed sets*. **Seventeen projects. Classes is complete.** From G048's product inventory (identity + counts) to G064's family graph (identity + self-reference + transitive invariants), the category has walked the full vocabulary of entity-based programming: identity, interaction, scheduled interaction, measured interaction, multi-sided atomicity, three-layer identity, conjunctive availability, structure-valued entities, external references, value classes, pipelines, polymorphism, shape-typed values, build-up/commit drafts, finite state machines, closed-form shortcuts, and now self-referential graphs. The next category — Threading (G065–G068) — takes this vocabulary and adds concurrency.

---

## G065 — Progress Bar for Downloads

**The producer and the consumer are distinct actors.**

Every prior project had a single actor that both recorded state changes and surfaced them. G065 is the first project where the recorder (a worker thread incrementing a byte counter) and the observer (a display thread reading the counter) are **different threads** with a contract mediated by a synchronisation primitive. This separation is what makes concurrency *necessary* — a single thread's operations are already ordered; two threads are two timelines that must be merged explicitly.

Every concurrency pattern in the noosphere has this shape: agent A streaming to agent B; the IPC daemon handling many clients; the choreography runner polling spawned tasks; the UI polling the vault for updates. G065 is the primitive, isolated from every other concern.

**Atomics are the minimum synchronisation.**

A counter that is only incremented by one thread and only read (not modified) by another needs no lock — atomic add + atomic load is sufficient. Rust and Go use this directly. Python and Common Lisp idiomatically take a mutex, but the critical section holds one addition, so the lock functions *as* an atomic.

**Choose the lightest primitive that works.** A mutex costs more than an atomic, a channel costs more than a mutex, an RwLock is overkill for single-writer state. Every concurrency primitive has a cost; the design question is which is cheap enough for the required throughput. G065 presents the cost-reasoning on the smallest possible case, and the answer is "atomic counter."

**Completion is a state, not a flag.**

A boolean "done" is insufficient: the tracker must distinguish running, completed, cancelled, and failed. Same FSM pattern as G062 Vending Machine — but now observed from another thread, which introduces memory-ordering concerns that never existed in single-threaded state machines.

Terminal states are sticky: `compare_exchange` / `CompareAndSwap` enforces "only transition from Running; ignore if already terminal." First Rosetta Stone pattern where **state transitions are conditional on current state at the CPU instruction level**, not behind a mutex. The ordering matters: worker writes bytes, then writes status; observer reads status, then reads bytes — if these aren't ordered, the observer can see `status == Completed && bytes < total`, a corrupt view. Rust's `SeqCst` and Go's default sequential-consistency atomics enforce the ordering. This is the concurrency specialist's permanent vigilance: every multi-field read is a place interleavings can betray you.

**Cancellation is cooperative, not preemptive.**

The controller does not kill the worker thread; it sets a flag, and the worker checks it between chunks. **Preemptive cancellation is unsafe in almost every language** — killing a thread mid-operation leaves locks held, files open, refcounts wrong. Every production concurrency library (Go contexts, Rust tokio, Java interruption, Python Event) uses cooperative cancellation because the alternatives are nightmares.

Cooperative cancellation imposes a contract on the worker: **check frequently**. Chunk size is itself a design parameter — coarser grain means cheaper work but slower cancellation response; finer means faster response at more overhead. The noosphere's choreography runner uses the same pattern: between steps, it checks cancellation. An agent that runs 10 minutes without checkpoints is uncancellable; the fix is to insert more checkpoints, not to "force-kill" the agent.

**Output throughput decouples from input throughput.**

A 1 GB download in 1 KB chunks is a million increments; the eye distinguishes 30 frames/sec. The display polls on its own schedule, not the producer's — **every 50 ms regardless of whether one chunk or ten thousand arrived in between**. First Rosetta Stone pattern where the consumer deliberately doesn't react to every producer event: coalescing, downsampling, debounce.

Every prior project had 1:1 event-to-response mapping. G065 introduces deliberate output-rate independence. Log aggregation, metrics collection, the vault's editor-update debounce, the quickshell bar's 10s bluetooth poll — all instances of "consumer chooses its own cadence, ignoring the producer's rate." G065 is the minimal teaching example.

**Time becomes a first-class input.**

Every prior project computed outputs as pure functions of data inputs. G065 is the first where **wall-clock time is part of the computed output**: rate is bytes-over-elapsed, ETA is remaining-over-rate. These values depend on a clock, not just on the data.

This is the edge where pure-functional code ends. Time-dependent output means repeated calls return different values — a violation of referential transparency. Testing gets harder: either inject a fake clock or accept non-determinism. G065 lets `elapsed_seconds` be real time because the tests that matter only assert `eta > 0`, not a specific value — the first instance of **accepting imprecision in tests because the system is inherently non-deterministic**.

Any real system that ignores time as an input fails as soon as it deals with durations: daily-note schedule blocks, agent-dispatch SLA checks, cache expiration — all need the clock. G065 introduces it at minimal scale.

**Classes vocabulary + concurrency = Threading category.** Where Classes taught entity, state, invariant, traversal, G065 adds: shared state across timelines, atomic primitives, FSM observed from outside, cooperative termination, consumer-scheduled rendering, time-as-input. The three remaining Threading projects build on this foundation — G066 Download Manager (multiple concurrent workers sharing a pool), G067 Chat App-Threading (bidirectional message passing between threads), G068 Bulk Thumbnail Creator (fan-out, fan-in, work-stealing). The category is short because the primitives are few; the projects test each in combination.

---

## G066 — Download Manager

**The queue is the rendezvous.**

G065 synchronised one producer and one consumer through a counter. G066 synchronises **one queue and N consumers** through a shared mutable collection. The queue is the rendezvous point: producers don't know which worker runs a job, workers don't know about each other, and a single queue-lock mediates every hand-off.

First Rosetta Stone project with a **shared mutable collection** under concurrent access. The queue must be safe against simultaneous pops (two workers returning the same job), push-during-pop (torn state), length-during-mutation (garbage readings). Every language's answer has the same shape: a mutex around pop/push with a tiny critical section. Go's channels hide the mutex inside the runtime's channel implementation — same primitive, different ergonomics.

**Pull scheduling balances load for free.**

No dispatcher assigns jobs to workers. Workers **pull** jobs whenever they're ready. A slow worker (big file, slow disk, congested network) pulls less often; a fast worker pulls more. The queue drains at the combined rate of all workers.

**Load balancing is emergent, not designed.** The system doesn't predict which job is slow or which worker is loaded — the pull model makes the right decision automatically. A bad guess about job size at enqueue doesn't penalise throughput; a fast worker blows through many small jobs while a slow worker grinds on its one big one, and both finish near the same time.

Push scheduling (dispatcher-routes-to-specific-worker) is worse in almost every case: it requires the dispatcher to guess worker load, guess job cost, and correct its guesses as reality diverges. Pull scheduling punts the whole problem to the workers themselves. The noosphere's agent-dispatch will use pull: idle agents request the next task, no dispatcher picks winners.

**Worker count is a resource cap.**

More workers = more parallelism = more throughput, up to a point. Past that point, more workers = more context-switching, memory pressure, open connections, and the system slows down. The optimal count depends on workload and machine. G066 presents it as a constructor parameter.

First Rosetta Stone project with a **concurrency limit as an explicit resource cap**. Setting it to 1 is legal (degenerate serial case). Setting it to 1000 is legal but almost certainly wrong. Every real pool exposes this as configuration: the agent-dispatch pool size, the max-IPC-client count, the parallel-tool-call cap when an LLM spawns agents. G066 introduces the primitive.

**Fan-out, fan-in — the map-reduce shape.**

The queue fans out work across N workers; the results collection fans in outcomes into one list. This is the classic **map-reduce** shape, minimal instance. Every large-scale batch pipeline (Hadoop, Spark, Beam, the noosphere's future bulk reindex job) has this exact shape. Worker count tunes parallelism; reduce step determines what the answer is.

**Failure isolation is the whole point of per-job transactions.**

If one download fails, the others must keep going, and remaining jobs must still run. A worker that lets an exception propagate kills itself and orphans remaining jobs. A manager that aborts on first failure loses all successful work.

G066 treats each job as a **transaction**. The worker wraps `simulate` in try/catch / Result match / error-value propagation; errors become the outcome of that specific job and the worker moves on. First project where **partial failure is a first-class outcome, not an abort condition**. Every prior project treated failure as fatal for the entire operation. G066 promotes failure to a per-job outcome co-equal with success and cancellation.

Production systems live on this distinction. A batch of 10,000 tasks with 3 failures is a report-and-continue case, not a crash-the-pipeline case. The result list is the audit trail: "10,000 attempted, 9,997 succeeded, 3 failed with these errors." G066 is the pattern at minimum scale.

**Cancellation now has scope.**

G065 had one worker and one cancel flag. G066 has N workers and **cancellation applies to all of them at once**. A single `cancel()` call causes every running worker to produce `Cancelled` outcomes on the next check (or exit, for jobs not yet claimed).

The scope is **the whole manager** — no per-job cancel. Finer control (cancel-job-by-id) is possible but invites complexity (job might already have started, might be partially complete, might be in flight). G066 makes the coarse choice. Go's `context.Context`, Rust tokio's `CancellationToken`, and Python's `threading.Event` all encode the same pattern; ergonomics differ.

G065 producer-consumer → G066 *shared-queue worker pool + pull scheduling + concurrency limit + fan-out/fan-in + failure isolation + batch-scoped cancellation*. The next two Threading projects take this and add: G067 Chat App — bidirectional message passing (readers and writers on the same queues, not just producer→consumer); G068 Bulk Thumbnail — CPU-bound fan-out with result aggregation into an index.

---

## G067 — Chat Application (Threading)

**Every actor is both producer and consumer.**

G065 separated the producer and consumer onto different threads. G066 had one producer and N consumers. G067 is the first project where **every actor is both**: the communication graph is fully connected, not a pipeline. Alice sends to Bob's inbox and reads from her own; Bob sends to Alice's inbox and reads from his own.

This is what "chat" means structurally, and what **peer-to-peer**, **federated**, and **multi-agent** mean structurally. IRC/Matrix/Discord, agent meshes, shared-document collaboration, gossip protocols — all have G067's shape. The noosphere's agent-to-agent broadcast will look exactly like G067 at the communication layer.

**Broadcast is fan-out on every send.**

A send doesn't go to "the next available consumer" (G066's pull). It goes to **every joined user's inbox simultaneously**. The fan-out happens per send, not once at startup; the target set changes dynamically as users join and leave. A 1,000-user room where Alice says "hello" creates 1,000 inbox entries. Broadcast is expensive at scale — real chat systems use server-side fan-out, pub/sub brokers, per-room partitioning to mitigate cost. G067 presents the naive version: loop over inboxes, push to each.

**Dynamic membership is a new synchronisation problem.**

Users join and leave while others are mid-conversation. Three races not present in earlier projects:
1. Alice sends while Bob is leaving — does Bob receive?
2. Alice reads while Carol is joining — does Carol see past messages?
3. Bob leaves then Alice sends — silent skip or error?

G067's answer: membership and sends serialise on a single lock, giving deterministic resolution. Bob-leaves-then-Alice-sends means Alice's send sees post-leave membership and skips Bob. Alice-sends-then-Bob-leaves means Bob receives the message before leaving. There is no "in between" state; the lock makes operations atomic with respect to each other. Carol-joining-mid-conversation does NOT retroactively receive past messages — **join-time-forward delivery** — matching every real chat system's convention. Past messages live in the log; joining users query the log if they want backfill.

**Total ordering from a monotonic counter under a single lock.**

Wall-clock timestamps collide or drift; network delivery order depends on recipient's position. G067 establishes a total order by assigning a **monotonically increasing sequence number** at send time, **under the same lock** that protects the user list.

Every recipient sees strictly-increasing sequence numbers. Within a single sender, numbers also strictly increase (serialised by the lock). Across senders, interleaving is determined by lock-acquisition order — arbitrary, but **once determined, consistent across all recipients**. Alice's message seq=5 reaches Bob as seq=5 and Carol as seq=5.

This is the **logical clock** pattern at minimal scale. Real distributed systems use Lamport clocks (per-node counter), vector clocks (per-node counters tracked across all nodes), or hybrid logical clocks (physical+logical). G067 is the single-machine case: one counter, one lock, total order. Any multi-agent choreography with events from many sources needs this primitive — wall-clock is unsafe, a room-scoped counter is the right answer for any single-broker scenario.

**The log and the inboxes are two different data structures.**

The room maintains both:
- **log** — every message ever sent, append-only, permanent. Audit trail.
- **inbox per user** — unread messages for that user. Drain-on-read.

Their consistency requirements differ, but both are maintained atomically inside `send`. They serve distinct queries: "what did we say?" (log) vs "what do I need to look at?" (inbox). First Rosetta Stone project where **the same events are recorded in two data structures serving different questions**.

The split — **journal vs queue** — is everywhere in real systems. Email has a sent-folder log + per-recipient inbox queues. The bank has transaction journals + account-balance state. The noosphere's conversations table is a log; the agent-dispatch queue is an inbox. G067 introduces the primitive.

**Leave doesn't retroactively break history.**

When Bob leaves, his inbox is discarded with his user entry. But his past messages remain in the log with his name intact — the log is immutable history. A user who rejoins later has a fresh empty inbox but can query the log to see what happened while away. **Membership and memory are orthogonal**: losing your seat at the table doesn't erase your name from the minutes.

This separation matches every real chat system and is deliberate. The noosphere's conversations table works the same way: a deactivated agent's messages are still attributed to that agent in the historical record; only future messages are skipped.

G065 single producer/consumer → G066 N consumers pulling → G067 *every-actor-both-roles + broadcast-per-send + dynamic-membership synchronisation + total-ordering via logical clock + journal/queue split + membership-memory orthogonality*. One Threading project remains: G068 Bulk Thumbnail, which takes G066's fan-out worker pool and applies it to CPU-bound work where the result-aggregation is the point (an index of thumbnails-by-source-hash), closing the category with fan-out feeding into a merged output structure.

---

## G068 — Bulk Thumbnail Creator

**The result is an aggregation, not a list of independent outcomes.**

G066 produced a list of `DownloadResult`s — each independent, the list was bookkeeping. G068 produces a **shared index** that workers cooperatively build: `content_hash → thumbnail`, `source_id → content_hash`. The result is a single aggregated artifact shaped by every worker's contribution, not a collection of isolated outputs.

First Rosetta Stone project where this distinction is explicit. Many real parallel pipelines produce an aggregate: search indexes, dependency graphs, code maps, embedding stores. Workers don't compute independent answers — each contributes a piece of a final whole. The output IS the point of the computation.

**Content-addressing makes dedup trivial and parallel-safe.**

The naive check-if-exists-then-generate has a race: two workers see "not present," both generate, one overwrites the other. Wasted work at best, torn writes at worst.

**Content-addressing dissolves the race.** The primary key of the index is the content hash itself. Two workers with identical content compute the same hash, which collides in the index under a single lock — and *only one worker succeeds in claiming the slot*. The other observes the collision and records the mapping without regenerating. The expensive thumbnailer runs **exactly once per unique content**, regardless of source or worker count.

This is the pattern behind every content-addressed system: Nix/Guix package caches (hash of inputs → build output), Docker image layers, Git objects, IPFS, the vault's future blob-backed image store. G056 introduced content-addressed identity; G068 shows why it's the right choice under concurrency.

**The reservation pattern decouples lock hold time from work time.**

Naive check-and-insert holds the lock while generating the thumbnail — serialising all workers and destroying the point of the pool. The **reservation pattern** fixes this:

1. Acquire lock.
2. If hash present, release, record dedup.
3. Otherwise, insert placeholder (empty string / None), release.
4. Run expensive work **unlocked**.
5. Reacquire briefly to install real value.

Concurrent workers arriving between steps 3 and 5 see the placeholder as "present" and treat it as dedup — semantically correct, because someone else IS generating this thumbnail. The lock is held for two tiny windows; expensive work runs in parallel. Initial Rust test (with thumbnailer running *inside* the lock) took 241ms for 8 jobs on 4 workers — fully serialised. Reservation-pattern version took 60-80ms — real parallelism.

This is the primitive behind `sync.Once` (Go), `OnceLock` (Rust), memoisation under concurrency, content-addressed cache lookups, and any "compute once, share many" scenario. Production libraries hide it; G068 presents it explicitly.

**CPU-bound parallelism has different ergonomics from I/O-bound.**

G066 was I/O-bound; G068 is CPU-bound. The differences:

- **Optimal worker count.** I/O-bound: many workers win (most time spent waiting). CPU-bound: workers beyond CPU count produce no speedup, may slow via context-switching. G068's best worker count is usually `N = num_CPUs`.
- **Lock contention matters more.** CPU-bound workers are constantly ready to run; any serialisation directly caps throughput.
- **Python's GIL matters.** Python threads don't provide CPU-bound parallel speedup for pure-Python code. Dedup correctness still works; speedup doesn't materialise unless the thumbnailer releases the GIL (most image libraries do). Real Python CPU-parallelism needs `multiprocessing`. G068's Python version documents this honestly rather than faking a speedup claim.

**Aggregation state requires partitioned locks.**

G066 had effectively one hot lock (the queue). G068 has two:
- Queue lock (every pop).
- Index lock (every claim and install).

These are **orthogonal** — operations on the queue don't affect the index. Two separate mutexes are correct and preferred; a worker waiting on the queue lock doesn't block another worker installing into the index. First Rosetta Stone project where **partitioning locks** is visibly the right choice.

Production systems partition further: shard the index lock by hash prefix, or replace with a lock-free concurrent map (DashMap, sync.Map, java.util.concurrent.ConcurrentHashMap). G068 uses one global index lock because the demo doesn't need more — but the seam for partitioning is there.

**Closing Threading — all four primitives are now present.**

The category's four projects introduced, in order:
- G065 **shared counter + FSM** (one producer, one consumer, atomic primitives, cooperative cancellation, time as input).
- G066 **shared queue + fan-out** (one producer, N consumers, pull scheduling, concurrency limit, failure isolation).
- G067 **shared broadcast** (N producers, N consumers, total ordering via monotonic counter, journal/queue split, dynamic membership).
- G068 **shared index** (N producers contributing to an aggregated output, content-addressed dedup, reservation pattern, partitioned locks).

Together these cover the four fundamental concurrency patterns. Any real multi-threaded program is assembled from these: a thread pool (G066) with per-worker progress tracking (G065), broadcasting completions to subscribers (G067), and contributing results to a shared index (G068). The noosphere's agent-dispatch layer will use all four. **Threading is complete.**

G048 (identity + counts) → G064 (self-referential graph) → G065–G068 (concurrency added to everything above). Four Threading projects + 17 Classes projects = 21 projects of entity/concurrency vocabulary, the foundation that Web (G069–G084), Files (G085–G100), Databases (G101–G113), and Graphics (G114–G130) will build on top of. 68/130 complete. The foundational third of the milestone (numbers, text, networking, classes, threading = 68 projects) is done; the remaining 62 projects are applications and integrations using the vocabulary now established.

---

## G069 — WYSIWYG Editor

**Edit IS render — no intermediate representation.**

Every pre-WYSIWYG text format had a source form distinct from rendered output. LaTeX source ≠ PDF. HTML source ≠ web page. Markdown ≠ its preview. A separate parser lifted source into a render tree; a separate renderer projected the tree back to pixels. WYSIWYG is the philosophical opposite: **the model you edit IS the model that displays**. There is no source form, no parse step; the user manipulates the display directly and the underlying representation is what they see.

First Rosetta Stone project where this property is the *point*. Opens the Web category and sets its convention: structured internal model, render function, coordinate-translation at the API boundary. The vault itself is Markdown-native (source-projection world); Google Docs, Notion's editor, rich-text fields in most CMSes are WYSIWYG (no source form). G069 is the minimum structure needed to support the WYSIWYG model.

**Runs are the right granularity.**

Character-by-character attribute storage is correct but wasteful. Region-by-region storage (ranges like `bold: [0..5], [12..20]`) is compact but hard to update. **Runs** — contiguous spans of uniformly-formatted text — are the middle path. Three invariants:

1. All characters in a run share the same attribute set.
2. Adjacent runs have *different* attribute sets (else they'd merge).
3. Concatenation of run texts = document's plain text.

These hold after every operation; `normalize` re-establishes them — drop empty runs, merge adjacent-identical runs. It runs after every mutation. First Rosetta Stone project where **a normalization pass is a mandatory part of the API contract**, not an optimisation. A denormalised document (two adjacent bold runs) is a bug, not a valid state.

**Range operations reshape runs.**

User-facing operations work on character ranges (`bold positions 5..12`). The implementation splits runs at range boundaries (creating new run boundaries if necessary), modifies attributes of runs fully inside the range, then re-normalises. User thinks in positions; storage lives in runs; `split_at` is the bridge.

First Rosetta Stone project where the **user coordinate system and the storage coordinate system are deliberately different** and an operation translates between them. Parallels: spreadsheet cells by (row,col) but storage is sparse maps; DOM selection by character offset but storage is a tree; git line-level operations on content-hashed blobs; vault `[[wiki-link]]` text resolved against a graph. G069 is the minimal instance.

**Attributes compose as sets, not scalar overwrite.**

Applying bold to bold-and-italic text must preserve the italic. Removing bold from bold-italic leaves italic. Attributes compose as **set union** / **set removal**. G056 had tags as set-algebra but tags were one dimension (tag sets ∩ tag sets). G069 has many orthogonal dimensions (bold, italic, underline, heading, link, color, font) on the same character. Each is independent; each composes.

Most real text formats get this wrong. CSS `font-weight: bold` conflicts with `font-weight: bold italic` (latter is a font name, not a composition). HTML `<b>` nested inside `<strong>` produces ambiguous semantics. WYSIWYG done right stores attributes as a set and renders to whatever the output format needs — **the model is the source of truth; every render format is a projection.** G069 renders to HTML; the same model renders to RTF, DOCX, Markdown-with-extensions, or ANSI terminal codes with a different function.

**Rendering is a trivial projection of the model.**

Given the document is already structured, `to_html` is a walk: for each run emit opening tags, escape and emit text, emit closing tags in reverse order. No parsing, no round-trip. `to_plain` is an even simpler walk. Every output format is a 20-line function because the model has already done the hard work.

First Rosetta Stone case where **render is a trivial projection**. G058 Chart Making had render as a pipeline stage (data → scale → encode → layout → draw). G069 is one walk. The difference: G058's model was *data* (values to chart); G069's model is already *structured display intent*.

**Opens the Web category.**

Web (G069–G084) is about **interfaces that render meaningful output users interact with directly**. Every project in the category has a structured internal model that projects to an output format. G069 sets the convention (model is source of truth, operations mutate model never output, rendering is projection, user coordinates differ from storage coordinates). Page scrapers, CMSes, template makers, image browsers — all inherit this shape.

68 foundational projects done (numbers/text/networking/classes/threading) → 69 begins the applications third of the milestone. The vocabulary from Classes (entity, state, invariant, traversal) and Threading (shared state, fan-out, broadcast, content-addressed index) is now being applied rather than extended; G069 is the first project where the primary lesson is *how to use the vocabulary*, not *what the vocabulary is*.

---

## G070 — Web Browser with Tabs

**Scope is the central design decision.**

Every piece of state in the browser belongs to exactly one scope. Per-tab: URL, back stack, forward stack, scroll position, session cookies. Browser-wide: bookmarks, downloads, global history, persistent cookies, user settings. **Getting the scope wrong shows up as user-surprise moments** — "why did closing this tab lose my bookmark?" (bookmarks in tab scope), "why does Back go to a page from a different tab?" (back-stack in browser scope). G070 makes scope explicit; tests enforce it (closing a tab preserves bookmarks, per-tab history is independent across tabs, closed-tab stack survives tab-switches).

First Rosetta Stone project where **"which thing owns this state?" is itself the load-bearing lesson**. The vault/noosphere will ask this constantly: does this metadata live on the note or on the project? Does this preference live on the agent or on the session? Does this cache live in the choreography-run or in the user-session? G070 is the minimal case where the answer determines correctness.

**Per-tab history is independent timelines.**

Each tab's back/forward stacks are completely independent of every other tab's. Navigate Tab 1 `a → b → c`, switch to Tab 2, navigate `x → y`, switch back to Tab 1, click Back — land on `b`, not `y`. Per-tab stacks are **branching timelines**, not shared state.

Early browsers had one shared navigation stack and it was awful — every tab-switch was a navigation event, Back went to whatever you last clicked regardless of which tab. Modern browsers moved to per-tab history and nobody misses the old model. G070 implements only the modern one because only one design is defensible. Analogues in the vault: each open note has its own cursor/scroll/folding state; each agent conversation has its own context history; each choreography execution has its own step-by-step state. G070 in a domain everyone recognises.

**Navigate truncates forward — the "branching timelines" pattern.**

When the user clicks Back then navigates to a new URL, the forward stack is cleared. The previous timeline (where they were going to visit `c` after `b`) is gone; they've branched onto a new path.

Correct and subtle. The naive alternative "keep the old forward stack, the user can push Forward to get back to it" gives the user two possible futures and one current state — semantically incoherent. Every browser, editor-with-undo, and version-control system works this way: redo/forward is lost as soon as you branch. Git reflog and the closed-tab stack are the *workarounds* for this — they let you recover a branched-away timeline, but only because they live in a different scope: they're recovery mechanisms, not first-class history.

**Closed-tab recovery is a second, orthogonal undo stack.**

Ctrl+Shift+T reopens the last-closed tab with URL and full per-tab history intact. This is a **completely separate** undo mechanism from per-tab back/forward:
- Per-tab back/forward — within a tab, navigate between visited URLs.
- Closed-tab recovery — within the browser, undo the decision to close a tab.

Scopes different, lifecycles different, storage different, they don't interact. First Rosetta Stone project where **two undo-like stacks coexist with different scopes and different semantics**. Vim has this (per-buffer undo tree vs global `:bdelete` stack); the noosphere will eventually have this (per-choreography step-undo for aborting a step vs global session-recovery for cancelled choreographies). G070 is the minimal teaching case.

**Active tab is a pointer, not a state.**

Which tab is active is a single `Option<TabId>` on the browser. No per-tab `is_active` flag — that would be redundant and could desync from the browser's pointer. The single source of truth is the browser's pointer; every "is tab X active?" query consults it.

First Rosetta Stone case where the DRY / single-source-of-truth principle is applied to **mutually-exclusive state** — exactly one answer at a time, storing it in one place guarantees consistency. Parallels: filesystem CWD pointer, terminal-focused-window pointer, vault's "current note" as UI-level pointer not note-level property. Cost: every mutation (close, open, switch) must maintain the pointer invariant. G070 implements all three close-and-reactivate cases (next tab if exists, previous if no next, null if empty) because leaving `active` pointing at a closed tab crashes every subsequent operation.

G069 edit-is-render → G070 *scope-as-design-decision + independent per-tab timelines + branching-timeline truncation + orthogonal recovery stacks + pointer-not-state for mutual exclusion*. Two Web projects done (WYSIWYG + Browser), fourteen to go.

---

## G071 — Page Scraper

**Parse is inverse of render, but render is total and parse is not.**

G069's render is a *total function*: every document has one correct rendering, no such thing as an "unrenderable" document. G071's parse is *partial*: not every string is a valid HTML document. The implementation chooses what to do with malformed input.

Two philosophies: **strict** (refuse malformed, produce error) vs **forgiving** (accept anything, produce best interpretation, recover). HTML parsing is the canonical forgiving-parser domain. Real web pages are messy; browsers encounter every malformed input imaginable and users expect something, not a parse error. **Postel's principle**: strict in what you send, liberal in what you accept.

G071 is forgiving. Unknown constructs become text; stray close tags drop; unclosed tags let content escape to parent. First Rosetta Stone project where **recovery is the spec, not an afterthought**. The noosphere will have many such domains: user-typed search queries, external RSS feeds with broken XML, Markdown files with invalid frontmatter, third-party choreographies. All need forgiving parsers.

**Selectors are a domain-specific query language.**

Once the tree exists, querying needs a language. CSS selectors are the natural fit — web developers already know them. G071 implements tag, class, id, combinations, descendant combinator — the subset covering most real scraping.

First Rosetta Stone project with a **domain-specific query language**. Prior projects queried collections through direct method calls (`room.user_count()`, `tree.ancestors(id)`); G071 parses a string into a selector AST then evaluates against the tree. Two parsers — one for HTML, one for selectors — both forgiving in the same way.

DSLs are everywhere in the noosphere: wiki-link paths `[[namespace/note#section]]`, tag queries `#foo -#bar`, search queries. Every DSL has the same shape: parse user string into AST, walk AST against data. G071 is the minimal teaching case.

**Descendant combinator is path-based pattern matching on trees.**

`article p` selects any `p` anywhere inside any `article`, at any depth. The algorithm: walk tree, match first selector element, continue searching descendants with remaining chain. Subtle rule: a matched element's descendants must keep searching (for more matches at deeper levels) AND unmatched elements' descendants must keep searching (so nested structures aren't skipped). Both rules apply simultaneously.

Same algorithm generalises to XPath, JSONPath, vault wiki-link resolution, filesystem glob descent (`src/**/*.ts`), git ref-matching (`refs/heads/feature/*`). G071 presents the minimum viable case; every tree-query-language extends it.

**Text extraction is a different walk with the same tree.**

`text_of(node)` returns the concatenation of text descendants. Ignores attrs, ignores structure except to traverse. A **different projection** of the same tree than `to_html` would be.

First Rosetta Stone case where **the same data structure supports multiple independent traversals for different questions**. G064 Family Tree had this at small scale (ancestors/descendants/siblings all BFS queries); G071 generalises: tree-walk is the universal interface to structured data, each "question" is a specific walk. One tree, many walks — why the vault's note-graph is useful: text-of-note, links-from-note, tags-on-note, notes-within-3-hops are all different walks on one structure.

**Parsing is the first place untrusted input enters the system.**

G069 was trusted; operations produced valid documents by construction. G071 takes arbitrary strings from outside — potentially network, potentially adversarial, potentially just broken. The parser is the first line of defence and must not crash or loop forever.

Safety properties G071 maintains: **termination** (every input produces output in bounded time — loops advance on every iteration, recursion bounded by input size), **memory bound** (output linear in input, no pathological amplification), **no crashes** (mismatched tags, truncated input, unusual UTF-8 don't panic). Production-hardening (rate limiting, depth limits, max attribute size) is orthogonal to the teaching point.

G069 WYSIWYG → G070 Browser → G071 *parse-as-partial-inverse-of-render + forgiving-parser + DSL-selectors + path-pattern-matching + tree-as-multi-query + untrusted-input-defence*. Three Web projects done, thirteen to go. The Web category's recurring theme is emerging: **structured model + render function + query language + robust input handling** — the four components every real document-processing system needs.

---

## G072 — File Downloader

**The file on disk is the durable state.**

In G065/G066 state lived in memory; process death = state death. G072 is the first project where **state survives the process** — the `.partial` file on disk IS the state, and the process can die arbitrarily without losing work. A 10-minute download must not restart from zero when the network blips at minute 9; bytes already received live on disk in a file whose name encodes "transfer in progress," and resume picks up from that file's size.

This is the boundary between "memory is state" and "disk is state." Every production download manager, rsync, torrent client, and backup system uses this pattern. The noosphere's future file-sync and media-fetch will use it exactly. First Rosetta Stone project where durability-across-process-lifetime is the property.

**Atomic rename is the promotion primitive.**

The download never writes to `dest` directly. It writes to `dest.partial` during transfer, then `rename(.partial, dest)` on success. `rename` on POSIX is atomic with respect to observers of `dest`: there is no instant where `dest` exists as a partial file. An observer sees either absence (transfer in progress) or the complete file (done).

Standard primitive for **making a write appear instantaneous** from observer's perspective even though the work took minutes. Databases commit this way (WAL + fsync + rename); config files update this way (write to `.tmp`, rename); package managers replace binaries this way; editors save this way. First Rosetta Stone project where **atomic promotion** is the correctness property. The vault's future document-write flow needs this — a half-saved note is worse than no save.

**Resume requires the hash to account for already-downloaded bytes.**

Subtle correctness bug: if the hasher starts fresh on the second attempt, it hashes only the bytes received on that attempt, not the bytes already on disk from the first. The final hash is wrong even though the disk bytes are correct.

Fix: on resume, open `.partial`, read its bytes, seed the hasher, continue appending new bytes to BOTH file AND hasher. Final hash now reflects the entire byte stream regardless of attempt count. First Rosetta Stone project where an invariant **spans the process lifetime** — hash state must be recoverable across restart, either by serialising it alongside `.partial` (complex) or re-deriving from `.partial` at resume time (what G072 does). Every real incremental-hash system faces this choice.

Parallels: rsync's block-level checksum table (recovered from both ends), `git fsck` (redirives integrity from stored objects), BitTorrent clients (re-hash pieces on startup to find what they already have).

**Transport is a parameter.**

The downloader doesn't know HTTP, FTP, local copy, or in-memory test transports. It asks an abstract `Transport` for "bytes starting at offset N." Protocol behind the trait is irrelevant to the correctness properties.

First Rosetta Stone project with **dependency injection as the transport boundary**. Tests use `FakeTransport` that can be told to fail at specific offsets; production uses HTTP clients; downloader code is identical. Pattern foundation of testable networked systems — every integration test of noosphere's agent-dispatch will use this (fake transport plays back pre-recorded interactions, production transport is real HTTP).

**Hash-mismatch keeps `.partial` for inspection.**

Refuse rename on mismatch; `dest` stays absent; `.partial` stays present. Deliberate: corrupt bytes available for inspection, debugging, content-addressed recovery (G068 could identify the mismatch if bytes had been seen correctly before). Alternative of "delete `.partial` on mismatch" loses the evidence.

First Rosetta Stone case where **failure produces diagnostic artifacts rather than returning to clean state**. Parallels: core dumps on crashes, `.rej` files from patch conflicts, failed-migration logs in database tools — all leave evidence. Principle: fail noisily with forensics.

G065 atomic counter → G066 worker pool → G067 broadcast → G068 shared index → G069 WYSIWYG → G070 browser state → G071 parser → G072 *durable state on disk + atomic rename + resume-aware hashing + abstract transport + forensic-on-failure*. Four Web projects done, twelve to go. The category's theme has expanded: structured model + render function + query language + robust input handling + **durable state with atomic promotion** = five-component architecture for any real document/file system.

---

## G073 — Telnet Application

**Protocol ≠ transport.**

A telnet *application* is not a telnet *server*. The server binds a socket and reads/writes bytes; the application decides what a line means once the socket delivers it. G073 implements the application; the socket is a detail slotted underneath.

This separation is the single most important lesson. Every real protocol — HTTP, SMTP, IRC, Redis RESP, Postgres wire protocol — gains testability the moment protocol logic splits from byte transport. Protocol tests call `handle_line(session, "login alice")` directly; transport tests mock byte streams; the two tests never need each other. First Rosetta Stone project where **protocol/transport split IS the design**. Every noosphere agent-dispatch integration test over IPC will use this shape: protocol is `{cmd, args, response}`; transport is Unix socket today, TCP tomorrow, WebSocket next.

**Session state is a three-state FSM with a terminal state.**

`Unauthenticated → Authenticated → Disconnected`. G062's vending machine had two states; G073 has three, and the third is terminal — no operation brings a session back from Disconnected.

**Terminal state is load-bearing.** Naive servers let commands run on disconnected sessions "because the data is still there" — that's a bug, not a feature. Disconnected means the protocol has ended; subsequent commands represent either a server logic error or an adversary holding a stale session id. G073 refuses all commands on Disconnected with an error.

First Rosetta Stone protocol with a **terminal state** — G062's terminal states (Completed/Cancelled/Failed) were optional for its purpose; G073's Disconnected is mandatory, every session eventually reaches it. Every real connection-oriented protocol has this: TCP CLOSED, HTTP/2 closed stream, IRC QUIT, Postgres terminated connection.

**Commands are a dispatch map, not a switch.**

G073's command handling is: parse line into `(cmd, args)`, look up `cmd`, delegate to handler. The handler-map pattern makes the command set **data, not code**. Adding a new command is an entry; removing is deleting an entry. Production protocol servers (Redis, Postgres, most chat servers) implement commands exactly this way.

Pattern matters for the noosphere: every choreography resolver is this. `@agent/dispatch{cmd, args}` looks up `cmd` in the resolver table, delegates, returns. The resolver's command set is extensible at runtime; new natives register without recompiling.

**State-gated commands map the security model.**

- `help` — any state (zero-trust)
- `login` — Unauthenticated only (double-login is error, not silent reassignment)
- `whoami` / `who` — Authenticated only (unauthenticated sessions cannot enumerate users; real information-disclosure concern)
- `echo` — any state (trivially safe)
- `quit` — any non-terminal state

**The legal-in-state table IS the protocol's security policy.** What commands are available to unauthenticated callers *is* the attack surface; anything beyond `help` and `login` leaks information. Every protocol bug has had a command accidentally legal in a state where it shouldn't have been.

First Rosetta Stone project where **access control is expressed through state-gating** rather than a separate permission system. Real servers often have both (RBAC + session-state); G073 shows state-gating alone is sufficient for many protocols.

**Per-session history is audit, not replay.**

Every line is recorded as `(line, reply)`. Not a replay mechanism — G073 doesn't re-execute history. It's an **audit trail**: who typed what, what the server answered. Survives until the session is GC'd.

**Journal-vs-queue** pattern from G067 Chat App applied to per-session scope. History is a journal (permanent, append-only); session state is the queue-like output of replaying it. Production analogues: HTTP access logs, Postgres query logs, IRC channel logs, the vault's future choreography-execution log. All append-only, all per-session-scoped, all independent of the application state they describe.

**Sessions are isolated — this is the whole point.**

Alice's login does not affect Bob's session. Alice's quit does not terminate Bob's. First Rosetta Stone project where **state isolation across callers is a correctness property**. G067 had similar isolation for chat inboxes but inboxes were symmetric data structures. G073 has asymmetric state (one user's login) and must guarantee independence — a regression letting Alice's login leak into Bob's session is a protocol-level security failure.

G065/G066/G067/G068 taught concurrency primitives; G069/G070/G071/G072 taught document/transfer primitives; G073 is the first *interactive* application — where a remote caller holds a session and issues commands over a line-oriented protocol. Five Web projects done, eleven to go. The Web category's architecture is now **model + render + query + robust input + durable state + interactive protocol** — six components, the foundation of any real application server.

---

## G074 — Online White Board

**Spatial data is a new coordinate system.**

Every prior Rosetta Stone project with "coordinates" had 1D coordinates: byte offsets in text (G069), seconds in a schedule (G065), URLs in a history stack (G070). G074 is the first with **2D spatial coordinates**. Distance is `sqrt((dx)² + (dy)²)`, bounding boxes are `(min_x, min_y, max_x, max_y)`, queries are "near this point" rather than "at this offset."

The jump from 1D to 2D is load-bearing. **1D has a natural total order; 2D does not.** Neither `(3, 5)` nor `(4, 4)` is "before" the other. Everything that assumed total order — sorted storage, binary search, before/after predicates — doesn't generalise. Operations must be redesigned around **nearness**, not order.

First Rosetta Stone project where **order is replaced by proximity** as the organising principle. The noosphere's spatial compositor (per memory `project_spatial_compositor.md`) depends on this primitive: rooms are 2D regions, agents are at positions, queries are "what's nearby." G074 is the minimal case.

**Strokes are first-class entities with identity.**

A stroke isn't a pixel path drawn onto a buffer. It's an entity with id, author, color, thickness, points. You can remove a specific stroke, filter strokes by author, find strokes near a point, render just one stroke.

The **retained-mode** vs **immediate-mode** graphics distinction. Immediate-mode draws into a buffer and loses the structure; retained-mode keeps the structure and renders from it on demand. G074 is retained-mode, which is what makes every interesting operation possible: undo (remove), edit (replace), query (who drew this?), export (render to SVG). Every real drawing app from Illustrator to Figma to whiteboard tools is retained-mode for this reason.

**Z-order is insertion order — storage order is semantic.**

Strokes draw in insertion order. No per-stroke z-index field, no depth calculation — the storage order IS the rendering order. This is the simplest possible z-model and it's what Figma, Sketch, Illustrator use at the layer level. Bring-to-front, send-to-back, move-up are **list rearrangements**, not field mutations.

First Rosetta Stone project where **storage order is semantic**. G067 Chat log order was chronological (meaningful); G070 tab list was UI arbitration (not meaningful to the protocol); G074's stroke order is **rendering-critical** — reorder the list and the image changes. `remove_preserves_z_order_of_remaining` verifies this: removing a middle stroke doesn't renumber; surviving strokes keep their relative order.

**Spatial queries are linear scans (until they aren't).**

`strokes_near(point, radius)` walks every stroke and checks distance. O(n × m) where n = stroke count, m = points per stroke. Fine for hundreds; catastrophic for millions. The production solution is a **spatial index** — quadtree, R-tree, grid hash, k-d tree — answering spatial queries in O(log n) or better.

G074 doesn't implement one; the linear scan is *correct*, and the index is a performance optimisation added only when profiling demands. First Rosetta Stone project where **the naive algorithm and the optimised algorithm have the same interface**. You can swap linear scan for R-tree without changing callers.

Same pattern throughout systems: hash maps over flat arrays (lookup), B-trees over sorted arrays (range), vector indexes over flat arrays (NN-search). The queries are the API; the index is an implementation detail. Protect this property in every geometric API — expose queries, not the index.

**Render is a format projection.**

`to_svg` walks strokes in z-order and emits `<polyline>` tags. Same shape as G069's `to_html`: structured model, walk, emit markup. Content differs; code shape is identical. Other formats (PNG by rasterising, PDF by path commands, LaTeX/TikZ for publishing, ASCII art for terminal) are 20-line walks because the model did the hard work.

First Rosetta Stone project where **the render format is genuinely parameterised**. G069 rendered to HTML only; G074 renders to SVG/PNG/PDF/ASCII with equal ease because the model doesn't care what the output format is.

**Per-author attribution enables collaboration queries.**

Every stroke carries its author. First Rosetta Stone project where **every atom of the data structure carries its provenance**. G067 had author on messages but the chat log was linear; G074 has author on every geometric element, enabling rich collaboration-aware queries (bbox of just-alice's-work, per-author undo, count-by-author).

The noosphere's agent-provenance tracking will use this pattern: every choreography output, vault edit, deliverable carries the agent/user that produced it. "Show me only what Alice's agents produced this week" becomes a filtered walk over provenance-annotated data.

G069 WYSIWYG (1D text model) → G070 Browser → G071 Parser → G072 File Downloader → G073 Telnet → G074 *spatial coordinates + retained-mode entities + z-order-as-storage-order + spatial-index seam + parameterised render + per-atom provenance*. Six Web projects done, ten to go. G074 opens the door to every remaining spatial/visual project in the milestone, particularly Graphics (G114–G130), by establishing the fundamental primitives of 2D entities and spatial queries.

---

## G075 — Bandwidth Monitor

**The useful signal is the derivative, not the counter.**

A network interface tells you one thing: a monotonically-increasing byte counter. "47,385,219 bytes since boot" is not actionable. The **signal** is bytes/sec right now, on average over the last minute, at peak over the last five minutes.

G075 stores the counter stream; it derives every rate. Every query walks the counters, picks samples spanning the window, divides byte-delta by time-delta. Counter is source of truth; rates are projections.

First Rosetta Stone project where **stored form and useful form are different**. Previous projects returned values stored directly (balance, user count, tags). G075 stores counters but never returns counters — only derived rates, computed on every query. Parallels: CPU percent, disk I/O, request throughput — every metrics system (Prometheus, StatsD, the vault's future observability) has this shape.

**Sampling cadence trades precision for overhead.**

Too infrequent: misses bursts (1s-average over 100ms spike sees 10× smaller). Too frequent: wastes CPU (sampling every 1ms on a GB/s link generates 1M samples/s overhead to detect rates that change 10×/s). Real monitors choose ~1 Hz for general use, higher for burst debugging.

First Rosetta Stone project where **observation frequency is itself a design parameter** separate from the observation primitive. The noosphere's agent-metric collection faces this: sample on a cadence that catches interesting behaviour without drowning in data.

**Average smooths, peak preserves — different statistics on same data.**

Steady 1 MB/s with a 100ms 10 MB/s spike: average ~1.9 MB/s (nearly invisible), peak 10 MB/s (exactly the spike). Saturated-link spikes are visible in peak but hidden in average; slow-sustained-growth is visible in average but not peak. Production monitors report both.

First Rosetta Stone project where **two summary statistics answer different questions about the same data**. Same shape as G063 (survivor vs elimination-order), G067 (log vs inbox), G074 (structure vs render). Pick the algorithm to match the question; store raw data so multiple algorithms can apply.

**Counter wraparound is an adversary, not an error.**

32-bit byte counters wrap at 4 GB — on a 100 Mbit link, every 5.5 minutes. Naive `(bytes_now - bytes_before)` goes negative across the wrap and reports implausible rates.

G075 treats wraparound as a **gap in observation** — if new count < old count, that pair is skipped; interval contributes zero, not negative. Rate resumes with next consecutive increasing pair. Alternatives (assume one wrap, compute `(new + 2^32 - old)`) work if you know counter width; G075's skip approach is safe without that knowledge.

First Rosetta Stone project where **the adversarial-data model is explicit**. Previous projects trusted inputs; G075 treats input as potentially misbehaved because it is. Every real metrics system has this: Prometheus reset detection, statsd counter-reset handling, Linux `/proc/net/dev` documenting counters can reset.

**Ring buffer is the right shape.**

Fixed-capacity deque that evicts oldest when full. Queries walk the buffer; no pagination, no external aggregation, no database. Bounded memory, bounded query cost.

First Rosetta Stone project where **bounded storage** is an explicit design property. Previous projects assumed unbounded growth (Family Tree nodes, Chat Room log, WYSIWYG runs); G075 caps because time-series is "most recent N" semantics. Same pattern: kernel perf ring buffer, syslog rotation, Prometheus TSDB chunks, vault activity log with rolling archive.

G069 → ... → G074 Whiteboard → G075 *counter-to-rate derivation + cadence-as-parameter + average-vs-peak orthogonality + wraparound-as-gap + bounded ring buffer*. Seven Web projects done, nine to go. Web architecture now eight components: model + render + query + robust input + durable state + interactive protocol + spatial data + time-series observation.

---

## G076 — Bookmark Collector and Sorter

**Folders and tags are orthogonal — both first-class.**

Early bookmark systems forced a choice. Netscape's hierarchical folders won the UI war but lost the model war — the same bookmark often belongs to multiple conceptual categories, and a tree can only place it once. Tag-based systems (delicious, pinboard) abandoned hierarchy entirely but users still want a default-location story.

G076 does both. A bookmark has **one folder** (primary filing location) AND **a set of tags** (secondary many-to-many categorisation). Neither is subordinate. First Rosetta Stone project where **two organisational axes coexist without hierarchy between them**. G056 had tags without folders; G064 had hierarchy without tags. G076 is the first with both intentionally.

Parallels: notes have folder path AND wiki-link tags; tasks have project (hierarchy) AND status/labels (flat); agents have team AND capability tags. Every real "stuff organiser" ends up with both.

**URL is the natural key.**

Re-adding a bookmark with an existing URL updates the existing entry, doesn't create a duplicate. URL is the natural key — two bookmarks with the same URL ARE the same bookmark. Re-add is update, not insert.

First Rosetta Stone project where **natural keys** (semantic keys the domain provides) are preferred over **surrogate keys** (numeric ids we assign). Surrogate id exists for reference stability (move/remove/tag); identity is the URL. This distinction matters throughout the vault: notes by path, agents by name, conversations by timestamp+participants. Choosing "what makes two of these the same thing?" is the deepest schema question.

**Sort is a view, never a storage property.**

The collection stores bookmarks in insertion order. No "sorted by title" mode, no re-shuffling on add, no index. Every sort is a **view** — computed on query, consumed by caller, discarded. Storage's natural order stays insertion order.

Multiple callers can request different sorts without fighting over "the order." Adding is O(1) — no sort maintenance. Sort is referentially transparent (same collection, same sort → same result). First Rosetta Stone project where **sort is deliberately decoupled from storage**. G074 Whiteboard had the opposite (insertion order WAS render order). G076 has neither storage order being semantic nor any sort being canonical; every sort is a lens.

**Stable sort** is the right default: ties preserve insertion order. Unstable sort produces different orderings on different runs, breaks tests, confuses users. Every language here provides stable sort primitives (Rust stable by default, Python Timsort, Go sort.SliceStable, CL stable-sort, Lean mergeSort).

**Folder paths as strings, not trees.**

Folders are path-like strings: `/news`, `/dev/languages`, `/personal`. No tree structure, no parent pointers, no folder entities. "Bookmarks in /dev" (recursive) works by prefix-match on `/` boundaries.

**Flat representation of a hierarchical concept** — what every real system uses (filesystem paths, URL paths, CSS selectors, vault wiki-links). Tree is implicit in the string structure, not materialised separately. Contrast G064 Family Tree's explicit graph: there relationships mattered for traversal algorithms; bookmarks don't need those — "what's in this folder" is a prefix query, "move to folder" is a string reassignment. Materialising the tree would be expensive bookkeeping for no benefit.

First Rosetta Stone project where **a hierarchical concept is represented flatly because no hierarchical operation is actually needed**. General lesson: don't materialise structure you don't query.

**Search crosses axes; upsert is soft idempotence.**

`search(query)` matches title, URL, AND tags with one substring query. Doesn't ask "which field?" — searches everywhere. First Rosetta Stone project where **the query crosses schema boundaries** (G064/G067/G071 had precise field-scoped queries); introduces the union-of-fields pattern behind the vault's future full-text search.

Re-add is upsert (keeps id, updates properties) — **soft idempotence**: same identity, possibly different content. First Rosetta Stone project with **upsert semantics by default**. Previous dedup was content-addressed (G056/G068); G076 is upsert at the API level, visible to callers. Production REST APIs often have both (POST = insert-only, PUT = upsert); G076 picks PUT as default because it matches user intuition.

G069 → ... → G075 Bandwidth → G076 *orthogonal multi-axis organisation + natural-key dedup + sort-as-view + flat-string hierarchy + cross-field search + upsert-default semantics*. Eight Web projects done, eight to go. Web is now exactly halfway; the category's architectural vocabulary is complete (model + render + query + robust input + durable state + interactive protocol + spatial data + time-series observation + multi-axis organisation), and remaining projects (password safe, media player widget, MUD-style game, scheduled action, e-card, CMS, template maker, CAPTCHA) are applications combining these components.

---

## G077 — Password Safe

**NOT real crypto.** Byte-sum pseudo-hash and XOR keystream demonstrate the *structure* of encryption-at-rest without pretending to be secure. Production would use Argon2id + AES-GCM. The STRUCTURE is faithful; the CRYPTOGRAPHY is pedagogical.

**The key lives only when unlocked.**

Safe has two states: Locked (key absent) and Unlocked (key in memory). When Locked there is no way to decrypt — ciphertext is persistent, key is ephemeral. Locking wipes the key; unlocking re-derives it from master+salt.

This is the fundamental pattern of **encryption at rest**. Encrypted data is always present (durable); decryption key is present only during active sessions (ephemeral). First Rosetta Stone project where **memory state and persistent state are deliberately different** — G072 had durable state on disk; G077 has *two kinds* of state (durable ciphertext + ephemeral key) with strict rules about when each exists. The noosphere's future encrypted-vault layer will use this exact pattern.

**Zero-knowledge is the feature.**

The safe has NO way to recover the master. No "email reset link," no security-question escape, no admin override. Losing the master loses the data.

**Zero-knowledge storage** — the safe doesn't know the master, can only verify it. 1Password, Bitwarden, KeePass all explicitly disclaim recovery; the feature this enables ("even if the database leaks, attackers can't decrypt without the master") is the entire point of a password manager.

First Rosetta Stone project where **the system explicitly refuses a recovery path** as a security property. Every prior project had some recovery path (rebuild from log, re-derive from source, reparse from storage). G077 has none by design. Noosphere will face this tension: more recovery = more attack surface; less = more user frustration. Different data classes justify different answers.

**Lock/unlock is an FSM (same pattern as G073).**

G073 had Unauthenticated → Authenticated → Disconnected. G077 has Locked ↔ Unlocked (no terminal — user can cycle indefinitely). Same primitive (state-gated operations), different lifecycle.

Every mutating operation requires Unlocked; they fail cleanly (return None / false / empty) when Locked. `get_summary` is the interesting exception — metadata (service/username/notes) is stored as plaintext and can be read without the key. **Only the password is encrypted.**

First Rosetta Stone project where **some fields are encrypted and some are not within the same record**. Real production choice: encrypting service name hurts search without adding security; encrypting password is load-bearing; encrypting everything uniformly is wasteful.

**Auto-lock is a time-based state transition.**

Unlocked state has a timeout. `auto_lock_after_ms` elapsed without activity → auto-lock. Activity (add/read/search) resets the timer. **Time as an implicit state-transition trigger** — same family as G062 Vending Machine state transitions but fired by clock, not by user action.

First Rosetta Stone project with **time as state-transition trigger**. Previous projects had time as input (G065 elapsed, G075 rates); never as the cause of a transition. G077 introduces the pattern. Parallels: HTTP session timeouts, SSH idle disconnect, OS screen lock, future vault "auto-dim" behaviour. Design knobs: duration + reset-on-activity policy.

**Password generation and strength are orthogonal.**

`generate_password(policy, seed)` and `password_strength(pw)` are separate functions. Generation doesn't check strength; strength doesn't care where the string came from. Rationale: users paste existing passwords ("save my Gmail password, don't rewrite it") and strength must work on those too; generation might produce Weak with a restrictive policy and user should see the score to tighten.

First Rosetta Stone project where **two related-but-orthogonal concerns are deliberately decoupled**.

**Verifier hash is NOT the key.**

Two different derivations from the same master+salt: verifier (stored, used for yes/no authentication) and key (derived fresh each unlock, used to encrypt/decrypt). Different functions so neither can be derived from the other. Attacker with stolen state gets verifier + ciphertext but cannot derive key from verifier.

First Rosetta Stone project where **two outputs derived from the same secret are NOT interchangeable**. Subtle but critical; forgetting this distinction is a headline vulnerability.

**Summaries never include the password.**

Explicit API-level separation: `get_summary` returns metadata only; `get_password` returns plaintext. Search returns summaries. Rationale: logs, UI scrollback, error messages all tend to contain "whatever the function returned"; if summaries included passwords they'd leak everywhere.

First Rosetta Stone project with **explicit API-level separation between metadata and secret material**. Production managers do this: clipboard-write ≠ display-in-UI; display is summary-only by default with explicit "reveal" gesture required. G077 models the API shape of that.

G069 → ... → G076 Bookmarks → G077 *encryption-at-rest + zero-knowledge + lock/unlock FSM + auto-lock via time + orthogonal generation/strength + verifier-≠-key + metadata-secret API split*. Nine Web projects done, seven to go. Web's architectural vocabulary adds **security primitives** (key-lifecycle, state-gated secrets, time-based transitions) — the tenth component.

---

## G078 — Media Player Widget

**Three-state playback is the minimum for pausable playback.**

G062 had two states (Idle, Accepting). G073 had three with one terminal (Unauthenticated, Authenticated, Disconnected). G077 had two (Locked, Unlocked). G078 has three **non-terminal** states — Stopped, Playing, Paused — and every state can transition to every other state (full 3×3 transition table).

Stopped = nothing loaded-and-ready-to-resume; Playing = time advances on tick; Paused = loaded and positioned but tick is a no-op, resume is cheap. Skip Paused and every lift of Play resets to track 0. Merge Stopped into Paused and you lose "start fresh" vs "pick up where I left off." First Rosetta Stone project where a three-state FSM is **load-bearing**, not incidental. The noosphere's choreography engine will likely use this exact pattern.

**Transport actions are context-dependent.**

`play()` has different behaviour in each state: Stopped → start track 0 + emit TrackStarted; Paused → resume, emit StateChanged only; Playing → no-op. The caller doesn't check state first — the handler encapsulates the policy.

Compare a naive design with separate `start()`/`resume()`/`continue()` — caller must check state. G078's `play()` absorbs the dispatch. **State-interpreted operations** (works in every state, does the right thing) vs G062's **state-gated operations** (refused if wrong state). First Rosetta Stone project with context-dependent operations on a running state machine.

**The tick function is the engine.**

`tick(delta_ms)` advances position, checks track end, fires boundary events, advances to next track (consulting repeat/shuffle), loops until `delta_ms` is consumed or the playlist runs out. The **game-loop pattern**: world has a tick that takes a time delta and advances everything. Every simulation, physics engine, game, scheduled-task executor has this shape.

First Rosetta Stone project with **tick-driven state advancement**. G062's transitions were instantaneous; G065's time was elapsed-since-start. G078 is the first where time is **consumed** by a function that turns "time passed" into "state changed and events fired."

**Playlist modes (repeat + shuffle) are orthogonal.**

Four combinations: repeat=(None|One|All) × shuffle=(off|on). Every combination is valid. Shuffle changes which track is next; repeat changes whether to stop or continue at end. They don't interact; they're independent axes. First Rosetta Stone project with **orthogonal mode dimensions** on the same operation. Previous projects had single-mode operations; G078 shows mode axes compose.

**Events fire on boundaries, not every tick.**

Track end → TrackEnded. New track start → TrackStarted. Playlist end → PlaylistEnded. State change → StateChanged. **Events are boundary signals**, not continuous streams. UI subscribed to events updates on changes; polls `position_ms` for the scrubber; subscribes to events for "now playing" display.

First Rosetta Stone project with **event stream as supplement to state query**. G065 had state queries only; G066 had outcomes returned from run. G078 has both: queries for continuous display, events for boundary-triggered reactions.

G069 → ... → G077 Password Safe → G078 *three-state non-terminal FSM + context-dependent transport + tick-as-engine + orthogonal mode dimensions + events-on-boundaries*. Ten Web projects done, six to go. Web adds **tick-driven engines** as the eleventh component — a primitive that will power every simulation, scheduled-task runner, or choreography executor in the noosphere.

---

## G079 — Text Based Game (Utopia-like)

**The world is a graph whose nodes have state.**

G064 had a self-referential graph (family trees); G070 Browser had state bags organised hierarchically; G079 combines them: **rooms form a graph**, and **each room is a state bag**. Exits connect rooms; items live in rooms; enemies stand in rooms; player occupies one room at a time. When the player moves, the world's topology doesn't change — only the player's pointer into it does.

First Rosetta Stone project where **a graph structure IS the simulation world**. Every room is both a node (outgoing edges named by direction) and a container (items and enemies inside). Graph and containers are orthogonal: moving doesn't change room contents; picking up an item doesn't change the graph.

The shape of every MUD, text adventure, dungeon-crawler. Generalises to: Kubernetes pods-on-nodes, DB tables with rows, vault notes with tag-memberships.

**Every command is a turn — user input drives the simulation clock.**

Turn-based: each input advances the world by one tick. Player's action resolves, world reacts (enemies strike), turn counter increments, end-conditions check. First Rosetta Stone project where **user input drives the simulation clock** — unlike G078's media player (self-ticking engine the caller advances on a schedule), G079 ticks on input only.

Both styles have their place. Turn-based games use input-driven ticking; real-time games use self-ticking with input as a separate event stream. Most real systems mix them: vault's daily-note system advances at midnight (self-ticking) but reacts to each agent message as an event (input-driven). G079 picks the simpler pure-turn-based model.

**Verb-noun parsing with aliases.**

First word = verb, rest = argument. Aliases normalise common shorts (`n` → `go north`, `take` → `get`, `l` → `look`). Unknown verbs produce polite errors; empty input is no-op.

First Rosetta Stone project with **human-oriented command parsing at the input boundary**. G073's telnet had a small fixed vocabulary; G079 has a richer one with synonyms and direction shortcuts because the user is typing from memory. Parser is forgiving (G071's pattern) — malformed input is an error message, not a crash.

**Items have kind-specific behaviour behind a uniform API.**

Items have a `kind` (Weapon / Potion / Treasure) and kind-specific fields. `use` dispatches on kind: potion heals, treasure adds gold, weapon equips. First Rosetta Stone project where **one command has polymorphic effects dispatched by item kind** — type-class / trait-dispatch pattern applied to domain objects. A single user-facing command can span multiple internal code paths based on receiver type — and the user doesn't have to know which.

**Win and lose conditions are checked every turn as invariants on state.**

After each turn: health ≤ 0 → Dead; gold ≥ win threshold → Won. Checks run **every turn**, not only after specific actions. Doesn't matter *how* health reached zero — any combination of effects that drops it ends the game.

First Rosetta Stone project where **end conditions are checked as invariants on state, not as side effects of specific actions**. Previous projects had specific operations moving to terminal states (G073 `quit`, G077 auto-lock). G079's terminal states are **derived from state after every turn** — much more robust because it handles combinations the programmer didn't explicitly code.

Analogues: OS process termination (any write to a bad page triggers SIGSEGV), DB consistency checks (any transaction leaving dangling FK is rejected), health-check probes (process goes unhealthy when it stops responding).

**The log is the game's narrative; turns are a logical clock.**

Every action appends to `world.log` — look, movement, combat, enemy retaliations, state changes, all in one append-only list. The log IS the narrative. First Rosetta Stone project where **the log is the primary output medium** — G073 had per-session history; G079's log is world-scoped and universal.

`world.turn` increments on provoking verbs only (movement, take, use, fight); observation verbs (look, stats, inv) don't provoke. First Rosetta Stone project where **a logical clock (turns) is distinct from the wall clock** — some mechanics tick on turns not time (enemy regeneration, poison-over-N-turns, hunger-at-turn-50). Production systems have this for rate limiting (per-request counters), fairness queues (round-robin counters), game-like simulations.

G069 → ... → G078 Media Player → G079 *graph-as-world + input-driven simulation clock + verb-noun parsing + kind-dispatched polymorphic commands + invariant end-conditions + log-as-narrative + logical clock distinct from wall clock*. Eleven Web projects done, five to go. Web adds **simulation world** as the twelfth component — the shape of every game, MUD, choreography executor, or agent training environment.

---

## G080 — Scheduled Auto Login and Action

**Tasks are data.**

The most important design choice in G080 is that tasks are **data, not code**. A `Task` is a struct (name, schedule enum, action string, credential ref). Serialisable to JSON, reconstructible on restart. Opposite of naive closure-based scheduling which is easy to start but painful to persist, audit, or migrate — can't inspect what's scheduled, can't pause/modify without re-writing the code that created it, can't restart the scheduler and recover.

First Rosetta Stone project where **data-driven scheduling** is explicit. Systemd timers, cron's `/etc/crontab`, Kubernetes CronJobs, Airflow DAGs, the vault's future scheduled-automations — all use this model. Task is a declarative description; executor is separate and pluggable.

**Catch-up policy is "skip missed, not backfill."**

If the scheduler goes offline for an hour and a task was due to fire 30 times, what happens when it comes back?

Two choices: **backfill** (fire 30 times in rapid succession) or **skip-missed** (fire once, reschedule to next natural boundary). G080 picks skip-missed — same as cron. Reason: a thundering herd of 30 simultaneous backups / credential refreshes / API calls is almost never what the user wants.

First Rosetta Stone project where **a visible policy choice about missed work** is the design's core decision. Backfill is correct for some semantics (event replay, idempotent work) and wrong for others (rate-limited APIs, resource-expensive backups). G080 documents the choice and sticks to it. Analogues: systemd's `Persistent=false` vs `Persistent=true`; Kubernetes CronJobs' `startingDeadlineSeconds`. Most production schedulers default to skip-missed because backfill blows up under downtime.

**Schedule is a sum type, not a cron string.**

G080 uses `Schedule` enum with `EveryMs` / `AtMs` / `Daily` variants rather than parsing cron syntax. Less expressive than cron strings but trivially typecheckable and testable. First Rosetta Stone project where **the schedule grammar is an enum, not a string DSL**. Same pattern in the noosphere: represent trigger conditions as a sum type the runtime can exhaustively match, rather than parse YAML strings.

**Disabled tasks are retained, not deleted.**

`set_enabled(id, false)` flips a flag; doesn't remove the task. Re-enabling restores firing. Lets users pause, debug, test without losing metadata. First Rosetta Stone project with **soft-disable** as a first-class state (previous projects had hard state changes: G077 lock/unlock, G078 stop/play; G080 adds present-but-dormant).

**History is per-task, queryable, immutable.**

Every run (succeeded or failed) appends to `history` with task-id, start/finish times, status, message. Diagnostic gold: why did yesterday's backup fail, is this task chronically slow, which tasks fired in the last hour.

First Rosetta Stone project where **execution history is a first-class observability primitive**. G073 had per-session history for one connection; G080 has global execution history across all tasks. The noosphere's future observability will have this exact shape.

**Dispatch function is the extension point.**

Scheduler doesn't know what tasks *do* — only when they fire and that they return (succeeded, message). Actual work happens inside the user-provided `dispatch` function. Same **abstract-transport** pattern as G072 File Downloader: tests use fakes, production plugs in real handlers.

First Rosetta Stone project with **dependency injection at the core execution boundary**. Scheduler is 200 lines of scheduling logic; dispatch function is where the real work lives, and it's a parameter. The split is what makes the scheduler reusable across tasks with wildly different semantics (HTTP polling, vault backups, agent heartbeats, credential refreshes).

G069 → ... → G079 Text Game → G080 *data-driven tasks + skip-missed catch-up + sum-type schedules + soft-disable + per-task history + pluggable dispatch*. Twelve Web projects done, four to go. Web adds **scheduled execution with history** as the thirteenth component — the shape of every automation, every cron-style background job, every scheduled agent action.

---

## G081 — E-Card Generator

**Content and presentation separate via a template.**

Every prior project that produced formatted output had the format baked into code. G069's render knew about `<h1>/<b>/<i>`; G074's `to_svg` knew about `<polyline>`. G081 is the first where **the rendering format is template-driven** — HTML surface is data (a string in the template record), swapping templates swaps output without code changes.

First Rosetta Stone project where **presentation is data, not code**. Same philosophical move as G080's "tasks are data" — declarative over imperative for anything user-visible or user-editable. Every production CMS, email system, invoice generator, document-generation pipeline works this way; the vault's note-template system uses this exact shape.

**Slots are a schema, validated at creation.**

A `Slot` declares name, kind, required, optional default. Validation at card creation: unknown slot → reject, missing required → reject, optional without value → apply default. **Stored cards are always valid** — every required slot is filled, no unknown slots, defaults materialised. Rendering is pure substitution; cannot fail on stored cards.

First Rosetta Stone project where **validation is a creation-time operation, not render-time**. G069 rendered whatever was in the model; if model was broken, render was broken. G081 won't let you store a broken card. **Fail-fast at the data-layer boundary**.

Trade-off: strict validation means sloppy/exploratory creation is harder. Production systems often add "draft" states with relaxed validation; G081 keeps it simple with only valid cards.

**Defaults are materialised, not lazy.**

Unfilled optional slot → default value is **copied into the card's filled map** at creation. Card is self-contained (render without referring back to template), template edits don't retroactively change existing cards, serialisation produces complete snapshots.

Alternative (store user values only, fall back to template defaults at render) is more compact but less stable — template edits change rendering of old cards, surprising users.

First Rosetta Stone project where **defaults are materialised at creation** rather than looked up at render. Same pattern: database NULL-with-default (materialised on insert), React default props (materialised at mount), vault note creation from template (slots filled, then frontmatter written).

**Multiple output formats from one card.**

`render` → HTML; `render_plain` → text. Both are projections. More formats (markdown, email-safe, ASCII) are mechanical to add; none require changing template or card. Pattern established in G069/G074: **model + multiple render functions = multiple output formats without duplicating content**. G081 applies it explicitly through the template mechanism.

**Unknown placeholders are visible bugs.**

`{{nonexistent_slot}}` in a template — placeholder whose slot isn't declared — is left as-is in output. Template-author's typo is **visible**, not silently hidden by empty-string substitution.

First Rosetta Stone project where **a design choice actively preserves visible evidence of bugs**. Parallels: Python's KeyError vs JavaScript's undefined; Rust's Result forcing error handling vs silently swallowing. Domain matters — G071's page scraper is forgiving (external, often-broken input); G081's renderer is strict (user-authored templates where typos should be fixed not hidden).

**Categories are an axis; slots are not.**

Templates have a `category` for user browsing. Slots belong to individual templates; one template's "age" is not the same as another's. First Rosetta Stone project where **category-instance hierarchy is explicit** — categories like folders/tags (organisational across instances), slots like fields (meaningful only within one record). Mixing them creates the "tag that's really a field" anti-pattern.

G069 → ... → G080 Scheduler → G081 *presentation-as-data + validation-at-creation + materialised-defaults + multi-format rendering + visible-bugs over silent-swallow + category-vs-slot scoping*. Thirteen Web projects done, three to go. Web adds **template-data rendering** as the fourteenth component — the shape of every CMS, every note template, every email merge, every document-generation pipeline.

---

## G082 — Content Management System

**Content has a lifecycle, not a flag.**

Every prior project with stored content used simple states (public/private, active/inactive) or none at all. G082 is the first with a **real lifecycle**: five states — Draft, InReview, Scheduled, Published, Archived — with specific transitions between specific states.

First Rosetta Stone project with **a lifecycle FSM that has more than three states**. G062 two, G073 three-with-terminal, G077 two, G078 three-non-terminal; G082's five capture the phases of editorial work (thinking, reviewing, waiting, live, retired). Every CMS in production — WordPress, Ghost, Contentful, Strapi, the noosphere's future note-publishing flow — has some version of this lifecycle. State names vary; phases don't.

Illegal transitions (publish an archived article, submit an already-published article) return a structured error. Only declared-legal transitions are accepted — **the state table IS the policy**.

**Revisions are append-only.**

Every save appends a new Revision with an incrementing version number. Edits don't overwrite history; they add. `revert_to_revision(v)` restores an old version's body **by creating a NEW revision** with that content — not by rolling back the counter.

This is the **Git model** for content: history is immutable, changes are new commits, reverts are new commits that undo previous ones. Once a revision is written, it's permanent; you can walk back through history and see exactly what the article said at any point.

First Rosetta Stone project where **history is append-only and reverts are additive**. G070 browser had forward-truncation on navigate (reverts lose the forward path); G078 media player had no history. G082 keeps everything. Cost is storage; benefit is perfect auditability.

**Scheduled publish is a deferred transition.**

`schedule_publish(id, at_ms)` puts the article in Scheduled state. Nothing else happens immediately. Later, `tick(now_ms)` transitions any Scheduled article whose time has arrived to Published. This is G080's scheduler pattern applied to state transitions — scheduler lives inside the CMS rather than as separate system because content lifecycle is the CMS's concern.

First Rosetta Stone project where **a state machine's transitions can be time-triggered in addition to user-triggered**. G077 had time-triggered auto-lock (only transition on a timer); G082 has both (submit/approve/archive = user; scheduled publish = time). Mixing is natural once the tick mechanism exists.

**Archived is frozen.**

Once archived, an article cannot be edited. Save, revert — all refused. Unarchive to Draft is allowed (reactivates editing). First Rosetta Stone project with **a non-terminal frozen state** — archived articles aren't dead (can be restored) but while archived they're read-only. Different from G073 Disconnected (terminal) or G077 Locked (inaccessible). Archived is **visible-but-frozen**.

DB parallel: soft-deleted rows with `deleted_at` are visible to admins, editable by no one. Wiki parallel: protected pages are readable but not editable without permission.

**Slug is the natural key; ID is the surrogate.**

Articles have both a numeric id (surrogate) and a slug (natural). Uniqueness enforced on slug. Same pattern as G076 Bookmarks. **Dual keying** because id provides stable references for code (revisions, internal mentions) while slug provides human-friendly identifiers (URLs, navigation).

Production CMSes uniformly do this. Changing a slug is a renaming operation requiring URL rewrites or 301 redirects; the id never changes.

G069 → ... → G081 E-Card → G082 *multi-state lifecycle FSM + append-only revisions + time-triggered transitions + archived-as-frozen + slug-as-natural-key + dimension-scoped queries*. Fourteen Web projects done, two to go. Web adds **content lifecycle with revision history** as the fifteenth component — the shape of every CMS, every versioned document system, every note-publishing flow.

---

## G083 — Template Maker

**The code that produces templates is itself a data model.**

G081 treated a Template as data: slots, HTML, category. The template was produced once (hand-written struct) and consumed many times. G083 makes the **production step** itself data-driven: `TemplateBuilder` is an editable record holding the in-progress definition; `TemplateMaker` is the tool that mutates it, validates it, produces the frozen Template.

First Rosetta Stone **meta-level** project — the tool that creates tools. Same philosophical shift as G080's "tasks are data" and G081's "templates are data" — but at a higher level. Not "store the output as data"; "store the process as data." Every good authoring tool has this shape: Figma stores design files, Excel stores spreadsheets, the vault's note-template system stores templates (all data, not code).

**Cross-reference validation between two representations.**

Builder has two representations of "what slots exist":
1. **Slots list** — declarative (name, kind, required, default).
2. **HTML placeholders** — operational (every `{{name}}` in the draft).

These must agree. Validator finds mismatches: **UnusedSlot** (declared but not referenced — clutters UI, indicates editing without updating slot list) and **UndeclaredPlaceholder** (referenced but not declared — produces raw `{{typo}}` in output).

First Rosetta Stone project where **two representations of the same concept are validated against each other**. G082's lifecycle states and scheduled publishes were different concepts. G083 has one concept (slots) with two representations (declared + referenced) and validation detects drift.

Every schema-migration tool does this (DB schema vs ORM model must agree). Every protobuf-to-code generator does this (.proto vs generated code). G083 is a tiny version of the same pattern.

**Preview uses sample data, not real data.**

Template author has no real cards yet. `preview` uses per-slot sample values the author provides; missing samples fall back to defaults; still missing shows the raw placeholder.

First Rosetta Stone project with **design-time sample data** separate from runtime data. G081 only had real card values; G083 introduces samples on the builder itself, so the author can preview without creating cards. The vault's note-template system does exactly this.

**Issue-kind-typed validation results.**

Four bug families: Structural (invalid names, duplicates), Completeness (unused, undeclared), Runtime-preview (missing sample for required). Each is a different kind of problem with different authoring mistakes and different fixes. `validate()` returns an itemised list with kind+name, letting the UI render specifics per kind.

First Rosetta Stone project with **structured validation results** rather than single-error returns. Production linters (ESLint, Clippy, pylint) work this way — validate everything, return a list, let the caller render. G083 at minimum scale.

**Reserved names as schema extensions.**

`{{_recipient}}` and `{{_sender}}` can be referenced without declaration. Validator skips `_`-prefixed names (they're system-provided). First Rosetta Stone project with **a naming convention as a schema extension mechanism** — same pattern as Python's `__dunder__`, Ruby's `@@class_vars`, HTML's `data-*`. The noosphere's choreography language can reserve `@_ctx`, `@_out`, `@_err` by the same convention.

G069 → ... → G082 CMS → G083 *meta-level tooling + cross-reference validation + design-time samples + issue-kind typing + reserved-name conventions*. Fifteen Web projects done, one to go. Web adds **authoring tools** as the sixteenth component — the shape of every schema editor, template builder, form designer, or content-creation IDE.

---

## G084 — CAPTCHA Maker

**Challenges are ephemeral, single-use, time-bounded.**

G077 had bounded time + bounded attempts for unlock; G082 had scheduled publishing (time bounds on state). G084 narrows: **the challenge IS an ephemeral authentication token**. Once solved, consumed. Once expired, refused. Once attempts exhausted, refused. Only one terminal outcome is success.

First Rosetta Stone project where **the primitive explicitly encodes single-use semantics** — G077's unlock happens many times; G082's publish happens once but the article persists; G084's challenge is like a password-reset token or OTP code: fire once, never again. The shape of every cryptographic nonce, every OAuth authorization code, every email-confirmation link.

**Four kinds of outcome are structurally distinct.**

VerifyResult has six outcomes: Success, WrongAnswer (+remaining), Expired, AlreadySolved, TooManyAttempts, Unknown. Each is a different kind of refusal requiring different UI response (retry makes sense for "wrong" but not "expired"; "already solved" is idempotent-click, not error). Collapsing into boolean accepted/rejected loses information.

First Rosetta Stone project where **the refusal outcome is itself structured data** — G081 had validation errors as enum variants; G084 extends to the per-request verification path. Production APIs converge on this pattern (OAuth returns invalid_grant/invalid_client/access_denied/expired_token).

**Attempt counter is a rate limit.**

`max_attempts` + `attempts_used` is a rate limit at the token level. Infinite-attempts is a brute-force vuln. 3-attempt limit means attacker has 3/(space_size) chance per token. Same pattern as login lockouts, API 429s, password-safe auto-lock (G077, time-based rather than count-based).

First Rosetta Stone project with **dual-bound refusal** — either expiry OR attempt-count triggers refusal independently. How most security tokens work (OAuth: expire by time AND revocable by use-count or explicit revocation).

**Challenge kinds are polymorphic through one generator.**

`issue(kind)` dispatches to kind-specific generator producing (prompt, expected) pair. Four kinds — Math, WordRecall, ReverseString, SequenceCount — produce different challenge types through one API.

First Rosetta Stone project where **challenge generation is dispatched at the construction boundary** based on kind. Matters for accessibility — math excludes non-native speakers, word recall excludes visually impaired, reverse-string excludes dyslexic users; offering multiple kinds lets frontends pick based on accessibility preferences.

**Answer matching is lenient.**

Trim whitespace, lowercase both sides. Typing a CAPTCHA is already user-hostile; nit-picking case/whitespace is worse for no security gain. First Rosetta Stone project where **input normalisation is part of the correctness contract**. G071 forgave malformed HTML; G080 forgave missed schedule windows; G084 forgives trivial input variations. Forgiveness is always **scoped to the specific noise the domain produces**.

**Cleanup is explicit garbage collection.**

`cleanup(now_ms)` removes solved + expired. Without it, the challenge list grows. Calling cleanup periodically (cron-style, using G080) keeps memory bounded. Alternative: auto-cleanup on every verify/issue — simpler but makes hot paths pay scan cost.

First Rosetta Stone project where **garbage collection is explicit and caller-scheduled**. G064/G076 retained indefinitely; G075 auto-evicted (ring buffer). G084 adds the third model: retain until explicit sweep.

**Closes the Web category — sixteen components.**

The Web category's architecture, complete and composable:

1. Structured model (G069)
2. Render function (G069, G074, G081)
3. Query language (G071, G076, G082)
4. Robust input handling (G071, G084)
5. Durable state (G072)
6. Interactive protocol (G073)
7. Spatial data (G074)
8. Time-series observation (G075)
9. Multi-axis organisation (G076)
10. Security primitives (G077)
11. Tick-driven engines (G078, G082, G084)
12. Simulation world (G079)
13. Scheduled execution with history (G080)
14. Template-data rendering (G081)
15. Content lifecycle with revision history (G082)
16. Authoring tools (G083)

G084 is the synthesis: combines time bounds (10), outcome-typed results (3, 16), polymorphic dispatch (via kind), attempt-counter refusal (10), and garbage collection. It doesn't add a new component — it uses every existing one.

**The Web category is done. 16 projects, 16 components.** The noosphere has its full Web-layer vocabulary. Remaining in the milestone: Files (G085–G100), Databases (G101–G113), Graphics (G114–G130) — 46 more projects, each building on these 16 components plus their own domain-specific additions.

G069 → ... → G083 Template Maker → G084 *ephemeral single-use tokens + outcome-typed verification results + dual-bound refusal + polymorphic challenge generation + lenient input normalisation + explicit GC*. **Sixteen Web projects done. Web category closed at 84/130.**

---

## G085 — Quiz Maker (Opens Files)

**Round-trip equivalence is the Files category's contract.**

Every Files project centres on one promise: **what you write, you can read back, unchanged**. Serialise a quiz to text, re-parse that text, get the original quiz. The defining contract of every file format in existence — JSON, CSV, PNG, PDF, Git objects, SQLite.

First Rosetta Stone project where **serialisation and deserialisation are co-defined** — you can't design one without the other, because they must compose to the identity function on valid inputs.

**Line-based text format is the hobbyist default.**

G085 doesn't use JSON or YAML. Invents a simple line-based format (`key: value`, `---` separators). Reasons: no dependency (every language reads lines), human-readable (edit in nano), diffable (each line independent), extensible (add a `key: value`). Cost: no nested structures, no quoting rules. Adequate for a small domain, far cheaper than a JSON parser.

First Rosetta Stone project where **the file format is an explicit design decision** rather than "whatever JSON library is lying around." The vault's note format (Markdown + YAML frontmatter) is similar.

**Polymorphism through tagged structures; scoring is a fold.**

Three question kinds (MC/TF/SA). Each language models tagged unions at its idiomatic level — Rust/Lean enum variants; Python/Go tagged structs; CL keyword-dispatched. The contract is the same: `check_answer(question, answer)` dispatches on kind. Scoring is a **fold over (question, answer) pairs** — kind-agnostic at the outer level, kind-specific inside check_answer.

First Rosetta Stone project where **the parse error is a structured type** the caller can destructure — enum of specific failure modes (BadHeader, MissingField, InvalidIndex, UnknownKind). G071 forgave; G083 returned a list of issues; G085 has a single error path returning structured diagnostics.

G084 closed Web → G085 opens **Files**. The category's theme (round-trip equivalence) will recur in every project — different data shapes, different file formats, same contract. 85/130 complete. 1 of 16 Files done.

---

## G086 — Quick Launcher

**Frecency is one number that replaces two.**

Every launcher answers the same question: "when the user types 'fi', show Firefox (500 launches last week) or Files (one launch yesterday)?" Pure frequency says Firefox. Pure recency says Files. Neither alone is right.

Frecency collapses both into a scalar: `log(1 + count) / (1 + age_hours)`. The log tames explosive frequency so a 1000x-launched entry doesn't crush everything. The age denominator decays recency. No weights to tune, no two-number comparison — one number orders them all.

First Rosetta Stone project where **the ranking function is a composition of two scalar sub-scores** rather than a lexicographic tuple comparison. Every vault recommender (recent notes, related articles, completion suggestions) will take this shape.

**Text match dominates; frecency breaks ties.**

Naive launchers rank by pure frecency — then typing "fire" ranks Firefox last if the user has never launched it. Wrong. The user stated intent by typing. Frecency cannot overrule intent.

So text match is the primary score (0..1); frecency is a small bonus (weight 0.1). A perfect name match (1.0) beats any frecency. A prefix tier (0.8) beats a substring tier (0.6) even with boosted frecency. Frecency only reshuffles entries that matched the query equally well.

First Rosetta Stone project with **two-layer ranking** explicit in its structure: layer 1 determines visibility (zero text match → filtered out), layer 2 orders the survivors. Every vault search will follow this: the query decides what's relevant, usage stats decide what's prioritised among relevant results.

**Usage state is a round-trip file too.**

G085 proved that a static quiz serialises round-trip. G086 extends the contract to **mutable accumulated state** — launch_count and last_launched_ms are persisted per entry, reloaded next session, continue accumulating. A launcher that forgets usage between sessions has to re-learn the user daily. One with persistence learns the user once.

First Rosetta Stone project where **the persistent representation carries accumulated state**, not just static configuration. The vault's note-metadata sidecars (last-viewed timestamps, access counts) will use this exact pattern.

**Empty query is a defined case, not an error.**

When the user hasn't typed yet, the launcher should surface everything — sorted by frecency, putting recent/frequent at top. Zero results on empty input is broken UX.

So `text_match("") == 0.5` for every entry: every entry passes the filter, and only frecency orders them. The moment the user types one character, text match reasserts. This is the only state where frecency solely determines rank.

First Rosetta Stone project where **the ranking function has a documented degenerate case**. No special code path — the same scoring function handles it via a known constant. Every vault search box will follow this convention.

G085 (round-trip equivalence of static data) → G086 *frecency + two-layer ranking + round-trip of mutable usage state + documented empty-query behaviour + structured launch errors*. Files 2/16. 86/130 complete.

---

## G087 — File Explorer

**Path resolution is the filesystem's type checker.**

Every file operation starts with a string the user typed. Turning that string into a location the filesystem understands is the single most important job a file manager does. Get it wrong and the user ends up somewhere unexpected, or — worst case — escapes the root.

`resolve(path)` turns user input into a canonical segment list. Absolute paths start from root, relative from cwd. `.` and empty segments drop, `..` pops one, `..` at root is a no-op. Output is type-correct: a list of bare names with no metacharacters. Every subsequent operation (`cd`, `ls`, `stat`) takes the canonical list, not the raw string.

First Rosetta Stone project where **parsing and semantic analysis are separate phases**. G085 and G086 parsed text into structures in one step; G087 separates `resolve(path) → segments` from `lookup(segments) → node`. That split is exactly what makes root escape impossible — the normaliser catches `..` before lookup runs.

**Virtual filesystem is the right test surface — and the right production abstraction.**

Real disk I/O is slow, nondeterministic, and host-dependent. The Rosetta Stone needs byte-identical behaviour across six languages. So the explorer operates on an in-memory tree, no syscalls.

But the virtual filesystem isn't just for tests — it's the production abstraction. The explorer doesn't know whether its data lives in RAM, on disk, in Postgres, or in vault markdown. Later projects plug in a backend; G087's logic is storage-agnostic.

First Rosetta Stone project where **the data model is an internal tree, not a reflection of the OS**. The vault's file views will all layer over this — what the user sees as a "folder" might be a SQL query, a glob pattern, or a real directory.

**Dirs first, alphabetical second.**

Every file manager ever shipped follows this ordering. Naive alphabetical mixes folders and files and makes listings hard to scan. Dirs-first partitions by kind, then orders within each group.

Two-stage comparator: partition by kind (dir < file), then alphabetical within kind. One pass, O(n log n), stable. Every test across six languages sees the same order.

First Rosetta Stone project with **category-before-alphabetical sort**. G076 used axis-based sorting; G087 uses kind-based grouping. Partition first, order within partitions.

**Recursive size is a fold.**

`total_size(path)` on a directory sums every descendant file's size. For a dir, fold over children; for a file, return size. Two-line recursion, arbitrary tree depth.

First Rosetta Stone project that **recurses over a non-linear structure** via dispatching on node kind. G064 transitive-closure did DAG work; G087 is a pure tree. The pattern — self-recursive function that dispatches on kind — reappears in G089, G097, and throughout Graphics.

G086 (usage state in a file) → G087 *path resolution as type-check + virtual filesystem abstraction + dirs-first listing + recursive fold over tree*. Files 3/16. 87/130 complete.

---

## G088 — Sort File Records Utility

**A file can be a table.**

G085's quiz file was structured blocks; G086's launcher state was entry records with heterogeneous data. G088 is the first project where the file is a **uniform tabular dataset** — every row has the same fields, interpreted the same way. The contract CSV has served for fifty years.

No schema is declared — the parser infers types per field value. Cheap and fragile. Cheap because any language reads delimited lines without a dependency; fragile because embedded commas or quoted strings break it. G088 accepts the fragility: the point is the *abstraction*, not production-ready CSV handling.

First Rosetta Stone project where **each row has a uniform type** rather than a polymorphic kind. G085 had three question kinds dispatched by tag; G088 rows are identical structures. That shift — polymorphic items to uniform rows — is what enables `sort_by(spec)` to work over every row with the same logic.

**Sort spec composes primitives.**

Single-key sort is obvious. Multi-key sort is useful: "age asc, score desc, name asc as final tiebreak". The spec is a list of `(field, order, kind)` tuples; the comparator walks the spec and returns the first non-zero comparison.

This is exactly SQL's `ORDER BY` clause. Composition works because comparison returns -1/0/1 (a total order), composition of total orders is total, and stable sort preserves insertion order on ties so unspecified axes don't reshuffle.

First Rosetta Stone project where **a data structure describes a sort operation**. Previous projects had implicit sort keys (G076's axis constants). G088 makes the sort explicit and configurable — a spec is data, which means it can be stored, passed, serialised.

**Stable sort is the contract, not a convenience.**

A stable sort preserves the relative order of records with equal keys. For a file utility, stability is the difference between "run it twice, get the same file" and "run it twice, get different files". Version control, diffs, audit logs depend on deterministic output.

**Stable sort is correctness**, not an optimisation choice. Each language uses its stable-sort primitive — Rust's `sort_by` is stable, Python's `sort` is stable, Go's `SliceStable`, CL's `stable-sort`, Lean's `mergeSort`. Six idioms, one contract.

First Rosetta Stone project where **determinism of output** is an explicit invariant, not an accidental property.

**Missing fields sort first.**

Real datasets have holes. A record with only two fields when the spec asks for field 3 must still sort — and must not crash. Convention: missing fields sort first in ascending order (last in descending). Equivalent to SQL's `ORDER BY col ASC NULLS FIRST`.

Minor but load-bearing. Partial records cluster so the UI can style them; the invariant "sort is total" never breaks.

First Rosetta Stone project where **a default policy for missing data** is part of the contract. G085 returned ParseError; G086 returned LaunchError. G088's choice is different — missing is valid, just orders specially. Sometimes the right answer is to define the behaviour, not error out.

G087 (path resolution + virtual FS) → G088 *uniform tabular records + multi-key sort spec + stable-sort contract + missing-field policy*. Files 4/16. 88/130 complete.

---

## G089 — Transaction Averages

**Group-by is a single-pass fold.**

Naïve aggregation does three passes: extract keys, group rows, aggregate each group. The one-pass version is a single fold: for each record, look up (or create) the group accumulator, observe the value, move on. Finalise each accumulator at the end.

The accumulator has two roles — **running state** (count, sum, min, max updated incrementally) and **finalised output** (mean derived at end). Splitting them is what makes the aggregation single-pass. min/max/sum are monoids, combinable pairwise; mean is not a monoid, but is computable from sum and count at the end.

First Rosetta Stone project where **observation (`observe`) and finalisation (`finalise`) are explicit separate methods** on the same accumulator. G008 summed with a fold; G065 accumulated progress with atomics. G089 formalises the pattern: update-step + finalise-step.

**Output must be sorted, not map-ordered.**

Hash map iteration order is unspecified in most languages (Python is the exception). Tests that depend on it become flaky; diffs show spurious reshuffles. Every `group_by` in G089 **sorts group keys** before emission, at the final step after observation is complete.

Not automatic — a choice. Returning an unsorted hash would be faster; insertion-ordered would be cheaper. But same-input-same-output beats microseconds. Stability is correctness, just like G088's sort.

First Rosetta Stone project where **a deterministic wrapper around a non-deterministic data structure** is the explicit design. Internally fast, externally always sorted — the vault's query layer will use this pattern.

**Integer cents throughout kills float error.**

Financial floats are a classic bug. `0.1 + 0.2 != 0.3`. Summing millions of transactions, each off by fractional cents, produces a visible error. Every transaction processor since the 90s uses integer cents.

G089 uses integer types throughout. Sum is exact. Min/max are exact. Only `mean = sum / count` produces a float, at the reporting step. Downstream code needing exact means computes `(sum, count)` tuples instead.

First Rosetta Stone project with **explicit fixed-point arithmetic for correctness**. G003/G024 converted number bases; G013 approximated pi. G089 is the first where representation matters for *accounting* correctness, not just precision.

**Key function, not key field.**

`group_by` takes a **key function** — a closure that extracts the group key from a record. "Group by category" and "group by month" (YYYY-MM prefix) are the same operation with different extractors.

This is what every functional `groupBy` does, what Python's `itertools.groupby` codifies, what SQL's `GROUP BY expression` matches. Aggregator is fixed; extractor is pluggable. Multi-axis grouping, derived keys, custom bucketing — all expressible as different key functions.

First Rosetta Stone project that **parameterises over a function, not a value**. G008's fold took a combiner; G089's `group_by` takes an extractor. Take-a-function is now table stakes for the library.

G088 (sort spec + stable sort) → G089 *single-pass accumulator + sorted output + integer cents + key-function parameterisation*. Files 5/16. 89/130 complete.

---

## G090 — Zip File Maker

**Byte-level round-trip is the compression contract.**

G085 proved textual round-trip (quiz → text → quiz). G090 tightens the contract: **every byte** of input must be recoverable from the compressed output. No lossiness, no approximation. A single lost bit corrupts the file.

Stricter than textual round-trip, because binary data includes bytes that mean different things in different encodings. The compressor is indifferent to content — it sees bytes, not characters. Decoder reverses exactly.

First Rosetta Stone project where **the data is bytes, not text**. G085's parser could normalise whitespace; G090's RLE cannot normalise anything, because every byte is semantically load-bearing.

**RLE is the minimal interesting compression.**

Five lines to decode. Visibly reduces repetitive data (`xxxxxxx` → `7 x`). Visibly fails on random data (every byte becomes two bytes).

That third property is the interesting one. RLE is terrible for general-purpose compression — it only helps when input has actual runs. `add_file` runs RLE and compares against raw size, picking whichever is smaller. **The compressor must know when not to compress.**

First Rosetta Stone project where **the algorithm makes a choice based on its own output size**. G053 fizzbuzz, G071 HTML parse had no such choice. `add_file` explicitly branches on "did compression help?". This meta-level — algorithm evaluating itself — recurs in every adaptive system.

**Per-entry method is the archive pattern.**

Real archive formats (ZIP, 7z, tar.gz) don't choose one method for the whole archive. Each entry picks the best method — or no method. A JPEG inside a ZIP is `STORED` because recompression achieves nothing; a text file inside the same ZIP is `DEFLATED`.

G090 adopts this with two methods. Archive header declares per-entry method; decoder dispatches on the tag. Adding LZ77, Huffman, Deflate later is: add the variant, implement encode/decode, update `add_file` to consider it. Archive **structure** doesn't change.

First Rosetta Stone project where **the archive is an extensible dispatch table**. PNG chunks, PDF objects, OCI image layers all use this.

**Archive = manifest + payload.**

Two conceptual parts: **manifest** (what's in it, how stored, size) and **payload** (compressed bytes). `compressed_size` and `compression_ratio` read only the manifest. `extract` reads manifest to find the method, then payload to decode.

This separation is load-bearing. Lightweight operations (list contents, check existence) don't touch payload; payload representation (in-memory bytes, file handles, object-store URLs) can change without breaking manifest API.

First Rosetta Stone project with explicit **manifest-vs-payload split**. Every indexed storage system (databases, S3, Git object store) layers this over raw bytes.

G089 (group-by aggregation) → G090 *byte-level round-trip + self-evaluating compression choice + per-entry method dispatch + manifest/payload split*. Files 6/16. 90/130 complete.

---

## G091 — PDF Generator

**Pagination is flow layout, nothing more.**

The kernel of PDF generation is trivial: start at y=0 on fresh page; for each block, compute height; if it fits, place it at y and advance; if not, emit page and start fresh. Headings may force an early break to avoid orphans.

The fancy parts of PDF (fonts, glyphs, colour spaces, embedded images, annotations, signatures) layer over this kernel. LaTeX is this algorithm with better heuristics. ReportLab is this algorithm with a richer block vocabulary. iText is this algorithm with full binary PDF output.

First Rosetta Stone project where **the domain's apparent complexity reduces to a simple loop**. G070's tab model looked complex, G077's encryption looked complex, G082's CMS looked complex — all reduced to small invariants. G091's pagination reduces to "fit or overflow". Domain complexity often hides a simple kernel.

**Word wrap is the paragraph's own layout.**

A paragraph becomes some number of lines. Start with first word; each subsequent word appends if it fits, otherwise starts a new line. Long unbreakable words overflow the line rather than truncate — a 30-char word on a 10-char line is bad typography but preserves information; truncation silently loses content.

First Rosetta Stone project where **the layout of a leaf block is itself non-trivial**. G083 had substitution but no layout; G081 had rendering but no wrap. G091 delegates `wrap_text(text, width)` → list of lines, then pagination treats each line as unit height. Content-to-lines vs lines-to-pages is a clean separation.

**Rendered document is renderer-independent.**

G091 doesn't emit PDF bytes. It emits **a list of pages**, each page with **a list of placed lines**. That's the input any renderer needs — a PDF renderer turns it into content-stream operators; an HTML renderer turns it into positioned divs; a terminal renderer prints with position markers.

Every publishing pipeline's killer feature. LaTeX's `.aux`, HTML's layout tree, PDF's content stream — all intermediate between source and final bytes. Having them as **data** rather than an output stream means you can serialise, test deterministically, ship to another process, render to multiple targets from one source.

First Rosetta Stone project with an explicit **IR (intermediate representation)**. G082's CMS had revisions as a content-history IR; G091 has pages as a visual-layout IR. Source → IR → output is every modern compiler and renderer.

**Heading orphan avoidance is built-in.**

A heading at the bottom of a page with no body below is an **orphan** — ugly typography. G091 avoids it naively: if a heading needs 2 lines (title + blank below) and only 1 is left, start a new page before placing.

First Rosetta Stone project where **visual aesthetics constrain layout logic**. Everything prior had pure-correctness targets; G091's correctness is partly aesthetic, and aesthetics are rules the algorithm must encode.

G090 (compression round-trip) → G091 *flow-layout kernel + word-wrap + rendered-document IR + orphan avoidance as aesthetic-as-code*. Files 7/16. 91/130 complete.

---

## G092 — Bulk Renamer and Organizer

**Preview is the non-destructive surface.**

Every bulk operation has a destructive phase. Once 100 files are renamed, reversing is expensive or impossible. Preview is read-only: same inputs (directory, rule), same output (list of ops), without touching anything. GUI shows preview; user confirms; apply runs.

Not cosmetic. Preview must produce **exactly** what apply will do — same ops, same order, same error conditions. If preview misses a collision apply hits, user trust is destroyed. Preview and apply share logic; apply is "run preview, then mutate".

First Rosetta Stone project where **the same logic executes twice** — once to show, once to do. G073's telnet had commands-with-effects; G084's captcha had single-action verification. G092 explicitly separates "show me what would happen" from "do it", making that separation API-level.

**Undo log = reverse ops.**

Apply returns an undo log — a list of ops that, if applied, restore original state. Structurally identical to input ops with `from`/`to` swapped. To undo: apply the log.

Minimal. Richer undo (history, redo) layers on top. But the primitive covers every real "I meant to rename `.txt` → `.md` but matched my `.tex` files too" scenario.

First Rosetta Stone project where **mutation returns its own inverse**. G077's safe had state changes but no inverse; G082's CMS had revisions-as-history but no undo primitive. G092 makes undo trivial — do, keep the log, apply the log.

**Collision detection happens before any mutation.**

Two rules collapsing distinct filenames into the same target is catastrophic. `{foo, bar}` both → `baz` means one gets silently overwritten in most real filesystems. Preview detects this before any mutation and returns `Collision`.

A second conflict: rename whose target is an existing untouched file. `a.txt → b.txt` when `b.txt` exists and isn't being renamed is also a collision. Preview catches with `ToExists`.

Combined: **preview is a total validator**. If it returns ops, apply is guaranteed to succeed. If it errors, the user gets a specific reason.

First Rosetta Stone project where **validation is a separate data path from execution**. G085's parse errors happened during consumption; G083's template validator was standalone. G092 runs validation in preview, and apply trusts preview's output.

**Atomic batch rename handles circular swaps.**

A batch `{a → b, b → a}` fails if done naively op-by-op: `a → b` leaves both pointing at the same thing; `b → a` fails because `a` was removed.

The atomic pattern:
1. Remove all `from` names.
2. Install all `to` names.

Removal first for all ops means circular swaps work. Same pattern SQL uses for swapping two tables.

First Rosetta Stone project where **batch atomicity is explicit**. G068's thumbnails had per-item atomicity; G077 had transactional locking. G092 applies atomicity to batch mutation — batch succeeds entirely or fails entirely; circular deps inside the batch just work.

G091 (flow pagination) → G092 *preview-as-total-validator + apply-as-mutation + undo log from reverse ops + atomic batch for circular swaps*. Files 8/16 — half of Files done. 92/130 complete.

---

## G093 — Mp3 Tagger

**Metadata lives separately from data.**

The tag store doesn't own the files. It owns path → tag. Files exist on disk without tags; tags exist for deleted paths. Intentional — metadata is an independent concern.

This is Git (objects vs refs vs commit messages), iTunes (library database vs bytes), every package manager (manifest vs binary). **Metadata is first-class, not a property of data.**

First Rosetta Stone project where **data and metadata are two layers with independent lifecycles**. G086's launcher had metadata alongside entries; G087 had mtime on nodes. G093 decouples: `TagStore` operates on paths as strings, no requirement they resolve.

**Indexes trade space for query speed.**

10k tagged files with naïve "filter by artist" = 10k compares. An artist index (`HashMap<Artist, Vec<Path>>`) is O(1). Cost: a copy of each (artist, path) edge.

Not every field justifies an index. Artist/album/year are equality-queried — indexes earn their keep. Title is substring-queried — index doesn't help; trigram/suffix-array would but is overkill. Per-field decision.

First Rosetta Stone project where **some operations are fast and some are slow by explicit design**. G087 avoided indexes; G085 optimised uniformly. G093 makes the trade-off visible — "by artist is fast; title substring scans".

**Missing is not empty.**

A field can be `None` (unset) or `Some("")` (explicitly blank). Different: first is "unknown, please fill"; second is "I set it to blank". UI that treats them the same hides information.

Rust `Option<String>`, Python `str | None`, Go `*string`, CL `(or null string)`, Lean `Option String`. Every language models this. `MissingField` query returns paths where field is `None`.

First Rosetta Stone project where **nullability is a first-class data contract**. Previous projects used sentinels (`""`, `-1`) or errors. G093 makes "not set" a valid, queryable state.

**Query is a data structure, not a function.**

One `query(q)` method where `q` is an enum. Every query shape is a variant. This means:
1. Queries are data — storable, loggable, shippable, serialisable.
2. New query types add variants, not methods.
3. Compound queries become natural — future `And`/`Or` can reference other `Query` values recursively (filter AST).

First Rosetta Stone project where **the query is reified**. G088's sort spec was data; G093 extends the same move to filtering — action is a value, not a method call.

G092 (preview/apply/undo) → G093 *metadata as independent layer + selective indexing + missing/empty distinction + reified query AST*. Files 9/16. 93/130 complete.

---

## G094 — Log File Maker

**Append-only is the simplest write discipline.**

Logger never updates an existing entry. `log()` pushes onto end of active; rotation pops from front. No `edit_entry`, no `delete_entry`. The restriction is the primary property of a log: **once written, never changed**.

Append-only is why logs are auditable. A system that rewrites logs can lie about its past. Linear history: every entry is present (and happened) or rotated (and happened then too). Same discipline as Git commits, Kafka streams, database WALs, blockchain ledgers.

First Rosetta Stone project where the **primary mutation is append-only by design**. G082 had revisions (mutable content with immutable history); G086 updated counters in place. G094 makes the mutation discipline explicit — writes go to end, reads scan, rotation moves to archive never back.

**Level filter at write time, not read time.**

`log()` drops entries below threshold at write time — they never enter the active buffer. Naïve would keep everything and filter at read; wastes memory and exposes debug output to production consumers who shouldn't see it.

Write-time filtering is what `log`, Python's `logging`, Go's `slog`, log4j all do. **Threshold is policy** — operator sets `INFO` in production, `DEBUG` in dev, application forgets about it. No per-call `if log_enabled` boilerplate.

First Rosetta Stone project where **policy decisions happen at capture, not consumption**. G085 was permissive at capture (accept everything); G094 takes the opposite stance for logs.

**Rotation is a ring buffer with an archive.**

Unbounded logs fill disks. Naive "delete when big" loses data. Rotation is the middle ground: active holds recent N entries; overflow moves oldest to archive.

Rosetta Stone G094 keeps archive in memory for testability. Real systems (log4j, journald, logrotate) write to numbered files or compress. Same data structure: fresh ring + growing append-only archive. Queries run against active for speed; compliance audits scan archive.

First Rosetta Stone project with **two-tier storage**: hot (active, small, fast) and cold (archive, growing, scanned rarely). G090 had manifest vs payload but both hot. G094 is the first where recency dictates tier.

**Tab-delimited is the bash-native format.**

Not JSON, not binary, not custom. Tabs separate fields, newlines separate rows. Why: `grep` filters; `awk -F'\t'` projects columns; `sort` just works; `cut -f2` extracts a column. Unix pipe-ready.

Modern systems add JSON for structured logging. But tab-delimited is still every ops person's first tool. G094 picks the format that composes with the universal toolbox.

First Rosetta Stone project where **on-disk format is chosen for tool interop**, not parser convenience. G085's quiz was for human edits; G094's tabs are for `grep | awk | sort`.

**Query results preserve insertion order.**

Every query returns entries in the order logged. Chronology is a contract — users expect time to march forward. A hash-based query could return random order effortlessly; G094 iterates active and filters.

Cheaper than G093's tag store (which sorts). Append-only + no reordering = O(n) queries with zero sort cost.

First Rosetta Stone project where **data-structure ordering is output ordering**. G093 paid for a sort; G094 gets order for free.

G093 (metadata/query layer) → G094 *append-only writes + write-time policy + two-tier rotation + tab-delimited tool-native format + ordering for free*. Files 10/16. 94/130 complete.

---

## G095 — Excel Spreadsheet Exporter

**The cell address is the primary key.**

Everything keyed by `(row, col)`. Not position in a list, not name, not pointer — coordinates. The address is a compound primary key; every operation uses one (get) or many (range).

This is what makes spreadsheets **forgiving to sparse data**. Most cells empty; sheet stores only non-empty in a hash by `(row, col)`. 1000 filled cells across a million addresses costs 1000 entries.

First Rosetta Stone project where **the primary structure is hash-keyed by a compound address**. G093 used single-string paths. G095's `(Nat, Nat)` tuples enable range queries — `A1:A5` is "all (r, 0) for r in 0..=4", expressible only if the address decomposes cleanly.

**Lazy evaluation is what makes formulae formulae.**

If `=A1+B1` were computed at write, changing `A1` wouldn't update `C1`. So every spreadsheet ever built **stores the formula, not the result**, and re-evaluates on read.

Bigger architectural choice than it looks:
1. State is minimal — only leaves stored; derived computed.
2. Updates are cheap — change one cell, downstream updates automatically next read.
3. Dependencies are implicit — formula text mentions `A1`, reading `C1` consults `A1`; no dependency graph built.

First Rosetta Stone project where **computation happens at query time, not write time**. G089's aggregation was eager (compute when asked, once); G094's filter was eager (at write). G095 evaluates every cell read. Every reactive framework (React, Vue, signals) generalises this move.

**Cycle detection is a visit-set.**

`A1 = B1 + 1`, `B1 = A1 + 1` would loop forever. Standard trick: track cells currently being evaluated; re-entering means cycle.

Each eval pushes its cell onto a set before recursing; pops on return. Set is **per evaluation tree**, not per sheet — two independent `evaluate()` calls each start fresh.

Same algorithm as topological sort, GC mark phases, Prolog occurs-check. G095 makes it load-bearing.

First Rosetta Stone project where **a visit-set prevents infinite recursion**. G087's filesystem had no cycles; G079's room graph could loop but was simple. G095's formulae are cyclic dependency graphs waiting to happen.

**Range expansion is address arithmetic.**

`SUM(A1:A5)` expands into five evaluations. Range parser reads two addresses, computes all `(r, c)` in the bounding box, evaluates each. The key move: **ranges are notation, not a new primitive** — they unroll into the existing cell-evaluation path.

Same way SQL's `IN (1,2,3)` unrolls into three equals; how APL and NumPy treat `a[0:5]` as five slot operations.

First Rosetta Stone project where **notation sugar expands into a uniform operation**. Ranges add no primitives; adding new function names costs nothing — the expansion logic is already there.

**CSV is the universal interop format.**

Excel's native XLSX is a zip of XML + fonts + styles + pivot tables + macros. Multi-year project. CSV is comma-separated values: every program on earth reads it, none of the above features exist.

G095 exports CSV because the Rosetta Stone's job is the **calculation model**, not the presentation format. An XLSX writer layers over a CSV-capable sheet; not the other way around. Quoting rules: fields containing `,`, `"`, or `\n` get wrapped in double quotes with internal `"` doubled. Five lines per language.

First Rosetta Stone project where **export format is chosen for interop breadth over fidelity**. G090's archive carried full payload; G095's CSV is a lossy projection (no formulae, no formatting) because lossiness buys universal readability.

G094 (append-only logs) → G095 *compound-key sparse grid + lazy read-time evaluation + visit-set cycle detection + range-as-notation-sugar + CSV for universal interop*. Files 11/16. 95/130 complete.

---

## G096 — RPG Character Stat Creator

**Class is a template, not a subclass.**

A class (Warrior, Mage, Rogue) is not a separate code path. It's a **multiplier table**: `{Strength → 1.5, Stamina → 1.3}` for Warrior, different for Mage/Rogue. Every character runs through the same `adjusted(stat) = base * multiplier[stat]` — class is which table is active.

The **data-over-dispatch** pattern. OOP would tempt us to make `Warrior` a subclass with `getStrength()` overrides. But adding a new class then means adding code, not data. Rosetta Stone: classes are rows; adding a Bard is adding a row.

First Rosetta Stone project where **class-like behaviour is configuration, not inheritance**. G085 had polymorphic questions via kind tag; G096 applies the same — one function dispatched on config data.

**Derived stats are computed, not stored.**

`max_hp = 10 + stamina*5 + level*10`. Never written to the character. Every read runs the formula. When stamina changes (point spend) or level changes (level up), the next read reflects it automatically.

Exactly G095's lazy-formula model applied to a character sheet. Base stats and level are **state**; derived stats are **derivations**. If someone forgets to update HP after a level-up, nothing breaks — there's no HP to update; it's always current.

First Rosetta Stone project where **lazy derivation is the character-sheet design**. G095 had cells with formulae; G096 has characters with derived getters. Same shape, different domain.

**Seeded RNG makes tests deterministic.**

`Character::roll(name, class, seed)` produces the same character for the same seed. LCG is four lines, identical across languages (same constants, same bit shift). That's what lets all six languages claim "same Rook the Warrior from seed 42".

Real games use system RNG or world-seed RNG. Rosetta Stone needs determinism — every test must hold in every language.

First Rosetta Stone project with **a reproducible RNG as a cross-language contract**. LCG constants (6364136223846793005, 1442695040888963407, shift 33) are the committed values — any language using different constants produces non-matching characters.

**Point budget is an invariant.**

Level up grants 5 points. Points spend on stats with per-class caps. `spend_point(stat, n)` checks budget AND cap. Invariant: `unspent_points ≥ 0`, `base_stat ≤ cap`.

Database-style constraints — defined once, checked on every write. G082's CMS had state-transition rules; G096 applies the same idea to incremental state updates.

First Rosetta Stone project where **the character sheet has database-like invariants** — not just data but legal-change rules.

**Save format round-trips cleanly.**

`name: X\nclass: Y\nlevel: N\n...stat:strength=K\n...` — decomposable into `key: value` atoms without ceremony. Line-based format G085/G086/G088/G093/G094 all picked scales to G096.

First Rosetta Stone project where **the character is a file**. Every real RPG (tabletop, CRPG, MMO) has this as primitive unit — character exists on paper or save file, loadable later.

G095 (spreadsheet model) → G096 *data-over-dispatch classes + lazy derived stats + deterministic seeded RNG + budget-and-cap invariants*. Files 12/16. 96/130 complete.

---

## G097 — Image Map Generator

**Hit testing is one method on every shape.**

Every shape answers `contains(x, y) → bool`. Shapes differ in data and test logic; caller sees them as interchangeable. Shape polymorphism via kind tag (Python/Go/CL) or enum variant (Rust/Lean).

G059's polymorphism applied to geometry. G059 had `shape_area(Shape) → f64`; G097 has `contains(Shape, x, y) → bool`. Same pattern: one method, many shapes, dispatched on kind.

First Rosetta Stone project where **the polymorphic method produces a boolean, not a value**. Hit testing is a **predicate**, not a function. That subtle shift — "does this shape claim this point?" — enables composable hit-testing pipelines.

**Point-in-polygon is ray casting.**

Textbook algorithm: shoot horizontal ray from `(x, y)` rightward; count edge crossings; **odd = inside, even = outside**. Works for convex, concave, self-intersecting polygons (even-odd rule).

Each edge `(xi, yi)` to `(xj, yj)` is tested: `(yi > y) != (yj > y)` means the edge spans `y`. Compute x-intersection; if right of `x`, toggle inside.

Same algorithm every GIS library (PostGIS, GEOS), every CAD tool, every SVG renderer uses. First published 1962; unchanged for general polygons.

First Rosetta Stone project with **a named classical algorithm** as the payload. Ray casting is worth implementing six times — cross-language consistency verifies the algorithm's definition, not just our implementation.

**Z-order resolves overlaps.**

Two rects stacked — which does the click land on? Higher `z` wins. `hit_test(x, y)` sorts regions by z descending, returns first containing.

Same model as every UI framework (CSS z-index, tkinter raise/lower, CAD layer stacking). Z is a small integer; bigger = on top. G097 doesn't enforce uniqueness — shared z relies on insertion order via stable sort.

First Rosetta Stone project with **explicit z-ordering as a contract**. G074's whiteboard used z for drawing; G097 applies it to hit testing. Both use stable sort so insertion order breaks z ties deterministically.

**HTML image maps are the simplest output target.**

`<map>` + `<area>` is how you made image parts clickable before CSS/SVG were universal. 25-year-old spec, browser-universal, no dependency to read.

G097 generates `<area shape="rect" coords="0,0,100,100" href="/home" alt="Home">` etc. SVG would be richer but needs scaffolding; CSS would need positioning logic. `<map>` just works.

First Rosetta Stone project where **the export target is an old-but-universal standard** chosen for ubiquity over features.

**Rect coords for HTML differ from internal.**

Internal rect is `(x, y, w, h)`; HTML `<area>` takes `(x1, y1, x2, y2)`. Converter emits `(x, y, x+w, y+h)`. Circles and polygons are unchanged.

Tiny but illustrative: **internal and export representations can differ**, and the exporter mediates. G090 serialised bytes as-is; G094 serialised fields as-is. G097 transforms some fields to match the target format.

First Rosetta Stone project with **explicit coordinate-system translation at export time**.

G096 (character sheets) → G097 *predicate-polymorphism + ray-casting canonical algorithm + z-ordered hit resolution + legacy-standard export target*. Files 13/16. 97/130 complete.

---

## G098 — File Copy Utility

**Copy is three orthogonal concerns.**

Naive `copy_tree(src, dest)` conflates: what to traverse, what to include, how to resolve conflicts, how to report progress. G098 separates: `copy_tree(src, dest, policy, filter) → events`. Each is a first-class parameter or return.

Separation of mechanism from policy — rsync, robocopy, every modern sync tool is organised around it. Mechanism (walk, copy) is fixed; policy (collision behaviour) and filter (inclusion) are configurable.

First Rosetta Stone project where **a single operation takes a policy enum and a filter struct as inputs**. G092 took one rule; G098 takes rule plus filter, and the rule is a named variant of several.

**Overwrite policy is three named cases.**

Every real copy tool has Skip / Overwrite / Rename-with-suffix. G098 models them as an enum; the copy loop branches on the enum when collision is detected. Adding a fourth ("interactive: ask") would be one variant plus a callback.

First Rosetta Stone project where **a user-facing policy decision is explicit data**. G092's rules were action shapes; G098's `Overwrite` is "what to do when the action would conflict". Orthogonal concerns, both reified.

**Filter is a pure predicate.**

`filter.accepts(path) → bool`. Include patterns need ≥1 match; exclude patterns require 0. Substring only, no regex. Applied before the policy check, so filtered items never reach the branch.

First Rosetta Stone project where **a filter is a separate object with its own API**. Previous filters were inline (G094) or embedded (G093). G098's `Filter` is a small struct with one method; it composes with the copier by being passed as argument.

**Events are the replay log.**

Every item produces an event (Copied/Skipped/Renamed/CreatedDir + source + dest + bytes). Event list is returned; caller can sum bytes, count skips, replay, serialise to audit log. **Not print, not callbacks — data.**

Kafka, Git reflog, Postgres WAL, every event-sourced system. G094 captured domain events; G098 captures operation events. Both return data that can be processed, not discarded.

First Rosetta Stone project where **the operation's side effects are captured as a value**.

**Rename-with-suffix cascades.**

If `dest/a.txt` exists and policy is rename, copy goes to `dest/a.txt.1`. If that exists too, `dest/a.txt.2`. `next_available_name(items, base)` loops until a free slot.

Same algorithm every browser uses for downloads (`cover.jpg`, `cover (1).jpg`), every email client for attachments. Convention (`.N`, `(N)`, `-copy`, date-stamp) layers over the same loop.

First Rosetta Stone project with **cascading name resolution**. G092 rejected collisions; G098 resolves them by generating new names.

G097 (image maps) → G098 *policy-as-data + filter-as-predicate + events-as-data + cascading rename resolution*. Files 14/16. 98/130 complete.

---

## G099 — Code Snippet Manager

**Tab stops are the editor's contract.**

A snippet expansion is two things the editor needs: (1) literal text to insert, (2) cursor positions to visit as user tabs through. Parser must produce both in lockstep; offsets stay correct as defaults substitute in.

First Rosetta Stone project where **the return value is a two-part structured result** matching an external contract (the IDE's snippet protocol). G091 returned pages + lines; G099 returns text + stops. Both are IRs external tools consume.

**`$0` is the final cursor, not a regular stop.**

Non-zero stops (`$1`, `$2`, ...) are ordered positions — user tabs forward. `$0` is the **final resting place** — after exhausting regular stops, cursor lands on `$0` and snippet mode exits.

Universal across every snippet system back to TextMate 1. Sort comparator: non-zero ascending by number; `$0` last regardless of text position.

First Rosetta Stone project where **a convention from an entire software ecosystem is the correctness target**. Getting it wrong doesn't crash — makes snippets feel wrong to anyone who's used any other editor.

**Placeholder syntax has three forms.**

`${N:default}`, `${N}`, `$N` — three variants of the same concept. Parser handles body character-by-character: `$` + `{` → find matching `}`, parse contents; `$` + digits → scan digits, no-default stop; otherwise copy verbatim (including `$` followed by anything else).

First Rosetta Stone project where **the parser handles three variant forms of the same concept** without exploding into three separate parsers. Variants share structure (all produce a TabStop); only presentation differs.

**Duplicate stop numbers mirror edits.**

`${1:name}` twice in the same snippet is not a bug — means "type once, mirror to both". G099 doesn't implement mirroring (UI concern); parser preserves both stops so the editor can group by number.

First Rosetta Stone project where **the model permits logically-duplicate data that downstream code treats specially**. Library doesn't deduplicate; consumer does.

**Library indexes by trigger, language, tag.**

Three query patterns: trigger+language ("what does `for` expand to in Python?"), language only, tag. G099 stores each as sorted list or hash by relevant key. Insertion updates all indexes; queries hit exactly the one that matches.

First Rosetta Stone project where **a small library indexes its contents multiple ways**. G093 indexed music metadata; G099 applies the same pattern to code snippets.

G098 (file copy + events) → G099 *two-part IR (text + stops) + $0-last ordering + three-form placeholder parser + duplicate-preservation for UI mirroring*. Files 15/16. 99/130 complete.

---

## G100 — Versioning Manager (Closes Files)

**Content addressing is the dedupe primitive.**

A blob's identity **is its content hash**. Same bytes → same hash → same blob. Two commits with the same file point to the same blob — no copies. Storage deduplicates automatically; no "check if exists" logic because the hash answered.

Git's object store, IPFS's entire protocol, Nix's package cache, Docker image layers, every system that wants "identical things stored once". G100 reduces it to three tables (blob, tree, commit) keyed by hash.

First Rosetta Stone project where **the identifier is derived from the data**. G093 keyed by external path; G094 by insertion order; G100's objects are keyed by their content hash.

**The commit chain is a linked list of trees.**

Commit has tree hash + parent hash. Walking parent pointers from HEAD reconstructs full history. G100's chain is linear (one parent); real Git is a DAG (merges). Deliberate simplification — the pattern is established; branching/merging is layered on.

First Rosetta Stone project with **a deliberate simplification of a well-known system**. G079 didn't try to be Zork; G094 didn't try to be syslog. G100 picks Git's object-store subset that fits the category.

**Canonical encoding is the cross-language contract.**

For commit hashes to match across six languages, the byte sequence fed to the hash function must match byte-for-byte. Tree entries sorted by filename. Hex lowercase. Newlines `\n`. No trailing spaces. Tab-separated fields.

Every language produces these exact bytes. Test: commit same files, same timestamp, same message in Python and Rust; same commit hash. **Behavioural equivalence at the bit level** — strongest cross-language contract the milestone has.

First Rosetta Stone project where **all six languages must produce identical bytes** from the same input. G085 had textual round-trip within one language; G090 had byte-level within one; G100 requires byte equivalence across six.

**Blob storage is deduplicated by construction.**

Ten files containing `"hello"` in one commit produce one blob, not ten. Committing the same file twice produces zero new blobs. `blob_count()` grows only when truly-new content arrives.

First Rosetta Stone project where **storage efficiency is an emergent property**, not an optimisation. G089 used integer cents for correctness; G100 uses content addressing for dedup. Both cases: choosing the right representation makes the hard problem go away.

**Diff is tree comparison, not byte comparison.**

`diff(from, to)` compares two commits' tree entries. Hash differs → Changed. Name exists in one but not other → Added or Removed. O(n) in tree entries, not O(bytes). Same speedup `git diff --name-status` gets.

First Rosetta Stone project where **a comparison operation exploits content addressing for speed**. Two commits are identical iff their trees hash identically — a single 64-bit compare replaces a full file walk.

**Closing Files: the journey.**

G085 → G086 → G087 → G088 → G089 → G090 → G091 → G092 → G093 → G094 → G095 → G096 → G097 → G098 → G099 → G100. Sixteen patterns: round-trip equivalence, frecency ranking, path resolution, multi-key sort, group-by aggregation, byte compression, flow layout, preview/apply/undo, metadata/data separation, append-only logs, lazy formula evaluation, class templates, ray-cast hit testing, policy-as-data copy, tab-stop IR, content-addressed storage.

Every file-format-adjacent codebase draws on these. The noosphere's vault files, save states, binary assets, backups, git-style history will compose these patterns rather than invent new ones.

G099 (snippet IR) → G100 *content-addressed identity + commit-chain history + canonical cross-language encoding + structural dedup + tree-level diff*. **Files 16/16 — CATEGORY CLOSED. 100/130 complete.**

---

## G101 — SQL Query Analyzer (Opens Databases)

**Parse then analyse is the universal pipeline.**

Every compiler, linter, SQL engine follows the same shape: tokenise → parse → analyse. G101 implements all three. Tokeniser handles keywords, identifiers, numbers, strings, operators, punctuation. Parser is recursive descent — one function per grammar rule. Analyser walks the AST against a schema, emits findings.

First Rosetta Stone project with the full **three-phase pipeline** as explicit stages. G071 did forgiving HTML extraction but not structural parsing; G085 parsed a line-based format but didn't build an AST. G101 is the first where a real grammar produces a typed tree.

**Recursive descent is one function per rule.**

Grammar:
```
query ::= 'select' projection 'from' identifier [where] [order_by] [limit]
```

Becomes `parse_query`, `parse_projection`, `parse_where`, `parse_order_by` — one function per nonterminal. Each consumes tokens from a shared position pointer, returns an AST node or error. No parser generator, no combinators, just functions.

Rosetta Stone commits to this style because it maps directly into every language. Same organisation as Rust's compiler's Pratt parser, Python's `ast` module, every hand-written SQL parser in production.

First Rosetta Stone project where **grammar rules map 1-to-1 to parser functions**.

**AST is data; analysis is a second pass.**

`SqlQuery` struct has no methods for "is this correct?" — pure data. All validation in `analyse(query, schema)`, a separate function. Split is load-bearing: parser errors are syntax ("I don't know what you meant"); analyser errors are semantics ("I understood but it's wrong").

Keeping them separate lets tooling parse without analysing (formatters), analyse without re-parsing (caching), or both. Same AST feeds every use case.

First Rosetta Stone project where **the data structure has no methods that judge itself**. G093's tag store had a query method; G097's shape had `contains`. G101's `SqlQuery` has no methods — analysis is a free function.

**Findings are data, not panics.**

Every analysis issue is a `Finding` — severity + code + message. Multiple findings per query are normal. Severities ladder: Info < Warning < Error. Codes are stable identifiers (`UNKNOWN_TABLE`, `SELECT_STAR`, `LIMIT_WITHOUT_ORDER`) so tooling can filter/suppress specific lints.

Pattern clippy, ESLint, pylint, SQLFluff all use. Output is data the UI renders however it wants.

First Rosetta Stone project where **errors are first-class values with codes and severities**. G085 returned a single `ParseError`; G101 returns a list of `Finding` with severity levels. That shift — from "one fatal error" to "a prioritised list of issues" — is what every production linter does.

**Schema context turns syntax into semantics.**

Without a schema, parser can say `SELECT name FROM users` is well-formed but not whether `users` exists. Schema provides that context — a map from table name to column set. Analyser passes query + schema together; findings like `UNKNOWN_TABLE` only fire when the schema says so.

Exactly how Postgres resolves names at planning, how every ORM validates at query-build, how SQL LSPs provide completion.

First Rosetta Stone project where **validation is parameterised by external context**. G092 had a directory as context; G101's schema is more abstract. Future Database projects (G102+) will compose over this schema abstraction.

G100 (content-addressed storage) → G101 *recursive-descent parser + AST-as-data + findings-with-severity + schema-parameterised validation*. Databases 1/13. 101/130 complete.

---

## G102 — Remote SQL Tool

**Parser + executor = database.**

G101 built the parser — text into AST. G102 is the executor — AST into rows. Together they form the minimum viable database: SQL string in, result set out.

Splitting lets each phase be tested independently. Parser doesn't need storage; executor doesn't need text. They meet at the AST, which is just data.

First Rosetta Stone project where **two prior projects compose into a complete system**. Future Database projects (G103+) layer over this same AST+executor foundation.

**Typed columns prevent silent corruption.**

Every column has a declared type (Int or Text). INSERT validates: type mismatch → structured error. No silent coercion, no "1" where 1 was meant.

Real databases extend with NUMERIC, DATE, JSON, arrays. The **principle** doesn't change: table declares shape; inserts must match; SELECTs rely on it.

First Rosetta Stone project where **type safety is enforced at the storage boundary**, not just at the language level. G093 had typed Option fields but didn't reject inserts; G102 rejects with `TypeMismatch`.

**Row storage is positional, not keyed.**

Row is a `Vec<Cell>` positioned to match the column list. Column name → position via `column_index(name)`. INSERT provides values in column order; SELECT rewrites to same order.

Positional wins for O(1) index access, compact storage, stable order. Real DBs use positional rows for the same reasons. Column stores (Parquet, Arrow) push further — each column a separate array for columnar scans.

First Rosetta Stone project where **storage layout is chosen explicitly for access pattern**.

**Transactions are a buffer that collapses on commit.**

`begin()` starts a pending-buffer. INSERTs during the txn go to the buffer. `commit()` moves buffer to table rows; `rollback()` discards.

SELECTs inside the txn see both base + pending — the **read-your-own-writes** guarantee every SQL transaction provides. After COMMIT pending becomes normal; after ROLLBACK it vanishes.

Minimalist — no isolation levels, no conflict detection, no logging. But the shape (buffer, commit-or-discard) is the kernel every real transaction system extends. Postgres's WAL, SQLite's journal, Redis's MULTI/EXEC all bolt isolation + durability on top.

First Rosetta Stone project with **atomic rollback of data mutations**. G082 had revisions (append-only); G092 had undo logs (reverse ops). G102 is the first with proper "abandon these changes entirely" semantics.

**SELECT is filter-project-sort-limit in order.**

Executor runs SELECT as a pipeline: scan → filter → sort → project → limit. Sort **before** project because ORDER BY column may not appear in projection (`SELECT name ORDER BY age` is legal).

Real engines push projection before sort when safe — via query planner. G102 doesn't have a planner; canonical ordering is stable.

First Rosetta Stone project where **a multi-stage pipeline has a canonical ordering**. Changing the order breaks queries that sort on non-projected columns.

**Result set is data, not an iterator.**

SELECT returns a `ResultSet` — columns + rows, all in memory. Not a cursor, not a generator. Materialisation is eager.

Real engines stream for large results; trade-off is predictability vs memory. For Rosetta Stone's scale, materialise.

First Rosetta Stone project where **query output is eager, fully-materialised data**. Consistent with G091's rendered documents, G094's event lists, G098's copy events.

G101 (SQL parser + findings) → G102 *executor completing the database + typed columns + positional rows + transactional buffer + filter-sort-project-limit pipeline*. Databases 2/13. 102/130 complete.

---

## G103 — Card Collector

**Multiset is the right shape for inventory.**

A collection isn't a set — sets don't track duplicates. Isn't a list — lists care about order. It's a **multiset**: map from key to count. `{DPN-001/Mint: 4, DPN-001/Played: 2, DPN-003/NearMint: 1}`.

G103's key is `(card_id, condition)` — same card at different conditions are different inventory entries. `add` increments; `remove` decrements (errors on underflow); key disappears at zero. Pattern every counting app uses: inventory, vote tallies, bag-of-words, Kubernetes replica counts.

First Rosetta Stone project where **the primary data structure is a multiset**. G047 was a map; G093 was a map. G103's `Collection` is the first where the value is explicitly a *count*, not an attribute.

**Valuation composes multipliers.**

`value = base_price × rarity_mult × condition_mult`. Three inputs, one pipeline. New axis (foil, first edition, signed) = new multiplier; formula stays flat.

Exactly how every real pricing system works — TCGplayer, PSA, Beckett. G103 keeps integers throughout (cents), rounds at the final step — preserves G089's integer-cents discipline.

First Rosetta Stone project where **pricing is a multiplicative pipeline**. G096's RPG derived stats were additive; G103's pricing is multiplicative. Both styles appear in real systems; Rosetta Stone shows both.

**Missing list is set difference.**

"What cards do I need?" = `catalogue_ids - owned_ids`. Simple, total, O(n). G103's `missing(catalogue)` iterates the catalogue's sorted IDs, filters out ones owned, returns ordered list.

First Rosetta Stone project where **set difference is a first-class query**. G093's `MissingField` was metadata-level (which fields unset); G103's `missing` is inventory-level (which cards absent). Both operationalise the same set-theoretic primitive.

**Trade bundle is two-sided multiset comparison.**

A trade has `offered` and `asked` — both are bundles of `(card_id, condition, qty)`. Evaluate: sum each side's value; compare against a tolerance.

Three verdicts: Fair (within tolerance), OfferedMore (asker gets better end), AskedMore (offerer gets better end). Diff reported in cents so users can decide whether to accept, renegotiate, or walk.

First Rosetta Stone project with **explicit two-sided comparison and verdict**. G092 had preview/apply for one side; G103's trade evaluation is symmetric — neither side privileged, evaluator just reports the gap.

**Inventory key is a compound type.**

`InventoryKey { card_id, condition }` — two fields needed to uniquely identify a stackable slot. Derives Hash/Eq/Ord so it works as a hash-map key.

Same pattern as Amazon SKUs (product + variant), weather data (station + timestamp), time-series points (metric + tags). Compound key, scalar value.

First Rosetta Stone project where **a struct is used as a hash-map key**. G088's sort key was a struct but config, not key. G103's `InventoryKey` identifies rows in the multiset.

G102 (executor completing database) → G103 *multiset inventory + multiplicative valuation + set-difference missing list + symmetric trade evaluation + compound hash key*. Databases 3/13. 103/130 complete.

---

## G104 — Report Generator

**Report is a list of kind-tagged rows.**

A report isn't a 2D array. It's a **list of rows**, each carrying a kind (Header/Data/Subtotal/Total). UIs dispatch on kind — bold header, indent data, emphasise subtotal, underline total.

First Rosetta Stone project where **the output format is a sequence of typed rows, not homogeneous data**. G094 had uniform entries; G091 had lines all the same kind. G104's rows each carry semantic role — consumer doesn't guess.

**Column definitions are first-class.**

`ColumnDef` declares name, field, kind (plain/summable), format-cents flag. Engine is fixed; columns are data. Pattern pandas, Excel pivots, Tableau all ship.

First Rosetta Stone project where **the output schema is declarative**. G095's Excel had cells-by-address; G104 has columns-as-objects. Adding a new column is adding a `ColumnDef`, not modifying engine code.

**Subtotals are per-group aggregations emitted inline.**

G104 emits data rows under each group PLUS subtotal row — user sees detail + summary in one pass. SQL's `GROUP BY WITH ROLLUP`, pandas' `.groupby().sum()` with data intact, every accounting ledger.

Subtotals per-group; grand totals across all groups. No grouping → skip subtotals, keep grand total if aggregates exist.

First Rosetta Stone project where **an aggregate is emitted alongside the data it aggregates**. G089's group-by returned only aggregates; G104 preserves source rows AND adds aggregates — hybrid view.

**Grand total label in first cell.**

"TOTAL" anchors the first cell; aggregate columns show summed values; non-aggregate middle cells blank. Spreadsheet convention: label anchors, numbers line up under headers.

First Rosetta Stone project with **label-plus-values row layout**. G091 had rendered lines with no structural roles; G104's cells map 1:1 to columns.

**Currency formatting is a column property.**

`format_cents=true` renders `1500` as `$15.00`. Per-column because some columns are money (revenue) while others are raw counts (qty). Integer stays integer in the record; formatter runs at report-generation time.

First Rosetta Stone project where **formatting is attached to the column schema**.

G103 (multiset inventory) → G104 *kind-tagged row output + declarative column schema + inline group subtotals + per-column currency formatting*. Databases 4/13. 104/130 complete.

---

## G105 — Database Backup Script Maker

**Code generation is template + validation.**

A generator has two halves: validator rejects malformed configs, emitter turns valid configs into deterministic text. G105 separates them: `validate(spec) → errors` then `emit_script(spec) → script | errors`. Invalid configs never reach the emitter; valid ones produce byte-reproducible output.

First Rosetta Stone project where **the output is code, not data**. G085's quiz text was data; G094's logs were data. G105's bash script is **executable** — runs in a different interpreter, has its own security surface.

**Shell quoting uses single quotes + `'\''` escape.**

Bash single-quoted strings are literal; a single quote cannot appear inside. Canonical workaround: close, emit backslash-escaped quote, reopen. `it's` → `'it'\''s'`.

G105's `shell_quote` does this mechanically. Every shell-script generator must implement it — mistakes here are CVEs.

First Rosetta Stone project where **shell-safe quoting is a security primitive**. G101 handled SQL string literals; G105 handles shell string literals. Different syntax, same class of problem.

**Identifier allow-list beats shell-quoting alone.**

Table names flow into **unquoted** positions (`--table=users`). Shell-quoting the value doesn't prevent SQL syntax injection if the name itself has SQL metacharacters. Instead, G105 validates identifiers up front: must match `[a-zA-Z_][a-zA-Z0-9_]*`.

Defence-in-depth: identifier is shell-quoted (bash sees it literal) AND validated (no SQL metacharacters reach pg_dump). Either layer alone has holes.

First Rosetta Stone project with **an explicit allow-list for user-provided strings that flow into multiple interpreters**.

**Determinism enables diff-and-review.**

`emit_script(spec) == emit_script(spec)` is a hard invariant. Same inputs → same bytes. Enables the ops pattern: version-control config, regenerate script, `diff` old vs new, review what changed.

G105 keeps determinism by sorting table lists, quoting deterministically, and **not embedding timestamps or randoms in the script text** (the `STAMP=$(date ...)` line runs at script-execution time, not emission time — that's the right place).

First Rosetta Stone project where **deterministic emission is load-bearing** for downstream diff review.

**Validation errors are a list.**

`validate(spec)` returns a `Vec<ValidationError>`. Multiple problems surface at once, not "fix one, resubmit, find next". Mirrors G101's `Finding` pattern: errors as data, plural.

First Rosetta Stone project where **config validation surfaces all errors in one pass**. G085 stopped at first error; G101 and G105 emit everything the checker found.

G104 (kind-tagged reports) → G105 *validate-then-emit code gen + shell-safe quoting + identifier allow-list + deterministic-for-diff + multi-error validation*. Databases 5/13. 105/130 complete.

---

## G106 — Event Scheduler and Calendar

**Half-open intervals are the universal convention.**

An event from 10:00 to 11:00 occupies `[10:00, 11:00)`. ISO-8601, RFC 5545, every calendar API. Adjacent events `[10:00, 11:00)` and `[11:00, 12:00)` don't overlap — their endpoint touches but neither contains 11:00 on both sides.

Inclusive-inclusive `[10, 11]` vs `[11, 12]` would both contain 11 → would "overlap" — nonsensical. Half-open makes the math clean.

First Rosetta Stone project where **the interval convention is a correctness premise**. G079 didn't deal with continuous time; G080 used point-in-time triggers. G106 is the first with range semantics.

**Overlap check is two inequalities.**

`a` and `b` overlap iff `a.start < b.end AND b.start < a.end`. Total, symmetric, no branching on which is earlier.

Naïve checks have bugs — single inequality misses cases; containment check misses partial overlaps. The two-inequality form is the shortest correct form.

First Rosetta Stone project with **a canonical interval algorithm** every language implements identically. G097's point-in-polygon was similar — a classical algorithm worth implementing six times.

**Recurrence is template + expansion.**

Recurring event stores the **first occurrence** plus a **rule** (Daily/Weekly/Monthly). Query range expands rule into concrete occurrences. Template doesn't materialise all futures — many recur forever.

Expansion driven by `[from, to)`: start at first occurrence, advance by step, emit each in window, stop at `to` or optional series-end.

First Rosetta Stone project where **a single template represents an unbounded set**. G082's revisions were concrete; G094 was eager. G106 is the first where **data is generated on demand** from a rule.

**Fast-forward skips irrelevant occurrences.**

Naïve expansion iterates from first occurrence forward. For a daily event 10 years old with query "next week" — 3650+ wasted iterations.

G106 computes how many steps to skip via divmod and adds them in one multiply-and-add. O(1) setup + O(range/step) emission.

First Rosetta Stone project where **an optimisation is load-bearing at scale**. Slower versions are correct but intractable; fast-forward makes daily+1-year-lookback tractable.

**Conflict detection is "any overlap exists".**

Candidate `[new_start, new_end)` conflicts with existing event iff any occurrence overlaps. For recurring, expand into the window and check each. `find_conflicts(start, end)` returns event IDs.

First Rosetta Stone project with **a batch query returning matching identifiers, not details**. G102's SELECT returned rows; G106's conflicts returns IDs — caller looks up details separately.

**Integer milliseconds avoid timezone complexity.**

Time is an `i64` count of milliseconds from an arbitrary epoch. No timezones, no leap seconds, no DST. Arithmetic is integer; comparisons are integer. Caller converts wall-clock to ms outside the engine.

First Rosetta Stone project where **time is explicitly a monotonic integer** — cross-language confidence depends on this, since no language has to agree on timezone database versions.

G105 (code generation) → G106 *half-open intervals + two-inequality overlap + recurrence-as-template-plus-expansion + fast-forward to window + integer-ms time*. Databases 6/13. 106/130 complete.

---

## G107 — Budget Tracker

**Envelope budgeting is category → allocation.**

A budget isn't one number — a **set of envelopes**, each with its own allocation. Groceries: $500. Rent: $2000. Each tracks independently; overrunning groceries doesn't affect rent's balance.

YNAB, cash-envelope method, 50/30/20 rule. G107 formalises: `Envelope { category, budgeted_cents, rollover, alert_threshold_pct }`. Budget aggregates envelopes + transaction stream.

First Rosetta Stone project where **a "budget" is structured per-category, not a single scalar**. G089's aggregations *grouped by* category; G107 makes categories also a *specification*.

**Variance = actual − budgeted.**

Signed difference. Positive = overspent; negative = underspent. Variance_pct = variance × 100 / budgeted. Accounting convention: positive variance is bad news for expenses.

Zero-budget envelopes get sentinel variance_pct of -101. UI renders as "∞" or "no budget set" — any ratio over zero base is undefined; the sentinel surfaces that explicitly.

First Rosetta Stone project where **a sentinel value represents "undefined"** in a numeric field.

**Alert thresholds scale with envelope size.**

Not a fixed-dollar line — a **percentage** of the envelope. Rent at 90% ($1800/$2000) alerts; entertainment at 90% ($90/$100) also alerts. Attention scales with exposure.

Each envelope has its own threshold; strict envelopes alert early, predictable envelopes late. Policy is per-envelope, not per-budget.

First Rosetta Stone project where **a scale-invariant threshold replaces an absolute one**. G086's frecency was scale-invariant (log); G107's alert is too (percentage).

**Refunds subtract from spent, not add.**

Refund is positive amount_cents. Computing `spent`: `spent += -amount_cents` handles both — expense (negative) adds to spent; refund (positive) subtracts. Clean, symmetric.

Why not separate refund transactions? Because accounting-wise they offset the original. A $50 refund against a $100 purchase yields $50 spent, not $50 + -$50 = $0 with no record.

First Rosetta Stone project where **a sign flip carries semantic meaning** (positive = money in, negative = money out).

**Uncategorised spending surfaces as synthetic envelope.**

Real data has categories the user never budgeted for. G107's status walks transactions, groups by category, emits a row for every category — **including ones that match no envelope**.

Synthetic envelopes have `budgeted_cents = 0`, `over_budget = true`, `variance_pct = -101`. UI renders them as "?" categories the user should assign or accept as overflow.

First Rosetta Stone project where **the output includes entries not present in the input schema**. G094 synthesised group keys from transactions. G107 synthesises *envelopes* when the user's schema didn't cover the data.

G106 (intervals + recurrence) → G107 *per-category envelopes + signed variance + scale-invariant alert thresholds + refund-as-negative-expense + synthetic envelopes for uncategorised*. Databases 7/13. 107/130 complete.

---

## G108 — Address Book

**Deduplication requires canonicalisation.**

`Alice.Foo@Gmail.com` and `alicefoo@gmail.com` route to the same inbox but differ by string equality. Correct address books maintain a **canonical form** — lowercase, trimmed, gmail-dot-stripped, `+tag`-stripped — and compare on that form.

Per field type: email (lowercase + gmail dots + plus-tag + googlemail→gmail), phone (digit-only + US leading-1 dropped). Original preserved for display; canonical powers comparison. iCloud, Google Contacts, Salesforce, Outlook all do this.

First Rosetta Stone project where **the same data has two representations — display form and canonical form — and comparisons always happen on canonical**. G093's tag store did exact match; G108 defines an equivalence relation on strings.

**Duplicate grouping is union-find.**

Contacts can share an email OR a phone. Transitively: A↔B via phone, B↔C via email → A, B, C are one group. Classic **union-find**.

G108 builds reverse indexes per identifier (email→{ids}, phone→{ids}), then for each index entry unions all IDs that share it. Parent pointers with path compression collapse transitive closure in near-linear time.

First Rosetta Stone project that **uses union-find as a primary algorithm**. G097 was point-in-polygon; G108 is union-find — classic algorithms translate cleanly across languages; their cross-language parity is a confidence test.

**Merge is union of fields + longest name.**

Merging produces one contact with: union of emails/phones/tags (deduplicated, sorted), longest non-empty name, earliest creation timestamp, concatenated addresses and notes.

Longest name because duplicates often accumulate richer metadata — one record has the full name, another has just a first name. Preserving the fuller form recovers detail.

First Rosetta Stone project where **merge produces new data from multiple sources** with explicit conflict-resolution rules (longest for name, earliest for timestamp, union for lists, concat for text).

**The original survives canonicalisation.**

G108 stores `emails: ["Alice.Foo@Gmail.com"]` and computes canonical forms on demand. Canonicalisation is **derived**, not stored.

If stored: rule updates (new gmail policy, new phone format) make all records stale. Derived on demand: rule updates are a library deploy, not a database migration.

First Rosetta Stone project where **a computed view is the comparison basis while the source form is the source of truth**.

**Synthetic IDs let merged contacts round-trip.**

Merging 1 and 2 doesn't reuse either ID — merged contact gets a fresh ID (e.g., 47). Sources are deleted. Why: auditability ("47 was merged from 1+2"), reversibility, reference stability for external systems.

First Rosetta Stone project where **merge is a creation operation, not a mutation**. G092 mutated in place; G108 creates fresh records.

**Search goes through the same canonicalisation.**

`search_by_email(needle)` canonicalises the needle and matches against contacts' canonical forms. Same pipeline the duplicate detector uses. Read and write go through the same normaliser — same pattern G100 used for content-addressed tree hashing.

First Rosetta Stone project where **both read-side and write-side go through the same normalisation function**.

G107 (budget envelopes) → G108 *canonical email + phone + union-find transitive grouping + longest-name merge heuristic + synthetic merged IDs + canonicalised search*. Databases 8/13. 108/130 complete.

---

## G109 — TV Show Tracker

**Library and watchlist are separate concerns.**

A TV tracker has two data stores with different meanings:
* **Library** — canonical shows/episodes, shared across users. Source of truth for what exists, when it aired, how long it is.
* **Watchlist** — per-user watched-episode keys. Private. Source of truth for what this person has seen, what's next.

Netflix/Trakt/Plex keep these separate; conflating them (bad CSV exports) breaks when episodes are added or renumbered. G109 models the split explicitly: `Library { shows }` and `Watchlist { watched: Set<EpisodeKey> }`, joined via key type.

First Rosetta Stone project where **shared data and per-user data are distinct types**. G093 mixed metadata and collection; G109 separates canonical knowledge from user-specific state.

**Progress rolls up from leaves.**

Each episode has a binary state (watched/unwatched). Show progress = watched_count / total. Season progress: same restricted to season.

Roll-up is one-directional — episode state is truth, aggregates derive. Changing one flag updates all aggregates on next read. G095's lazy-formula model applied to watch state.

First Rosetta Stone project where **percentage progress is derived, not stored**. Storing risks drift when episodes are added or marked.

**Next-up is first-unwatched-in-order.**

`next_up(show) = first episode in (season, episode) order the user hasn't watched`. Sort by tuple, scan until unwatched, return. Five lines.

Edge cases handled uniformly: out-of-order viewing fills gaps (watched S1E1, S1E3 → next up is S1E2); fully watched returns None; nothing watched returns S1E1.

First Rosetta Stone project where **a UI-critical primitive reduces to one line of search + predicate**. Every "Continue Watching" shelf is one call to this.

**Airing-since is filter + sort.**

"What's new since last week?" — filter episodes by `aired_ms >= since`, sort by aired_ms. Library doesn't track per-user last-checked time; caller passes cutoff. Keeps the library reusable across users.

First Rosetta Stone project where **a library query takes the cutoff from the caller**, not from per-user state.

**EpisodeKey is a compound foreign key across stores.**

Library stores full `Episode` records keyed by `(show_id, season, episode)`. Watchlist stores just the key. "Is this watched?" constructs the key and hash-looks up — no need to pass the whole Episode around.

Same pattern every SQL foreign-key uses: joining table stores key, not row. Denormalising would store title/duration in watchlist too, creating sync burden.

First Rosetta Stone project where **a compound struct is a canonical foreign key** across two stores. G108 compared on compound identifiers but within one store.

**Currently-watching is "started but not finished".**

`currently_watching(library, watchlist)` returns shows where `0 < watched_count < total`. Not-started filtered out (candidates). Finished filtered out (no next-up). Only middle state matters.

The "Continue Watching" row. Every streaming service ships it. G109 makes it a pure function.

First Rosetta Stone project where **a view is defined by a range predicate on a derived quantity**.

G108 (contact dedup) → G109 *library/watchlist split + derived hierarchical progress + first-unwatched-in-order next-up + filter+sort airing-since + compound foreign key across stores + in-progress filter*. Databases 9/13. 109/130 complete.

---

## G110 — Travel Planner System

**An itinerary is a chain of graph edges.**

A leg is an edge in a transportation graph: `origin --(mode, depart_ms → arrive_ms)--> destination`. A trip is a walk along that graph — sequence where each edge's target equals the next edge's source.

G110 validates the chain composition property: **leg[N].destination == leg[N+1].origin**. Any mismatch is a `LocationMismatch` — someone's supposed to teleport. Same invariant `git log --graph` checks when parents don't link, same invariant rail routing uses when a transfer makes no sense.

First Rosetta Stone project where **the data structure is a typed chain** and the invariant is that consecutive elements compose. G106 were independent points; G109 were ordered but independent. G110 is the first where the Nth element *depends* on the N-1st.

**Time chain is a second invariant.**

Beyond location, the temporal chain must be monotone: `leg[N+1].depart_ms >= leg[N].arrive_ms`. Otherwise the traveller is in two places. `TimeConflict` as Error.

Time and location chains are **independent** invariants — a trip can violate either or both. G110 reports both separately so the user can fix independently.

First Rosetta Stone project with **two independent consecutive-element invariants**, each reported separately.

**Tight layovers are warnings, not errors.**

A 15-minute layover is legal (traveller has time) but unwise — missed connections happen. G110 flags gaps below `min_layover_minutes` as Warning, not Error. User may accept the risk.

Zero-minute layover (connecting at instant) is neither TightLayover nor TimeConflict — theoretically legal but physically tight. G110 treats it as acceptable (no issue), matching how airlines model "legal connection time".

First Rosetta Stone project where **a continuum of connection time produces different severity responses** — Error for negative, no-issue for zero, Warning for positive-but-below, no-issue for above.

**Leg duration has its own sanity checks.**

Independent of the chain: `arrive_ms >= depart_ms`. Zero = Warning (teleport); negative = Error (time travel). Per-leg, not per-pair.

First Rosetta Stone project where **the validator has two layers**: per-element (is this leg internally consistent?) and per-pair (do these two legs compose?). Independent, merge into one issue list.

**Layovers are derived from consecutive legs.**

`layovers(trip)` walks pairs, emits `Layover { location, arrive_ms, depart_ms, duration_ms }`. Pure function. UI renders as "3h 45m in SFO" between legs.

Derived not stored: edit a leg's depart time and layover updates on next read. G095's formulae, G109's progress roll-up, same pattern.

First Rosetta Stone project where **per-pair derivations are a distinct view** of the same underlying sequence.

**Trip metrics separate span from transit.**

Three quantities: **span** (first depart to last arrive), **in-transit** (sum of leg durations), **layover** (span − transit). Linear identity. Users care about all three — span for "when am I back?", transit for "how much flying?", layover for "airport time?".

First Rosetta Stone project where **three derived metrics relate via a linear identity**. Same decomposition every trip-planning app ships.

G109 (TV tracker) → G110 *chain-of-edges composition invariant + independent location+time chains + severity-laddered layover warnings + two-layer validator (per-leg + per-pair) + linearly-related span/transit/layover metrics*. Databases 10/13. 110/130 complete.

---

## G111 — ERD Creator

**A schema is a typed directed graph.**

Entities are **nodes**; relationships are **edges** with typed endpoints (which attribute on each side) and a cardinality label. The ERD is a directed graph: `from_entity --(from_attr, to_attr, cardinality)--> to_entity`.

Every schema tool (dbdiagram, drawSQL, SchemaSpy, prisma's ERD generator) treats it this way. Once the graph is explicit, standard graph algorithms apply: reachability, topological sort, cycle detection, dependency closure. G111 runs three of them.

First Rosetta Stone project where **the data model is a general directed graph** (with typed endpoints) rather than a tree, sequence, or map. G100's commit chain was a DAG but linearly structured; G111's ERD can have arbitrary shape.

**Cardinality is enum, not magic string.**

`OneToOne`, `OneToMany`, `ManyToMany` — three values, sealed set. Not "1-1", "1:1", "one_to_one", "1-*" depending on who typed it. Closed enum eliminates a class of bugs and enables exhaustive handling.

First Rosetta Stone project where **a domain term is constrained to a small finite enum** to prevent string-representation drift. G101 did this for severity; G111 for cardinality.

**Topological sort orders dependencies first.**

`topo_sort()` returns entities in dependency-first order. `User` before `Order` before `OrderItem`. Produces valid SQL schema-creation order.

G111 uses **Kahn's algorithm**: start with zero-in-degree nodes, remove them, repeat. If any nodes remain after completion, there's a cycle.

First Rosetta Stone project with **a named graph algorithm as a primary operation**. G097 was ray casting; G108 was union-find; G111 is Kahn's. Each cross-language implementation is a confidence test of the algorithm's definition.

**Cycle detection is DFS with path-membership tracking.**

Recurse down the graph maintaining the set of nodes currently on the stack. If you encounter one, you've found a back-edge (cycle).

Self-references (`Node.parent_id -> Node.id` for tree structures) are deliberately **excluded** — legitimate pattern, not a structural error.

First Rosetta Stone project with **graph-colouring-style DFS** for cycle detection.

**Validation catches structural errors pre-render.**

Before emitting DOT: does every `from_entity`/`to_entity` resolve? Does every `from_attr`/`to_attr` exist on its entity? Rendering a DOT with dangling refs produces a broken diagram.

First Rosetta Stone project where **pre-render validation is a distinct phase**. G105 validated configs before emitting scripts; G111 validates ERDs before emitting DOT.

**DOT export is the canonical visualisation target.**

Graphviz DOT is the de facto format for directed-graph visualisation. Every ERD tool, UML generator, CI graph view, k8s dependency viewer speaks DOT. G111 emits it directly — `digraph`, record-shape nodes, cardinality-mapped arrowhead attributes.

First Rosetta Stone project where **the export target is DOT**. G090 was ZIP; G100 was content-addressed hashing. DOT is the graph analog — text-based, tool-pipeline-friendly serialisation.

G110 (trip chain) → G111 *typed directed graph + closed cardinality enum + Kahn's topological sort + DFS cycle detection + pre-render validation + DOT export*. Databases 11/13. 111/130 complete.
