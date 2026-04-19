# G084 — CAPTCHA Maker

> The Rosetta Stone's **closing Web project**. A CAPTCHA is an ephemeral, single-use, time-bounded challenge: issue a prompt with a known expected answer, accept answers for a limited window with a bounded retry count, refuse after solved / expired / exhausted. Composes every time-based and security-flavored primitive from earlier Web projects.

```yaml
id: G084
title: CAPTCHA Maker
category: web
requires: [G077-password-safe, G080-scheduler, G082-cms, G083-template-maker]
provides: [challenge-response-pattern, ephemeral-bounded-validity, attempt-counter-refusal, outcome-typed-verification, polymorphic-challenge-kinds]
```

## Insight: Challenges Are Ephemeral, Single-Use, Time-Bounded

G077 had the same structural primitive for the Locked→Unlocked transition (bounded time, bounded attempts). G082 had scheduled publishing (time bounds on state). G084 combines and narrows them: **the challenge IS an ephemeral authentication token**. Once solved, it's consumed. Once expired, it's refused. Once attempts are exhausted, it's refused. Only one terminal outcome is success.

First Rosetta Stone project where **the primitive explicitly encodes single-use semantics** — G077's unlock can happen many times; G082's publish happens once but the article persists; G084's challenge is like a password-reset token or a one-time-code: fire once, never again.

This is the shape of every nonce in cryptography, every OAuth authorization code, every "click this link in your email" confirmation. The token exists only to be consumed. After consumption, it's garbage. G084 models this at minimum scale.

## Insight: Four Kinds of Outcome Are Structurally Distinct

The `VerifyResult` enum has five outcomes (plus Unknown): Success, WrongAnswer (with remaining count), Expired, AlreadySolved, TooManyAttempts. Each is a **different kind of refusal**, triggered by a different condition, requiring a different UI response:

- Success: proceed.
- WrongAnswer: show "incorrect, N attempts left".
- Expired: show "session expired, request a new challenge".
- AlreadySolved: show "you've already completed this" (treat as duplicate click, not error).
- TooManyAttempts: show "maximum attempts exceeded, try a new challenge".
- Unknown: likely bug or tampering; show generic error.

Collapsing these into a single boolean (accepted/rejected) loses information the UI needs to guide the user correctly. "Wrong answer" and "expired" are both rejections but mean different things to the user — retry makes sense for one and not the other.

First Rosetta Stone project where **the refusal outcome is itself structured data**. G081 had validation errors as enum variants; G084 extends this to the per-request verification path: every way a request can fail is a distinct kind, discoverable by the caller.

Production APIs converge on this pattern. OAuth returns `invalid_grant`, `invalid_client`, `access_denied`, `expired_token` — each triggers different client behaviour.

## Insight: Attempt Counter Is a Rate Limit

The `max_attempts` + `attempts_used` pair is a **rate limit at the token level**. Infinite-attempts is a brute-force vulnerability (attacker tries every possible answer until they hit the right one). A 3-attempt limit means the attacker has a 3/(space_size) chance per token, and if the space is large enough (all 1-20 math, thousands of word recalls, etc.) the expected attempts-per-success is prohibitively high.

This is the same pattern as login attempt limits (lock account after N failures), API rate limits (429 after N requests), and password-safe auto-lock (G077, but time-based rather than count-based). G084 combines both: bounded by count AND by time.

First Rosetta Stone project with **a dual-bound refusal**: either expiry or attempt-count triggers refusal independently. The challenge is invalid once EITHER bound is exceeded. This is how most security tokens work (OAuth tokens expire by time AND are revocable by use-count or explicit revocation).

## Insight: Challenge Kinds Are Polymorphic Through One Generator

G079 had the `use` command dispatching to different handlers based on item kind. G084 extends this to the *generator* side: `issue(kind)` dispatches to a kind-specific generator that produces a (prompt, expected) pair. Four kinds — Math, WordRecall, ReverseString, SequenceCount — produce different challenge types through one API.

