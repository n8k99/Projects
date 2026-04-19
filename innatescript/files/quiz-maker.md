# G085 — Quiz Maker

> The Rosetta Stone's first **Files-category** project. Introduces the category's central theme: **round-trip equivalence** — write the model to a file, read it back, get the same model. Polymorphic question kinds (MC/TF/SA) dispatched through a uniform Question/Answer pair. Scoring is a fold over (question, answer) tuples.

```yaml
id: G085
title: Quiz Maker
category: files
requires: [G079-text-game, G081-ecard, G083-template-maker]
provides: [round-trip-serialisation, line-based-text-format, polymorphic-questions, fold-scoring]
```

## Insight: Round-Trip Equivalence Is the Files Category's Contract

Every Files project centres on a single promise: **what you write, you can read back, unchanged**. A quiz serialised to text, re-parsed from that text, must equal the original quiz. Break that promise and the file format is useless; preserve it and the format is reliable forever.

This is the defining contract of every file format in existence — JSON, CSV, PNG, PDF, Git objects, SQLite databases. The Files category's projects all satisfy this contract for their specific data. G085 starts with the simplest case: a quiz model with a small, self-designed text format.

First Rosetta Stone project where **serialisation and deserialisation are co-defined** — you can't design one without thinking about the other, because they must compose to the identity function on valid inputs.

## Insight: Line-Based Text Format Is the Hobbyist Default

G085 doesn't use JSON, YAML, TOML, or any existing format. It invents a simple line-based format (`key: value`, with `---` as the question separator). Reasons:

- **No dependency**: every language can read and write lines without a library.
- **Human-readable**: you can edit a quiz file with `nano` or any text editor.
- **Diffable**: each line is independent; version control shows sensible diffs.
- **Extensible**: adding a new field is adding a new `key: value` line.

Cost: no nested structures, no quoting rules, no type safety beyond what the parser validates. For a domain with known kinds and a small set of fields, this is adequate and far cheaper than a full JSON parser.

First Rosetta Stone project where **the file format is an explicit design decision** rather than "whatever JSON library is lying around." The vault's note format (Markdown with YAML frontmatter) is similar — line-based, human-readable, no dependency for parsing.

## Insight: Polymorphism Through Tagged Structures

Questions come in three kinds. Rust and Lean model them as enum variants with kind-specific fields. Python and Go use a tagged struct (one `kind` field plus all possible value fields; only fields relevant to the kind matter). Common Lisp uses a keyword-dispatched struct with all fields present.

The difference is cosmetic — all five language families model **tagged unions at their idiomatic abstraction level**. The behavioral contract is the same: given a Question and an Answer, produce a boolean indicating whether the answer is correct, dispatched on kind.

Same pattern as G079's item-kind dispatch (Potion heals / Treasure adds gold / Weapon equips), but G085 makes it explicit at serialisation: the `kind:` field is what the parser reads first to know how to interpret the rest of the block.

## Insight: Scoring Is a Fold, Not a Switch

`score_attempt` walks each question in order, checks the corresponding answer, and accumulates totals. It doesn't branch on question kind at the outer level — that dispatch is hidden inside `check_answer(question, answer)`. The scoring function is **kind-agnostic**.

First Rosetta Stone project with **fold-over-pairs scoring**. G059 did polymorphism (shape_area summed across shapes); G085 applies the same pattern to correctness checking. Each language has its own idiom (Rust match, Python isinstance + attributes, Go switch on kind), but the shape is the same.

## Insight: Error Handling at the Parse Boundary

`from_text` returns a `Result<Quiz, ParseError>` (or equivalent). Every failure mode is an enum variant: missing header, missing field, invalid kind, invalid integer. The caller knows exactly what went wrong and can produce a useful error message.

First Rosetta Stone project where **the parse error is a structured type** the caller can destructure. G071 HTML parser was forgiving (no errors, best-effort parse); G083 template validator returned a list of issues; G085 is the first with a single error path returning structured diagnostics.

