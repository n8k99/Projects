package numbers

import (
	"fmt"
	"strconv"
	"strings"
	"unicode"
)

// Calculator — evaluate arithmetic expressions.

// token types for the calculator tokenizer.
const (
	tokNumber = iota
	tokPlus
	tokMinus
	tokStar
	tokSlash
	tokLParen
	tokRParen
)

type calcToken struct {
	kind int
	num  float64
}

// calcTokenize splits an expression string into tokens.
func calcTokenize(expr string) ([]calcToken, error) {
	var tokens []calcToken
	runes := []rune(expr)
	i := 0
	for i < len(runes) {
		c := runes[i]
		switch {
		case unicode.IsSpace(c):
			i++
		case c == '+':
			tokens = append(tokens, calcToken{kind: tokPlus})
			i++
		case c == '-':
			tokens = append(tokens, calcToken{kind: tokMinus})
			i++
		case c == '*':
			tokens = append(tokens, calcToken{kind: tokStar})
			i++
		case c == '/':
			tokens = append(tokens, calcToken{kind: tokSlash})
			i++
		case c == '(':
			tokens = append(tokens, calcToken{kind: tokLParen})
			i++
		case c == ')':
			tokens = append(tokens, calcToken{kind: tokRParen})
			i++
		case unicode.IsDigit(c) || c == '.':
			start := i
			for i < len(runes) && (unicode.IsDigit(runes[i]) || runes[i] == '.') {
				i++
			}
			s := string(runes[start:i])
			n, err := strconv.ParseFloat(s, 64)
			if err != nil {
				return nil, fmt.Errorf("invalid number '%s': %w", s, err)
			}
			tokens = append(tokens, calcToken{kind: tokNumber, num: n})
		default:
			return nil, fmt.Errorf("unexpected character: %c", c)
		}
	}
	return tokens, nil
}

// calcParser is a recursive descent parser for arithmetic expressions.
//
// Grammar:
//
//	expr   -> term (('+' | '-') term)*
//	term   -> unary (('*' | '/') unary)*
//	unary  -> '-' unary | atom
//	atom   -> NUMBER | '(' expr ')'
type calcParser struct {
	tokens []calcToken
	pos    int
}

func (p *calcParser) peek() (calcToken, bool) {
	if p.pos < len(p.tokens) {
		return p.tokens[p.pos], true
	}
	return calcToken{}, false
}

func (p *calcParser) consume() calcToken {
	tok := p.tokens[p.pos]
	p.pos++
	return tok
}

func (p *calcParser) expect(kind int) error {
	tok, ok := p.peek()
	if !ok {
		return fmt.Errorf("unexpected end of input")
	}
	if tok.kind != kind {
		return fmt.Errorf("unexpected token")
	}
	p.consume()
	return nil
}

func (p *calcParser) parseExpr() (float64, error) {
	left, err := p.parseTerm()
	if err != nil {
		return 0, err
	}
	for {
		tok, ok := p.peek()
		if !ok || (tok.kind != tokPlus && tok.kind != tokMinus) {
			break
		}
		p.consume()
		right, err := p.parseTerm()
		if err != nil {
			return 0, err
		}
		if tok.kind == tokPlus {
			left += right
		} else {
			left -= right
		}
	}
	return left, nil
}

func (p *calcParser) parseTerm() (float64, error) {
	left, err := p.parseUnary()
	if err != nil {
		return 0, err
	}
	for {
		tok, ok := p.peek()
		if !ok || (tok.kind != tokStar && tok.kind != tokSlash) {
			break
		}
		p.consume()
		right, err := p.parseUnary()
		if err != nil {
			return 0, err
		}
		if tok.kind == tokStar {
			left *= right
		} else {
			if right == 0 {
				return 0, fmt.Errorf("division by zero")
			}
			left /= right
		}
	}
	return left, nil
}

func (p *calcParser) parseUnary() (float64, error) {
	tok, ok := p.peek()
	if ok && tok.kind == tokMinus {
		p.consume()
		val, err := p.parseUnary()
		if err != nil {
			return 0, err
		}
		return -val, nil
	}
	if ok && tok.kind == tokPlus {
		p.consume()
		return p.parseUnary()
	}
	return p.parseAtom()
}

func (p *calcParser) parseAtom() (float64, error) {
	tok, ok := p.peek()
	if !ok {
		return 0, fmt.Errorf("unexpected end of input")
	}
	switch tok.kind {
	case tokLParen:
		p.consume()
		result, err := p.parseExpr()
		if err != nil {
			return 0, err
		}
		if err := p.expect(tokRParen); err != nil {
			return 0, fmt.Errorf("expected ')'")
		}
		return result, nil
	case tokNumber:
		p.consume()
		return tok.num, nil
	default:
		return 0, fmt.Errorf("unexpected token")
	}
}

// Evaluate parses and evaluates an arithmetic expression string.
// Supports +, -, *, /, parentheses, negative numbers, and decimals.
func Evaluate(expr string) (float64, error) {
	_ = strings.TrimSpace(expr)
	tokens, err := calcTokenize(expr)
	if err != nil {
		return 0, err
	}
	if len(tokens) == 0 {
		return 0, fmt.Errorf("empty expression")
	}
	p := &calcParser{tokens: tokens}
	result, err := p.parseExpr()
	if err != nil {
		return 0, err
	}
	if p.pos < len(p.tokens) {
		return 0, fmt.Errorf("unexpected token after expression")
	}
	return result, nil
}

// Demo usage (in a main package):
//
//	func main() {
//		examples := []string{
//			"2 + 3 * 4",
//			"(2 + 3) * 4",
//			"10 / 3",
//			"-(5 + 3)",
//		}
//		for _, expr := range examples {
//			result, err := numbers.Evaluate(expr)
//			if err != nil {
//				fmt.Printf("%s = ERROR: %v\n", expr, err)
//			} else {
//				fmt.Printf("%s = %g\n", expr, result)
//			}
//		}
//	}
