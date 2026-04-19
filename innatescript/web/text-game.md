# G079 — Text Based Game (like Utopia)

> The Rosetta Stone's first **interactive simulation world** — a room graph, a player state, and a command loop where every input mutates the world and the world reacts. Composes G064's graph (rooms connected by exits), G073's command dispatch (verbs and arguments), G077/G078's state FSMs (Playing / Dead / Won), and G078's tick engine (every command is a turn).

```yaml
id: G079
title: Text Based Game (Utopia-like)
category: web
requires: [G064-family-tree, G073-telnet, G077-password-safe, G078-media-player]
provides: [room-graph-world, turn-based-simulation, command-loop, npc-reactive-ai, win-lose-conditions, text-adventure-shape]
```

## Insight: The World Is a Graph Whose Nodes Have State

G064 had a self-referential graph (family trees); G070 Browser had state bags organised hierarchically; G079 combines them: **rooms form a graph**, and **each room is a state bag**. Exits connect rooms; items live in rooms; enemies stand in rooms; the player occupies one room at a time. When the player moves, the world's topology doesn't change — only the player's pointer into it does.

First Rosetta Stone project where **a graph structure IS the simulation world**. Every room is both a node (with outgoing edges named by direction) and a container (with items and enemies inside). The graph and the containers are orthogonal: moving between rooms doesn't change room contents; picking up an item doesn't change the graph.

This is the shape of every MUD, every text adventure, every dungeon-crawler. It generalises to:
- Kubernetes pods-on-nodes (nodes form a graph of network topology; pods live on nodes).
- Database tables and their rows (tables are nodes in a schema graph; rows live in tables).
- Vault notes and their tags (notes form a link-graph; tag-memberships live on notes).

## Insight: Every Command Is a Turn

The command loop is **turn-based**: each input the player submits advances the world by one tick. After the player's action resolves, the world reacts — enemies in the current room strike, the turn counter increments, end-of-game conditions are checked.

This is the **lock-step simulation** pattern at minimal scale. The world doesn't advance on its own (no real-time clock); it advances exactly when the player does something. Every action produces a response; no action produces silence. First Rosetta Stone project where **user input drives the simulation clock** — unlike G078's media player (self-ticking engine that the caller advances on a schedule), G079 ticks on input only.

Both styles have their place. Turn-based games use input-driven ticking. Real-time games use self-ticking with input as a separate event stream. Most real systems are some mix: the vault's daily-note system advances automatically at midnight (self-ticking) but reacts to each agent message as an event (input-driven). G079 picks the simpler pure-turn-based model.

## Insight: Verb-Noun Parsing with Aliases

The parser is minimal: first word is the verb, the rest is the argument. Aliases normalise common short forms (`n` → `go north`, `take` → `get`, `l` → `look`). Unknown verbs produce a polite error; empty input is a no-op.

First Rosetta Stone project with **human-oriented command parsing at the input boundary**. G073's telnet had a small fixed vocabulary; G079 has a richer one with synonyms and direction shortcuts because the user is typing from memory, not reading help. The parser is forgiving (like G071's HTML parser) — malformed input is an error message, not a crash.

The verb-noun grammar is what every old text adventure used and what modern natural-language interfaces still simulate underneath. It scales to "verb noun preposition noun" for transitive commands ("put sword in chest") without fundamentally changing the dispatch shape.

## Insight: Items Have Kind-Specific Behaviour Behind a Uniform API

Items have a `kind` (Weapon / Potion / Treasure) and kind-specific fields (damage, heal, worth_gold). The `use` command dispatches on kind: use-a-potion heals, use-a-treasure adds gold, use-a-weapon equips. First Rosetta Stone project where **one command has polymorphic effects dispatched by item kind**.

This is the type-class / trait-dispatch pattern applied to domain objects. In Rust / Lean / Go / CL, it would be natural to model each item kind as a trait implementation; G079's simpler enum-based dispatch works identically and keeps the six-language port straightforward. The lesson is that **a single user-facing command can span multiple internal code paths** based on the receiver's type — and the user doesn't have to know which.

## Insight: Win and Lose Conditions Are Checked Every Turn

After each turn, the world checks:
- Player health ≤ 0 → state = Dead, log a death message, all future commands rejected.
- Player gold ≥ win threshold → state = Won, log a victory message, all future commands rejected.

These checks run **every turn**, not only after specific actions. It doesn't matter *how* the player's health reached zero — it might have been a goblin strike during a `go north`, or a trap that hasn't been coded yet, or a `use cursed item` effect — the condition is evaluated from the player's final state after the turn, and the game ends if met.

First Rosetta Stone project where **end conditions are checked as invariants on state, not as side effects of specific actions**. Every prior project had specific operations that moved the system into terminal states (G073's `quit`, G077's auto-lock). G079's terminal states are **derived from the state after every turn** — this is much more robust because it handles combinations the programmer didn't explicitly code.

