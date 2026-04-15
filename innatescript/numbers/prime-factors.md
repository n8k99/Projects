# Prime Factorization

Pure computation again — but with a structural twist.

## Resolver native

```dpn
@prime_factors{n: @n} -> @factors
@prime_factorization{n: @n} -> @factor_map
```

## Where it gets interesting

Factorization *decomposes*. It breaks a whole into its irreducible parts. That's structurally analogous to what InnateScript does with choreographies — projection decomposes a global intention into each agent's irreducible local slice.

A choreography that uses factorization as metaphor:

```dpn
[decompose_project @project
    @prime_factors{n: @project:complexity_score} -> @factors
    concurrent [
        @SylviaInkweaver{assign narrative weight to each factor}
        @KathrynLyonne{assign resource cost to each factor}
    ]
    join
    <- @ElianaRiviera{verify decomposition covers the whole}
] where [no factor is left unassigned and the product of assignments reconstructs the project]
```

The `where` here echoes the fundamental theorem of arithmetic — the factorization is unique and the product of the factors reconstructs the original. Applied to project decomposition: did the agents' assignments cover every irreducible piece, and do the pieces multiply back into the whole?

## Design note

Three projects in. Pure computation still resolves natively. But this is the first project where the *structure* of the computation (decomposition into irreducibles) mirrors a coordination concept (projection into local slices). The Rosetta Stone is starting to show where math and choreography share vocabulary.