First Rosetta Stone project where **challenge generation is dispatched at the construction boundary** based on kind. The caller picks the challenge style; the service generates an appropriate prompt; verification works the same way regardless of kind because the expected answer is stored.

This matters for user experience. Math challenges exclude non-native speakers; word recall excludes visually impaired users; reverse-string excludes dyslexic users. **Offering multiple kinds** lets the frontend pick based on accessibility preferences. G084 provides the mechanism; accessibility-aware selection is the caller's job.

## Insight: Answer Matching Is Lenient

`answers_match(expected, user)` trims whitespace and lowercases both sides. "5" and " 5 " match; "Butterfly" and "BUTTERFLY" match. This is deliberate: typing a CAPTCHA answer is already user-hostile, and nit-picking case / whitespace makes it worse for no security gain.

First Rosetta Stone project where **input normalisation is part of the correctness contract**. G071 forgave malformed HTML (input from external systems); G080 forgave missed schedule windows (input from time drift); G084 forgives trivial input variations from the user. In each case the forgiveness is **scoped to the specific noise the domain produces** — whitespace and case in user text entry, not punctuation (which might matter: "9.5" vs "9,5" are different answers in different locales).

Production captcha systems (reCAPTCHA, hCaptcha) go further with phonetic variants, common-typo matching, etc. G084 picks the minimum reasonable level.

## Insight: Cleanup Is a Deliberate Garbage Collection

