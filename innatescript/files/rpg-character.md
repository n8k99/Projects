# G096 — RPG Character Stat Creator

> The Rosetta Stone's twelfth **Files-category** project. Introduces the **class-template + derived-stats** pattern for character sheets. Base stats are rolled from a seeded RNG (deterministic across languages); class multipliers adjust each base stat; derived stats (HP, MP, armour, crit chance) compute lazily from adjusted bases — never stored, always computed. Level progression grants points that spend against per-class caps. The save format is line-based text that round-trips cleanly.

```yaml
id: G096
title: RPG Character Stat Creator
category: files
requires: [G055-dice-roller, G059-shape-polymorphism, G085-quiz-maker, G095-excel-exporter]
provides: [class-template-pattern, derived-stats-from-base, seeded-rng-for-determinism, point-budget-allocation]
```

## Insight: Class Is a Template, Not a Subclass

A class (Warrior, Mage, Rogue) is not a separate code path. It's a **multiplier table**: `{Strength → 1.5, Stamina → 1.3, ...}` for Warrior; different numbers for Mage and Rogue. Every character runs through the same `adjusted(stat) = base * multiplier[stat]` function; the class is just which multiplier table is active.

This is the **data over dispatch** pattern. OOP would tempt us to make a `Warrior` class with a `getStrength()` method overriding a `Character` base class. But then adding a new class means adding code, not data. The Rosetta Stone version: classes are rows in a table; adding a Bard is adding a row.

First Rosetta Stone project where **class-like behaviour is modelled as configuration, not inheritance**. G085's quiz had polymorphic questions via a kind tag; G096 uses the same pattern — one adjustment function dispatched on class config.

## Insight: Derived Stats Are Computed, Not Stored

`max_hp = 10 + stamina*5 + level*10`. Never written into the character. Every time something asks for max HP, the formula runs. When stamina changes (point spend) or level changes (level up), the next read reflects it automatically.

This is exactly G095's lazy-formula model applied to a character sheet. The base stats and level are the **state**; the derived stats are **derivations**. If someone introduces a bug that forgets to update HP after a level-up, nobody notices — because there's no HP to update; it's always current.

First Rosetta Stone project where **lazy derivation is the character-sheet design**. G095 had cells with formulae; G096 has characters with derived getters. Same shape, different domain.

## Insight: Seeded RNG Makes Tests Deterministic

`Character::roll(name, class, seed)` always produces the same character for the same seed. The LCG is four lines and identical across languages (same constants, same bit-shift). That's what lets Rust, Python, CL, Go, Lean, and InnateScript each claim to produce *the same* Rook the Warrior from seed 42.

Real games either use the system RNG (random every time) or seed from the world seed (Minecraft). The Rosetta Stone needs determinism: every test assertion about the rolled character has to hold in every language.

First Rosetta Stone project with **a reproducible RNG as a cross-language contract**. The LCG choice (coefficients 6364136223846793005 and 1442695040888963407) is the specific one the Rosetta Stone commits to — any language that wants to produce matching outputs uses these constants.

## Insight: Point Budget Is an Invariant

Level up grants 5 unspent points. Points can be spent on any stat, but each stat has a **per-class cap**. Spending must check both budget (enough points) and cap (not over max). `spend_point(stat, n)` returns an error on either violation.

The invariant: **unspent_points ≥ 0**, and **base_stat ≤ cap for all stats**. These are checked at every mutation. They're the same kind of invariants databases call "constraints" — defined once, checked on every write.

First Rosetta Stone project where **the character sheet has database-like invariants** — not just data, but rules about what changes are legal. G082's CMS had state-transition rules; G096 applies the same idea to incremental state updates.

## Insight: Save Format Round-Trips Cleanly

A character serialises to `name: X\nclass: Y\nlevel: N\nunspent: M\nstat:strength=K\n...`. The reverse direction (parse) is implied but not implemented here — every Rosetta Stone Files project should be able to load what it saved. For G096, the point is demonstrating that the save format is **decomposable into atoms** (`key: value` lines) without ceremony.

A binary format would be denser but opaque; JSON would need a dependency; YAML is over-powered. The line-based format G085, G086, G088, G093, and G094 all picked scales to G096's needs.

First Rosetta Stone project where **the character is a file**. Every real RPG (including tabletop ones) has this as the primitive unit: a character exists on paper or in a save file, loadable later. G096 establishes that format for future project use.

## Choreographic Case: Vault Character Gallery

```innate
(@vault-character-gallery){
  @files <- @fs/ls{path: "characters/"}
  @characters <- @for file in @files {
    @text <- @file/read-string{path: @file.path}
    @rpg/parse-character{text: @text}
  }

  @ui/render-gallery{characters: @characters}

  @on-user-creates-new (@name @class){
    @c <- @rpg/roll{name: @name, class: @class, seed: @clock/now}
    @file/write-string{path: "characters/${@name}.char",
                        content: @rpg/to-text{character: @c}}
  }

  @on-user-level-ups (@name){
    @c <- @characters.find(.name == @name)
    @rpg/level-up{character: @c}
    ;; Persist immediately so sheet is always latest.
    @file/write-string{path: "characters/${@name}.char",
                        content: @rpg/to-text{character: @c}}
  }
}
```

A vault of character files; the UI reads all of them into memory, renders a gallery, and persists on every mutation. Each character is independent — no server, no sync — just files on disk.

## Structures

```innate
(defenum stat STRENGTH | DEXTERITY | INTELLECT | STAMINA | LUCK)
(defenum class WARRIOR | MAGE | ROGUE)

(defstruct character
  name            : String
  class           : Class
  level           : Int
  base            : {Stat -> Int}
  unspent-points  : Int)

(defstruct derived
  max-hp          : Int
  max-mp          : Int
  armour          : Int
  carry-capacity  : Int
  crit-chance     : Float)
```

## Resolver Natives

```innate
@rpg/roll{name, class, seed}                      -> Character
@rpg/adjusted{character, stat}                    -> Int
@rpg/derived{character}                            -> Derived
@rpg/level-up{character}                           -> Unit
@rpg/spend-point{character, stat, points}          -> Unit | RpgError
@rpg/to-text{character}                            -> String
```

## Demo

```innate
(@demo){
  @c <- @rpg/roll{name: "Rook", class: "warrior", seed: 42}
  ;; Deterministic across languages:
  ;; STR base=16 adj=24 (1.5x warrior bonus)
  ;; STA base=4 adj=5 (1.3x rounds down to 5)
  ;; level=1 → HP = 10 + 5*5 + 1*10 = 45

  @rpg/level-up{character: @c}
  @rpg/level-up{character: @c}
  @rpg/spend-point{character: @c, stat: "strength", points: 5}
  @rpg/spend-point{character: @c, stat: "stamina", points: 3}

  @rpg/derived{character: @c}
  ;; level=3, STR=21 (adj=31), STA=7 (adj=9)
  ;; HP = 10 + 9*5 + 3*10 = 85
}
```

## Where

Class MUST be data (a multiplier table), NOT a code path (a subclass) — adding a new class is adding a row, not a module. Derived stats MUST be computed, NOT stored — storing HP and forgetting to update it on level-up is the classic bug; derivation makes the bug impossible. Seed MUST produce the same character across all six languages — that's what "same logic in six languages" means for randomness. Point spending MUST enforce budget AND cap — either check alone leaves a gap for the other to exploit. Save format MUST be line-based `key: value` — human-readable, diff-friendly, no dependency. LCG constants MUST be the exact Rosetta Stone committed values (6364136223846793005 and 1442695040888963407 with shift of 33) — any language using different constants produces non-matching characters from the same seed.
