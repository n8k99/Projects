# Unit Converter

A choreography that converts between measurement units.

## Design feedback

Unit conversion is about context. The same physical quantity — a distance, a mass, a temperature — expressed in different frames of reference. This maps directly to InnateScript's agent model: agents operate in their own frames. Kathryn thinks in dollars, Eliana thinks in server-hours, Vincent thinks in pixels. Unit conversion is what happens at the boundaries where agents with different frames need to agree on meaning.

The `@convert` native is a bridge between contexts, just like `@base` was a bridge between representations. The algorithm is simple — normalize to a common base, then express in the target frame — but the insight is structural: every time two agents exchange a quantity, there is an implicit unit negotiation. InnateScript makes that negotiation explicit.

Temperature reveals something the ratio-based categories don't: some conversions aren't proportional. Celsius to Fahrenheit involves an offset. This means the conversion isn't just scaling — it's transformation. The resolver handles both, but the distinction matters. Ratio conversions compose (km->m->ft is the same as km->ft). Offset conversions don't (you can't chain C->F->K by multiplying factors). The resolver must know which kind of conversion it's performing.

```dpn
@convert{value: 100, from: "celsius", to: "fahrenheit"} -> 212.0
@convert{value: 1, from: "mi", to: "km"} -> 1.609344
@convert{value: 2.5, from: "kg", to: "lb"} -> 5.51156
@convert{value: 1, from: "gal", to: "L"} -> 3.78541
```

The normalize-to-base pattern maps cleanly to the resolver's generic protocol. Each unit knows its relationship to the base unit. The resolver looks up two relationships and composes them. This is the same thing `@base` does with number representations — normalize to an internal canonical form, then express in the requested output form.

## Choreographic case

Cross-domain coordination where agents need to express compatible units. A logistics choreography where one agent reports distance in km, another needs it in miles for a US shipping API. Neither agent changes its internal representation — the conversion happens at the boundary, in the choreography itself.

```dpn
# International shipping coordination
concurrent [
    @warehouse_tokyo{distance_to_port: "nearest"} -> @dist_km
    @carrier_us{rate_schedule: "pacific_route"} -> @rate_per_mile
]
join

@convert{value: @dist_km, from: "km", to: "mi"} -> @dist_mi
@calc{expr: "@dist_mi * @rate_per_mile"} -> @shipping_cost

@warehouse_tokyo{confirm: "shipping cost is @shipping_cost USD"}
```

The warehouse thinks in kilometers. The carrier thinks in miles. Neither needs to know the other's unit system. The choreography handles the translation at the boundary — `@convert` bridges the frames.

Multi-unit coordination chains naturally:

```dpn
# Recipe scaling across measurement systems
@chef_lyon{recipe: "ratatouille", servings: 4} -> @recipe

# Convert everything to US customary for the American kitchen
@convert{value: @recipe.flour_g, from: "g", to: "oz"} -> @flour_oz
@convert{value: @recipe.water_mL, from: "mL", to: "cup"} -> @water_cups
@convert{value: @recipe.oven_celsius, from: "celsius", to: "fahrenheit"} -> @oven_f

@kitchen_boston{prepare: @recipe, flour: @flour_oz, water: @water_cups, oven: @oven_f}
```

Each `@convert` is a boundary crossing. The French chef's recipe exists in metric. The Boston kitchen operates in US customary. The choreography doesn't rewrite the recipe — it translates at each boundary point.

## What this means

Unit conversion reveals that InnateScript's agent model is fundamentally about frames of reference. Every agent has a native frame — the units it thinks in, the representations it uses internally. Choreographies coordinate across frames. The `@convert` native makes frame translation explicit rather than burying it in each agent's implementation.

The normalize-to-base pattern (meters for distance, grams for weight, liters for volume) is the same pattern the resolver uses everywhere: find a canonical internal form, translate inputs to it, compute, translate output from it. Units are just another instance of the resolver's generic protocol — context in, canonical form, context out.

The temperature special case is instructive: not all frame translations are simple scaling. Some involve offsets, some involve non-linear transforms. The resolver must be prepared for conversion functions that are more complex than multiplication. This is true of agent coordination generally — translating between Kathryn's financial frame and Eliana's infrastructure frame isn't just scaling; it requires understanding the relationship between the frames.

## Native implementation

The resolver provides `@convert{value: V, from: F, to: T}` as a built-in. Ratio-based categories (distance, weight, volume) normalize to a base unit and rescale. Temperature uses dedicated formulas. The host language provides the arithmetic. Unknown units or cross-category conversions produce resolver errors.
