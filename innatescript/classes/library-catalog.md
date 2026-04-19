# G053 — Library Catalog

> Three layers of identity. Subjects form a tree. Holds form a FIFO queue. Renewal is a mutation of an obligation already in flight.

```yaml
id: G053
title: Library Catalog
category: classes
requires: [G049-movie-store, G050-reservation-system]
provides: [three-layer-identity, tree-prefix-query, fifo-hold-queue, in-flight-renewal]
```

## Insight: Three Layers of Identity

G048 had one entity type. G049 had two (movie and customer, fungible copies, rentals joined them). G050 had two (resource and customer, specific resources, reservations joined them). G053 has **three**:

- **Title** — the abstract work. "1984" by Orwell. Identity persists across editions, reprints, translations.
- **Copy** — a physical item at a branch with its own barcode. Has its own state: available, checked out, lost, withdrawn.
- **Loan** — the temporal claim on one copy by one patron.

G049's movie copies were fungible — no one cared which of the two Casablanca copies you got. G053's copies have identity because they can be lost, damaged, moved, withdrawn *individually*. Copy-level state matters. A title with three copies, one lost, two available, is a different catalog record from a title with three available copies.

This three-layer pattern appears throughout the noosphere. A *project* (abstract work), its *phases* (specific instances with their own state), and the *tasks* within each phase (temporal claims on agent attention). A *conversation thread* (abstract channel), its *messages* (specific posts with their own state), and *reactions* (temporal claims attached to messages). Every rich vault entity is a Title/Copy/Loan triple in disguise.

## Insight: Subjects Are a Tree, Queried by Prefix

Dewey Decimal isn't a flat category list. It's a **prefix-closed namespace**. `"500"` is natural sciences. `"512"` is algebra. `"512.5"` is a specific algebraic subfield. A title classified at `"500.512.5"` sits three levels deep. Asking "show me everything in 500" means "all titles whose subject_path starts with `500`" — a tree-prefix query, not an equality lookup.

Same algebra as the vault's wiki-link structure. `[[The Forge]]` is a namespace; `[[The Forge/Temporal]]` is a subspace; `[[The Forge/Temporal/Daily Notes]]` is a leaf zone. Asking "everything under The Forge" is a prefix query. The temporal calendar itself is a tree: year → quarter → month → week → day.

The `.` separator in the subject path is informationally the same as `/` in a vault path: both are tree delimiters. Prefix queries over either are the same operation. Every `@search{scope: "The Forge/..."}` in an InnateScript choreography is a Dewey-style lookup wearing different punctuation.

## Insight: Holds Are a FIFO Queue With a Wakeup Trigger

When all copies of a title are out and a patron wants it, they join a queue. First-come, first-served by default. When a copy is returned, the front of the queue wakes up: their hold becomes `fulfilled`, attached to the returned copy.

This is the first project in the Rosetta Stone with a **passive waiting primitive with automatic fulfillment**. G050's reservation system was explicit — someone called `book` and either got it or didn't. G053's holds sit in a queue until state elsewhere changes (a return happens) and the queue wakes the front.

This is the shape of every task queue, email inbox, interrupt handler, and patient-waiting-room in the noosphere. Kathryn's pending trade orders form a FIFO queue until market conditions fire; Sarah's pending task assignments form a FIFO queue until an agent becomes available; the nightly summary forms a FIFO queue of the day's completions. The library hold is where the noosphere's wait-and-wake primitive becomes concrete.

**The wakeup is not free — it's coupled to the return event.** When `return_copy` runs, it does two things atomically: mark the copy available, and fulfill the next hold. If those two weren't coupled, a race would open a gap where the copy is available but no one has been notified. G052 taught us atomic multi-entity updates; G053 uses the pattern to keep the queue consistent with shelf state.

## Insight: Renewal Is a Mutation of an Obligation in Flight

G050 had two lifecycle transitions for a reservation: cancel (retract a future claim) and complete (the interval ended). G053 introduces a third: **renew** — the claim continues, but its deadline moves forward.

This is qualitatively new. Previous obligations were monotone — once the interval was set, you could retract or complete it, not extend. Renewal is *in-flight mutation of a deadline*. It's different from rescheduling (which moves both start and end). It's different from cancellation (which ends the claim). It's mid-flight extension.

