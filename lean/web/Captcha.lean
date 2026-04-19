-- CAPTCHA Maker — challenge-response with expiration and attempt limits.
-- Pure-functional: every operation returns a new Service.

namespace Web.Cap

inductive Kind where | math | wordRecall | reverseString | sequenceCount
  deriving Repr, DecidableEq, BEq

structure Challenge where
  id : Nat
  kind : Kind
  prompt : String
  expected : String
  createdAtMs : Nat
  expiresAtMs : Nat
  attemptsUsed : Nat := 0
  maxAttempts : Nat := 3
  solved : Bool := false
  deriving Repr

inductive VerifyKind where
  | success | wrongAnswer | expired | alreadySolved
  | tooManyAttempts | unknown
  deriving Repr, DecidableEq, BEq

structure VerifyResult where
  kind : VerifyKind
  attemptsRemaining : Option Nat := none
  deriving Repr

structure Service where
  challenges : List Challenge := []
  nextId : Nat := 1
  defaultTTLMs : Nat
  maxAttempts : Nat
  rngState : UInt64
  deriving Repr

def Service.new (ttl maxAtt : Nat) (seed : UInt64) : Service :=
  { defaultTTLMs := ttl, maxAttempts := maxAtt, rngState := seed }

private def nextRandom (s : Service) : Service × UInt64 :=
  let r := s.rngState * 6364136223846793005 + 1442695040888963407
  ({ s with rngState := r }, r >>> 32)

private def generate (s : Service) (kind : Kind) : Service × String × String :=
  match kind with
  | .math =>
    let (s, r1) := nextRandom s
    let a := r1.toNat % 20 + 1
    let (s, r2) := nextRandom s
    let b := r2.toNat % 20 + 1
    let (s, r3) := nextRandom s
    let op := r3.toNat % 3
    match op with
    | 0 => (s, s!"What is {a} + {b}?", toString (a + b))
    | 1 =>
      let hi := max a b
      let lo := min a b
      (s, s!"What is {hi} - {lo}?", toString (hi - lo))
    | _ => (s, s!"What is {a} × {b}?", toString (a * b))
  | .wordRecall =>
    let words := ["butterfly", "chronicle", "lighthouse", "symphony", "mountain"]
    let (s, r) := nextRandom s
    let w := words.get! (r.toNat % words.length)
    (s, s!"Type this word: {w}", w)
  | .reverseString =>
    let inputs := ["hello", "rosetta", "captcha", "innate"]
    let (s, r) := nextRandom s
    let w := inputs.get! (r.toNat % inputs.length)
    let rev := w.toList.reverse.asString
    (s, s!"Reverse this: {w}", rev)
  | .sequenceCount =>
    let (s, rLen) := nextRandom s
    let length := rLen.toNat % 6 + 5
    let rec loop (s : Service) (i : Nat) (seq : String) (digits : Nat)
        : Service × String × Nat :=
      if i == 0 then (s, seq, digits)
      else
        let (s, r) := nextRandom s
        let v := r.toNat % 36
        if v < 10 then loop s (i - 1) (seq ++ toString v) (digits + 1)
        else loop s (i - 1) (seq.push (Char.ofNat (65 + (v - 10)))) digits
    let (s, seq, digits) := loop s length "" 0
    (s, s!"How many digits in: {seq}?", toString digits)

def Service.issue (s : Service) (kind : Kind) (nowMs : Nat) : Service × Challenge :=
  let id := s.nextId
  let (s, prompt, expected) := generate s kind
  let c : Challenge := {
    id, kind, prompt, expected,
    createdAtMs := nowMs,
    expiresAtMs := nowMs + s.defaultTTLMs,
    maxAttempts := s.maxAttempts
  }
  ({ s with challenges := s.challenges ++ [c], nextId := id + 1 }, c)

private def updateChallenge (s : Service) (id : Nat) (f : Challenge → Challenge)
    : Service :=
  { s with challenges := s.challenges.map fun c => if c.id == id then f c else c }

def Service.verify (s : Service) (id : Nat) (userAnswer : String) (nowMs : Nat)
    : Service × VerifyResult :=
  match s.challenges.find? (·.id == id) with
  | none => (s, { kind := .unknown })
  | some c =>
    if c.solved then (s, { kind := .alreadySolved })
    else if nowMs > c.expiresAtMs then (s, { kind := .expired })
    else if c.attemptsUsed ≥ c.maxAttempts then (s, { kind := .tooManyAttempts })
    else
      let matches := c.expected.trim.toLower == userAnswer.trim.toLower
      let s' := updateChallenge s id fun x =>
        { x with attemptsUsed := x.attemptsUsed + 1, solved := x.solved || matches }
      if matches then (s', { kind := .success })
      else
        let remaining := c.maxAttempts - (c.attemptsUsed + 1)
        (s', { kind := .wrongAnswer, attemptsRemaining := some remaining })

def Service.cleanup (s : Service) (nowMs : Nat) : Service × Nat :=
  let before := s.challenges.length
  let kept := s.challenges.filter fun c => !c.solved && nowMs ≤ c.expiresAtMs
  ({ s with challenges := kept }, before - kept.length)

end Web.Cap
