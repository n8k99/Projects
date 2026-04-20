# G103 — Card Collector

> The Rosetta Stone's third **Databases project**. Introduces **multiset semantics** — a data structure that counts multiple occurrences of the same key, the natural model for "I have 4 copies of this card in Mint condition and 2 in Played condition". Card identity is `(set_code, number)`; the same card at different conditions are the same *card* but different *inventory entries*. Layer in a **valuation function** (base_price × rarity_multiplier × condition_multiplier) and a **trade evaluator** that compares two bundles by total value — and you have the kernel every card-collection app ships.

```yaml
id: G103
title: Card Collector
category: databases
requires: [G093-mp3-tagger, G096-rpg-character, G102-remote-sql-tool]
provides: [multiset-inventory, multiplier-based-valuation, trade-bundle-comparison, set-difference-missing]
```

## Insight: Multiset Is the Right Shape for Inventory

A collection isn't a set — sets don't track duplicates. It isn't a list — lists care about order. It's a **multiset**: a map from key to count. `{DPN-001/Mint: 4, DPN-001/Played: 2, DPN-003/NearMint: 1}`.

G103 makes the key `(card_id, condition)` so the multiset distinguishes the same card at different conditions. `add` increments; `remove` decrements (errors if underflow); key disappears at zero. Every counting application follows this shape — inventory, vote tallies, language bag-of-words, Kubernetes replica counts.

First Rosetta Stone project where **the primary data structure is a multiset**. G047's dictionary was a map; G093's tag store was a map. G103's `Collection` is the first where the value is explicitly a *count*, not an attribute.

## Insight: Valuation Composes Multipliers

`value = base_price * rarity_mult * condition_mult`. Three inputs, one pipeline. Adding a new axis (foil vs. non-foil, first edition, signed-by-artist) is adding a multiplier — the formula stays flat.

This is exactly how every real pricing system works: Magic: The Gathering's TCGplayer, Pokémon's PSA, baseball card grading. Each grade/edition/condition is a multiplier; the product is the price. Rosetta Stone uses integers throughout (cents), rounding at the final step — preserves G089's integer-cents discipline for correctness.

First Rosetta Stone project where **pricing is a multiplicative pipeline**. G096's RPG derived stats were additive (sta*5 + level*10); G103's pricing is multiplicative (base × rarity × condition). Both styles appear in real systems; the Rosetta Stone shows both.

## Insight: Missing List Is Set Difference

"What cards do I need to complete my set?" — that's `catalogue_ids - owned_ids`. Set difference: everything in the full catalogue, minus everything we own. Simple, total, O(n).

G103's `missing(catalogue)` iterates the catalogue's sorted IDs, filtering out ones in the collection. Returns a list, ordered. The UI can render it directly as a wishlist or feed it into a trade-search tool.

First Rosetta Stone project where **set difference is a first-class query**. G093's `MissingField` query was metadata-level (which fields are unset); G103's `missing` is inventory-level (which cards are missing entirely). Both operationalise the same set-theoretic primitive.

## Insight: Trade Bundle Is a Two-Sided Multiset Comparison

A trade has an `offered` bundle and an `asked` bundle. Both are lists of `(card_id, condition, qty)`. Evaluate: sum each side's total value; compare.

Three verdicts:
* **Fair** — values within a tolerance window.
* **Offered more valuable** — the asker is getting the better end; difference reported in cents.
* **Asked more valuable** — the offerer is getting the better end; difference reported.

Tolerance is a parameter — 100-cent tolerance is "within a dollar", 1000 is "within $10". Tooling surfaces the diff so users can decide whether to accept, renegotiate, or walk away.

First Rosetta Stone project with **explicit two-sided comparison and verdict**. G092's bulk rename had preview/apply for one side; G103's trade evaluation is symmetric — neither side is privileged, the evaluator just reports the numeric gap.

## Insight: Inventory Key Is a Compound Type

`InventoryKey { card_id, condition }` — two fields, both needed to uniquely identify a stackable inventory slot. The struct derives `Hash`/`Eq`/`Ord` so it can be a hash-map key and sort predictably.

This is the pattern every catalogue system uses: Amazon SKUs (product + variant), e-commerce inventory (item + size + colour), weather data (station + timestamp), time-series points (metric + tags). The key is compound; the value is scalar.