And it has a **coordination precondition**: renewal fails if other patrons have holds. You can't extend your claim at the expense of a queue behind you. This is the first project where a lifecycle transition is gated on the *state of a separate entity set* — the hold queue. The precondition isn't about the loan's own invariants; it's about the downstream consumers' expectations.

In the noosphere: a choreography running past its scheduled end-time can "renew" if nothing downstream is waiting. If a downstream choreography has a hold on the same agent or resource, the renewal is blocked. The library's renewal rule is the general shape of "yield to the queue." This prevents the familiar anti-pattern where the current user keeps extending indefinitely while newcomers wait.

## Choreographic Case: Book Club Coordinator

```innate
(@book-club-coordinator){
  @title  <- @catalog/search-by-name{query: "1984"}.first
  @copies <- @catalog/copies-of{title-id: @title.id}

  concurrent {
    @for each member in @club {
      @hold <- @catalog/place-hold{title-id: @title.id, patron-id: @member.id}
      @queue_position <- @catalog/hold-queue{title-id: @title.id}.index_of(@hold.id)
      @sarah/notify{member: @member, position: @queue_position}
    }
  } join as @queue

  where {
    all_placed:        @queue.every(.hold.status == waiting)
    queue_monotonic:   @queue.is_sorted_by(.placed_at)
    positions_unique:  @queue.map(.position).distinct.length == @queue.length
  }
}
```

Members place holds; positions reported. The `where` ensures the queue is correctly formed. When copies return, fulfillment happens automatically — the coordinator doesn't need to poll. This is the wait-and-wake primitive used at the choreography level.

## Structures

```innate
(defstruct title
  id           : String
  name         : String
  authors      : [String]
  subject-path : String)                          ;; dot-delimited Dewey

(defstruct copy
  id        : String
  title-id  : String
  location  : String
  status    : "available" | "checked-out" | "lost" | "withdrawn")

(defstruct loan
  id             : Nat
  copy-id        : String
  patron-id      : String
  checked-out-at : Timestamp
  due-at         : Timestamp
  returned-at    : Timestamp?
  renewal-count  : Nat)

(defstruct hold
  id                 : Nat
  title-id           : String
  patron-id          : String
  placed-at          : Timestamp
  status             : "waiting" | "fulfilled" | "cancelled"
  fulfilled-copy-id  : String?)
```

## Resolver Natives

```innate
@catalog{}                                                   -> LibraryCatalog
@catalog/add-title{title}                                    -> LibraryCatalog
@catalog/add-copy{copy}                                      -> LibraryCatalog
@catalog/check-out{copy-id, patron-id, duration-days}        -> Loan
@catalog/return-copy{loan-id}                                -> (Loan, Hold?)
@catalog/renew{loan-id, additional-days}                     -> Loan
@catalog/place-hold{title-id, patron-id}                     -> Hold
@catalog/cancel-hold{hold-id}                                -> Hold
@catalog/hold-queue{title-id}                                -> [Hold]
@catalog/available-copies{title-id}                          -> [Copy]
@catalog/search-by-subject{prefix}                           -> [Title]    ;; tree-prefix
@catalog/search-by-author{name}                              -> [Title]
@catalog/overdue{now}                                        -> [Loan]
```

## Demo

```innate
(@demo){
  @cat <- @catalog{}
    .add-title{id: "T1", name: "1984", authors: ["Orwell"], subject-path: "800.813"}
    .add-copy{id: "C1", title-id: "T1"}
    .add-copy{id: "C2", title-id: "T1"}
    .add-patron{id: "P1", name: "Nathan"}
    .add-patron{id: "P2", name: "Korrallan"}

  @l1 <- @cat/check-out{copy-id: "C1", patron-id: "P1"}
  @l2 <- @cat/check-out{copy-id: "C2", patron-id: "P2"}
  @h1 <- @cat/place-hold{title-id: "T1", patron-id: "P1"}

  ;; renewing l1 fails: h1 is waiting
  ;; @cat/renew{loan-id: @l1.id}  -> error: blocked by hold

  @l2.returned <- @cat/return-copy{loan-id: @l2.id}
  ;; h1 now fulfilled to copy C2 — automatically, atomically with the return
}
```

## Where

The three identity layers (title/copy/loan) MUST be distinct records. Subject search MUST treat the path as a tree, matching by prefix. The hold queue MUST be FIFO, and return MUST atomically fulfill the next waiting hold in the same operation. Renewal MUST refuse when waiting holds exist on the title. Those four rules are what separates a library catalog from a reservation system wearing different labels.
