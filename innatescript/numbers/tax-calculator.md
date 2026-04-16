# Tax Calculator

Tax brackets are the first encounter with RULES in the Rosetta Stone. Not computation from first principles (pi), not world-state lookup (city coordinates), not structural validation (credit cards). Rules are human-created policies encoded as conditional logic. This introduces a fifth resolver primitive: the rule engine.

## Resolver native

```dpn
@tax{income: 50000} -> @result
@tax{income: 50000, year: 2024} -> @result
@tax_breakdown{income: @income} -> @brackets
@effective_rate{income: @income} -> @rate
@marginal_rate{income: @income} -> @rate
```

## Rules are neither facts nor computations

`@tax{income: 50000}` doesn't compute from first principles (like `@pi{}`). It doesn't look up a fact (like `@distance_cities{from: "NYC", to: "London"}`). It doesn't validate structure (like `@luhn_check{}`). It applies a rule set — a cascade of conditions and rates that a legislature defined. The resolver needs a new category:

```dpn
[tax_resolution @income
    <- @rules:tax_brackets{year: 2024} -> @brackets   ;; fetch the rule set
    @apply_brackets{income: @income, brackets: @brackets}
       -> @result
] where [rules are versioned policy, not immutable math]
```

The `@rules:` prefix signals the resolver that this isn't a computation or a lookup — it's a policy fetch. The rule set is versioned by year because legislatures change it. This is different from fact freshness (city coordinates shift over geological time) — rules change because humans DECIDE to change them.

## Versioned rules vs. mutable facts

```dpn
;; Facts change because the world changes:
@distance_cities{from: "Istanbul", to: "Ankara"}  ;; same answer in 2024 and 2025

;; Rules change because humans decide to change them:
@tax{income: 50000, year: 2024}  ;; $6,053
@tax{income: 50000, year: 2025}  ;; different brackets, different result

;; The resolver needs both:
[tax_planning @client
    @rules:tax_brackets{year: @current_year} -> @current_brackets
    @rules:tax_brackets{year: @next_year} -> @proposed_brackets
    @compare_outcomes{
        income: @client:projected_income,
        current: @current_brackets,
        proposed: @proposed_brackets
    } -> @comparison
] where [planning requires reasoning across rule versions]
```

The resolver maintains a rule registry alongside its fact store. Facts are keyed by identity. Rules are keyed by identity AND version. `@rules:tax_brackets{year: 2024}` and `@rules:tax_brackets{year: 2025}` are different rule sets, not stale vs. fresh copies of the same data.

## Range dispatch: continuous pattern matching

The bracket cascade is a `match` on continuous ranges, not discrete values. Credit card validation dispatched on discrete prefixes (Visa/MC/Amex). Tax brackets dispatch on ranges (10%/12%/22%...). InnateScript's `match` needs both:

```dpn
;; Discrete dispatch (G013 credit cards):
match @card:network [
    "Visa"       -> @visa_handler
    "Mastercard" -> @mc_handler
]

;; Range dispatch (G014 tax brackets):
match @income [
    0..11600       -> @apply{rate: 0.10}
    11601..47150   -> @apply{rate: 0.12}
    47151..100525  -> @apply{rate: 0.22}
    100526..191950 -> @apply{rate: 0.24}
    191951..243725 -> @apply{rate: 0.32}
    243726..609350 -> @apply{rate: 0.35}
    609351..       -> @apply{rate: 0.37}
]
```

But tax brackets aren't simple dispatch — they're CUMULATIVE. Each dollar passes through every bracket up to its level. The match isn't "which ONE bracket" but "which brackets, and how much in each." This is a fold over ranges, not a single dispatch:

```dpn
[progressive_tax @income
    @fold_brackets{
        income: @income,
        brackets: @rules:tax_brackets{year: 2024},
        accumulator: 0
    } -> @total_tax
] where [each bracket applies to its slice, not the whole income]
```

## Effective vs. marginal: reporting as agent concern

The calculation produces one number: total tax. The interpretation is an agent concern. Effective rate ("you're paying 18%") is what Kathryn reports to Nathan — the big picture. Marginal rate ("your next dollar is taxed at 24%") is what JMax uses for tax planning — the decision-relevant edge.

```dpn
[tax_report @income
    @tax{income: @income, year: 2024} -> @result
    concurrent [
        @KathrynLyonne{
            summarize @result:effective_rate for Nathan
            "Your effective tax rate is {effective_rate}%"
        } -> @summary
        @JMaxMontague{
            analyze @result:marginal_rate for tax planning
            flag if approaching bracket boundary
        } -> @planning_notes
    ]
] where [same data, different agent perspectives]
```

Same `@result`, different extractions. Kathryn cares about the average burden. JMax cares about the marginal incentive. The choreography doesn't privilege either view — both are valid projections of the same calculation. This is why InnateScript separates computation from reporting: the resolver computes, agents interpret.

## Choreographic case: annual tax planning

The full tax planning choreography coordinates across rule versions and agent perspectives:

```dpn
[annual_tax_planning @client
    ;; Kathryn projects income from multiple sources
    @KathrynLyonne{
        project @client:salary,
        project @client:freelance_income,
        project @client:investment_returns
    } -> @projected_income

    ;; JMax applies current and proposed bracket rules
    concurrent [
        @JMaxMontague{
            @tax{income: @projected_income, year: 2024} -> @current_tax
        }
        @JMaxMontague{
            @tax{income: @projected_income, year: 2025} -> @proposed_tax
        }
    ]

    ;; Coordinate to minimize tax burden through timing
    @JMaxMontague{
        compare @current_tax vs @proposed_tax,
        identify income deferral opportunities,
        calculate optimal timing of income recognition
    } -> @strategy

    @KathrynLyonne{
        present @strategy to Nathan,
        flag decisions that need action before year end
    } -> @action_items
] where [multi-agent tax planning across rule versions]
```

## Design note

Fourteen projects in. First encounter with policy-as-data — rules that are neither computed nor observed but legislated. The rule engine is a new resolver primitive alongside computation, lookup, validation, and scheduling. Tax brackets reveal that InnateScript needs versioned rule sets, continuous range matching, cumulative fold over ranges, and the separation between computation (total tax) and interpretation (effective vs. marginal, Kathryn vs. JMax). The resolver doesn't just compute and look things up — it also applies human-authored policy. And policy changes on a legislative schedule, not a physical one.