Analogues: operating-system process termination (any write to a special page triggers SIGSEGV regardless of which line of code made it), database consistency checks (any transaction that leaves foreign keys dangling is rejected, whether by design or accident), health-check probes (a process goes to "unhealthy" when it stops responding, regardless of why).

## Insight: The Log Is the Game's Narrative

Every action appends to `world.log`. Look commands, movement, item pickups, combat strikes, enemy retaliations, state changes — all go into one append-only list. The log IS the game's narrative; replay the log and you replay the game (though without the player's input, this would be a monologue rather than re-execution).

First Rosetta Stone project where **the log is the primary output medium**. G073 had per-session history; G079's log is world-scoped and universal — everything that happens anywhere in the world goes through it. This is how every MUD is implemented (the room broadcast is a per-room log, globally concatenated at the player's view).

The vault's future choreography-execution log will have this shape: every step, every agent message, every state change in one append-only timeline, with views that filter by agent or by choreography for readability.

## Insight: Turn Counters Are the Pattern Beneath Scheduled Actions

`world.turn` increments on every command that "provokes" — actions that legitimately pass time. Look, inventory, stats don't provoke (they're observation commands). Go, take, use, fight do provoke.

This distinction is important because **some mechanics tick on turns, not on wall-clock time**. Enemy regeneration ("goblin heals 1 hp per turn"), poison damage ("lose 1 hp per turn for 5 turns"), hunger ("eat food or starve at turn 50") — all use the turn counter, not the clock. The tick engine is the same as G078 but the clock source is different: user-driven rather than wall-clock.

First Rosetta Stone project where **a logical clock (turns) is distinct from the wall clock**. Production systems have this for rate limiting (per-request counters independent of wall time), fairness queues (round-robin counters), and game-like simulations.

## Choreographic Case: Interactive Agent Training Dungeon

```innate
(@agent-training-dungeon){
  @world <- @tg/new-demo-world
  @agent <- @spawn{policy: @policy, initial-state: @world/player-state{world: @world}}

  @loop {
    @observation <- @world/describe{world: @world}
    @command <- @agent/decide{observation: @observation}
    @result <- @tg/execute{world: @world, cmd: @command}

    @score <- @reward{before: @agent.state, after: @world/player-state{world: @world}}
    @agent <- @learn{agent: @agent, reward: @score, observation: @observation}

    break-if (@world/state{world: @world} != "playing")
  }

  @final-score <- @world/player-gold{world: @world}
}
```

A text-adventure dungeon is a natural environment for agent training: discrete actions, deterministic transitions, clear rewards (gold), clear failure (death). The LLM-based agent decides; the world executes; the outcome feeds back into training. Many reinforcement-learning experiments use exactly this shape.

## Structures

```innate
(defenum item-kind Weapon | Potion | Treasure)
(defstruct item
  id : Int, name : String, kind : ItemKind,
  damage : Int, heal : Int, worth-gold : Int)

(defstruct enemy id : Int, name : String, health : Int, damage : Int)

(defstruct room
  id : Int, name : String, description : String,
  exits : {String -> RoomId}, items : [Item], enemies : [Enemy])

(defstruct player
  location : RoomId, health : Int, max-health : Int, gold : Int,
  inventory : [Item], equipped-weapon-id : ItemId?, equipped-damage : Int)

(defenum game-state Playing | Dead | Won)

(defstruct world
  rooms : {RoomId -> Room}, player : Player,
  state : GameState, turn : Int,
  win-gold : Int, log : [String])
```

## Resolver Natives

```innate
@tg/new-demo-world                       -> World
@tg/execute{world, cmd}                  -> [String]       ;; new log lines
@tg/state{world}                         -> GameState
@tg/current-room{world}                  -> Room
@tg/player-state{world}                  -> Player
```

## Demo

```innate
(@demo){
  @w <- @tg/new-demo-world
  @tg/execute{world: @w, cmd: "look"}         ;; describes Entrance Hall, items, exits
  @tg/execute{world: @w, cmd: "take sword"}
  @tg/execute{world: @w, cmd: "equip sword"}
  @tg/execute{world: @w, cmd: "go east"}
  @tg/execute{world: @w, cmd: "take chalice"}
  @tg/execute{world: @w, cmd: "use chalice"}  ;; +50 gold, triggers win condition
  @tg/state{world: @w}                         ;; -> Won
}
```

## Where

Every player command MUST update the log with at least one line — even failures ("I don't understand X", "There is no Y here") are visible feedback; silent failures break the command loop's trust contract. Only specific verbs MUST provoke a turn (movement, take, use, fight, equip) — observation verbs (look, inventory, stats) MUST NOT advance time because the player is building a mental model, not acting. End conditions MUST be checked after every provoking turn, not after specific actions — death by goblin strike while moving must be caught the same way as death by a future poison mechanic. When the game state becomes non-Playing, all subsequent commands MUST return a "game is over" message — allowing actions on a dead/won world would silently corrupt state. Direction aliases (n/s/e/w) MUST normalise to full names inside the dispatch — otherwise every downstream check needs to handle both, scattering the alias-expansion logic throughout the code.
