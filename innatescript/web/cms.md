# G082 — Content Management System

> The Rosetta Stone's first **content lifecycle FSM** with revision history and scheduled publishing. Articles progress through Draft → InReview → Scheduled → Published → Archived. Every save creates a revision; revert appends rather than erases; scheduled publishes fire on tick. Composes G080's scheduler pattern, G077's state-gated operations, G081's separation of content from presentation, and G076's tagging.

```yaml
id: G082
title: Content Management System
category: web
requires: [G076-bookmarks, G077-password-safe, G080-scheduler, G081-ecard]
provides: [content-lifecycle-fsm, append-only-revision-history, scheduled-publish, archived-as-frozen, slug-as-natural-key]
```

## Insight: Content Has a Lifecycle, Not a Flag

Every prior project with stored content used simple states (public/private, active/inactive) or no state at all. G082 is the first with a **real lifecycle**: five states that a single article moves through over its life, with specific transitions allowed between specific states.

- **Draft**: author is writing. Full edit permissions.
- **InReview**: submitted for editorial review. Still editable; waiting for approval.
- **Scheduled**: approved and queued to publish at a specific future time.
- **Published**: live. Readable by the audience. Still editable (revisions continue to append).
- **Archived**: removed from view but preserved. Frozen — no more edits.

Transitions are explicit: Draft → InReview (`submit_for_review`); Draft/InReview/Scheduled → Published (`approve_and_publish`); any active → Scheduled (`schedule_publish`); any → Archived (`archive`); Archived → Draft (`unarchive`). Illegal transitions (e.g., "publish an archived article") return an error.

First Rosetta Stone project with a **lifecycle FSM that has more than three states**. G062 had two (Idle/Accepting). G073 had three (one terminal). G077 had two. G078 had three (all non-terminal). G082's five capture the phases of editorial work: thinking, reviewing, waiting, live, retired.

Every CMS in production — WordPress, Ghost, Contentful, Strapi, the noosphere's future note-publishing flow — has some version of this lifecycle. The exact state names vary; the phases don't.

## Insight: Revisions Are Append-Only

Every `save_revision` appends a new `Revision` record with an incrementing version number. Edits don't overwrite history; they add to it. `revert_to_revision(v)` restores an old version's body **by creating a new revision** with that content — not by rolling back the version counter.

This is the **Git model** for content: history is immutable, changes are new commits, reverts are new commits that undo previous ones. Once a revision is written, it's permanent; you can walk back through history and see exactly what the article said at any point.

First Rosetta Stone project where **history is append-only and reverts are additive**. G070 browser had forward-truncation on navigate (reverts lose the forward path); G078 media player had no history. G082 keeps everything. The cost is storage; the benefit is perfect auditability ("what did this article say on Tuesday?").

Production systems add compaction (delete revisions older than N years, compress old versions) but the invariant is the same: **no silent rewrites**.

## Insight: Scheduled Publish Is a Deferred Transition

`schedule_publish(id, at_ms, now_ms)` puts the article into Scheduled state with a target time. Nothing else happens immediately. Later, `tick(now_ms)` checks every Scheduled article; any whose time has arrived transitions to Published.

This is G080's scheduler pattern applied to state transitions. The scheduler lives inside the CMS rather than being a separate system, because content lifecycle is the CMS's concern — but the shape is identical: declarative "when", imperative "what", tick-driven advancement.

First Rosetta Stone project where **a state machine's transitions can be time-triggered in addition to user-triggered**. G077 had time-triggered auto-lock (the only transition that fired on a timer); G082 has both: user-triggered transitions (submit, approve, archive) and time-triggered transitions (scheduled publish). Mixing them is natural once the tick mechanism exists.

## Insight: Archived Is Frozen

Once archived, an article cannot be edited. `save_revision` on an archived article returns an error. `revert_to_revision` is also refused. You can unarchive back to Draft (which reactivates editing), but during the archived state, nothing changes.

This is **deliberate immutability at a lifecycle stage**. The alternative — allowing edits on archived content — creates a confusing user model: "is this still archived? why did the archived copy change?" Locking down the archived state removes ambiguity.

First Rosetta Stone project with **a non-terminal frozen state** — archived articles aren't dead (they can be restored), but while they're archived they're read-only. Different from G073's Disconnected (terminal) or G077's Locked (just inaccessible). Archived is visible-but-frozen.

Database parallel: soft-deleted rows with `deleted_at` timestamps are visible to admins, editable by no one. Wiki parallel: protected pages are readable but not editable without permission. G082 uses the same primitive.

## Insight: Slug Is the Natural Key; ID Is the Surrogate

Articles have both a numeric `id` (surrogate, generated) and a `slug` (natural, user-meaningful). Uniqueness is enforced on slug, not id — two articles cannot share the same slug. Lookups work either way: `article(id)` and `article_by_slug(slug)`.

Same pattern as G076 Bookmarks (URL is natural key, id is surrogate). First Rosetta Stone project with **dual keying** for the same reasons: the numeric id provides stable references for code (foreign keys in revisions, mentions in internal links) while the slug provides human-friendly identifiers (URLs, navigation, permalinks).

Production CMSes uniformly do this. Changing a slug is a renaming operation that requires either URL rewrites or a permanent 301 redirect to the new slug; the id never changes. G082 doesn't implement slug rename (a future exercise) but the structure supports it.

