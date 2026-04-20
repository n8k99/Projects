# G109 — TV Show Tracker

> The Rosetta Stone's ninth **Databases project**. Introduces **hierarchical state** — show → season → episode — with **progress roll-up** from leaf-level watched flags to show-level percentages. The defining novel move: **next-up computation** as `first_unwatched_in_sorted_order` — the primitive every "Continue Watching" shelf relies on. Also splits **library** (shared canonical view) from **watchlist** (per-user state), the pattern every streaming service uses.

```yaml
id: G109
title: TV Show Tracker
category: databases
requires: [G093-mp3-tagger, G102-remote-sql-tool, G106-event-scheduler, G108-address-book]
provides: [hierarchical-progress, next-up-computation, library-vs-watchlist-split, airing-since-query]
```

## Insight: Library and Watchlist Are Separate Concerns

A TV tracker has two data stores that look similar but mean different things:
* **Library** — the *canonical* set of shows and episodes. Shared across all users. Source of truth for "does this episode exist?", "when did it air?", "how long is it?".
* **Watchlist** — the *user's* set of watched episode keys. Private to one person. Source of truth for "have I seen this?", "what's next?", "am I caught up?".

Every streaming service that gets this right (Netflix, Trakt, Plex) keeps them separate. Every one that conflates them (old Windows Media Center, bad CSV exports) has nightmares when episodes get added, renumbered, or renamed — because the user's "watched" flags either vanish or point to stale data.

G109 models this split explicitly: `Library { shows }` and `Watchlist { watched: Set<EpisodeKey> }`. They share the `EpisodeKey` type as the join key.

First Rosetta Stone project where **shared data and per-user data are distinct types**. G093's tag store mixed metadata and collection; G109 separates canonical knowledge from user-specific state.

## Insight: Progress Rolls Up From Leaves

Each episode has a binary state (watched / unwatched). The show-level progress is computed: `watched_count / total_count`. Season-level progress: same computation restricted to the season. "My library's progress": sum across shows.

Roll-up is one-directional — episode-level state is the source of truth, and every aggregate derives from it. Changing one episode's flag automatically updates all containing aggregates on the next read. This is G095's lazy-formula model applied to watch state.

First Rosetta Stone project where **percentage progress is derived, not stored**. Storing percentages risks drift when episodes are added or marked; computing them avoids it.

## Insight: Next-Up Is First-Unwatched-In-Order

`next_up(show) = first episode in (season, episode) order that the user has not watched`. One function, universal across every streaming UI.

The (season, episode) ordering is canonical — seasons are natural numbers, episodes within a season are natural numbers. Sort by the tuple, scan until you hit an unwatched one, return it. Five lines of code.

Edge cases:
* If the user has watched S1E1 and S1E3 but not S1E2, `next_up` returns S1E2 — out-of-order viewing doesn't skip the gap.
* If the entire show is watched, `next_up` returns `None` — a completed show has no next-up.
* If nothing is watched, `next_up` returns S1E1 — the sensible default.

First Rosetta Stone project where **a UI-critical primitive reduces to one line of search + predicate**. Every "Continue Watching" shelf, every "Mark as next" feature is one call to this primitive.

## Insight: Airing-Since Is a Filter + Sort

"What's new since last week?" — filter all episodes by `aired_ms >= since`, sort by aired_ms. The library doesn't track per-user last-checked time; the caller passes the cutoff. Keeps the library reusable across users, each of whom has their own notion of "last checked".

First Rosetta Stone project where **a library query takes the cutoff from the caller**, not from per-user state. Separation of concerns in action.

## Insight: EpisodeKey Is a Compound Key That Joins Across Stores

The library stores full `Episode` records keyed by `(show_id, season, episode)`. The watchlist stores just the key in a set. When you ask "is this episode watched?", you construct the key and hash-lookup — no need to pass the whole `Episode` around.

This is the pattern every foreign-key relationship in SQL uses: the joining table stores the key, not the row. Denormalisation would store the episode's title/duration in the watchlist too, but that creates two copies to keep in sync; keyed join avoids it.

First Rosetta Stone project where **a compound struct is a canonical foreign key** across two stores. G108's deduplication also compared on compound identifiers, but within one store.

## Insight: Currently-Watching Is "Started But Not Finished"

