# Calculator

A choreography that evaluates arithmetic expressions.

## Design feedback

A calculator is an evaluator. InnateScript IS an evaluator. This is the project where the Rosetta Stone confronts itself — building an expression evaluator in a language that is an expression evaluator. The resolver already does what a calculator does: parse, evaluate, return a result.

The recursive descent parser that every other language implements from scratch — tokenize, parse precedence levels, reduce to a value — is exactly what the resolver does with every `@reference` it encounters. `@calc{expr: "2 + 3 * 4"}` is a resolver native. The resolver reads the expression, respects structure (bindings nest like parentheses), and produces a value. The algorithm is the same. The question is whether we even need to reimplement it, or whether we name the capability that already exists.

The answer: we name it. The resolver's generic protocol already handles `@calc` the same way it handles `@binary` or `@base` — dispatch to a native, return a value. But unlike those single-function natives, `@calc` reveals that the resolver itself is a recursive descent evaluator. Every `@reference{with: @nested{references}}` is an expression tree being walked.

```dpn
@calc{expr: "2 + 3 * 4"} -> 14
@calc{expr: "(2 + 3) * 4"} -> 20
@calc{expr: "10 / 3"} -> 3.333...
@calc{expr: "-(@revenue - @costs)"} -> @net_loss
```

The last example is where it gets interesting. When the expression contains `@references`, the resolver must evaluate them first — exactly like a recursive descent parser evaluating subexpressions before applying the outer operator. The resolver IS the parser.

## Choreographic case

The calculator becomes meaningful in InnateScript when the terms of the expression come from different agents. A calculator that only evaluates literal strings is just a function call. A calculator that evaluates expressions whose terms are live references to agent-produced values is coordination.

```dpn
# Budget reconciliation across departments
concurrent [
    @kathryn{report: "Q4 revenue"} -> @revenue
    @eliana{report: "infrastructure cost"} -> @infra_cost
    @marcus{report: "staffing cost"} -> @staff_cost
]
join

@calc{expr: "@revenue - @infra_cost - @staff_cost"} -> @net
@calc{expr: "@net / @revenue * 100"} -> @margin_pct

@kathryn{present: "Margin is @margin_pct%"}
```

Each agent contributes a term. The calculator evaluates in context — it doesn't need to know where the numbers came from, only that the references resolve. The choreography is the coordination; the calculation is just the final reduction.

Multi-step calculations chain naturally:

```dpn
@calc{expr: "(@base_price * @quantity) * (1 + @tax_rate)"} -> @total
@calc{expr: "@total - @discount"} -> @final_price
```

The resolver walks the expression tree, resolving `@references` as it descends — exactly the way a recursive descent parser evaluates subexpressions. The difference: in Python or Rust, the "subexpressions" are literal numbers. In InnateScript, they are promises that other agents fulfill.

## What this means

The calculator is the identity project for InnateScript. Every other Rosetta Stone entry implements an algorithm in a language. This one implements an evaluator in an evaluator. The resolver's generic protocol — parse, resolve, return — is the same algorithm as recursive descent. The Rosetta Stone's calculator column doesn't just show InnateScript solving a problem; it shows InnateScript recognizing itself.

The choreographic contribution is the insight that expressions with agent-resolved terms are inherently concurrent: the terms can be computed in parallel, and the expression evaluation is the synchronization point where results combine.

## Native implementation

The resolver provides `@calc{expr: E}` as a built-in. Arithmetic expressions are parsed with standard precedence. References within the expression (`@name`) are resolved before evaluation. The host language provides the actual arithmetic.
