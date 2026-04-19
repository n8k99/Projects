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
