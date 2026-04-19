//! Quiz Maker — polymorphic questions with file round-trip serialisation.
//!
//! Opens the Files category. The key Files-category theme is **round-trip
//! equivalence**: write the model to a file, read it back, get the same
//! model. That property is the contract between in-memory representation
//! and persistent storage.
//!
//! Questions come in three kinds — MultipleChoice, TrueFalse, ShortAnswer —
//! with different answer spaces and scoring rules, dispatched through a
//! uniform `Question`/`Answer` pair. Scoring is a fold over `(question,
//! user_answer)` tuples producing a `Score` struct.

use std::collections::HashMap;

pub type QuizId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Question {
    MultipleChoice {
        prompt: String,
        options: Vec<String>,
        correct_index: usize,
    },
    TrueFalse {
        prompt: String,
        correct: bool,
    },
    ShortAnswer {
        prompt: String,
        expected: String,
    },
}

impl Question {
    pub fn prompt(&self) -> &str {
        match self {
            Question::MultipleChoice { prompt, .. } => prompt,
            Question::TrueFalse { prompt, .. } => prompt,
            Question::ShortAnswer { prompt, .. } => prompt,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    MultipleChoice(usize),
    TrueFalse(bool),
    ShortAnswer(String),
}

#[derive(Debug, Clone)]
pub struct Quiz {
    pub id: QuizId,
    pub title: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone)]
pub struct QuizAttempt {
    pub quiz_id: QuizId,
    pub answers: Vec<Option<Answer>>,
    pub submitted_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    pub total: u32,
    pub correct: u32,
    pub skipped: u32,
    pub wrong: u32,
}

impl Score {
    pub fn percent(&self) -> f64 {
        if self.total == 0 { return 0.0; }
        self.correct as f64 / self.total as f64 * 100.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    BadHeader(String),
    MissingField(String),
    InvalidIndex,
    UnknownKind(String),
}

impl Quiz {
    pub fn new(id: QuizId, title: &str) -> Self {
        Self { id, title: title.into(), questions: Vec::new() }
    }

    pub fn add_question(&mut self, q: Question) {
        self.questions.push(q);
    }

    /// Score an attempt: for each question, check if user_answer matches.
    pub fn score(&self, attempt: &QuizAttempt) -> Score {
        let total = self.questions.len() as u32;
        let mut correct = 0u32;
        let mut skipped = 0u32;
        let mut wrong = 0u32;
        for (i, question) in self.questions.iter().enumerate() {
            match attempt.answers.get(i).and_then(|o| o.as_ref()) {
                None => skipped += 1,
                Some(ans) => {
                    if check_answer(question, ans) { correct += 1; }
                    else { wrong += 1; }
                }
            }
        }
        Score { total, correct, skipped, wrong }
    }

    /// Serialise the quiz to a line-based text format that round-trips.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("QUIZ\n");
        out.push_str(&format!("id: {}\n", self.id));
        out.push_str(&format!("title: {}\n", self.title));
        for q in &self.questions {
            out.push_str("---\n");
            match q {
                Question::MultipleChoice { prompt, options, correct_index } => {
                    out.push_str("kind: mc\n");
                    out.push_str(&format!("prompt: {prompt}\n"));
                    for opt in options { out.push_str(&format!("option: {opt}\n")); }
                    out.push_str(&format!("correct: {correct_index}\n"));
                }
                Question::TrueFalse { prompt, correct } => {
                    out.push_str("kind: tf\n");
                    out.push_str(&format!("prompt: {prompt}\n"));
                    out.push_str(&format!("correct: {correct}\n"));
                }
                Question::ShortAnswer { prompt, expected } => {
                    out.push_str("kind: sa\n");
                    out.push_str(&format!("prompt: {prompt}\n"));
                    out.push_str(&format!("expected: {expected}\n"));
                }
            }
        }
        out
    }

