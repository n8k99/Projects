# G077 — Password Safe

> The Rosetta Stone's first project with **encryption at rest** — data stored in the safe is unreadable without a key derived from the master password. The key lives only in memory, only while the safe is unlocked; locking wipes it. First project where **forgetting the master password means losing the data** — zero-knowledge is the design, not a bug.

```yaml
id: G077
title: Password Safe
category: web
requires: [G031-cd-key-generator, G062-vending-machine, G073-telnet]
provides: [encryption-at-rest, key-derivation, lock-unlock-fsm, auto-lock-timer, password-generation, strength-scoring]
```

> **NOT A REAL PASSWORD MANAGER.** G077 uses a byte-sum pseudo-hash and XOR keystream to demonstrate the *structure* of encryption-at-rest without pretending to be cryptographically secure. A production version would use Argon2id for key derivation and AES-GCM or ChaCha20-Poly1305 for authenticated encryption. The STRUCTURE is faithful; the CRYPTOGRAPHY is pedagogical.

## Insight: The Key Lives Only When Unlocked

The safe has two states: Locked (key is absent) and Unlocked (key is in memory). When Locked, there is no way to decrypt the stored passwords — the ciphertext is on disk / in the struct, the key is not. Locking wipes the key; unlocking re-derives it from the master password plus the salt.

This is the fundamental pattern of **encryption at rest**. The encrypted data is always present (durable storage); the decryption key is present only during active sessions (ephemeral). When the session ends (process exits, safe locks, auto-lock fires), the key disappears and the ciphertext is the only state — indecipherable without re-authentication.

First Rosetta Stone project where **memory state and persistent state are deliberately different**. G072 had durable state on disk; G077 has *two kinds* of state — durable ciphertext and ephemeral key — with a strict rule about when each exists. The noosphere's future encrypted-vault layer will use this pattern: documents encrypted at rest, key derived on each unlock, auto-locked on idle.

## Insight: Zero-Knowledge Is the Feature

The safe has NO way to recover the master password. There is no "email me a reset link," no security-question escape hatch, no admin override. The master password is the only way in; losing it loses the data.

This is **zero-knowledge storage** — the safe doesn't know the master, can only verify it (via the verifier hash). Every real password manager works this way: 1Password, Bitwarden, KeePass all explicitly disclaim recovery. The feature this enables — "even if the database leaks, attackers can't decrypt without the master" — is the entire point of having a password manager.

First Rosetta Stone project where **the system explicitly refuses a recovery path** as a security property. Every prior project had some way to recover state from other state (rebuild from log, re-derive from source, reparse from storage). G077 has no such path by design.

The noosphere's design will face this tension repeatedly: more recovery paths = more attack surface; fewer recovery paths = more user frustration. Production password managers pick "no recovery" because the alternative is worse. The vault may pick differently depending on the specific data — journal entries probably want recovery; encrypted credential stores probably don't.

## Insight: Lock/Unlock Is an FSM (Same Pattern as G073)

G073 Telnet had Unauthenticated → Authenticated → Disconnected. G077 has Locked → Unlocked with no terminal state — the user can lock and unlock as many times as they want (or as often as auto-lock fires). Same primitive (state-gated operations), different lifecycle.

Every operation is gated on state: `add_entry`, `get_password`, `remove_entry`, `search` all require Unlocked; all fail cleanly (return None / false / empty) when Locked. `get_summary` is the interesting exception — metadata (service name, username, notes) can be read without the key, because they're stored as plaintext. Only the password itself is encrypted.