The service retains challenges in a list; `cleanup(now_ms)` removes solved and expired ones. Without cleanup, the list grows without bound. Calling cleanup periodically (cron-style, using G080's scheduler) keeps memory bounded.

Alternative: auto-cleanup on every `verify` and `issue`. Simpler but makes those hot paths pay the scan cost. G084 decouples: verify/issue are O(n) due to linear search (in production, a map would make them O(1)), and cleanup is an explicit call the operator schedules.

First Rosetta Stone project where **garbage collection is explicit and caller-scheduled**. Every prior project either retained indefinitely (G064 Family Tree, G076 Bookmarks) or auto-evicted (G075 Bandwidth Monitor's ring buffer). G084 adds the third model: **retain until explicit sweep**.

## Insight: Closes the Web Category — Sixteen Components

The Web category's architecture, complete:

1. **Structured model** (G069 WYSIWYG, runs + attrs)
2. **Render function** (G069 to_html, G074 to_svg, G081 template substitution)
3. **Query language** (G071 selectors, G076 multi-axis, G082 by_state/tag/author)
4. **Robust input handling** (G071 forgiving parser, G084 lenient matching)
5. **Durable state** (G072 atomic rename + resume)
6. **Interactive protocol** (G073 line-oriented, state-gated commands)
7. **Spatial data** (G074 2D coords, bbox, proximity)
8. **Time-series observation** (G075 counter-to-rate)
9. **Multi-axis organisation** (G076 folders + tags orthogonal)
10. **Security primitives** (G077 encryption at rest, lock/unlock, auto-lock)
11. **Tick-driven engines** (G078 media player, G082 scheduled publish, G084 time-bounded challenges)
12. **Simulation world** (G079 graph + turn-based)
13. **Scheduled execution with history** (G080)
14. **Template-data rendering** (G081)
15. **Content lifecycle with revision history** (G082)
16. **Authoring tools** (G083 template maker)

G084 is the synthesis: combines time bounds (10), outcome-typed results (3, 16), polymorphic dispatch (via kind), attempt-counter refusal (10), and garbage collection. It doesn't add a new architectural component — it uses every existing one.

The category is done. 16 projects, 16 components, all composable. The noosphere has its full Web-layer vocabulary.

## Choreographic Case: Signup Gate

```innate
(@signup-gate){
  @service <- @captcha/new{ttl_ms: 120000, max_attempts: 3, seed: @rand}

  @on-signup-form-requested (@session){
    @c <- @captcha/issue{service: @service, kind: "math", now_ms: @now}
    @ui/render{prompt: @c.prompt, challenge_id: @c.id}
  }

  @on-user-submits (@session, @challenge_id, @answer){
    @result <- @captcha/verify{service: @service, id: @challenge_id,
                                user_answer: @answer, now_ms: @now}
    case @result.kind {
      "success" => @proceed-with-signup{@session}
      "wrong_answer" => @ui/show-error{msg: "wrong (${@result.attempts_remaining} left)"}
      "expired" => @ui/show-error{msg: "expired; refreshing"}
                    @redirect-to-signup
      "too_many_attempts" => @ui/show-error{msg: "too many tries"}
                              @redirect-to-signup
      "already_solved" => @proceed-with-signup{@session}   ;; idempotent double-click
      "unknown" => @ui/show-error{msg: "unknown challenge"}
    }
  }

  @every 5.min {
    @captcha/cleanup{service: @service, now_ms: @now}
  }
}
```

A signup flow composes on G084 directly: issue on form request, verify on submit, dispatch on outcome kind, periodically sweep.

## Structures

```innate
(defenum challenge-kind Math | WordRecall | ReverseString | SequenceCount)

(defstruct challenge
  id                : Int
  kind              : ChallengeKind
  prompt, expected  : String
  created-at-ms     : Int
  expires-at-ms     : Int
  attempts-used     : Int
  max-attempts      : Int
  solved            : Bool)

(defenum verify-kind
  Success | WrongAnswer | Expired | AlreadySolved | TooManyAttempts | Unknown)

(defstruct verify-result
  kind              : VerifyKind
  attempts-remaining : Int?)

(defstruct service
  challenges        : [Challenge]
  default-ttl-ms    : Int
  max-attempts      : Int
  rng-state         : Int)
```

## Resolver Natives

```innate
@captcha/new{ttl_ms, max_attempts, seed}           -> Service
@captcha/issue{service, kind, now_ms}              -> Challenge
@captcha/verify{service, id, user_answer, now_ms}  -> VerifyResult
@captcha/cleanup{service, now_ms}                  -> Int   ;; removed count
@captcha/challenge{service, id}                    -> Challenge?
@captcha/active-count{service}                     -> Int
```

## Demo

```innate
(@demo){
  @s <- @captcha/new{ttl_ms: 60000, max_attempts: 3, seed: 1234}
  @c <- @captcha/issue{service: @s, kind: "math", now_ms: 1000}
  ;; @c.prompt = "What is X + Y?"  (deterministic for seed=1234)
  @captcha/verify{service: @s, id: @c.id, user_answer: @c.expected, now_ms: 2000}
    ;; -> {kind: "success"}
  @captcha/verify{service: @s, id: @c.id, user_answer: "anything", now_ms: 3000}
    ;; -> {kind: "already_solved"}

  @c2 <- @captcha/issue{service: @s, kind: "math", now_ms: 3000}
  @captcha/verify{service: @s, id: @c2.id, user_answer: "0", now_ms: 4000}
    ;; -> {kind: "wrong_answer", attempts_remaining: 2}

  @c3 <- @captcha/issue{service: @s, kind: "math", now_ms: 5000}
  @captcha/verify{service: @s, id: @c3.id, user_answer: @c3.expected,
                    now_ms: 100000}
    ;; -> {kind: "expired"}
}
```

## Where

Each verification outcome MUST be a distinct enum variant — collapsing Expired, AlreadySolved, TooManyAttempts into a single "rejected" loses information the UI needs to guide the user. Challenge state (attempts_used, solved) MUST update on every verify call, even rejected ones — otherwise the attempt counter is bypassable by replaying verifications. The attempt counter MUST NOT reset when the attempt is accidentally submitted wrong (network blip, user typo); 3 attempts is the whole budget for the whole challenge's lifetime. Expiration MUST be checked on every verify call with the current `now_ms`, not cached at issue time — if `now_ms` drifts past `expires_at_ms` between issue and first verify, the challenge MUST already be expired. The RNG MUST be seeded (deterministic for tests) but its state MUST be isolated per service instance so parallel services don't produce duplicate challenges. Answer comparison MUST normalise trivial input variations (trim whitespace, lowercase) but MUST NOT attempt semantic normalisation (typo-correction, phonetic matching) at this level — those belong to a higher layer.