`currently_watching(library, watchlist)` returns shows where `0 < watched_count < total_episodes`. Not-started shows (watched=0) are filtered out (they're candidates, not in progress). Finished shows (watched=total) are filtered out (no next-up). Only the middle state matters.

This is the "Continue Watching" row. Every streaming service ships it. G109 makes it a pure function over library + watchlist.

First Rosetta Stone project where **a view is defined by a range predicate on a derived quantity**. The inequality `0 < watched < total` is a shape that appears wherever things have a "some but not all" state.

## Choreographic Case: Vault TV Dashboard

```innate
(@vault-tv-dashboard){
  @lib <- @tv/library-load{path: "tv/library.json"}
  @wl <- @tv/watchlist-load{path: "tv/mine.json"}

  @continue <- @tv/currently-watching{library: @lib, watchlist: @wl}
  @ui/render-row{title: "Continue Watching",
                 items: @continue.map(@s => ({
                   show: @s,
                   next-up: @tv/next-up{show: @s, watchlist: @wl},
                   progress: @tv/show-progress{show: @s, watchlist: @wl}
                 }))}

  @airing <- @tv/airing-since{library: @lib, since-ms: @user.last-dashboard-open-ms}
  @ui/render-row{title: "New Episodes", items: @airing}

  @on-user-marks-watched (@show-id @season @episode){
    @tv/watchlist-mark-watched{watchlist: @wl, show-id: @show-id,
                                season: @season, episode: @episode}
    @tv/watchlist-save{watchlist: @wl, path: "tv/mine.json"}
  }
}
```

Dashboard renders Continue Watching and What's New rows; marking as watched updates the per-user file without touching the library. User switches accounts → load different watchlist, same library.

## Structures

```innate
(defstruct episode-key
  show-id : Int
  season  : Int
  episode : Int)

(defstruct episode
  show-id       : Int
  season        : Int
  episode       : Int
  title         : String
  duration-min  : Int
  aired-ms      : Int)

(defstruct show
  id       : Int
  title    : String
  episodes : [Episode])

(defstruct library
  shows : {Int -> Show})

(defstruct watchlist
  watched : {EpisodeKey})

(defstruct progress
  show-id : Int
  watched : Int
  total   : Int
  percent : Int)
```

## Resolver Natives

```innate
@tv/library{}                                     -> Library
@tv/library-add-show{library, show}               -> Unit
@tv/library-add-episode{library, episode}         -> Unit | Error
@tv/library-airing-since{library, since-ms}       -> [Episode]
@tv/watchlist{}                                   -> Watchlist
@tv/watchlist-mark-watched{watchlist, show-id, season, episode}      -> Unit
@tv/watchlist-is-watched{watchlist, show-id, season, episode}        -> Bool
@tv/show-progress{show, watchlist}                -> Progress
@tv/next-up{show, watchlist}                      -> Episode | null
@tv/currently-watching{library, watchlist}        -> [Show]
```

## Demo

```innate
(@demo){
  @lib <- @tv/library{}
  @show <- {id: 1, title: "Breaking Bad", episodes: [
    {season: 1, episode: 1, title: "Pilot", duration-min: 47, aired-ms: ...},
    ...
  ]}
  @tv/library-add-show{library: @lib, show: @show}

  @wl <- @tv/watchlist{}
  @tv/watchlist-mark-watched{watchlist: @wl, show-id: 1, season: 1, episode: 1}
  @tv/watchlist-mark-watched{watchlist: @wl, show-id: 1, season: 1, episode: 2}

  @tv/next-up{show: @show, watchlist: @wl}
  ;; -> Episode S1E3

  @tv/show-progress{show: @show, watchlist: @wl}
  ;; -> {watched: 2, total: 6, percent: 33}
}
```

## Where

Library and watchlist MUST be separate structures — one is canonical (shared) and one is per-user (private), and conflating them creates per-user forks of canonical data. Episode identity MUST be the `(show_id, season, episode)` tuple — that's what users think with and what metadata providers emit. Progress MUST be derived from episode-level flags, NOT stored — storing aggregates creates drift when episodes are added or marked. Next-up MUST return None for fully-watched shows, NOT the last episode or first-of-a-new-season — "no next-up" is a valid state that UIs render differently (e.g., "Watched" badge vs "Resume" button). Currently-watching MUST exclude both not-started and completed shows — that's the "in the middle" row the UI wants. Marking an already-watched episode as watched MUST be a no-op, NOT an error — UIs often send mark-watched on page refresh and redundant calls must not break.
