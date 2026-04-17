# G039 — Mail Checker

> Monitor an inbox for new messages with filtering and notification.

```yaml
id: G039
title: Mail Checker
category: networking
requires: [G023-post-it-notes, G026-news-ticker, G032-regex-query, G036-fetch-weather]
provides: [polling-pattern, inbox-model, filter-pipeline, notification-callback]
```

## Insight: Polling Is the Alarm Clock Applied to External State

G011 introduced the alarm clock — a temporal trigger that fires at a scheduled time. G036 introduced live external data — facts that change continuously. The mail checker combines both: **poll an external source on a schedule, report what changed since the last check.** This is the temporal alarm (G011) driving a fact-freshness check (G036) with change detection (G038's scan comparison).

The pattern is universal. The daily note template IS a mail check: "what happened since yesterday?" The `(@SarahLin){task_audit}` line in the daily note is a poll — check the task table, filter for changes, report what's new. Every recurring check in the vault is a mail checker with a different inbox.

## Insight: The Inbox Is a Filtered Priority Stream

G026 (News Ticker) introduced priority streams with TTL. The inbox extends this: messages arrive unsolicited (push), accumulate until checked (queue), and can be filtered by sender, subject, content, and read-state. The inbox is a priority stream where the consumer controls when they look and what they look at.

The filter pipeline is G013's progressive gates applied to content: sender filter (cheap, string match) → subject filter (cheap) → body filter (moderate, substring search) → since filter (date comparison). Each gate reduces the result set. The `where` for an inbox check isn't "did mail arrive?" — it's "did mail arrive that matters."

## Insight: Read-State Is Agent Attention

Marking a message as read means "I have processed this." The unread count is the attention debt — how much input the agent hasn't processed yet. This connects to G021's cursor as attention: the cursor tracked *where* in a document the agent was focused. Read-state tracks *whether* the agent has engaged at all. Both are attention models at different granularities.

Flagging extends this: "I processed this AND it requires future action." Read-state is binary attention. Flagging is prioritized attention. Together they form a three-state attention model: unprocessed (unread) → acknowledged (read) → actionable (flagged). This is the agent's triage protocol.

## Insight: Notification Callbacks Are Event-Driven Choreography

The `on_new_mail(callback)` pattern inverts the polling model: instead of "check every N minutes," the mailbox pushes to the agent when something arrives. This is event-driven rather than poll-driven. Both models coexist — polling for batch review, callbacks for urgent items.

In InnateScript, this maps to the difference between `@alarm{every: 15min} -> @check_mail` (scheduled polling) and `@inbox{on_new: @handler}` (event-driven). The daily note uses polling — check once per section. The chathud uses events — new messages appear immediately. Same data, different temporal models.

## Choreographic Case: Morning Briefing as Mail Check

```innate
(@morning-briefing){
  concurrent {
    @finance <- @kathryn/inbox{filter: {since: @yesterday, unread_only: true}}
    @engineering <- @jay/inbox{filter: {since: @yesterday, unread_only: true}}
    @editorial <- @lena/inbox{filter: {since: @yesterday, unread_only: true}}
  }
  join {
    @briefing <- @compose{
      finance: @finance.matched,
      engineering: @engineering.matched,
      editorial: @editorial.matched
    }
  }
  where {
    all_checked: @finance.checked AND @engineering.checked AND @editorial.checked
    nothing_missed: @finance.new == 0 AND @engineering.new == 0 AND @editorial.new == 0
    // After the briefing runs, all inboxes should show zero new messages.
    // If new > 0 after check, the briefing missed something that arrived during assembly.
  }
}
```

The morning briefing is three mail checks running concurrently, joined into a composite view. Each inbox is independently filtered. The `where` ensures nothing slipped through.

## Structures

```innate
(defstruct email-address
  name    : String
  address : String)

(defstruct mail-message
  uid        : String
  sender     : EmailAddress
  recipients : [EmailAddress]
  subject    : String
  body       : String
  date       : Instant
  is-read    : Bool
  is-flagged : Bool)

(defstruct mail-filter
  sender-contains  : String?
  subject-contains : String?
  body-contains    : String?
  is-unread-only   : Bool
  since            : Instant?)

(defstruct check-result
  total-messages   : Nat
  new-messages     : Nat
  matched-messages : [MailMessage]
  checked-at       : Instant)
```

## Resolver Natives

```innate
@inbox{owner: String}                           -> Mailbox
@inbox/check{filter: MailFilter?}               -> CheckResult
@inbox/mark-read{uid: String}                   -> Bool
@inbox/mark-flagged{uid: String, flagged: Bool}  -> Bool
@inbox/unread-count                              -> Nat
@inbox/stats                                     -> MailboxStats
```

## Demo

```innate
(@demo){
  @server <- @mail-server{}
  @nathan <- @server/create-mailbox{owner: "nathan@em.com"}
  @server/send{
    from: @email{name: "Kathryn Lyonne", address: "kathryn@em.com"},
    to: [@email{address: "nathan@em.com"}],
    subject: "Q2 Finance Positions",
    body: "Tracking ahead of pace. Captivate and Anthropic covered."
  }
  @result <- @nathan/check{filter: {sender-contains: "em.com"}}
  @print{@result.summary}
  ;; => Total: 1, New: 1, Matched: 1
  ;;    ● kathryn@em.com  Q2 Finance Positions
}
```
