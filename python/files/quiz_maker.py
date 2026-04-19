"""Quiz Maker — polymorphic questions with file round-trip serialisation.

Opens the Files category. Theme: **round-trip equivalence** — write model
to file, read it back, get the same model. Questions come in three kinds
(MultipleChoice, TrueFalse, ShortAnswer) behind a uniform Question/Answer
pair. Scoring is a fold over (question, user_answer) tuples.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class QuestionKind(Enum):
    MC = "mc"
    TF = "tf"
    SA = "sa"


@dataclass
class Question:
    kind: QuestionKind
    prompt: str
    # MC
    options: list[str] = field(default_factory=list)
    correct_index: int = 0
    # TF
    correct_bool: bool = False
    # SA
    expected: str = ""


@dataclass
class Answer:
    kind: QuestionKind
    # MC
    choice_index: int | None = None
    # TF
    truth_value: bool | None = None
    # SA
    text: str | None = None


@dataclass
class Quiz:
    id: int
    title: str
    questions: list[Question] = field(default_factory=list)

    def add_question(self, q: Question) -> None:
        self.questions.append(q)


@dataclass
class QuizAttempt:
    quiz_id: int
    answers: list[Answer | None]
    submitted_at_ms: int


@dataclass(frozen=True)
class Score:
    total: int
    correct: int
    skipped: int
    wrong: int

    def percent(self) -> float:
        if self.total == 0:
            return 0.0
        return self.correct / self.total * 100.0


class ParseError(Exception):
    pass


def check_answer(q: Question, a: Answer) -> bool:
    if q.kind is not a.kind:
        return False
    if q.kind is QuestionKind.MC:
        return a.choice_index == q.correct_index
    if q.kind is QuestionKind.TF:
        return a.truth_value == q.correct_bool
    # SA
    if a.text is None:
        return False
    return q.expected.strip().lower() == a.text.strip().lower()


def score_attempt(quiz: Quiz, attempt: QuizAttempt) -> Score:
    correct = skipped = wrong = 0
    for i, q in enumerate(quiz.questions):
        a = attempt.answers[i] if i < len(attempt.answers) else None
        if a is None:
            skipped += 1
        elif check_answer(q, a):
            correct += 1
        else:
            wrong += 1
    return Score(total=len(quiz.questions), correct=correct,
                 skipped=skipped, wrong=wrong)


def to_text(quiz: Quiz) -> str:
    out = ["QUIZ", f"id: {quiz.id}", f"title: {quiz.title}"]
    for q in quiz.questions:
        out.append("---")
        if q.kind is QuestionKind.MC:
            out.append("kind: mc")
            out.append(f"prompt: {q.prompt}")
            for o in q.options:
                out.append(f"option: {o}")
            out.append(f"correct: {q.correct_index}")
        elif q.kind is QuestionKind.TF:
            out.append("kind: tf")
            out.append(f"prompt: {q.prompt}")
            out.append(f"correct: {str(q.correct_bool).lower()}")
        else:
            out.append("kind: sa")
            out.append(f"prompt: {q.prompt}")
            out.append(f"expected: {q.expected}")
    return "\n".join(out) + "\n"


def from_text(text: str) -> Quiz:
    lines = text.splitlines()
    it = iter(lines)

    def next_line() -> str | None:
        return next(it, None)

    header = next_line()
    if header != "QUIZ":
        raise ParseError("missing QUIZ header")

    id_line = next_line()
    if id_line is None or not id_line.startswith("id: "):
        raise ParseError("missing id")
    qid = int(id_line[4:])

    title_line = next_line()
    if title_line is None or not title_line.startswith("title: "):
        raise ParseError("missing title")
    title = title_line[7:]

    quiz = Quiz(id=qid, title=title)
    remaining: list[str] = list(it)
    while remaining:
        line = remaining[0]
        if line != "---":
            raise ParseError(f"expected --- got {line!r}")
        remaining.pop(0)
        # Gather lines until next --- or EOF
        block: list[str] = []
        while remaining and remaining[0] != "---":
            block.append(remaining.pop(0))
        quiz.questions.append(_parse_question_block(block))
    return quiz


def _parse_question_block(block: list[str]) -> Question:
    fields: dict[str, str] = {}
    options: list[str] = []
    for line in block:
        if line.startswith("option: "):
            options.append(line[8:])
        elif ": " in line:
            key, _, value = line.partition(": ")
            fields[key] = value
    kind_str = fields.get("kind")
    if kind_str == "mc":
        return Question(
            kind=QuestionKind.MC,
            prompt=fields.get("prompt", ""),
            options=options,
            correct_index=int(fields.get("correct", "0")),
        )
    if kind_str == "tf":
        return Question(
            kind=QuestionKind.TF,
            prompt=fields.get("prompt", ""),
            correct_bool=fields.get("correct", "").lower() == "true",
        )
    if kind_str == "sa":
        return Question(
            kind=QuestionKind.SA,
            prompt=fields.get("prompt", ""),
            expected=fields.get("expected", ""),
        )
    raise ParseError(f"unknown kind: {kind_str}")


if __name__ == "__main__":
    q = Quiz(id=42, title="Programming Languages")
    q.add_question(Question(
        kind=QuestionKind.MC,
        prompt="Which is Rust's package manager?",
        options=["npm", "cargo", "pip"],
        correct_index=1,
    ))
    q.add_question(Question(
        kind=QuestionKind.TF,
        prompt="Rust has a garbage collector.",
        correct_bool=False,
    ))
    q.add_question(Question(
        kind=QuestionKind.SA,
        prompt="Rust's ownership model prevents data races through...?",
        expected="ownership",
    ))

    text = to_text(q)
    print("--- serialised ---")
    print(text)

    # Round-trip.
    q2 = from_text(text)
    assert q2.id == q.id and q2.title == q.title
    assert len(q2.questions) == 3
    print(f"round-trip: {len(q2.questions)} questions preserved")

    # Score an attempt.
    attempt = QuizAttempt(
        quiz_id=42,
        answers=[
            Answer(QuestionKind.MC, choice_index=1),         # correct
            None,                                            # skipped
            Answer(QuestionKind.SA, text="  OWNERSHIP  "),   # correct (case+ws)
        ],
        submitted_at_ms=1000,
    )
    s = score_attempt(q, attempt)
    print(f"score: {s.correct}/{s.total} correct, {s.skipped} skipped, "
          f"{s.wrong} wrong — {s.percent():.1f}%")
