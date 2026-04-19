// Quiz Maker — polymorphic questions with file round-trip serialisation.
package files

import (
	"fmt"
	"strconv"
	"strings"
)

// QKind classifies a question.
type QKind string

const (
	QMC QKind = "mc"
	QTF QKind = "tf"
	QSA QKind = "sa"
)

// Question holds all kinds of question data; only fields relevant to Kind matter.
type Question struct {
	Kind         QKind
	Prompt       string
	Options      []string // MC
	CorrectIndex int      // MC
	CorrectBool  bool     // TF
	Expected     string   // SA
}

// Answer is a user's answer; only field matching Kind is meaningful.
type Answer struct {
	Kind        QKind
	ChoiceIndex int
	TruthValue  bool
	Text        string
}

// Quiz is a titled list of questions.
type Quiz struct {
	ID        int64
	Title     string
	Questions []Question
}

// Score is the result of scoring an attempt.
type Score struct {
	Total, Correct, Skipped, Wrong int
}

// Percent returns Correct/Total as a percentage.
func (s Score) Percent() float64 {
	if s.Total == 0 {
		return 0
	}
	return float64(s.Correct) / float64(s.Total) * 100
}

// ScoreAttempt computes a score for `answers`, where nil = skipped.
func (q *Quiz) ScoreAttempt(answers []*Answer) Score {
	score := Score{Total: len(q.Questions)}
	for i, question := range q.Questions {
		var a *Answer
		if i < len(answers) {
			a = answers[i]
		}
		if a == nil {
			score.Skipped++
			continue
		}
		if checkAnswer(question, *a) {
			score.Correct++
		} else {
			score.Wrong++
		}
	}
	return score
}

func checkAnswer(q Question, a Answer) bool {
	if q.Kind != a.Kind {
		return false
	}
	switch q.Kind {
	case QMC:
		return a.ChoiceIndex == q.CorrectIndex
	case QTF:
		return a.TruthValue == q.CorrectBool
	case QSA:
		return strings.EqualFold(
			strings.TrimSpace(a.Text), strings.TrimSpace(q.Expected))
	}
	return false
}

// ToText serialises the quiz to the round-tripping text format.
func (q *Quiz) ToText() string {
	var b strings.Builder
	fmt.Fprintf(&b, "QUIZ\nid: %d\ntitle: %s\n", q.ID, q.Title)
	for _, qn := range q.Questions {
		b.WriteString("---\n")
		switch qn.Kind {
		case QMC:
			fmt.Fprintf(&b, "kind: mc\nprompt: %s\n", qn.Prompt)
			for _, o := range qn.Options {
				fmt.Fprintf(&b, "option: %s\n", o)
			}
			fmt.Fprintf(&b, "correct: %d\n", qn.CorrectIndex)
		case QTF:
			fmt.Fprintf(&b, "kind: tf\nprompt: %s\ncorrect: %t\n", qn.Prompt, qn.CorrectBool)
		case QSA:
			fmt.Fprintf(&b, "kind: sa\nprompt: %s\nexpected: %s\n", qn.Prompt, qn.Expected)
		}
	}
	return b.String()
}

// QuizParseError is returned for malformed quiz text.
type QuizParseError struct{ Reason string }

func (e *QuizParseError) Error() string { return "quiz parse error: " + e.Reason }

// ParseQuiz reads a quiz from text form produced by ToText.
func ParseQuiz(text string) (*Quiz, error) {
	lines := strings.Split(strings.TrimRight(text, "\n"), "\n")
	if len(lines) < 3 || lines[0] != "QUIZ" {
		return nil, &QuizParseError{"missing QUIZ header"}
	}
	if !strings.HasPrefix(lines[1], "id: ") {
		return nil, &QuizParseError{"missing id"}
	}
	id, err := strconv.ParseInt(lines[1][4:], 10, 64)
	if err != nil {
		return nil, &QuizParseError{"id not a number"}
	}
	if !strings.HasPrefix(lines[2], "title: ") {
		return nil, &QuizParseError{"missing title"}
	}
	q := &Quiz{ID: id, Title: lines[2][7:]}
	i := 3
	for i < len(lines) {
		if lines[i] != "---" {
			return nil, &QuizParseError{"expected ---, got " + lines[i]}
		}
		i++
		var block []string
		for i < len(lines) && lines[i] != "---" {
			block = append(block, lines[i])
			i++
		}
		question, err := parseQuestionBlock(block)
		if err != nil {
			return nil, err
		}
		q.Questions = append(q.Questions, question)
	}
	return q, nil
}

func parseQuestionBlock(block []string) (Question, error) {
	fields := map[string]string{}
	var options []string
	for _, line := range block {
		if strings.HasPrefix(line, "option: ") {
			options = append(options, line[8:])
			continue
		}
		if idx := strings.Index(line, ": "); idx >= 0 {
			fields[line[:idx]] = line[idx+2:]
		}
	}
	kind := fields["kind"]
	switch kind {
	case "mc":
		ci, _ := strconv.Atoi(fields["correct"])
		return Question{Kind: QMC, Prompt: fields["prompt"],
			Options: options, CorrectIndex: ci}, nil
	case "tf":
		return Question{Kind: QTF, Prompt: fields["prompt"],
			CorrectBool: fields["correct"] == "true"}, nil
	case "sa":
		return Question{Kind: QSA, Prompt: fields["prompt"],
			Expected: fields["expected"]}, nil
	}
	return Question{}, &QuizParseError{"unknown kind: " + kind}
}
