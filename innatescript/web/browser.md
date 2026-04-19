# G070 — Web Browser with Tabs

> The Rosetta Stone's first project with **two orthogonal undo-like stacks** — per-tab back/forward history and browser-scoped closed-tab recovery. The first project where **scope of state** is the central design decision: which state belongs to a tab, which belongs to the browser, and which operations cross the boundary.

```yaml
id: G070
title: Web Browser with Tabs
category: web
requires: [G062-vending-machine, G069-wysiwyg]
provides: [per-tab-history, closed-tab-stack, scoped-undo, state-scope-orthogonality]
```

## Insight: Scope Is the Central Design Decision

Every piece of state in the browser belongs to exactly one scope:
- **Per-tab**: URL, back stack, forward stack, scroll position, cookies-for-this-origin-in-this-tab (session cookies).
- **Browser-wide**: bookmarks, downloads, history (the searchable global one, distinct from per-tab back/forward), persistent cookies, user settings.

Getting the scope wrong is a design bug that shows up as user-surprise moments. "Why did closing this tab lose my bookmark?" — because you put bookmarks in tab scope. "Why does Back go to a page from a different tab?" — because you put back-stack in browser scope. G070 makes the scope explicit and the tests enforce it: closing a tab preserves bookmarks, per-tab history is independent across tabs, closed-tab stack survives across tab-switches.

This is the first Rosetta Stone project where **"which thing owns this state?" is itself the load-bearing lesson**. The vault/noosphere will have the same question constantly: does this metadata live on the note or on the project? Does this preference live on the agent or on the session? G070 is the minimal case where the answer determines correctness.

## Insight: Per-Tab History Is Independent Timelines

Each tab's back/forward stacks are completely independent of every other tab's. Alice can navigate Tab 1 from `a → b → c`, switch to Tab 2, navigate `x → y`, switch back to Tab 1, and click Back — and land on `b`, not `y`. The per-tab stacks are **timelines** in the sense of branching timelines, not shared state.

This matters because the opposite — a single shared navigation stack — is what early browsers had, and it was awful. Every tab-switch was a navigation event; Back went to whatever you last clicked regardless of which tab. Modern browsers switched to per-tab history and nobody misses the old model. G070 implements only the modern one because only one design is defensible.

Analogues in the vault: each open note has its own cursor position, scroll offset, and folding state — not shared across notes. Each agent conversation has its own context history — not shared with other conversations. Each choreography execution has its own step-by-step state — not shared with other choreographies. G070 presents the pattern in a domain everyone recognises.

## Insight: Navigate Truncates Forward — The "Branching Timelines" Pattern

When the user clicks Back and then navigates to a new URL, the forward stack is **cleared**. The previous timeline (where they were going to visit c after b) is gone; they've branched onto a new path.

This is correct behaviour, and it's subtle. Naively you might think "keep the old forward stack; the user can push Forward to get back to it." But that gives the user two possible futures and only one current state, which is semantically incoherent — they're not on the old timeline anymore. Every browser, editor with undo, and version-control system works this way: redo/forward is lost as soon as you branch.

Git reflog and the closed-tab stack are the workarounds for this. They let you recover a timeline you've branched away from, but only because they live in a different scope: they're recovery mechanisms, not first-class history. G070 has both: the per-tab forward stack (first-class, truncated on navigate) and the browser-level closed-tab stack (second-class recovery, survives navigate).

## Insight: Closed-Tab Recovery Is a Second, Orthogonal Undo Stack

Pressing Ctrl+Shift+T reopens the last-closed tab with its URL and full per-tab history intact. This is a completely separate undo mechanism from per-tab back/forward:

- Per-tab back/forward: within a tab, navigate between URLs I've visited.
- Closed-tab recovery: within the browser, undo the decision to close a tab.

The scopes are different; the lifecycles are different; the storage is different. They do NOT interact. Closing a tab doesn't alter any other tab's history. Reopening a closed tab doesn't touch the browser-wide bookmark list.

First Rosetta Stone project where **two undo-like stacks coexist with different scopes and different semantics**. Vim has this: the undo tree lives per-buffer; the `:bdelete` unbuffered-buffer stack is global. The noosphere will eventually have this: per-choreography step-undo for aborting a step, separate from global session-recovery for recovering a cancelled choreography. G070 is the minimal teaching case.

## Insight: Active Tab Is a Pointer, Not a State

