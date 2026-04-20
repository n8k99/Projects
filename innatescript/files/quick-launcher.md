# G086 — Quick Launcher

> The Rosetta Stone's second **Files-category** project. Revisits round-trip equivalence through **persistent usage state** — a launcher's memory (what you ran, how often, how recently) is itself a file that survives sessions. Introduces **frecency ranking**: log(1+count)/(1+age_hours), a scalar that blends frequency and recency into a single value. Query-time composition: `text_match + frecency_weight * frecency`, text match dominating, frecency breaking ties.

```yaml
id: G086
title: Quick Launcher
category: files
requires: [G076-bookmarks, G085-quiz-maker]
provides: [frecency-ranking, two-layer-scoring, usage-stats-persistence, line-format-round-trip]
```

## Insight: Frecency Is One Number That Replaces Two

Every launcher the user has ever interacted with answers the same question: "when the user types 'fi', which entry should be first — Firefox (launched 500 times last week) or Files (launched once yesterday)?" Pure frequency says Firefox. Pure recency says Files. Neither is right alone.

Frecency collapses both axes into one scalar: `log(1 + launch_count) / (1 + age_hours)`. The log tames explosive frequency — a 1000x launched entry doesn't crush everything else. The `1 + age_hours` decay causes recent launches to dominate, but an old-and-frequent entry still outranks a new-and-never-used one. No configuration. No weights. One number.

First Rosetta Stone project where **the ranking function is a composition of two scalar sub-scores** (text match × frecency), not a single lexicographic comparison. Every recommender system the vault will build — recent-notes surfacing, related-article ranking, completion suggestions — follows this shape.

## Insight: Text Match Dominates, Frecency Breaks Ties

Naive launchers use pure frecency — but then typing "fire" surfaces Firefox last if you've never launched it. That is wrong. The user has stated their intent: they typed "fire". Frecency cannot overrule intent.

So text match is the **primary score** (0 to 1) and frecency is a small **bonus** (weight 0.1, max ~ln(1000)/1 ≈ 7, so bonus ≤ 0.7). A perfect name match (1.0) beats any frecency. A prefix match (0.8) beats a substring match (0.6) even with boosted frecency. Frecency only breaks ties between two entries that matched the query equally well.

First Rosetta Stone project where **two-layer ranking** is explicit: layer 1 determines visibility (zero text match = filtered out), layer 2 orders the surviving candidates. This is the pattern for every vault search: the query determines what's relevant, usage stats determine what's prioritised among the relevant.

## Insight: Usage State Is a Round-Trip File

G085 showed that a quiz serialises round-trip. G086 extends the pattern to **mutable usage statistics**: launch_count and last_launched_ms are written to disk, read back, and continue accumulating. A launcher that forgets its usage stats between sessions is useless — it has to re-learn the user every day. A launcher with persistent stats learns the user once.

The serialisation is again line-based (`key: value`, `---` separators). The per-entry block now includes `count: N` and `last: T` alongside `name:` and `action:`. Round-trip still holds: `from_text(to_text(l)) == l` for any launcher. The contract the Files category introduced in G085 extends to state that changes over time.

First Rosetta Stone project where **the persistent representation includes accumulated state**, not just static configuration. The vault's note-metadata sidecars (last-viewed timestamps, access counts) will use this pattern.

## Insight: Empty Query Is a Special Case, Not an Error

When the user hasn't typed anything yet, the launcher should show everything — probably sorted by frecency so "most recently/frequently launched" rises to the top. An empty query returning zero results is wrong UX.

So `text_match("") == 0.5` for every entry — every entry passes the filter, and the only discriminator left is frecency. This is the only place where frecency fully determines ranking. The moment the user types one character, text match reasserts dominance.

First Rosetta Stone project where **the ranking function has a documented degenerate case**. Empty query isn't an error or a separate code path; it's handled by the same scoring function with a known constant. Every search box the vault will ship follows this convention.

## Choreographic Case: Vault Launcher