## Insight: Queries Are Dimension-Scoped

`by_state` returns articles in a state; `by_tag` returns articles with a tag; `by_author` returns articles by a person. Each is a linear scan with a single-field filter. More complex queries (state=Published AND tag=tutorial) compose by intersection at the caller.

Same pattern as G076 Bookmarks — single-axis filters, composition at the caller. First Rosetta Stone project where **the CMS UI's filter panel** is directly implementable by calling these queries. "Show me all my Draft articles tagged as tutorials" = `by_author("alice") ∩ by_state(Draft) ∩ by_tag("tutorial")`.

Production CMSes add full-text search, sort orders, pagination, complex boolean queries. G082 provides the primitives; the composition is the caller's job.

## Choreographic Case: Daily Editorial Automation

```innate
(@editorial-automation){
  @cms <- @cms/load

  ;; Morning: notify editors of articles awaiting review
  @every day at 09:00 {
    @in-review <- @cms/by-state{cms: @cms, state: "in_review"}
    @for article in @in-review {
      @email/notify{to: @article.editor, subject: "Awaiting review",
                     article: @article.slug}
    }
  }

  ;; Every minute: tick the CMS to publish scheduled articles
  @every 1.min {
    @newly-published <- @cms/tick{cms: @cms, now_ms: @now}
    @for id in @newly-published {
      @article <- @cms/article{cms: @cms, id: @id}
      @email/broadcast{subject: "New post: ${@article.title}",
                        body: @article.body, segment: "subscribers"}
    }
  }
}
```

Editorial automation composes on G082's lifecycle + G080's scheduler: tick the CMS periodically, the state machine advances itself, side effects fire on transitions.

## Structures

```innate
(defenum content-state Draft | InReview | Scheduled | Published | Archived)

(defstruct article
  id                         : Int
  slug                       : String          ;; natural key
  title, body, author        : String
  state                      : ContentState
  tags                       : Set<String>
  created-at-ms, modified-at-ms : Int
  scheduled-publish-at-ms    : Int?
  published-at-ms            : Int?)

(defstruct revision
  article-id, version        : Int
  title, body                : String
  saved-at-ms                : Int
  saved-by                   : String)
```

## Resolver Natives

```innate
@cms/new                                                      -> CMS
@cms/create-article{cms, author, slug, title, body, now_ms}   -> ArticleId | error
@cms/save-revision{cms, id, user, title, body, now_ms}        -> Version | error
@cms/submit-for-review{cms, id, now_ms}                       -> Ok | error
@cms/approve-and-publish{cms, id, now_ms}                     -> Ok | error
@cms/schedule-publish{cms, id, at_ms, now_ms}                 -> Ok | error
@cms/archive{cms, id, now_ms}                                 -> Ok | error
@cms/unarchive{cms, id, now_ms}                               -> Ok | error
@cms/tick{cms, now_ms}                                        -> [ArticleId]
@cms/revert-to-revision{cms, id, version, user, now_ms}       -> Version | error
@cms/set-tags{cms, id, tags, now_ms}                          -> Ok | error
@cms/article{cms, id}                                         -> Article?
@cms/article-by-slug{cms, slug}                               -> Article?
@cms/by-state{cms, state}                                     -> [Article]
@cms/by-tag{cms, tag}                                         -> [Article]
@cms/by-author{cms, author}                                   -> [Article]
@cms/revisions-of{cms, id}                                    -> [Revision]
```

## Demo

```innate
(@demo){
  @cms <- @cms/new
  @hello <- @cms/create-article{cms: @cms, author: "alice", slug: "hello-world",
                                  title: "Hello", body: "First post", now_ms: 1000}
  @cms/save-revision{cms: @cms, id: @hello, user: "alice",
                      title: "Hello", body: "Edited body", now_ms: 2000}
  @cms/submit-for-review{cms: @cms, id: @hello, now_ms: 3000}
  @cms/approve-and-publish{cms: @cms, id: @hello, now_ms: 4000}

  @announce <- @cms/create-article{cms: @cms, author: "alice",
                                     slug: "announcement",
                                     title: "Big", body: "Coming soon",
                                     now_ms: 5000}
  @cms/schedule-publish{cms: @cms, id: @announce, at_ms: 100000, now_ms: 5000}
  @cms/tick{cms: @cms, now_ms: 200000}    ;; -> [@announce]; announcement is now published
  @cms/revert-to-revision{cms: @cms, id: @hello, version: 1, user: "alice",
                           now_ms: 300000}  ;; body returns to "First post", appended as v3
}
```

## Where

Slug MUST be the natural key — two articles MUST NOT share a slug; create with a duplicate slug MUST fail. The id is a surrogate; internal references (revisions, mentions) MUST use the id, not the slug (so slug renames don't break references). Every save MUST append a new revision, never overwrite — history is append-only; revert MUST be an additive operation (copy old content into a new revision) not a rollback (delete newer revisions). Scheduled articles MUST publish on tick, not on manual intervention — the scheduler IS the transition mechanism. Archived articles MUST refuse all modifications (save_revision, revert_to_revision) — the freeze is a correctness property, not a UX preference. Only state transitions declared as legal MUST be accepted — trying to move from a non-matching state MUST return a structured error (not throw, not silently accept), so the caller can present a helpful message.