The vault's future note-ingestion code will use this pattern: parse errors should be data, not opaque strings, so the error-handling UI can point at specific lines.

## Choreographic Case: Vault Quiz Archive

```innate
(@vault-quiz-archive){
  @files <- @vault/find{path: "quizzes/*.quiz"}
  @quizzes <- @for file in @files {
    @text <- @file/read-string{path: @file.path}
    @quiz <- @quiz/parse{text: @text}
    @quiz
  }

  @by-title <- @quizzes.index-by(.title)
  @ui/list-quizzes{quizzes: @by-title}

  @on-user-creates-attempt (@quiz-id @answers){
    @quiz <- @by-title.get{id: @quiz-id}
    @score <- @quiz/score{quiz: @quiz, attempt: {answers: @answers}}
    @vault/save{path: "attempts/${@now}.attempt",
                 content: @attempt/to-text{quiz: @quiz, answers: @answers, score: @score}}
  }
}
```

The vault stores quizzes as `.quiz` files in a directory; a loader walks the directory, parses each file, and assembles an in-memory quiz library. Attempts are persisted the same way. Simple, auditable, git-friendly.

## Structures

```innate
(defenum question-kind MC | TF | SA)

(defstruct question
  kind            : QuestionKind
  prompt          : String
  options         : [String]      ;; MC
  correct-index   : Int           ;; MC
  correct-bool    : Bool          ;; TF
  expected        : String)       ;; SA

(defenum answer
  MC(choice : Int) | TF(truth : Bool) | SA(text : String))

(defstruct quiz
  id        : Int
  title     : String
  questions : [Question])

(defstruct score
  total, correct, skipped, wrong : Int)
```

## Resolver Natives

```innate
@quiz/new{id, title}                             -> Quiz
@quiz/add-mc-question{quiz, prompt, options, correct-index}  -> Unit
@quiz/add-tf-question{quiz, prompt, correct}     -> Unit
@quiz/add-sa-question{quiz, prompt, expected}    -> Unit
@quiz/to-text{quiz}                              -> String
@quiz/from-text{text}                            -> Quiz | ParseError
@quiz/score{quiz, attempt}                       -> Score
```

## Demo

```innate
(@demo){
  @q <- @quiz/new{id: 42, title: "Programming Languages"}
  @quiz/add-mc-question{quiz: @q, prompt: "Rust's package manager?",
                         options: ["npm", "cargo", "pip"], correct-index: 1}
  @quiz/add-tf-question{quiz: @q, prompt: "Rust has a GC.", correct: false}
  @quiz/add-sa-question{quiz: @q, prompt: "Ownership model?", expected: "ownership"}

  @text <- @quiz/to-text{quiz: @q}
  @restored <- @quiz/from-text{text: @text}
  @restored == @q                                 ;; -> true (round-trip)

  @score <- @quiz/score{quiz: @q, attempt: {answers: [
    {kind: "mc", choice: 1},
    null,                               ;; skipped
    {kind: "sa", text: "  OWNERSHIP  "}
  ]}}
  @score.percent                                  ;; -> 66.67
}
```

## Where

Serialise-then-parse MUST be the identity function on valid quizzes — `from_text(to_text(q)) == q` is the category's defining contract. Parse errors MUST be structured (enum of specific failure reasons) not opaque strings — the caller needs to tell the user what's wrong. Short-answer comparison MUST be case-insensitive and whitespace-trimmed — matching the user's intent, not their exact keystrokes. Kind mismatch (answer's kind doesn't match question's kind) MUST be scored as wrong, NOT as a runtime error — the user submitting a bool for a multiple-choice question is a UI bug, not a crash-the-scorer bug. Skipped answers (null/None) MUST be counted separately from wrong answers — the UI wants to distinguish "didn't try" from "tried and failed."
