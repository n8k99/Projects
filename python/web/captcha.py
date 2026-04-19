"""CAPTCHA Maker — challenge-response with expiration and attempt limits.

Closes the Web category. A CAPTCHA is an ephemeral, single-use, time-bounded
challenge: issue a prompt with a known answer, accept answers for a limited
window with a bounded retry count, refuse after solved/expired/exhausted.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ChallengeKind(Enum):
    MATH = "math"
    WORD_RECALL = "word_recall"
    REVERSE_STRING = "reverse_string"
    SEQUENCE_COUNT = "sequence_count"


@dataclass
class Challenge:
    id: int
    kind: ChallengeKind
    prompt: str
    expected: str
    created_at_ms: int
    expires_at_ms: int
    attempts_used: int = 0
    max_attempts: int = 3
    solved: bool = False


class VerifyKind(Enum):
    SUCCESS = "success"
    WRONG_ANSWER = "wrong_answer"
    EXPIRED = "expired"
    ALREADY_SOLVED = "already_solved"
    TOO_MANY_ATTEMPTS = "too_many_attempts"
    UNKNOWN = "unknown"


@dataclass(frozen=True)
class VerifyResult:
    kind: VerifyKind
    attempts_remaining: int | None = None


class CaptchaService:
    def __init__(self, default_ttl_ms: int, max_attempts: int, seed: int) -> None:
        self._challenges: list[Challenge] = []
        self._next_id = 1
        self._default_ttl_ms = default_ttl_ms
        self._max_attempts = max_attempts
        self._rng_state = seed

    def _next_random(self) -> int:
        self._rng_state = (self._rng_state * 6364136223846793005
                           + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        return self._rng_state >> 32

    def _generate(self, kind: ChallengeKind) -> tuple[str, str]:
        if kind is ChallengeKind.MATH:
            a = self._next_random() % 20 + 1
            b = self._next_random() % 20 + 1
            op = self._next_random() % 3
            if op == 0:
                return f"What is {a} + {b}?", str(a + b)
            if op == 1:
                hi, lo = max(a, b), min(a, b)
                return f"What is {hi} - {lo}?", str(hi - lo)
            return f"What is {a} × {b}?", str(a * b)
        if kind is ChallengeKind.WORD_RECALL:
            words = ["butterfly", "chronicle", "lighthouse", "symphony", "mountain"]
            w = words[self._next_random() % len(words)]
            return f"Type this word: {w}", w
        if kind is ChallengeKind.REVERSE_STRING:
            inputs = ["hello", "rosetta", "captcha", "innate"]
            w = inputs[self._next_random() % len(inputs)]
            return f"Reverse this: {w}", w[::-1]
        # SEQUENCE_COUNT
        length = self._next_random() % 6 + 5
        seq = []
        digits = 0
        for _ in range(length):
            r = self._next_random() % 36
            if r < 10:
                seq.append(str(r))
                digits += 1
            else:
                seq.append(chr(ord("A") + (r - 10)))
        return f"How many digits in: {''.join(seq)}?", str(digits)

    def issue(self, kind: ChallengeKind, now_ms: int) -> Challenge:
        cid = self._next_id
        self._next_id += 1
        prompt, expected = self._generate(kind)
        c = Challenge(
            id=cid, kind=kind, prompt=prompt, expected=expected,
            created_at_ms=now_ms,
            expires_at_ms=now_ms + self._default_ttl_ms,
            max_attempts=self._max_attempts,
        )
        self._challenges.append(c)
        return c

    def verify(self, cid: int, user_answer: str, now_ms: int) -> VerifyResult:
        c = next((c for c in self._challenges if c.id == cid), None)
        if c is None:
            return VerifyResult(VerifyKind.UNKNOWN)
        if c.solved:
            return VerifyResult(VerifyKind.ALREADY_SOLVED)
        if now_ms > c.expires_at_ms:
            return VerifyResult(VerifyKind.EXPIRED)
        if c.attempts_used >= c.max_attempts:
            return VerifyResult(VerifyKind.TOO_MANY_ATTEMPTS)
        c.attempts_used += 1
        if c.expected.strip().lower() == user_answer.strip().lower():
            c.solved = True
            return VerifyResult(VerifyKind.SUCCESS)
        return VerifyResult(VerifyKind.WRONG_ANSWER,
                            attempts_remaining=c.max_attempts - c.attempts_used)

    def challenge(self, cid: int) -> Challenge | None:
        return next((c for c in self._challenges if c.id == cid), None)

    def cleanup(self, now_ms: int) -> int:
        before = len(self._challenges)
        self._challenges = [c for c in self._challenges
                            if not c.solved and now_ms <= c.expires_at_ms]
        return before - len(self._challenges)

    def active_count(self) -> int:
        return len(self._challenges)


if __name__ == "__main__":
    s = CaptchaService(default_ttl_ms=60_000, max_attempts=3, seed=1234)

    # Issue one of each kind and print the prompt + expected answer.
    for kind in ChallengeKind:
        c = s.issue(kind, now_ms=1000)
        print(f"{kind.value:20s} → {c.prompt!r}  (expected: {c.expected!r})")

    print()
    # Verify a correct answer.
    math_c = s.issue(ChallengeKind.MATH, now_ms=2000)
    print(f"prompt: {math_c.prompt}")
    result = s.verify(math_c.id, math_c.expected, now_ms=3000)
    print(f"  correct → {result.kind.value}")

    # Wrong answer, then correct.
    math_c2 = s.issue(ChallengeKind.MATH, now_ms=3000)
    wrong = s.verify(math_c2.id, "0", now_ms=3500)
    print(f"  wrong once → {wrong.kind.value} (remaining={wrong.attempts_remaining})")
    ok = s.verify(math_c2.id, math_c2.expected, now_ms=4000)
    print(f"  then correct → {ok.kind.value}")

    # Expired.
    old_c = s.issue(ChallengeKind.MATH, now_ms=5000)
    expired = s.verify(old_c.id, old_c.expected, now_ms=5000 + 61_000)
    print(f"  expired attempt → {expired.kind.value}")

    # Cleanup removes solved + expired.
    removed = s.cleanup(now_ms=5000 + 61_000)
    print(f"  cleanup removed {removed} ({s.active_count()} remaining)")
