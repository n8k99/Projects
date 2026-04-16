//! Calculator — evaluate arithmetic expressions.

/// Token types produced by the tokenizer.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

/// Tokenize an expression string into a list of tokens.
fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => i += 1,
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                let n = s.parse::<f64>().map_err(|e| format!("invalid number '{s}': {e}"))?;
                tokens.push(Token::Number(n));
            }
            c => return Err(format!("unexpected character: {c}")),
        }
    }
    Ok(tokens)
}

/// Recursive descent parser for arithmetic expressions.
///
/// Grammar:
///   expr   -> term (('+' | '-') term)*
///   term   -> unary (('*' | '/') unary)*
///   unary  -> '-' unary | atom
///   atom   -> NUMBER | '(' expr ')'
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.peek() {
            Some(tok) if tok == expected => { self.consume(); Ok(()) }
            Some(tok) => Err(format!("expected {expected:?}, got {tok:?}")),
            None => Err(format!("expected {expected:?}, got end of input")),
        }
    }

    fn parse_expr(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;
        while matches!(self.peek(), Some(Token::Plus) | Some(Token::Minus)) {
            let op = self.consume();
            let right = self.parse_term()?;
            left = match op {
                Token::Plus => left + right,
                Token::Minus => left - right,
                _ => unreachable!(),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek(), Some(Token::Star) | Some(Token::Slash)) {
            let op = self.consume();
            let right = self.parse_unary()?;
            left = match op {
                Token::Star => left * right,
                Token::Slash => {
                    if right == 0.0 {
                        return Err("division by zero".to_string());
                    }
                    left / right
                }
                _ => unreachable!(),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<f64, String> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.consume();
            let val = self.parse_unary()?;
            return Ok(-val);
        }
        if matches!(self.peek(), Some(Token::Plus)) {
            self.consume();
            return self.parse_unary();
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some(Token::LParen) => {
                self.consume();
                let result = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(result)
            }
            Some(Token::Number(_)) => {
                if let Token::Number(n) = self.consume() {
                    Ok(n)
                } else {
                    unreachable!()
                }
            }
            Some(tok) => Err(format!("unexpected token: {tok:?}")),
            None => Err("unexpected end of input".to_string()),
        }
    }
}

/// Evaluate an arithmetic expression string and return the result.
pub fn evaluate(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr)?;
    if tokens.is_empty() {
        return Err("empty expression".to_string());
    }
    let mut parser = Parser::new(tokens);
    let result = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        return Err(format!("unexpected token after expression: {:?}", parser.peek()));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_addition() {
        assert_eq!(evaluate("2 + 3").unwrap(), 5.0);
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap(), 14.0);
    }

    #[test]
    fn parentheses() {
        assert_eq!(evaluate("(2 + 3) * 4").unwrap(), 20.0);
    }

    #[test]
    fn division() {
        let result = evaluate("10 / 4").unwrap();
        assert!((result - 2.5).abs() < 1e-10);
    }

    #[test]
    fn negation() {
        assert_eq!(evaluate("-(5 + 3)").unwrap(), -8.0);
    }

    #[test]
    fn nested_parens() {
        assert_eq!(evaluate("((1 + 2) * (3 + 4)) / 7").unwrap(), 3.0);
    }

    #[test]
    fn decimal_numbers() {
        let result = evaluate("3.14 * 2").unwrap();
        assert!((result - 6.28).abs() < 1e-10);
    }

    #[test]
    fn division_by_zero() {
        assert!(evaluate("1 / 0").is_err());
    }
}
