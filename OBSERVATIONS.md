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
