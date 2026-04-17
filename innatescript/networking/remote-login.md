# G044 — Remote Login

> Authenticated session management with credentials and tokens.

```yaml
id: G044
title: Remote Login
category: networking
requires: [G013-credit-card-validator, G028-ciphers, G031-cd-key-generator, G033-ftp-protocol]
provides: [authentication, session-management, brute-force-protection, audit-trail]
```

## Insight: Identity Verification Across a Boundary

Every networking project so far assumed trust — the client connected, and the server answered. Remote login introduces **distrust**: the server doesn't know who the client is until they prove it. The credential is the proof. The session token is the ongoing assertion of that proof.

This is the first time the Rosetta Stone builds a **trust establishment** protocol. G013 (credit card validator) checked structural validity — is this number well-formed? Remote login checks **identity** — are you who you claim to be? Structural validity is stateless. Identity verification creates state: the session.

## Insight: Salted Hashing Is One-Way Trust

The server never stores the password. It stores a salted hash — a one-way transformation that can verify but never recover the original. This is G028's cipher pattern (shared secret between two parties) with a crucial difference: the cipher is reversible, the hash is not. The server can confirm "you know the password" without itself knowing the password.

In the noosphere, agent authentication follows the same model. An agent proves its identity by demonstrating knowledge (the password) without transmitting the secret itself. The verification is the hash comparison. The trust is established by the match.

## Insight: Sessions Are Time-Bounded Identity Assertions

The session token is G031's CD key (self-identifying artifact) combined with G026's TTL (temporal scope). The token asserts: "this client authenticated as this user at this time, and the assertion expires at this time." After expiry, the assertion is void — the client must re-authenticate.

This connects to every temporal pattern in the Rosetta Stone: G011's alarm (the session timeout is a deferred obligation), G026's ticker TTL (the session expires like a news item), G042's domain expiry (the session is a renewable lease). Time bounds trust.

## Insight: Brute-Force Protection Is Rate-Limited `<-` Gates

The lockout policy — lock the account after N failed attempts — is G013's progressive trust gates applied to authentication. Each failed attempt is a failed `<-` gate. After too many failures, the entire gate closes for a cooldown period. The ascending cost: first attempt is free, subsequent attempts accumulate risk, final attempt triggers lockout.

The audit log records every attempt — G025's immutable journal applied to security events. The log answers: who tried to log in, from where, when, and did they succeed? This is the security version of Lena's nightly summary: a chronological record of authentication events.

## Insight: Revoke-All Is Emergency Session Kill

`revoke_all(username)` invalidates every session for a user — the nuclear option when credentials are compromised. This is a new pattern: retroactive invalidation of previously-granted trust. The sessions were valid when created. They're now invalid because the trust basis changed. The revocation propagates backward through time.

## Choreographic Case: Agent Authentication

```innate
(@agent-auth){
  @credential <- @auth/verify{
    agent: @kathryn,
    capability: @finance_positions,
    context: @current_choreography
  }
  <- @credential.valid?                    ;; gate: is the agent authenticated?
  <- @credential.role == :exec             ;; gate: does the role permit this operation?
  <- @credential.session.remaining > 60s   ;; gate: enough time left in session?
  
  // Only after all three gates pass does the choreography proceed.
  @result <- @kathryn/finance_positions
  
  where {
    authenticated: @credential.valid?
    authorized: @credential.role in [:exec, :admin]
    session_alive: @credential.session.remaining > 0
  }
}
```

Three `<-` gates before the operation: authentication (identity), authorization (role), and session validity (temporal). Each gate is cheaper than the operation it protects. Progressive trust applied to agent choreography.

## Structures

```innate
(defstruct credential
  username      : String
  password-hash : String
  salt          : String
  role          : String
  locked        : Bool
  failed-attempts : Nat)

(defstruct session
  token     : String
  username  : String
  role      : String
  created   : Instant
  expires   : Instant
  ip-address : String
  is-active : Bool)

(defstruct login-result
  success           : Bool
  session           : Session?
  error             : String
  attempts-remaining : Nat?)
```

## Resolver Natives

```innate
@auth/register{username: String, password: String, role: String}  -> Bool
@auth/login{username: String, password: String, ip: String}       -> LoginResult
@auth/validate{token: String}                                      -> Session?
@auth/logout{token: String}                                        -> Bool
@auth/revoke-all{username: String}                                 -> Nat
@auth/refresh{token: String}                                       -> Session?
```

## Demo

```innate
(@demo){
  @server <- @auth-server{ttl: 30min, max-failed: 3}
  @server/register{username: "nathan", password: "rosetta2026", role: "admin"}
  @result <- @server/login{username: "nathan", password: "rosetta2026", ip: "127.0.0.1"}
  @print{@result.success}     ;; => true
  @print{@result.session.role} ;; => "admin"
}
```