    /// Parse the line-based format emitted by `to_text`.
    pub fn from_text(text: &str) -> Result<Self, ParseError> {
        let mut lines = text.lines().peekable();
        if lines.next() != Some("QUIZ") {
            return Err(ParseError::BadHeader("missing QUIZ header".into()));
        }
        let (id, title) = parse_header(&mut lines)?;
        let mut questions = Vec::new();
        while lines.peek() == Some(&"---") {
            lines.next();
            questions.push(parse_question(&mut lines)?);
        }
        Ok(Quiz { id, title, questions })
    }
}

fn check_answer(q: &Question, a: &Answer) -> bool {
    match (q, a) {
        (Question::MultipleChoice { correct_index, .. }, Answer::MultipleChoice(i)) => {
            i == correct_index
        }
        (Question::TrueFalse { correct, .. }, Answer::TrueFalse(b)) => b == correct,
        (Question::ShortAnswer { expected, .. }, Answer::ShortAnswer(s)) => {
            expected.trim().to_lowercase() == s.trim().to_lowercase()
        }
        _ => false,  // kind mismatch — never correct
    }
}

fn parse_header<'a, I: Iterator<Item = &'a str>>(
    lines: &mut std::iter::Peekable<I>,
) -> Result<(QuizId, String), ParseError> {
    let id_line = lines.next().ok_or_else(||
        ParseError::MissingField("id".into()))?;
    let id = id_line.strip_prefix("id: ")
        .ok_or_else(|| ParseError::MissingField("id".into()))?
        .parse::<u64>()
        .map_err(|_| ParseError::MissingField("id (unparseable)".into()))?;
    let title_line = lines.next().ok_or_else(||
        ParseError::MissingField("title".into()))?;
    let title = title_line.strip_prefix("title: ")
        .ok_or_else(|| ParseError::MissingField("title".into()))?
        .to_string();
    Ok((id, title))
}

fn parse_question<'a, I: Iterator<Item = &'a str>>(
    lines: &mut std::iter::Peekable<I>,
) -> Result<Question, ParseError> {
    let kind_line = lines.next().ok_or_else(||
        ParseError::MissingField("kind".into()))?;
    let kind = kind_line.strip_prefix("kind: ")
        .ok_or_else(|| ParseError::MissingField("kind".into()))?;

    let mut fields: HashMap<String, String> = HashMap::new();
    let mut options = Vec::new();
    while let Some(&line) = lines.peek() {
        if line == "---" { break; }
        lines.next();
        if let Some(rest) = line.strip_prefix("option: ") {
            options.push(rest.to_string());
        } else if let Some(colon_pos) = line.find(": ") {
            let key = &line[..colon_pos];
            let value = &line[colon_pos + 2..];
            fields.insert(key.to_string(), value.to_string());
        }
    }

    match kind {
        "mc" => {
            let prompt = fields.remove("prompt").ok_or_else(||
                ParseError::MissingField("prompt".into()))?;
            let correct_index = fields.remove("correct")
                .ok_or_else(|| ParseError::MissingField("correct".into()))?
                .parse::<usize>()
                .map_err(|_| ParseError::InvalidIndex)?;
            Ok(Question::MultipleChoice { prompt, options, correct_index })
        }
        "tf" => {
            let prompt = fields.remove("prompt").ok_or_else(||
                ParseError::MissingField("prompt".into()))?;
            let correct_str = fields.remove("correct").ok_or_else(||
                ParseError::MissingField("correct".into()))?;
            let correct = matches!(correct_str.as_str(), "true" | "True" | "TRUE");
            Ok(Question::TrueFalse { prompt, correct })
        }
        "sa" => {
            let prompt = fields.remove("prompt").ok_or_else(||
                ParseError::MissingField("prompt".into()))?;
            let expected = fields.remove("expected").ok_or_else(||
                ParseError::MissingField("expected".into()))?;
            Ok(Question::ShortAnswer { prompt, expected })
        }
        other => Err(ParseError::UnknownKind(other.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_quiz() -> Quiz {
        let mut q = Quiz::new(42, "Programming Languages");
        q.add_question(Question::MultipleChoice {
            prompt: "Which is Rust's package manager?".into(),
            options: vec!["npm".into(), "cargo".into(), "pip".into()],
            correct_index: 1,
        });
        q.add_question(Question::TrueFalse {
            prompt: "Rust has a garbage collector.".into(),
            correct: false,
        });
        q.add_question(Question::ShortAnswer {
            prompt: "Rust's ownership model prevents data races through...?".into(),
            expected: "ownership".into(),
        });
        q
    }

    #[test]
    fn perfect_attempt_scores_100_percent() {
        let q = sample_quiz();
        let attempt = QuizAttempt {
            quiz_id: q.id,
            answers: vec![
                Some(Answer::MultipleChoice(1)),
                Some(Answer::TrueFalse(false)),
                Some(Answer::ShortAnswer("ownership".into())),
            ],
            submitted_at_ms: 1000,
        };
        let score = q.score(&attempt);
        assert_eq!(score.total, 3);
        assert_eq!(score.correct, 3);
        assert_eq!(score.skipped, 0);
        assert_eq!(score.wrong, 0);
        assert!((score.percent() - 100.0).abs() < 0.01);
    }

    #[test]
    fn all_wrong_scores_zero() {
        let q = sample_quiz();
        let attempt = QuizAttempt {
            quiz_id: q.id,
            answers: vec![
                Some(Answer::MultipleChoice(0)),
                Some(Answer::TrueFalse(true)),
                Some(Answer::ShortAnswer("something else".into())),
            ],
            submitted_at_ms: 1000,
        };
        let score = q.score(&attempt);
        assert_eq!(score.correct, 0);
        assert_eq!(score.wrong, 3);
    }

    #[test]
    fn skipped_questions_counted() {
        let q = sample_quiz();
        let attempt = QuizAttempt {
            quiz_id: q.id,
            answers: vec![
                Some(Answer::MultipleChoice(1)),
                None, // skipped
                Some(Answer::ShortAnswer("wrong".into())),
            ],
            submitted_at_ms: 1000,
        };
        let score = q.score(&attempt);
        assert_eq!(score.correct, 1);
        assert_eq!(score.skipped, 1);
        assert_eq!(score.wrong, 1);
    }

    #[test]
    fn short_answer_is_case_and_whitespace_insensitive() {
        let q = Quiz::new(1, "");
        let mut q = q;
        q.add_question(Question::ShortAnswer {
            prompt: "Capital of France?".into(),
            expected: "Paris".into(),
        });
        let attempt = QuizAttempt {
            quiz_id: 1,
            answers: vec![Some(Answer::ShortAnswer("  PARIS  ".into()))],
            submitted_at_ms: 1000,
        };
        assert_eq!(q.score(&attempt).correct, 1);
    }

    #[test]
    fn kind_mismatch_in_answer_is_wrong_not_crash() {
        let q = sample_quiz();
        let attempt = QuizAttempt {
            quiz_id: q.id,
            answers: vec![
                Some(Answer::TrueFalse(true)),   // wrong kind for MC question
                Some(Answer::TrueFalse(false)),
                Some(Answer::ShortAnswer("ownership".into())),
            ],
            submitted_at_ms: 1000,
        };
        let score = q.score(&attempt);
        assert_eq!(score.correct, 2);
        assert_eq!(score.wrong, 1);
    }

    #[test]
    fn round_trip_preserves_every_question() {
        let q = sample_quiz();
        let text = q.to_text();
        let parsed = Quiz::from_text(&text).unwrap();
        assert_eq!(parsed.id, q.id);
        assert_eq!(parsed.title, q.title);
        assert_eq!(parsed.questions.len(), q.questions.len());
        for (a, b) in parsed.questions.iter().zip(q.questions.iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn round_trip_empty_quiz() {
        let q = Quiz::new(99, "empty quiz");
        let text = q.to_text();
        let parsed = Quiz::from_text(&text).unwrap();
        assert_eq!(parsed.id, 99);
        assert_eq!(parsed.title, "empty quiz");
        assert!(parsed.questions.is_empty());
    }

    #[test]
    fn parse_rejects_missing_header() {
        let err = Quiz::from_text("id: 1\ntitle: x");
        assert!(matches!(err, Err(ParseError::BadHeader(_))));
    }

    #[test]
    fn parse_rejects_unknown_kind() {
        let text = "QUIZ\nid: 1\ntitle: x\n---\nkind: xyz\nprompt: ?\n";
        let err = Quiz::from_text(text);
        assert!(matches!(err, Err(ParseError::UnknownKind(_))));
    }

    #[test]
    fn score_empty_quiz_is_empty_score() {
        let q = Quiz::new(1, "empty");
        let attempt = QuizAttempt {
            quiz_id: 1, answers: vec![], submitted_at_ms: 1000,
        };
        let score = q.score(&attempt);
        assert_eq!(score.total, 0);
        assert_eq!(score.percent(), 0.0);
    }
}