First Rosetta Stone project where **a struct is used as a hash-map key**. G088's sort key was a struct but it was a config, not a key. G103's `InventoryKey` is the identifier for a row in the multiset.

## Choreographic Case: Vault Card Binder

```innate
(@vault-card-binder){
  @cat <- @cc/catalogue-load{path: "catalogues/dpn-set-1.json"}
  @col <- @cc/collection-load{path: "binder/my-cards.json"}

  @ui/render-stats{
    total-value: @cc/total-value{collection: @col, catalogue: @cat},
    unique:      @cc/unique-cards{collection: @col},
    missing:     @cc/missing{collection: @col, catalogue: @cat}
  }

  @on-trade-proposed (@offered-bundle @asked-bundle){
    @result <- @cc/evaluate-trade{
      offered: @offered-bundle, asked: @asked-bundle,
      catalogue: @cat, tolerance-cents: 100
    }
    @ui/render-trade-verdict{result: @result}
  }
}
```

The vault's binder page shows value + unique count + missing wishlist. Trade proposals pipe through the evaluator, surfacing the fairness verdict for the user to decide.

## Structures

```innate
(defenum rarity COMMON | UNCOMMON | RARE | MYTHIC)
(defenum condition MINT | NEAR_MINT | PLAYED | DAMAGED)

(defstruct card
  set-code         : String
  number           : Int
  name             : String
  rarity           : Rarity
  base-price-cents : Int)

(defstruct inventory-key
  card-id   : String
  condition : Condition)

(defstruct collection
  counts : {InventoryKey -> Int})

(defstruct trade-bundle
  cards : [(String, Condition, Int)])

(defenum trade-verdict FAIR | OFFERED_MORE | ASKED_MORE)

(defstruct trade-evaluation
  verdict    : TradeVerdict
  diff-cents : Int)
```

## Resolver Natives

```innate
@cc/catalogue{}                                   -> Catalogue
@cc/catalogue-add{cat, card}                      -> Unit
@cc/collection{}                                  -> Collection
@cc/add{col, card-id, condition, qty}             -> Unit
@cc/remove{col, card-id, condition, qty}          -> Unit | Error
@cc/total-value{col, cat}                         -> Int
@cc/missing{col, cat}                             -> [String]
@cc/trade-bundle{}                                -> TradeBundle
@cc/bundle-add{bundle, card-id, condition, qty}   -> Unit
@cc/evaluate-trade{offered, asked, cat, tolerance} -> TradeEvaluation
```

## Demo

```innate
(@demo){
  @cat <- @cc/catalogue{}
  @cc/catalogue-add{cat: @cat, card: {set: "DPN", num: 4, name: "Mythic Four",
                                       rarity: "mythic", base: 100}}
  ;; ... other cards

  @col <- @cc/collection{}
  @cc/add{col: @col, card-id: "DPN-004", condition: "mint", qty: 1}
  @cc/total-value{col: @col, cat: @cat}   ;; -> 2500 cents

  @offered <- @cc/trade-bundle{}
  @cc/bundle-add{bundle: @offered, card-id: "DPN-004", condition: "mint", qty: 1}
  @asked <- @cc/trade-bundle{}
  @cc/bundle-add{bundle: @asked, card-id: "DPN-003", condition: "mint", qty: 12}
  @cc/bundle-add{bundle: @asked, card-id: "DPN-001", condition: "mint", qty: 4}
  @cc/evaluate-trade{offered: @offered, asked: @asked, cat: @cat, tolerance: 100}
  ;; -> {verdict: "fair", diff-cents: 0}
}
```

## Where

Inventory MUST be a multiset, NOT a set — duplicate counts are the whole point of a collection, and collapsing them loses information. Key MUST be `(card_id, condition)` — the same card at Mint and Played are different market goods and mixing them would produce wrong valuations. Zero-count keys MUST be deleted, NOT left as `{k: 0}` — the hash map should only represent what's present, not what used to be. Valuation MUST be multiplicative — additive valuations fail when adding new axes; multiplication composes cleanly. Final value MUST be an integer (rounded cents) — float money is a category error in accounting contexts. Trade evaluation MUST be symmetric — neither "offered" nor "asked" is privileged; the evaluator just reports the diff. Tolerance parameter MUST be explicit — different users have different fairness thresholds, and baking in a default is paternalism.