First Rosetta Stone project where **some fields are encrypted and some are not within the same record**. This is a real production choice: encrypting the service name hurts search without adding much security (if you know the user has a GitHub account, encrypting that fact doesn't hide it from anyone). Encrypting the password is load-bearing. Encrypting every field uniformly is wasteful.

## Insight: Auto-Lock Is a Time-Based State Transition

The Unlocked state has a **timeout**. If `auto_lock_after_ms` elapses without activity, the safe auto-locks. Activity (adding, reading, searching) resets the timer. This is **time as an implicit state-transition trigger** — same family as G062 Vending Machine's state transitions but fired by the clock, not by user action.

First Rosetta Stone project with **time as a state-transition trigger**. Previous projects had time as an input (G065 elapsed seconds, G075 rate calculations) but never as the cause of a state change. G077 introduces the pattern: the FSM has an edge triggered by "enough time passed since last activity."

Parallels: HTTP session timeouts, SSH idle disconnect, OS screen-lock on inactivity, vault's future "auto-dim" when unread notes exceed threshold. All are time-triggered transitions. The design choice is always the duration and the reset-on-activity policy.

## Insight: Password Generation and Strength Are Orthogonal Concerns

`generate_password(policy, seed)` produces a password matching a declarative policy (length, character classes). `password_strength(pw)` judges any string. They are **separate functions** — generation doesn't check strength, and strength doesn't care where the string came from.

Why separate? Because users often paste their own passwords into the safe ("I want to save my existing Gmail password, don't rewrite it"). The strength score should work on those passwords too, not just generated ones. And generation might produce a password that scores Weak if the policy is too restrictive (length 6, lower-only) — the user should see that score and tighten the policy.

First Rosetta Stone project where **two related-but-orthogonal concerns are deliberately decoupled**. G074 had render as projection of model; G075 had rate as projection of counter. G077 has strength as a *separate function*, not a method on Password — any string is judgeable, not just passwords from this safe.

## Insight: The Verifier Hash Is Not the Key

Two different derivations from the same master password:
- **Verifier**: stored in the safe; compared to a fresh hash on unlock attempt. Used only for yes/no authentication.
- **Key**: derived fresh each unlock, held in memory, used to encrypt/decrypt passwords.

They are **different functions** of (master, salt) so neither can be derived from the other. An attacker who steals the safe's stored state gets the verifier and the ciphertext, but cannot:
- Derive the key from the verifier (different derivation).
- Brute-force directly against the ciphertext (that's offline and unbounded).

Real KDFs (Argon2id, scrypt, PBKDF2) handle this with a single output that's then split, but the principle is the same: verification and encryption use different derived values so stealing one doesn't give you the other.

First Rosetta Stone project where **two outputs are derived from the same secret and are NOT interchangeable**. Subtle but critical; forgetting this distinction in a real crypto design is a headline-making vulnerability.

## Insight: Summaries Never Include the Password

`get_summary(id)` returns service/username/notes; `get_password(id)` returns the decrypted password. Search returns summaries — never passwords. This is deliberate: logs, UI scrollback, error messages, debug output all tend to contain "whatever the function returned," and if summaries included passwords, passwords would leak into places they shouldn't be.

First Rosetta Stone project with **explicit API-level separation between metadata and secret material**. Production password managers have this: clipboard-write is separate from display-in-UI; display is summary-only by default with explicit "reveal" gesture required to show plaintext. G077 models the API shape of that: the safe has two different functions and callers must actively choose to call the password-returning one.

## Choreographic Case: Agent Credential Retrieval

```innate
(@agent-credential-use){
  @safe <- @safe/load{path: "~/.dpn-safe.enc"}
  @master <- @secret/prompt{title: "unlock credential safe"}

  where { unlocked: @safe/unlock{safe: @safe, master: @master, now_ms: @now} }

  @cred <- @safe/search{safe: @safe, query: @target.service, now_ms: @now}.first
  @password <- @safe/get-password{safe: @safe, id: @cred.id, now_ms: @now}

  @use-credential{password: @password, target: @target}

  @safe/lock{safe: @safe}     ;; explicit lock at end of use
}
```

The safe is unlocked long enough to retrieve one credential, the password is used, the safe locks. Production agent workflows will follow this pattern: minimise the unlock window, explicit lock at end of need. Auto-lock is the safety net; explicit lock is the discipline.

## Structures

```innate
(defenum state
  Locked | Unlocked{key: Bytes, last-activity-ms: Int})

(defstruct entry
  id                 : Int
  service            : String
  username           : String
  encrypted-password : Bytes           ;; ciphertext, key-derived XOR
  notes              : String
  created-at-ms      : Int
  modified-at-ms     : Int)

(defstruct summary
  id                 : Int
  service            : String
  username           : String
  notes              : String)          ;; NEVER contains the password

(defstruct safe
  salt               : Bytes
  verifier           : Bytes            ;; verifies master; NOT the key
  entries            : [Entry]
  state              : State
  auto-lock-after-ms : Int)

(defstruct generate-policy
  length             : Int
  lower, upper, digits, symbols : Bool)

(defenum strength Weak | Fair | Strong | VeryStrong)
```

## Resolver Natives

```innate
@safe/new{master, auto_lock_ms}                 -> Safe
@safe/unlock{safe, master, now_ms}              -> Bool
@safe/lock{safe}                                -> Unit
@safe/is-locked{safe}                           -> Bool
@safe/check-auto-lock{safe, now_ms}             -> Unit

@safe/add-entry{safe, service, username, password, notes, now_ms}  -> EntryId?
@safe/get-password{safe, id, now_ms}            -> String?          ;; requires unlocked
@safe/get-summary{safe, id}                     -> Summary?         ;; does NOT require unlocked
@safe/remove-entry{safe, id, now_ms}            -> Bool
@safe/search{safe, query, now_ms}               -> [Summary]        ;; requires unlocked, returns summaries

@password/generate{policy, seed}                -> String
@password/strength{pw}                          -> Strength
```

## Demo

```innate
(@demo){
  @safe <- @safe/new{master: "my-master-passphrase", auto_lock_ms: 30000}
  @safe/is-locked{safe: @safe}                                   ;; -> true
  @safe/unlock{safe: @safe, master: "wrong", now_ms: 1000}       ;; -> false
  @safe/unlock{safe: @safe, master: "my-master-passphrase",
                now_ms: 1000}                                    ;; -> true

  @id <- @safe/add-entry{safe: @safe, service: "github.com",
                          username: "alice", password: "s3cret!",
                          notes: "main dev account", now_ms: 1000}
  @safe/get-password{safe: @safe, id: @id, now_ms: 2000}         ;; -> "s3cret!"

  @gen <- @password/generate{policy: {length: 20, lower: true, upper: true,
                                       digits: true, symbols: true}, seed: 42}
  @password/strength{pw: @gen}                                   ;; -> VeryStrong

  @safe/check-auto-lock{safe: @safe, now_ms: 40000}              ;; 40s idle → locks
  @safe/is-locked{safe: @safe}                                   ;; -> true
  @safe/get-password{safe: @safe, id: @id, now_ms: 41000}        ;; -> null (locked)
}
```

## Where

The safe MUST start in the Locked state — no exception for "trusted" contexts, no bypass on first creation. Unlock MUST verify via a hash-compare, not by comparing plaintext masters — the master is never stored. The verifier hash and the encryption key MUST be derived differently so that knowing one does not yield the other. Locking MUST clear the key from memory — in a real implementation this means zeroizing the buffer, not just dropping the reference. Auto-lock MUST trigger based on activity timestamp, and any user-visible operation (add, get, search, remove) MUST update the activity timestamp — checking the timestamp without updating it is a bug. Summaries MUST NEVER include the password — not in logs, not in error messages, not in search results; the password is returned ONLY from `get_password` which is an explicit, state-gated operation. Search MUST require unlocked even though it returns summaries only — searching the encrypted entries requires iterating them, which is itself information an attacker could exploit if done on a locked safe.