```innate
(@vault-launcher){
  @state <- @vault/read-string{path: ".vault/launcher-state.txt"}
  @launcher <- @launcher/from-text{text: @state}

  @on-user-types (@query){
    @matches <- @launcher/query{launcher: @launcher, text: @query, now: @now}
    @ui/render-hits{matches: @matches, resolver: (@id)=>@launcher/get{id: @id}}
  }

  @on-user-chooses (@entry-id){
    @entry <- @launcher/get{launcher: @launcher, id: @entry-id}
    @shell/run{command: @entry.action}
    @launcher/record-launch{launcher: @launcher, id: @entry-id, now: @now}
    @vault/save{path: ".vault/launcher-state.txt",
                content: @launcher/to-text{launcher: @launcher}}
  }
}
```

Launcher state lives in a single vault file. Every launch is a transaction: run the action, update stats, write the file back. No database needed — a flat text file diffs well in git, reads in any editor, and costs nothing.

## Structures

```innate
(defstruct entry
  id       : Int
  name     : String
  action   : String
  keywords : [String]
  count    : Int       ;; launches since creation
  last     : Int)      ;; last-launched millis

(defstruct launcher
  entries         : {Int -> Entry}
  frecency-weight : Float)   ;; default 0.1

(defstruct match
  entry-id : Int
  score    : Float)
```

## Resolver Natives

```innate
@launcher/new{}                                      -> Launcher
@launcher/add{launcher, entry}                       -> Unit
@launcher/get{launcher, id}                          -> Entry | null
@launcher/query{launcher, text, now}                 -> [Match]
@launcher/record-launch{launcher, id, now}           -> Unit | LaunchError
@launcher/to-text{launcher}                          -> String
@launcher/from-text{text}                            -> Launcher | ParseError
```

## Demo

```innate
(@demo){
  @l <- @launcher/new{}
  @launcher/add{launcher: @l, entry: {id: 1, name: "Firefox", action: "firefox",
                                       keywords: ["browser","web"]}}
  @launcher/add{launcher: @l, entry: {id: 2, name: "Files", action: "nautilus",
                                       keywords: ["folder"]}}
  @launcher/add{launcher: @l, entry: {id: 3, name: "Terminal", action: "kitty",
                                       keywords: ["shell","cli"]}}
  @launcher/add{launcher: @l, entry: {id: 4, name: "File Manager", action: "thunar",
                                       keywords: ["folder"]}}

  @launcher/query{launcher: @l, text: "file", now: 0}
  ;; -> [{id:2, 0.8}, {id:4, 0.8}]     — tie, exact-prefix both, no frecency yet

  @launcher/record-launch{launcher: @l, id: 2, now: 1_000_000}
  @launcher/record-launch{launcher: @l, id: 2, now: 1_000_000}
  @launcher/record-launch{launcher: @l, id: 2, now: 1_000_000}

  @launcher/query{launcher: @l, text: "file", now: 2_000_000}
  ;; -> [{id:2, ~0.94}, {id:4, 0.8}]   — frecency broke the tie

  @launcher/query{launcher: @l, text: "fire", now: 1_000_000}
  ;; -> [{id:1, 0.8}]                  — Firefox, never launched, still wins

  @text <- @launcher/to-text{launcher: @l}
  @restored <- @launcher/from-text{text: @text}
  @restored == @l                                    ;; -> true (round-trip)
}
```

## Where

Text match MUST be strictly primary — no amount of frecency can elevate an entry with zero text match, and no frecency bonus can overcome a one-step difference in text-match tier (exact vs prefix, prefix vs substring). Frecency MUST decay with real time — an entry launched 100 times a month ago MUST rank below one launched 10 times this hour when text match ties. Empty query MUST surface all entries ranked by frecency — it is not an error, it is the "show me everything" state. Round-trip MUST preserve launch_count and last_launched_ms exactly — the file is the source of truth for accumulated user preference, and losing it means re-learning the user. Unknown-entry launches MUST return a structured error, NOT silently no-op — a launcher asked to run a non-existent entry means something upstream (shortcut, pinned tile) is stale and the UI needs to know.