Which tab is active is a single `Option<TabId>` on the browser. No per-tab "am I active" flag — that would be redundant and couldable go out of sync with the browser's pointer. The single source of truth is the browser's active pointer; every "is tab X active?" query consults it.

This is the DRY / single-source-of-truth principle applied to concurrent state. First Rosetta Stone project where **the same question is answerable from two places and one is explicitly chosen as authoritative**. Parallels: the filesystem's working-directory pointer (no per-directory "am I the CWD" flag); the terminal's focused window (no per-window focus flag); the vault's "current note" is a UI-level pointer, not a note-level property.

When a tab is closed, the pointer must be updated — pick the next tab (most common UX), or the previous if there is no next, or null if the browser is now empty. G070 implements all three cases explicitly, as the pointer must stay in valid states or every operation on `active_tab` crashes. This is the cost of making the pointer authoritative: every mutation must maintain its invariant.

## Choreographic Case: Multi-Research Session

```innate
(@research-session){
  @browser <- @web/new-browser

  @tabs <- @for topic in @research-topics {
    @web/open-tab{browser: @browser, url: @search-url{topic}}
  }

  @for t in @tabs {
    @web/switch-to{browser: @browser, id: @t}
    @for result in @top-results{n: 5} {
      @web/navigate{browser: @browser, url: @result.url}
      @content <- @scrape{page: @result.url}
      where { relevant: @content.relevance > 0.7 }
      @web/add-bookmark{browser: @browser}
    }
  }

  @bookmarks <- @web/bookmarks{browser: @browser}
  @report <- @synthesize{sources: @bookmarks}
}
```

Multi-tab research composes naturally on G070's primitive: open a tab per topic, navigate through results within each tab, bookmark the relevant ones. The bookmarks accumulate at browser scope; per-tab history remains independent.

## Structures

```innate
(defstruct tab
  id        : Int
  title     : String
  url       : String
  back      : [String]
  forward   : [String])

(defstruct closed-tab
  url       : String
  title     : String
  back      : [String]
  forward   : [String])

(defstruct browser
  tabs      : [Tab]
  active    : TabId?
  closed    : [ClosedTab]    ;; newest first, LIFO reopen
  bookmarks : [(String, String)]
  next-id   : Int)
```

## Resolver Natives

```innate
@web/new-browser                          -> Browser
@web/open-tab{browser, url}               -> TabId
@web/close-tab{browser, id}               -> Unit
@web/switch-to{browser, id}               -> Bool
@web/active-tab{browser}                  -> Tab?
@web/navigate{browser, url}               -> Bool
@web/back{browser}                        -> String?
@web/forward{browser}                     -> String?
@web/reopen-closed{browser}               -> TabId?
@web/add-bookmark{browser}                -> Bool
@web/bookmarks{browser}                   -> [(String, String)]
```

## Demo

```innate
(@demo){
  @b <- @web/new-browser
  @t1 <- @web/open-tab{browser: @b, url: "https://example.org"}
  @web/navigate{browser: @b, url: "https://example.org/about"}
  @web/navigate{browser: @b, url: "https://example.org/contact"}

  @t2 <- @web/open-tab{browser: @b, url: "https://news.example.com"}
  @web/navigate{browser: @b, url: "https://news.example.com/story/1"}
  @web/add-bookmark{browser: @b}

  @web/switch-to{browser: @b, id: @t1}
  @web/back{browser: @b}                 ;; -> "https://example.org/about"

  @web/close-tab{browser: @b, id: @t2}
  @web/reopen-closed{browser: @b}        ;; -> new tab id, url restored
}
```

## Where

Per-tab history MUST be independent across tabs — switching tabs does NOT alter any tab's back or forward stack. Navigate MUST truncate the forward stack — once the user branches, the old future is unreachable (closed-tab recovery is the workaround, not forward-preservation). The active-tab pointer is the single source of truth for "which tab is current"; no per-tab `is_active` flag. Closing a tab MUST push it onto the closed-tab stack (LIFO); reopen-closed MUST restore the tab with its URL AND per-tab history intact. Bookmarks MUST be browser-scoped — closing a tab MUST NOT remove bookmarks, even bookmarks added from that tab. When the active tab is closed, activation MUST move to a sibling (next tab, or previous if there is no next, or none if the browser is empty) — leaving `active` pointing at the just-closed tab is a bug.
